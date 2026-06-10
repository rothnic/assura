//! Performance comparison report generation for CI artifacts and docs data.
// allow-reason: performance row factories keep measured dimensions explicit
// for benchmark auditability despite wide argument lists.

use crate::cli::args::PerformanceReportFormat;
use crate::cli::check::run_structure_check_with_timings;
use crate::cli::ExitCode;
use serde::Serialize;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;

mod assura_cli;
mod binary_profile;
mod cached_cli;
mod changed_path_cli;
mod check_sources;
mod claim_summary;
mod compiled_cli;
mod counterexample_fixtures;
mod dirty_project_socket;
mod environment;
mod external_fixture_catalog;
mod external_fixture_scenarios;
mod external_fixtures;
mod feasibility;
mod fixture_io;
mod fixture_metadata;
mod fixture_rows;
#[cfg(test)]
mod fixture_tests;
mod fixtures;
mod hot_cli;
mod io;
mod ls_lint;
mod metadata;
mod monorepo_policy;
mod phases;
mod prepared_rows;
mod process_floor;
mod real_project_feedback_fixture;
mod realistic_fixtures;
mod rows;
#[cfg(test)]
mod rows_tests;
mod session_cli;
mod stats;
mod traversal;

use assura_cli::{
    prepare_assura_check_cli, prepare_assura_check_client, prepare_assura_check_compile_config,
    prepare_assura_check_compiled, prepare_assura_check_noop, prepare_assura_check_session,
    prepare_assura_check_status, prepare_assura_checkd, prepare_assura_cli,
};
use claim_summary::{summarize_headline_claim, summarize_warm_claim, PerformanceClaimSummary};
use environment::{collect_environment, PerformanceEnvironment};
use feasibility::annotate_two_x_feasibility;
use fixture_rows::measure_scenario_rows;
pub(in crate::cli::performance_report) use fixtures::MaterializedFixture;
use fixtures::{materialize_fixture, scenarios};
use io::{append_history, render_jsonl, write_text, write_website_data};
use ls_lint::{prepare_ls_lint, PreparedLsLint};
use metadata::{git_value, utc_timestamp};
use phases::AssuraPhaseSamples;
pub(in crate::cli::performance_report) use rows::{row, RowMeasurement};
pub use rows::{PerformanceResultRow, RuntimeDistribution};

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
    /// Whether the source worktree had uncommitted changes during the run.
    pub source_worktree_dirty: bool,
    /// Environment and toolchain metadata for this measurement run.
    pub environment: PerformanceEnvironment,
    /// Baseline identifier used for longitudinal comparison.
    pub comparison_baseline_id: String,
    /// Command line used to generate this report.
    pub command_line: String,
    /// Number of measured iterations requested per tool and fixture.
    pub iterations: usize,
    /// Exact LS-Lint package spec requested for comparison.
    pub ls_lint_package: String,
    /// LS-Lint version output, or an explicit availability blocker.
    pub ls_lint_status: ToolAvailability,
    /// Machine-readable verdict for the public headline performance claim.
    pub claim_summary: PerformanceClaimSummary,
    /// Machine-readable verdict for the warm editor-session dirty-project row.
    pub warm_claim_summary: PerformanceClaimSummary,
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

/// Options passed from the CLI into performance report generation.
pub struct PerformanceReportCommandOptions {
    /// Output path for the current run report, or stdout when omitted.
    pub output: Option<PathBuf>,
    /// JSONL history path to append result rows to.
    pub history: Option<PathBuf>,
    /// Website public data directory to refresh.
    pub website_dir: Option<PathBuf>,
    /// Number of measured iterations per tool and fixture.
    pub iterations: usize,
    /// Baseline identifier used for longitudinal comparison.
    pub baseline_id: String,
    /// Output format for the current report.
    pub format: PerformanceReportFormat,
    /// LS-Lint package spec used for comparison.
    pub ls_lint_package: String,
    /// Whether to include opt-in pinned external Git fixtures.
    pub include_external_fixtures: bool,
}

/// Generate and write a performance report.
pub async fn performance_report_command(options: PerformanceReportCommandOptions) -> ExitCode {
    let command_line = std::env::args().collect::<Vec<_>>().join(" ");
    match generate_report(
        options.iterations.max(1),
        options.baseline_id,
        options.ls_lint_package,
        options.include_external_fixtures,
        command_line,
    ) {
        Ok(report) => {
            let rendered = match options.format {
                PerformanceReportFormat::Json => match serde_json::to_string(&report) {
                    Ok(rendered) => rendered,
                    Err(error) => {
                        eprintln!("Error: failed to serialize performance report: {error}");
                        return ExitCode::RuntimeError;
                    }
                },
                PerformanceReportFormat::Jsonl => render_jsonl(&report.results),
            };

            if let Some(output) = options.output {
                if let Err(error) = write_text(&output, &rendered) {
                    eprintln!("Error: failed to write {}: {error}", output.display());
                    return ExitCode::RuntimeError;
                }
            } else {
                println!("{rendered}");
            }

            if let Some(history) = options.history.as_ref() {
                if let Err(error) = append_history(history, &report.results) {
                    eprintln!("Error: failed to append {}: {error}", history.display());
                    return ExitCode::RuntimeError;
                }
            }

            if let Some(website_dir) = options.website_dir {
                if let Err(error) =
                    write_website_data(&website_dir, &report, options.history.as_deref())
                {
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
    include_external_fixtures: bool,
    command_line: String,
) -> Result<PerformanceReport, String> {
    let timestamp = utc_timestamp();
    let commit_sha = git_value(["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let branch = git_value(["branch", "--show-current"]).unwrap_or_else(|| "unknown".to_string());
    let source_worktree_dirty =
        git_value(["status", "--porcelain"]).is_some_and(|status| !status.trim().is_empty());
    let environment = collect_environment();
    let assura_cli = prepare_assura_cli();
    let assura_check_cli = prepare_assura_check_cli();
    let assura_check_compiled = prepare_assura_check_compiled();
    let assura_check_compile_config = prepare_assura_check_compile_config();
    let assura_check_client = prepare_assura_check_client();
    let assura_check_session = prepare_assura_check_session();
    let assura_check_status = prepare_assura_check_status();
    let assura_check_noop = prepare_assura_check_noop();
    let assura_checkd = prepare_assura_checkd();
    let ls_lint = prepare_ls_lint(&ls_lint_package);
    let ls_lint_status = ls_lint.status.clone();
    let mut results = Vec::new();

    for scenario in scenarios(include_external_fixtures) {
        results.extend(measure_scenario_rows(
            scenario,
            iterations,
            &timestamp,
            &commit_sha,
            &branch,
            &environment,
            &baseline_id,
            &ls_lint_status,
            &assura_cli,
            &assura_check_cli,
            &assura_check_compiled,
            &assura_check_compile_config,
            &assura_check_client,
            &assura_check_session,
            &assura_check_status,
            &assura_check_noop,
            &assura_checkd,
            &ls_lint,
        )?);
    }
    annotate_two_x_feasibility(&mut results);
    let claim_summary = summarize_headline_claim(&results, iterations);
    let warm_claim_summary = summarize_warm_claim(&results, iterations);

    Ok(PerformanceReport {
        schema_version: SCHEMA_VERSION.to_string(),
        timestamp,
        commit_sha,
        branch,
        source_worktree_dirty,
        environment,
        comparison_baseline_id: baseline_id,
        command_line,
        iterations,
        ls_lint_package,
        ls_lint_status,
        claim_summary,
        warm_claim_summary,
        results,
    })
}

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
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

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
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
            RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli"),
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
            RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli"),
            Vec::new(),
            Some("skipped because LS-Lint binary path was not prepared".to_string()),
            baseline_id,
        );
    };

    let mut samples = Vec::with_capacity(iterations);
    let mut failure = None;
    let expected_status = fixture.metadata.expected_ls_lint_exit_status;
    for _ in 0..iterations {
        let mut command = Command::new(binary_path);
        command
            .current_dir(&fixture.root)
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let started = Instant::now();
        let status = command.status();
        match status {
            Ok(status) if status.code() == Some(expected_status) => {
                samples.push(started.elapsed().as_secs_f64() * 1000.0);
            }
            Ok(status) => {
                failure = Some(format!(
                    "expected exit {expected_status}, got {:?}",
                    status.code()
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
        RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli")
            .with_ls_lint_binary(binary_path, ls_lint.execution_mode),
        samples,
        failure,
        baseline_id,
    )
}
