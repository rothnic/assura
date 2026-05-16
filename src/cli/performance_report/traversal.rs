//! Filesystem traversal-only measurements for migration evidence.

use super::{row, FixtureScenario, PerformanceEnvironment, PerformanceResultRow, ToolAvailability};
use std::path::Path;
use std::thread;
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
        "walkdir",
        count_walkdir_entries,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_serial_jwalk_traversal(
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
        "jwalk-serial",
        |path| count_jwalk_entries(path, jwalk::Parallelism::Serial),
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_parallel_jwalk_traversal(
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
        "jwalk-parallel",
        |path| count_jwalk_entries(path, parallel_jwalk_strategy()),
    )
}

#[allow(clippy::too_many_arguments)]
fn measure_traversal<F>(
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
    count_entries: F,
) -> PerformanceResultRow
where
    F: Fn(&Path) -> Result<usize, String>,
{
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

fn count_jwalk_entries(path: &Path, parallelism: jwalk::Parallelism) -> Result<usize, String> {
    let mut count = 0;
    for entry in jwalk::WalkDir::new(path)
        .skip_hidden(false)
        .parallelism(parallelism)
    {
        entry.map_err(|error| error.to_string())?;
        count += 1;
    }
    Ok(count)
}

fn parallel_jwalk_strategy() -> jwalk::Parallelism {
    let threads = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    if threads > 1 {
        jwalk::Parallelism::RayonNewPool(threads)
    } else {
        jwalk::Parallelism::Serial
    }
}
