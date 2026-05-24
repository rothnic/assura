//! Persistent session-client measurement for editor-loop validation evidence.

use super::assura_cli::PreparedAssuraCli;
use super::changed_path_cli::{changed_path_candidate, mark_changed_path};
use super::hot_cli::{start_hot_server, unavailable_row};
use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};
use std::io::{BufRead, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_dirty_project_session_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_session: &PreparedAssuraCli,
    assura_checkd: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new(
        "assura-check-dirty-project-session-cli",
        "assura-check-dirty-project-session-cli",
    );
    let Some(session_path) = assura_check_session.binary_path.as_ref() else {
        return unavailable_row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            measurement,
            "assura-check-session",
            assura_check_session,
        );
    };
    let Some(server_path) = assura_checkd.binary_path.as_ref() else {
        return unavailable_row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            measurement,
            "assura-checkd",
            assura_checkd,
        );
    };
    let changed_path = match changed_path_candidate(fixture) {
        Ok(path) => path,
        Err(error) => {
            return row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                ls_lint_status.version.as_deref().unwrap_or("unavailable"),
                measurement,
                Vec::new(),
                Some(error),
                baseline_id,
            )
        }
    };

    let mut server = match start_hot_server(server_path, fixture, None) {
        Ok(server) => server,
        Err(error) => {
            return row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                ls_lint_status.version.as_deref().unwrap_or("unavailable"),
                measurement,
                Vec::new(),
                Some(error),
                baseline_id,
            )
        }
    };
    let mut session = match SessionClient::start(session_path, &server.addr) {
        Ok(session) => session,
        Err(error) => {
            let _ = server.child.kill();
            let _ = server.child.wait();
            return row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                ls_lint_status.version.as_deref().unwrap_or("unavailable"),
                measurement,
                Vec::new(),
                Some(error),
                baseline_id,
            );
        }
    };

    let expected_status = 0;
    let mut samples = Vec::with_capacity(iterations);
    let mut failure = session
        .request("CHECK", expected_status)
        .err()
        .map(|error| format!("dirty-project session warmup failed: {error}"));

    for _ in 0..iterations {
        if failure.is_some() {
            break;
        }
        if let Err(error) = mark_changed_path(&fixture.root.join(&changed_path)) {
            failure = Some(error);
            break;
        }
        let command = format!("DIRTY-PROJECT-PATH\t{}", changed_path.display());
        match session.timed_request(&command, expected_status) {
            Ok(runtime_ms) => samples.push(runtime_ms),
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }

    let _ = session.shutdown();
    let _ = server.child.kill();
    let _ = server.child.wait();

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement
            .with_assura_binary(session_path, assura_check_session.binary_profile.as_deref()),
        samples,
        failure,
        baseline_id,
    )
}

struct SessionClient {
    child: Child,
    stdin: ChildStdin,
    stdout: std::io::BufReader<ChildStdout>,
}

impl SessionClient {
    fn start(session_path: &std::path::Path, addr: &str) -> Result<Self, String> {
        let mut child = Command::new(session_path)
            .arg(addr)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| format!("failed to start assura-check-session: {error}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "assura-check-session stdin was not captured".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "assura-check-session stdout was not captured".to_string())?;
        Ok(Self {
            child,
            stdin,
            stdout: std::io::BufReader::new(stdout),
        })
    }

    fn request(&mut self, command: &str, expected_status: i32) -> Result<(), String> {
        self.timed_request(command, expected_status).map(|_| ())
    }

    fn timed_request(&mut self, command: &str, expected_status: i32) -> Result<f64, String> {
        let started = Instant::now();
        writeln!(self.stdin, "{command}")
            .map_err(|error| format!("write session command: {error}"))?;
        self.stdin
            .flush()
            .map_err(|error| format!("flush session command: {error}"))?;

        let mut response = String::new();
        self.stdout
            .read_line(&mut response)
            .map_err(|error| format!("read session response: {error}"))?;
        let exit_code = parse_session_response(&response)?;
        if exit_code == expected_status {
            return Ok(started.elapsed().as_secs_f64() * 1000.0);
        }
        Err(format!("expected exit {expected_status}, got {exit_code}"))
    }

    fn shutdown(&mut self) -> Result<(), String> {
        writeln!(self.stdin, "QUIT").map_err(|error| format!("write session quit: {error}"))?;
        self.stdin
            .flush()
            .map_err(|error| format!("flush session quit: {error}"))?;
        self.child
            .wait()
            .map_err(|error| format!("wait for assura-check-session: {error}"))?;
        Ok(())
    }
}

fn parse_session_response(response: &str) -> Result<i32, String> {
    let mut parts = response.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some("OK"), Some(code)) => code
            .parse::<i32>()
            .map_err(|error| format!("invalid session status code: {error}")),
        _ => Err(format!("invalid session response: {}", response.trim())),
    }
}
