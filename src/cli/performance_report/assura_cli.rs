//! Assura CLI subprocess measurement for performance reports.

use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability, ASSURA_VERSION,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

pub(super) struct PreparedAssuraCli {
    status: ToolAvailability,
    binary_path: Option<PathBuf>,
    binary_profile: Option<String>,
}

pub(super) fn prepare_assura_cli() -> PreparedAssuraCli {
    match std::env::current_exe() {
        Ok(binary_path) if binary_path.is_file() => PreparedAssuraCli {
            binary_profile: Some(assura_binary_profile(&binary_path)),
            status: ToolAvailability {
                available: true,
                version: Some(ASSURA_VERSION.to_string()),
                blocker: None,
            },
            binary_path: Some(binary_path),
        },
        Ok(binary_path) => PreparedAssuraCli {
            status: ToolAvailability {
                available: false,
                version: None,
                blocker: Some(format!(
                    "current executable is not a file: {}",
                    binary_path.display()
                )),
            },
            binary_path: None,
            binary_profile: None,
        },
        Err(error) => PreparedAssuraCli {
            status: ToolAvailability {
                available: false,
                version: None,
                blocker: Some(format!("failed to resolve current executable: {error}")),
            },
            binary_path: None,
            binary_profile: None,
        },
    }
}

fn assura_binary_profile(binary_path: &Path) -> String {
    if binary_path
        .components()
        .any(|component| component.as_os_str() == "release")
    {
        "release".to_string()
    } else if binary_path
        .components()
        .any(|component| component.as_os_str() == "debug")
    {
        "debug".to_string()
    } else {
        "unknown".to_string()
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_cli: &PreparedAssuraCli,
) -> PerformanceResultRow {
    measure_assura_cli_row(
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        assura_cli,
        RowMeasurement::new("assura-cli", "assura-cli"),
        None,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_strategy(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_cli: &PreparedAssuraCli,
    strategy_name: &'static str,
    traversal_env: Option<&'static str>,
) -> PerformanceResultRow {
    measure_assura_cli_row(
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        assura_cli,
        RowMeasurement::new(strategy_name, strategy_name),
        traversal_env,
    )
}

#[allow(clippy::too_many_arguments)]
fn measure_assura_cli_row(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_cli: &PreparedAssuraCli,
    measurement: RowMeasurement<'_>,
    traversal_env: Option<&str>,
) -> PerformanceResultRow {
    let Some(binary_path) = assura_cli.binary_path.as_ref() else {
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
                "skipped because Assura CLI is unavailable: {}",
                assura_cli
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
    let expected_status = fixture.metadata.expected_assura_exit_status;
    for _ in 0..iterations {
        let started = Instant::now();
        let output = Command::new(binary_path)
            .arg("check")
            .arg(&fixture.root)
            .arg("--format")
            .arg("json")
            .env_remove("ASSURA_CHECK_TRAVERSAL")
            .envs(traversal_env.map(|value| ("ASSURA_CHECK_TRAVERSAL", value)))
            .output();
        match output {
            Ok(output) if output.status.code() == Some(expected_status) => {
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            Ok(output) => {
                failure = Some(format!(
                    "expected exit {expected_status}, got {:?}; stdout: {}; stderr: {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stdout).trim(),
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
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
        measurement.with_assura_binary(binary_path, assura_cli.binary_profile.as_deref()),
        samples,
        failure,
        baseline_id,
    )
}
