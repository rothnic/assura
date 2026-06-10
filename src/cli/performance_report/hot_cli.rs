//! Hot daemon/client measurement for repeated validation checks.
// allow-reason: performance row factories keep measured dimensions explicit
// for benchmark auditability despite wide argument lists.

use super::assura_cli::PreparedAssuraCli;
use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};
#[cfg(unix)]
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::BufRead;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::time::Instant;

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_hot_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_client: &PreparedAssuraCli,
    assura_checkd: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new("assura-check-hot-cli", "assura-check-hot-cli");
    let Some(client_path) = assura_check_client.binary_path.as_ref() else {
        return unavailable_row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            measurement,
            "assura-check-client",
            assura_check_client,
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

    let expected_status = fixture.metadata.expected_assura_exit_status;
    let mut samples = Vec::with_capacity(iterations);
    let mut failure = run_hot_client(client_path, &server.addr, expected_status)
        .err()
        .map(|error| format!("hot daemon warmup failed: {error}"));

    for _ in 0..iterations {
        if failure.is_some() {
            break;
        }
        match run_timed_hot_client(client_path, &server.addr, expected_status) {
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
        measurement.with_assura_binary(client_path, assura_check_client.binary_profile.as_deref()),
        samples,
        failure,
        baseline_id,
    )
}

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_status_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_status: &PreparedAssuraCli,
    assura_checkd: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new("assura-check-status-cli", "assura-check-status-cli");
    let Some(status_client_path) = assura_check_status.binary_path.as_ref() else {
        return unavailable_row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            baseline_id,
            ls_lint_status,
            measurement,
            "assura-check-status",
            assura_check_status,
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

    let status_dir = std::env::temp_dir().join(format!(
        "assura-checkd-{}-{}",
        std::process::id(),
        fixture.scenario.id
    ));
    let status_file = status_dir.join("assura-check.status");
    let _ = std::fs::create_dir_all(&status_dir);
    let _ = std::fs::remove_file(&status_file);

    let mut server = match start_hot_server(server_path, fixture, Some(&status_file)) {
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

    let expected_status = fixture.metadata.expected_assura_exit_status;
    let mut samples = Vec::with_capacity(iterations);
    let mut failure = run_status_client(status_client_path, &status_file, expected_status).err();

    for _ in 0..iterations {
        if failure.is_some() {
            break;
        }
        match run_timed_status_client(status_client_path, &status_file, expected_status) {
            Ok(runtime_ms) => samples.push(runtime_ms),
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }

    let _ = server.child.kill();
    let _ = server.child.wait();
    let _ = std::fs::remove_file(&status_file);
    let _ = std::fs::remove_dir(&status_dir);

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement.with_assura_binary(
            status_client_path,
            assura_check_status.binary_profile.as_deref(),
        ),
        samples,
        failure,
        baseline_id,
    )
}

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn unavailable_row(
    fixture: &MaterializedFixture,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    measurement: RowMeasurement<'_>,
    binary_name: &str,
    binary: &PreparedAssuraCli,
) -> PerformanceResultRow {
    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement,
        Vec::new(),
        Some(format!(
            "skipped because {binary_name} is unavailable: {}",
            binary
                .status
                .blocker
                .as_deref()
                .unwrap_or("unknown blocker")
        )),
        baseline_id,
    )
}

pub(super) struct HotServer {
    pub(super) child: Child,
    pub(super) addr: String,
}

pub(super) fn start_hot_server(
    server_path: &Path,
    fixture: &MaterializedFixture,
    status_file: Option<&Path>,
) -> Result<HotServer, String> {
    let listen = hot_listen_address(fixture);
    let mut command = Command::new(server_path);
    command
        .arg("--listen")
        .arg(&listen)
        .arg("--root")
        .arg(&fixture.root);
    if let Some(status_file) = status_file {
        command.arg("--status-file").arg(status_file);
    }
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to start assura-checkd: {error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "assura-checkd stdout was not captured".to_string())?;
    let mut reader = std::io::BufReader::new(stdout);
    let mut addr = String::new();
    reader
        .read_line(&mut addr)
        .map_err(|error| format!("failed to read assura-checkd address: {error}"))?;
    let addr = addr.trim().to_string();
    if addr.is_empty() {
        let _ = child.kill();
        let _ = child.wait();
        return Err("assura-checkd did not print a listening address".to_string());
    }

    Ok(HotServer { child, addr })
}

fn hot_listen_address(fixture: &MaterializedFixture) -> String {
    #[cfg(unix)]
    {
        let mut hasher = DefaultHasher::new();
        fixture.scenario.id.hash(&mut hasher);
        let socket_path = std::env::temp_dir().join(format!(
            "asr-{}-{:016x}.sock",
            std::process::id(),
            hasher.finish()
        ));
        format!("unix:{}", socket_path.display())
    }

    #[cfg(not(unix))]
    {
        let _ = fixture;
        "127.0.0.1:0".to_string()
    }
}

fn run_hot_client(client_path: &Path, addr: &str, expected_status: i32) -> Result<(), String> {
    run_timed_hot_client(client_path, addr, expected_status).map(|_| ())
}

fn run_timed_hot_client(
    client_path: &Path,
    addr: &str,
    expected_status: i32,
) -> Result<f64, String> {
    let mut command = Command::new(client_path);
    command
        .arg(addr)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let started = Instant::now();
    let status = command
        .status()
        .map_err(|error| format!("failed to run assura-check-client: {error}"))?;

    if status.code() == Some(expected_status) {
        return Ok(started.elapsed().as_secs_f64() * 1000.0);
    }

    Err(format!(
        "expected exit {expected_status}, got {:?}",
        status.code()
    ))
}

fn run_status_client(
    client_path: &Path,
    status_file: &Path,
    expected_status: i32,
) -> Result<(), String> {
    run_timed_status_client(client_path, status_file, expected_status).map(|_| ())
}

fn run_timed_status_client(
    client_path: &Path,
    status_file: &Path,
    expected_status: i32,
) -> Result<f64, String> {
    let mut command = Command::new(client_path);
    command
        .arg(status_file)
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let started = Instant::now();
    let status = command
        .status()
        .map_err(|error| format!("failed to run assura-check-status: {error}"))?;

    if status.code() == Some(expected_status) {
        return Ok(started.elapsed().as_secs_f64() * 1000.0);
    }

    Err(format!(
        "expected exit {expected_status}, got {:?}",
        status.code()
    ))
}

#[cfg(test)]
mod tests {
    use super::hot_listen_address;
    use crate::cli::performance_report::fixtures::{
        FixtureKind, FixtureMetadata, FixtureScenario, MaterializedFixture,
    };
    use std::path::PathBuf;

    #[test]
    fn hot_listen_address_stays_short_for_long_fixture_ids() {
        let fixture = MaterializedFixture {
            root: PathBuf::from("/tmp/project"),
            scenario: FixtureScenario {
                id: "many_configured_scopes_regression_with_a_long_name",
                source_revision: "test",
                rule_cohort: "test",
                dirs: 0,
                files_per_dir: 0,
                kind: FixtureKind::ManyConfiguredScopesRegression,
            },
            metadata: FixtureMetadata {
                source_type: "generated",
                source_revision: "test".to_string(),
                cohort: "realistic-equivalent",
                checked_file_count: 0,
                ignored_file_count: 0,
                directory_count: 0,
                rule_count: 0,
                rule_surface_summary: "test",
                native_ls_lint_parity: true,
                assura_config_path: ".assura/config.yml",
                ls_lint_config_path: ".ls-lint.yml",
                config_generation_method: "test",
                shared_config_id: "test:test".to_string(),
                expected_assura_exit_status: 0,
                expected_ls_lint_exit_status: 0,
            },
        };

        let address = hot_listen_address(&fixture);
        #[cfg(unix)]
        assert!(address.len() < 104, "{address}");
    }
}
