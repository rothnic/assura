//! Cached check CLI subprocess measurement.

use super::assura_cli::{measure_assura_cli_row, AssuraInvocation, PreparedAssuraCli};
use super::{
    MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_assura_check_cached_cli(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    assura_check_cli: &PreparedAssuraCli,
) -> PerformanceResultRow {
    let cache_dir = std::env::temp_dir().join(format!(
        "assura-check-cache-{}-{}",
        std::process::id(),
        fixture.scenario.id
    ));
    let _ = std::fs::remove_dir_all(&cache_dir);

    let row = measure_assura_cli_row(
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        assura_check_cli,
        RowMeasurement::new("assura-check-cached-cli", "assura-check-cached-cli"),
        None,
        AssuraInvocation::CheckOnlyCached {
            cache_dir: cache_dir.clone(),
        },
    );

    let _ = std::fs::remove_dir_all(&cache_dir);
    row
}
