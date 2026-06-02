//! Prepared checker rows for same-turn agent feedback latency evidence.

use super::changed_path_cli::mark_changed_path;
use super::{
    row, MaterializedFixture, PerformanceEnvironment, PerformanceResultRow, RowMeasurement,
    ToolAvailability,
};
use crate::cli::check::PreparedStructureCheck;
use std::path::{Path, PathBuf};
use std::time::Instant;

const FIVE_CHANGED_PATH_COUNT: usize = 5;

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_prepared_full_check(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
) -> PerformanceResultRow {
    let measurement =
        RowMeasurement::new("assura-prepared-full-check", "assura-prepared-full-check");
    let prepared =
        match PreparedStructureCheck::load_for_path(Some(fixture.root.clone()), None, false) {
            Ok(prepared) => prepared,
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
                    Some(format!("failed to prepare checker: {error}")),
                    baseline_id,
                );
            }
        };

    let mut samples = Vec::with_capacity(iterations);
    let mut failure = None;
    for _ in 0..iterations {
        let started = Instant::now();
        match prepared.check_path(fixture.root.clone()) {
            Ok(report) if report.success => samples.push(started.elapsed().as_secs_f64() * 1000.0),
            Ok(report) => {
                failure = Some(format!(
                    "prepared full check reported {} violations",
                    report.violations.len()
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
        measurement,
        samples,
        failure,
        baseline_id,
    )
}

#[allow(clippy::too_many_arguments)]
pub(super) fn measure_prepared_five_changed_paths(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
) -> PerformanceResultRow {
    let measurement = RowMeasurement::new(
        "assura-prepared-five-changed-paths",
        "assura-prepared-five-changed-paths",
    );
    let prepared =
        match PreparedStructureCheck::load_for_path(Some(fixture.root.clone()), None, false) {
            Ok(prepared) => prepared,
            Err(error) => {
                return unavailable_prepared_row(
                    fixture,
                    timestamp,
                    commit_sha,
                    branch,
                    environment,
                    baseline_id,
                    ls_lint_status,
                    measurement,
                    format!("failed to prepare checker: {error}"),
                );
            }
        };
    let changed_paths = match changed_path_candidates(&fixture.root, FIVE_CHANGED_PATH_COUNT) {
        Ok(paths) => paths,
        Err(error) => {
            return unavailable_prepared_row(
                fixture,
                timestamp,
                commit_sha,
                branch,
                environment,
                baseline_id,
                ls_lint_status,
                measurement,
                error,
            );
        }
    };

    let mut samples = Vec::with_capacity(iterations);
    let mut failure = None;
    for _ in 0..iterations {
        let started = Instant::now();
        for path in &changed_paths {
            if let Err(error) = mark_changed_path(&fixture.root.join(path)) {
                failure = Some(error);
                break;
            }
            match prepared.check_changed_path(fixture.root.join(path)) {
                Ok(report) if report.success => {}
                Ok(report) => {
                    failure = Some(format!(
                        "prepared changed-path check for {} reported {} violations",
                        path.display(),
                        report.violations.len()
                    ));
                    break;
                }
                Err(error) => {
                    failure = Some(error.to_string());
                    break;
                }
            }
        }
        if failure.is_some() {
            break;
        }
        samples.push(started.elapsed().as_secs_f64() * 1000.0);
    }

    row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        measurement,
        samples,
        failure,
        baseline_id,
    )
}

#[allow(clippy::too_many_arguments)]
fn unavailable_prepared_row(
    fixture: &MaterializedFixture,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
    measurement: RowMeasurement<'_>,
    error: String,
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
        Some(error),
        baseline_id,
    )
}

fn changed_path_candidates(root: &Path, count: usize) -> Result<Vec<PathBuf>, String> {
    let mut candidates = Vec::new();
    let mut stack = vec![PathBuf::new()];
    while let Some(rel) = stack.pop() {
        let abs = root.join(&rel);
        let entries = std::fs::read_dir(&abs).map_err(|error| {
            format!("read changed-path candidate dir {}: {error}", abs.display())
        })?;
        let mut entries = entries.flatten().collect::<Vec<_>>();
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            let child_rel = if rel.as_os_str().is_empty() {
                PathBuf::from(&name)
            } else {
                rel.join(&name)
            };
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if is_ignored_changed_path_dir(&name_str) {
                    continue;
                }
                stack.push(child_rel);
            } else if file_type.is_file() && !is_ignored_changed_path_file(&name_str) {
                candidates.push(child_rel);
                if candidates.len() == count {
                    return Ok(candidates);
                }
            }
        }
    }

    Err(format!(
        "needed {count} changed-path candidates, found {}",
        candidates.len()
    ))
}

fn is_ignored_changed_path_dir(name: &str) -> bool {
    matches!(
        name,
        ".assura" | ".git" | "node_modules" | "dist" | "build" | "coverage" | "target"
    )
}

fn is_ignored_changed_path_file(name: &str) -> bool {
    matches!(name, ".ls-lint.yml" | "assura-check.status")
}
