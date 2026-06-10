//! Direct hot-daemon socket profiling for dirty-project validation.
// allow-reason: performance row factories keep measured dimensions explicit
// for benchmark auditability despite wide argument lists.

use super::assura_cli::PreparedAssuraCli;
use super::changed_path_cli::{changed_path_candidate, mark_changed_path};
use super::hot_cli::{start_hot_server, unavailable_row};
use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};
use std::io::{Read, Write};
use std::net::TcpStream;
#[cfg(unix)]
use std::os::unix::net::UnixStream;
use std::time::Instant;

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_dirty_project_socket(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_checkd: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new(
        "assura-check-dirty-project-socket",
        "assura-check-dirty-project-socket",
    );
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

    let expected_status = 0;
    let mut samples = Vec::with_capacity(iterations);
    let mut failure = run_direct_hot_request(&server.addr, b"C\n", expected_status)
        .err()
        .map(|error| format!("dirty-project socket warmup failed: {error}"));

    for _ in 0..iterations {
        if failure.is_some() {
            break;
        }
        if let Err(error) = mark_changed_path(&fixture.root.join(&changed_path)) {
            failure = Some(error);
            break;
        }
        let request = format!("D\t{}\n", changed_path.display());
        match run_timed_direct_hot_request(&server.addr, request.as_bytes(), expected_status) {
            Ok(runtime_ms) => samples.push(runtime_ms),
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }

    let _ = server.child.kill();
    let _ = server.child.wait();

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement.with_assura_binary(server_path, assura_checkd.binary_profile.as_deref()),
        samples,
        failure,
        baseline_id,
    )
}

fn run_direct_hot_request(addr: &str, request: &[u8], expected_status: i32) -> Result<(), String> {
    run_timed_direct_hot_request(addr, request, expected_status).map(|_| ())
}

fn run_timed_direct_hot_request(
    addr: &str,
    request: &[u8],
    expected_status: i32,
) -> Result<f64, String> {
    let started = Instant::now();
    let exit_code = send_direct_hot_request(addr, request)?;
    if exit_code == expected_status {
        return Ok(started.elapsed().as_secs_f64() * 1000.0);
    }

    Err(format!("expected exit {expected_status}, got {exit_code}"))
}

fn send_direct_hot_request(addr: &str, request: &[u8]) -> Result<i32, String> {
    #[cfg(unix)]
    if let Some(socket_path) = addr.strip_prefix("unix:") {
        let mut stream = UnixStream::connect(socket_path)
            .map_err(|error| format!("connect unix socket: {error}"))?;
        return request_over_stream(&mut stream, request);
    }

    let mut stream =
        TcpStream::connect(addr).map_err(|error| format!("connect tcp socket: {error}"))?;
    request_over_stream(&mut stream, request)
}

fn request_over_stream(stream: &mut impl ReadWrite, request: &[u8]) -> Result<i32, String> {
    stream
        .write_all(request)
        .map_err(|error| format!("write daemon request: {error}"))?;
    let mut response = [0_u8; 32];
    let len = stream
        .read(&mut response)
        .map_err(|error| format!("read daemon response: {error}"))?;
    parse_hot_response(&response[..len])
}

fn parse_hot_response(response: &[u8]) -> Result<i32, String> {
    if response.len() == 1 && response[0].is_ascii_digit() {
        return Ok(i32::from(response[0] - b'0'));
    }

    let text = std::str::from_utf8(response).map_err(|_| "invalid UTF-8 response".to_string())?;
    let mut parts = text.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some("OK"), Some(code)) => code
            .parse::<i32>()
            .map_err(|error| format!("invalid daemon status code: {error}")),
        _ => Err(format!("invalid daemon response: {}", text.trim())),
    }
}

trait ReadWrite: Read + Write {}

impl<T: Read + Write> ReadWrite for T {}
