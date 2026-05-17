//! Performance comparison report generation for CI artifacts and docs data.

use crate::cli::args::PerformanceReportFormat;
use crate::cli::check::run_structure_check_with_timings;
use crate::cli::ExitCode;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

mod assura_cli;
mod environment;
mod fixture_io;
mod fixture_metadata;
mod fixtures;
mod io;
mod ls_lint;
mod metadata;
mod phases;
mod rows;
mod stats;
mod traversal;

use assura_cli::{measure_assura_cli, measure_assura_strategy, prepare_assura_cli};
use environment::{collect_environment, PerformanceEnvironment};
pub(in crate::cli::performance_report) use fixtures::MaterializedFixture;
use fixtures::{materialize_fixture, scenarios};
use io::{append_history, render_jsonl, write_text, write_website_data};
use ls_lint::{prepare_ls_lint, PreparedLsLint};
use metadata::{git_value, utc_timestamp};
use phases::AssuraPhaseSamples;
pub(in crate::cli::performance_report) use rows::{row, RowMeasurement};
pub use rows::{PerformanceResultRow, RuntimeDistribution};
use traversal::{
    measure_parallel_jwalk_traversal, measure_serial_jwalk_traversal, measure_walkdir_traversal,
};

const SCHEMA_VERSION: &str = "assura.performance.v1";
const ASSURA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Full performance report emitted by `assura performance-report`.
#[derive(Debug, Serialize)]
pub struct PerformanceReport {
    /// Schema version for the report envelope.
    pub schema_version: String,
    /// UTC timestamp for the report run.
    pub timestamp: String,
    /// Current git commit SHA when available.
    pub commit_sha: String,
    /// Current git branch when available.
    pub branch: String,
    /// Environment and toolchain metadata for this measurement run.
    pub environment: PerformanceEnvironment,
    /// Baseline identifier used for longitudinal comparison.
    pub comparison_baseline_id: String,
    /// Number of measured iterations requested per tool and fixture.
    pub iterations: usize,
    /// Exact LS-Lint package spec requested for comparison.
    pub ls_lint_package: String,
    /// LS-Lint version output, or an explicit availability blocker.
    pub ls_lint_status: ToolAvailability,
    /// Per-tool, per-fixture result rows.
    pub results: Vec<PerformanceResultRow>,
}

/// Tool availability metadata included when a comparator cannot run.
#[derive(Debug, Clone, Serialize)]
pub struct ToolAvailability {
    /// Whether the tool was available for this run.
    pub available: bool,
    /// Version string reported by the tool.
    pub version: Option<String>,
    /// Exact blocker text when unavailable.
    pub blocker: Option<String>,
}

/// Generate and write a performance report.
pub async fn performance_report_command(
    output: Option<PathBuf>,
    history: Option<PathBuf>,
    website_dir: Option<PathBuf>,
    iterations: usize,
    baseline_id: String,
    format: PerformanceReportFormat,
    ls_lint_package: String,
) -> ExitCode {
    match generate_report(iterations.max(1), baseline_id, ls_lint_package) {
        Ok(report) => {
            let rendered = match format {
                PerformanceReportFormat::Json => match serde_json::to_string(&report) {
                    Ok(rendered) => rendered,
                    Err(error) => {
                        eprintln!("Error: failed to serialize performance report: {error}");
                        return ExitCode::RuntimeError;
                    }
                },
                PerformanceReportFormat::Jsonl => render_jsonl(&report.results),
            };

            if let Some(output) = output {
                if let Err(error) = write_text(&output, &rendered) {
                    eprintln!("Error: failed to write {}: {error}", output.display());
                    return ExitCode::RuntimeError;
                }
            } else {
                println!("{rendered}");
            }

            if let Some(history) = history.as_ref() {
                if let Err(error) = append_history(history, &report.results) {
                    eprintln!("Error: failed to append {}: {error}", history.display());
                    return ExitCode::RuntimeError;
                }
            }

            if let Some(website_dir) = website_dir {
                if let Err(error) = write_website_data(&website_dir, &report, history.as_deref()) {
                    eprintln!(
                        "Error: failed to write website data under {}: {error}",
                        website_dir.display()
                    );
                    return ExitCode::RuntimeError;
                }
            }

            ExitCode::Success
        }
        Err(error) => {
            eprintln!("Error: failed to generate performance report: {error}");
            ExitCode::RuntimeError
        }
    }
}

fn generate_report(
    iterations: usize,
    baseline_id: String,
    ls_lint_package: String,
) -> Result<PerformanceReport, String> {
    let timestamp = utc_timestamp();
    let commit_sha = git_value(["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let branch = git_value(["branch", "--show-current"]).unwrap_or_else(|| "unknown".to_string());
    let environment = collect_environment();
    let assura_cli = prepare_assura_cli();
    let ls_lint = prepare_ls_lint(&ls_lint_package);
    let ls_lint_status = ls_lint.status.clone();
    let mut results = Vec::new();

    for scenario in scenarios() {
        let fixture = materialize_fixture(scenario)?;
        results.push(measure_assura_cli(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
            &assura_cli,
        ));
        results.push(measure_assura_strategy(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
            &assura_cli,
            "strategy:jwalk-serial-cli",
            None,
        ));
        results.push(measure_assura_strategy(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
            &assura_cli,
            "strategy:walkdir-cli",
            Some("walkdir"),
        ));
        results.push(measure_assura_strategy(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
            &assura_cli,
            "strategy:jwalk-parallel-cli",
            Some("parallel-jwalk"),
        ));
        results.extend(measure_assura(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
        ));
        results.push(measure_ls_lint(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint,
        ));
        results.push(measure_walkdir_traversal(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
        ));
        results.push(measure_serial_jwalk_traversal(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
        ));
        results.push(measure_parallel_jwalk_traversal(
            &fixture,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
        ));
        let _ = fs::remove_dir_all(&fixture.root);
    }

    Ok(PerformanceReport {
        schema_version: SCHEMA_VERSION.to_string(),
        timestamp,
        commit_sha,
        branch,
        environment,
        comparison_baseline_id: baseline_id,
        iterations,
        ls_lint_package,
        ls_lint_status,
        results,
    })
}

#[allow(clippy::too_many_arguments)]
fn measure_assura(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint_status: &ToolAvailability,
) -> Vec<PerformanceResultRow> {
    let mut samples = Vec::with_capacity(iterations);
    let mut phase_samples = AssuraPhaseSamples::with_capacity(iterations);
    let mut failure = None;
    for _ in 0..iterations {
        match run_structure_check_with_timings(Some(fixture.root.clone()), None, false) {
            Ok((report, timings)) if report.success => {
                samples.push(timings.total_ms);
                phase_samples.push(timings);
            }
            Ok(report) => {
                failure = Some(format!(
                    "Assura reported {} violations",
                    report.0.violations.len()
                ));
                break;
            }
            Err(error) => {
                failure = Some(error.to_string());
                break;
            }
        }
    }

    let mut rows = vec![row(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status.version.as_deref().unwrap_or("unavailable"),
        RowMeasurement::new("assura-in-process", "assura-in-process"),
        samples,
        failure.clone(),
        baseline_id,
    )];
    rows.extend(phase_samples.into_rows(
        fixture,
        timestamp,
        commit_sha,
        branch,
        environment,
        ls_lint_status,
        failure.as_deref(),
        baseline_id,
    ));
    rows
}

#[allow(clippy::too_many_arguments)]
fn measure_ls_lint(
    fixture: &MaterializedFixture,
    iterations: usize,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    baseline_id: &str,
    ls_lint: &PreparedLsLint,
) -> PerformanceResultRow {
    let ls_lint_status = &ls_lint.status;
    if !ls_lint_status.available {
        return row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            ls_lint_status.version.as_deref().unwrap_or("unavailable"),
            RowMeasurement::new("ls-lint-cli", "ls-lint-cli"),
            Vec::new(),
            Some(format!(
                "skipped because LS-Lint is unavailable: {}",
                ls_lint_status
                    .blocker
                    .as_deref()
                    .unwrap_or("unknown blocker")
            )),
            baseline_id,
        );
    }
    let Some(binary_path) = ls_lint.binary_path.as_ref() else {
        return row(
            fixture,
            timestamp,
            commit_sha,
            branch,
            environment,
            ls_lint_status.version.as_deref().unwrap_or("unavailable"),
            RowMeasurement::new("ls-lint-cli", "ls-lint-cli"),
            Vec::new(),
            Some("skipped because LS-Lint binary path was not prepared".to_string()),
            baseline_id,
        );
    };

    let mut samples = Vec::with_capacity(iterations);
    let mut failure = None;
    let expected_status = fixture.metadata.expected_ls_lint_exit_status;
    for _ in 0..iterations {
        let started = Instant::now();
        let output = Command::new(binary_path)
            .current_dir(&fixture.root)
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
        RowMeasurement::new("ls-lint-cli", "ls-lint-cli"),
        samples,
        failure,
        baseline_id,
    )
}
