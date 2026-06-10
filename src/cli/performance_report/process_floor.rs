//! Process launch floor measurement for interpreting tiny CLI benchmark rows.
// allow-reason: performance row factories keep measured dimensions explicit
// for benchmark auditability despite wide argument lists.

use super::assura_cli::PreparedAssuraCli;
use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};
use std::process::{Command, Stdio};
use std::time::Instant;

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_process_floor(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
) -> PerformanceResultRow {
    let mut samples = Vec::with_capacity(iterations);
    let mut failure = None;

    for _ in 0..iterations {
        let mut command = process_floor_command();
        command.stdout(Stdio::null()).stderr(Stdio::null());
        let started = Instant::now();
        let status = command.status();
        match status {
            Ok(status) if status.success() => {
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            Ok(status) => {
                failure = Some(format!("process floor command exited {:?}", status.code()));
                break;
            }
            Err(error) => {
                failure = Some(error.to_string());
                break;
            }
        }
    }

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        RowMeasurement::new("process-floor", "process-floor"),
        samples,
        failure,
        baseline_id,
    )
}

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_rust_cli_floor(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_noop: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new("assura-rust-cli-floor", "assura-rust-cli-floor");
    let Some(binary_path) = assura_check_noop.binary_path.as_ref() else {
        return row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            ls_lint_status.version.as_deref().unwrap_or("unavailable"),
            measurement,
            Vec::new(),
            Some(format!(
                "skipped because assura-check-noop is unavailable: {}",
                assura_check_noop
                    .status
                    .blocker
                    .as_deref()
                    .unwrap_or("unknown blocker")
            )),
            baseline_id,
        );
    };

    let mut samples = Vec::with_capacity(iterations);
    let mut failure = None;

    for _ in 0..iterations {
        let mut command = Command::new(binary_path);
        command.stdout(Stdio::null()).stderr(Stdio::null());
        let started = Instant::now();
        let status = command.status();
        match status {
            Ok(status) if status.success() => {
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            Ok(status) => {
                failure = Some(format!("assura-check-noop exited with {:?}", status.code()));
                break;
            }
            Err(error) => {
                failure = Some(error.to_string());
                break;
            }
        }
    }

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement.with_assura_binary(binary_path, assura_check_noop.binary_profile.as_deref()),
        samples,
        failure,
        baseline_id,
    )
}

#[cfg(unix)]
fn process_floor_command() -> Command {
    Command::new("/usr/bin/true")
}

#[cfg(windows)]
fn process_floor_command() -> Command {
    let mut command = Command::new("cmd");
    command.args(["/C", "exit", "0"]);
    command
}
