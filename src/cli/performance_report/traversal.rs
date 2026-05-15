//! Filesystem traversal-only measurements for migration evidence.

use super::{row, FixtureScenario, PerformanceEnvironment, PerformanceResultRow, ToolAvailability};
use std::path::Path;
use std::time::Instant;

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_walkdir_traversal(
    scenario: FixtureScenario,
    fixture: &Path,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
) -> PerformanceResultRow {
    measure_traversal(
        scenario,
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        "walkdir-before-jwalk",
        count_walkdir_entries,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_jwalk_traversal(
    scenario: FixtureScenario,
    fixture: &Path,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
) -> PerformanceResultRow {
    measure_traversal(
        scenario,
        fixture,
        iterations,
        timestamp,
        commit_sha,
        branch,
        environment,
        baseline_id,
        ls_lint_status,
        "jwalk-after-migration",
        count_jwalk_entries,
    )
}

#[allow(clippy::too_many_arguments)]
fn measure_traversal(
    scenario: FixtureScenario,
    fixture: &Path,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    tool_name: &str,
    count_entries: fn(&Path) -> Result<usize, String>,
) -> PerformanceResultRow {
    let mut samples = Vec::with_capacity(iterations);
    let mut failure = None;
    for _ in 0..iterations {
        let started = Instant::now();
        match count_entries(fixture) {
            Ok(_) => samples.push(started.elapsed().as_secs_f64() * 1000.0),
            Err(error) => {
                failure = Some(error);
                break;
            }
        }
    }

    row(
        scenario,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        tool_name,
        samples,
        failure,
        baseline_id,
    )
}

fn count_walkdir_entries(path: &Path) -> Result<usize, String> {
    let mut count = 0;
    for entry in walkdir::WalkDir::new(path) {
        entry.map_err(|error| error.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn count_jwalk_entries(path: &Path) -> Result<usize, String> {
    let mut count = 0;
    for entry in jwalk::WalkDir::new(path)
        .skip_hidden(false)
        .parallelism(jwalk::Parallelism::Serial)
    {
        entry.map_err(|error| error.to_string())?;
        count += 1;
    }
    Ok(count)
}
