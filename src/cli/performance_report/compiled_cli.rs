//! Compiled-config check CLI subprocess measurement.

use super::assura_cli::{measure_assura_cli_row, AssuraInvocation, PreparedAssuraCli};
use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};
use std::path::Path;
use std::process::Command;

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_compiled_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_compiled: &PreparedAssuraCli,
    assura_check_compile_config: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let compiled_config_path = fixture.root.join(".assura/check-config.bin");
    if let Err(error) =
        write_compiled_config(fixture, &compiled_config_path, assura_check_compile_config)
    {
        return row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            ls_lint_status.version.as_deref().unwrap_or("unavailable"),
            RowMeasurement::new("assura-check-compiled-cli", "assura-check-compiled-cli"),
            Vec::new(),
            Some(error),
            baseline_id,
        );
    }

    measure_assura_cli_row(
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        assura_check_compiled,
        RowMeasurement::new("assura-check-compiled-cli", "assura-check-compiled-cli"),
        None,
        AssuraInvocation::CheckOnlyCompiled,
    )
}

fn write_compiled_config(
    fixture: &MaterializedFixture,
    output_path: &Path,
    compiler: &PreparedAssuraCli,
) -> Result<(), String> {
    let Some(binary_path) = compiler.binary_path.as_ref() else {
        return Err(format!(
            "skipped because assura-check-compile-config is unavailable: {}",
            compiler
                .status
                .blocker
                .as_deref()
                .unwrap_or("unknown blocker")
        ));
    };

    let config_path = fixture.root.join(".assura/config.yml");
    let output = Command::new(binary_path)
        .arg("--config")
        .arg(&config_path)
        .arg("--output")
        .arg(output_path)
        .output()
        .map_err(|error| format!("run assura-check-compile-config: {error}"))?;
    if output.status.success() {
        return Ok(());
    }

    Err(format!(
        "assura-check-compile-config exited {:?}; stdout: {}; stderr: {}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).trim(),
        String::from_utf8_lossy(&output.stderr).trim()
    ))
}
