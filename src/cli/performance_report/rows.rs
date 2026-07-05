//! Performance report row schema and row construction helpers.
// allow-reason: performance row factories keep measured dimensions explicit
// for benchmark auditability despite wide argument lists.

use super::{
    metadata::source_provenance_from_env, stats, MaterializedFixture, PerformanceEnvironment,
    ASSURA_VERSION, SCHEMA_VERSION,
};
use serde::Serialize;
use std::path::Path;

/// Chart-ready performance result row.
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceResultRow {
    /// Row schema version.
    pub schema_version: String,
    /// UTC timestamp for the measurement run.
    pub timestamp: String,
    /// Current git commit SHA when available.
    pub commit_sha: String,
    /// Current git branch when available.
    pub branch: String,
    /// Source lane commit when a clean snapshot was materialized from another worktree.
    pub source_commit_sha: Option<String>,
    /// Source lane branch when a clean snapshot was materialized from another worktree.
    pub source_branch: Option<String>,
    /// Stable patch identifier for the source-lane diff used to materialize a clean snapshot.
    pub source_patch_id: Option<String>,
    /// Operating system identifier reported by the Rust target.
    pub os: String,
    /// CPU architecture identifier reported by the Rust target.
    pub arch: String,
    /// Rust compiler version used to build or run Assura.
    pub rust_version: String,
    /// Node.js version available when installing the pinned LS-Lint package.
    pub node_version: String,
    /// npm version used to install the pinned LS-Lint package.
    pub npm_version: String,
    /// Assura package version or binary path used for the run.
    pub assura_version: String,
    /// LS-Lint version or package spec used for comparison.
    pub ls_lint_version: String,
    /// Stable fixture identifier.
    pub fixture_id: String,
    /// Pinned fixture source revision.
    pub fixture_source_revision: String,
    /// Fixture cohort such as real-repo-headline, realistic-equivalent, or synthetic-stress.
    pub fixture_cohort: String,
    /// Historical fixture cohort retained for compatibility with older data.
    pub legacy_fixture_cohort: String,
    /// Rule cohort exercised by the fixture.
    pub rule_cohort: String,
    /// Explicit row family used to separate headline CLI rows from diagnostics.
    pub row_family: String,
    /// Execution model represented by this row.
    pub validation_execution_mode: String,
    /// Whether this row is headline-eligible evidence or diagnostic-only.
    pub evidence_role: String,
    /// True when this row must not drive public headline comparisons.
    pub diagnostic: bool,
    /// Fixture acceptance class used by no-slower gates and public docs.
    pub fixture_acceptance: String,
    /// Fixture source type such as generated or external pinned repo.
    pub source_type: String,
    /// Files expected to be checked after configured ignores are applied.
    pub checked_file_count: usize,
    /// Files present in configured ignored paths.
    pub ignored_file_count: usize,
    /// Directories materialized in the fixture tree, excluding the fixture root.
    pub directory_count: usize,
    /// Count of LS-Lint-compatible rule entries represented by the fixture.
    pub rule_count: usize,
    /// Human-readable summary of the exercised rule surface.
    pub rule_surface_summary: String,
    /// Whether the fixture uses native LS-Lint parity behavior.
    pub native_ls_lint_parity: bool,
    /// Stable Assura config reference within the materialized fixture.
    pub assura_config_path: String,
    /// Stable LS-Lint config reference within the materialized fixture.
    pub ls_lint_config_path: String,
    /// How the Assura and LS-Lint configs were generated.
    pub config_generation_method: String,
    /// Shared config-pair identifier for auditability.
    pub shared_config_id: String,
    /// Expected Assura process exit status for this fixture.
    pub expected_assura_exit_status: i32,
    /// Expected LS-Lint process exit status for this fixture.
    pub expected_ls_lint_exit_status: i32,
    /// Assura binary profile for CLI subprocess rows, when applicable.
    pub assura_binary_profile: Option<String>,
    /// Assura binary path for CLI subprocess rows, when applicable.
    pub assura_binary_path: Option<String>,
    /// LS-Lint executable path for LS-Lint subprocess rows, when applicable.
    pub ls_lint_binary_path: Option<String>,
    /// LS-Lint execution mode for LS-Lint subprocess rows, when applicable.
    pub ls_lint_execution_mode: Option<String>,
    /// Tool measured by this row.
    pub tool_name: String,
    /// Median runtime in milliseconds, when measured.
    pub median_runtime_ms: Option<f64>,
    /// Nearest-rank p95 runtime in milliseconds, when measured.
    pub p95_runtime_ms: Option<f64>,
    /// Half of the native LS-Lint median for this fixture, when available.
    pub two_x_target_runtime_ms: Option<f64>,
    /// Median process-launch floor for this fixture, when available.
    pub process_floor_runtime_ms: Option<f64>,
    /// Process floor divided by the two-times-faster target.
    pub process_floor_to_two_x_target_ratio: Option<f64>,
    /// True when process launch alone is slower than the two-times-faster target.
    pub process_floor_blocks_two_x: Option<bool>,
    /// Median runtime for the smallest measured Assura-built Rust CLI process, when available.
    pub rust_cli_floor_runtime_ms: Option<f64>,
    /// Assura Rust CLI floor divided by the two-times-faster target.
    pub rust_cli_floor_to_two_x_target_ratio: Option<f64>,
    /// True when the Assura Rust CLI floor is slower than the two-times-faster target.
    pub rust_cli_floor_blocks_two_x: Option<bool>,
    /// This row's median runtime minus the measured process launch floor.
    pub runtime_above_process_floor_ms: Option<f64>,
    /// Assura CLI median minus process floor and Assura in-process validation.
    pub assura_cli_overhead_ms: Option<f64>,
    /// This row's median runtime divided by the two-times-faster target.
    pub runtime_to_two_x_target_ratio: Option<f64>,
    /// True when this row's median runtime is at or below the two-times-faster target.
    pub meets_two_x_target: Option<bool>,
    /// Machine-readable status for a two-times-faster Assura CLI claim.
    pub two_x_claim_status: Option<String>,
    /// Whether this row proves full-project success rather than scoped feedback.
    pub proves_whole_project_success: bool,
    /// Number of changed paths validated by each measured sample, when scoped.
    pub changed_path_count: Option<usize>,
    /// Goal-specific latency threshold for this row, when applicable.
    pub latency_threshold_ms: Option<f64>,
    /// Whether this row's p95 latency meets the goal-specific threshold.
    pub latency_threshold_met: Option<bool>,
    /// Checked native baseline median for the matching native row, when applicable.
    pub native_regression_baseline_median_ms: Option<f64>,
    /// Number of checked native report rows that contributed to this row's baseline.
    pub native_regression_baseline_report_count: Option<usize>,
    /// Total measured samples across checked native report rows that contributed to this baseline.
    pub native_regression_baseline_sample_count: Option<usize>,
    /// Calibrated native-regression threshold derived from the checked baseline row.
    pub native_regression_threshold_ms: Option<f64>,
    /// Current median minus the checked native baseline median, when applicable.
    pub native_regression_delta_ms: Option<f64>,
    /// Machine-readable native regression status versus the checked baseline row.
    pub native_regression_status: Option<String>,
    /// Distribution details for charting and review.
    pub distribution: RuntimeDistribution,
    /// Whether this tool run passed.
    pub success: bool,
    /// Pass/fail/skipped status.
    pub status: String,
    /// Explicit skip or failure detail.
    pub details: Option<String>,
    /// Baseline identifier used for longitudinal comparison.
    pub comparison_baseline_id: String,
}

/// Runtime distribution details for a result row.
#[derive(Debug, Clone, Serialize)]
pub struct RuntimeDistribution {
    /// Number of samples in this row.
    pub samples: usize,
    /// Nearest-rank p95 runtime in milliseconds.
    pub p95_ms: Option<f64>,
    /// Individual sample runtimes in milliseconds.
    pub samples_ms: Vec<f64>,
    /// Minimum sample runtime in milliseconds.
    pub min_ms: Option<f64>,
    /// Maximum sample runtime in milliseconds.
    pub max_ms: Option<f64>,
}

#[derive(Clone, Copy)]
pub(in crate::cli::performance_report) struct RowMeasurement<'a> {
    pub(in crate::cli::performance_report) tool_name: &'a str,
    pub(in crate::cli::performance_report) row_family: &'a str,
    pub(in crate::cli::performance_report) assura_binary_path: Option<&'a Path>,
    pub(in crate::cli::performance_report) assura_binary_profile: Option<&'a str>,
    pub(in crate::cli::performance_report) ls_lint_binary_path: Option<&'a Path>,
    pub(in crate::cli::performance_report) ls_lint_execution_mode: Option<&'a str>,
    pub(in crate::cli::performance_report) expected_assura_exit_status: Option<i32>,
    pub(in crate::cli::performance_report) expected_ls_lint_exit_status: Option<i32>,
}

impl<'a> RowMeasurement<'a> {
    pub(in crate::cli::performance_report) fn new(tool_name: &'a str, row_family: &'a str) -> Self {
        Self {
            tool_name,
            row_family,
            assura_binary_path: None,
            assura_binary_profile: None,
            ls_lint_binary_path: None,
            ls_lint_execution_mode: None,
            expected_assura_exit_status: None,
            expected_ls_lint_exit_status: None,
        }
    }

    pub(in crate::cli::performance_report) fn with_assura_binary(
        self,
        binary_path: &'a Path,
        binary_profile: Option<&'a str>,
    ) -> Self {
        Self {
            assura_binary_path: Some(binary_path),
            assura_binary_profile: binary_profile,
            ..self
        }
    }

    pub(in crate::cli::performance_report) fn with_ls_lint_binary(
        self,
        binary_path: &'a Path,
        execution_mode: Option<&'a str>,
    ) -> Self {
        Self {
            ls_lint_binary_path: Some(binary_path),
            ls_lint_execution_mode: execution_mode,
            ..self
        }
    }

    pub(in crate::cli::performance_report) fn with_expected_assura_exit_status(
        self,
        expected_status: i32,
    ) -> Self {
        Self {
            expected_assura_exit_status: Some(expected_status),
            ..self
        }
    }
}

// allow-reason: performance row factory keeps measured dimensions explicit for benchmark auditability.
#[allow(clippy::too_many_arguments)]
pub(in crate::cli::performance_report) fn row(
    fixture: &MaterializedFixture,
    timestamp: &str,
    commit_sha: &str,
    branch: &str,
    environment: &PerformanceEnvironment,
    ls_lint_version: &str,
    measurement: RowMeasurement<'_>,
    samples: Vec<f64>,
    failure: Option<String>,
    baseline_id: &str,
) -> PerformanceResultRow {
    let source_provenance = source_provenance_from_env();
    let distribution = stats::distribution(samples);
    let median_runtime_ms = stats::median(&distribution.samples_ms);
    let p95_runtime_ms = distribution.p95_ms;
    let latency_threshold_ms = latency_threshold_ms(measurement.row_family);
    let skipped = median_runtime_ms.is_none() && failure.is_some();
    let success = failure.is_none() && median_runtime_ms.is_some();
    let metadata = &fixture.metadata;
    let diagnostic = is_diagnostic_row(measurement.row_family, metadata.cohort);
    PerformanceResultRow {
        schema_version: SCHEMA_VERSION.to_string(),
        timestamp: timestamp.to_string(),
        commit_sha: commit_sha.to_string(),
        branch: branch.to_string(),
        source_commit_sha: source_provenance.source_commit_sha,
        source_branch: source_provenance.source_branch,
        source_patch_id: source_provenance.source_patch_id,
        os: environment.os.clone(),
        arch: environment.arch.clone(),
        rust_version: environment.rust_version.clone(),
        node_version: environment.node_version.clone(),
        npm_version: environment.npm_version.clone(),
        assura_version: ASSURA_VERSION.to_string(),
        ls_lint_version: ls_lint_version.to_string(),
        fixture_id: fixture.scenario.id.to_string(),
        fixture_source_revision: metadata.source_revision.clone(),
        fixture_cohort: metadata.cohort.to_string(),
        legacy_fixture_cohort: "stable-baseline".to_string(),
        rule_cohort: fixture.scenario.rule_cohort.to_string(),
        row_family: measurement.row_family.to_string(),
        validation_execution_mode: validation_execution_mode(measurement.row_family).to_string(),
        evidence_role: if diagnostic {
            "diagnostic"
        } else {
            "headline-candidate"
        }
        .to_string(),
        diagnostic,
        fixture_acceptance: fixture_acceptance(metadata.cohort, metadata.native_ls_lint_parity)
            .to_string(),
        source_type: metadata.source_type.to_string(),
        checked_file_count: metadata.checked_file_count,
        ignored_file_count: metadata.ignored_file_count,
        directory_count: metadata.directory_count,
        rule_count: metadata.rule_count,
        rule_surface_summary: metadata.rule_surface_summary.to_string(),
        native_ls_lint_parity: metadata.native_ls_lint_parity,
        assura_config_path: metadata.assura_config_path.to_string(),
        ls_lint_config_path: metadata.ls_lint_config_path.to_string(),
        config_generation_method: metadata.config_generation_method.to_string(),
        shared_config_id: metadata.shared_config_id.clone(),
        expected_assura_exit_status: measurement
            .expected_assura_exit_status
            .unwrap_or(metadata.expected_assura_exit_status),
        expected_ls_lint_exit_status: measurement
            .expected_ls_lint_exit_status
            .unwrap_or(metadata.expected_ls_lint_exit_status),
        assura_binary_profile: measurement.assura_binary_profile.map(str::to_string),
        assura_binary_path: measurement
            .assura_binary_path
            .map(|path| path.display().to_string()),
        ls_lint_binary_path: measurement
            .ls_lint_binary_path
            .map(|path| path.display().to_string()),
        ls_lint_execution_mode: measurement.ls_lint_execution_mode.map(str::to_string),
        tool_name: measurement.tool_name.to_string(),
        median_runtime_ms,
        p95_runtime_ms,
        two_x_target_runtime_ms: None,
        process_floor_runtime_ms: None,
        process_floor_to_two_x_target_ratio: None,
        process_floor_blocks_two_x: None,
        rust_cli_floor_runtime_ms: None,
        rust_cli_floor_to_two_x_target_ratio: None,
        rust_cli_floor_blocks_two_x: None,
        runtime_above_process_floor_ms: None,
        assura_cli_overhead_ms: None,
        runtime_to_two_x_target_ratio: None,
        meets_two_x_target: None,
        two_x_claim_status: None,
        proves_whole_project_success: proves_whole_project_success(measurement.row_family),
        changed_path_count: changed_path_count(measurement.row_family),
        latency_threshold_ms,
        latency_threshold_met: latency_threshold_ms
            .zip(p95_runtime_ms)
            .map(|(threshold, p95)| p95 <= threshold),
        native_regression_baseline_median_ms: None,
        native_regression_baseline_report_count: None,
        native_regression_baseline_sample_count: None,
        native_regression_threshold_ms: None,
        native_regression_delta_ms: None,
        native_regression_status: None,
        distribution,
        success,
        status: if success {
            "pass"
        } else if skipped {
            "skipped"
        } else {
            "fail"
        }
        .to_string(),
        details: failure,
        comparison_baseline_id: baseline_id.to_string(),
    }
}

pub(super) fn is_diagnostic_row(row_family: &str, fixture_cohort: &str) -> bool {
    !matches!(
        fixture_cohort,
        "real-repo-headline" | "realistic-equivalent"
    ) || row_family == "assura-in-process"
        || row_family == "assura-check-cached-cli"
        || row_family == "assura-check-compiled-cli"
        || row_family == "assura-check-hot-cli"
        || row_family == "assura-check-changed-path-cli"
        || row_family == "assura-check-dirty-project-cli"
        || row_family == "assura-check-dirty-project-session-cli"
        || row_family == "assura-check-dirty-project-socket"
        || row_family == "assura-prepared-full-check"
        || row_family == "assura-prepared-five-changed-paths"
        || row_family == "assura-check-status-cli"
        || row_family == "assura-rust-cli-floor"
        || row_family == "process-floor"
        || row_family.starts_with("assura:phase:")
        || row_family.starts_with("strategy:")
        || row_family.starts_with("traversal:")
}

fn fixture_acceptance(fixture_cohort: &str, native_ls_lint_parity: bool) -> &'static str {
    if native_ls_lint_parity
        && matches!(
            fixture_cohort,
            "realistic-equivalent" | "real-repo-headline"
        )
    {
        "accepted-ls-lint-equivalent"
    } else if native_ls_lint_parity {
        "diagnostic"
    } else {
        "assura-native-diagnostic"
    }
}

fn proves_whole_project_success(row_family: &str) -> bool {
    !matches!(
        row_family,
        "assura-check-changed-path-cli"
            | "assura-prepared-five-changed-paths"
            | "assura-check-dirty-project-socket"
    )
}

fn changed_path_count(row_family: &str) -> Option<usize> {
    match row_family {
        "assura-prepared-five-changed-paths" => Some(5),
        "assura-check-changed-path-cli"
        | "assura-check-dirty-project-cli"
        | "assura-check-dirty-project-session-cli"
        | "assura-check-dirty-project-socket" => Some(1),
        _ => None,
    }
}

fn latency_threshold_ms(row_family: &str) -> Option<f64> {
    match row_family {
        "assura-prepared-full-check" => Some(250.0),
        "assura-prepared-five-changed-paths" => Some(100.0),
        _ => None,
    }
}

pub(super) fn validation_execution_mode(row_family: &str) -> &'static str {
    match row_family {
        "assura-cli" | "assura-check-cli" | "ls-lint-cli" => "cold-cli",
        "assura-check-cached-cli" => "warm-cache-cli",
        "assura-check-compiled-cli" => "precompiled-config-cli",
        "assura-check-hot-cli" => "hot-daemon-cli",
        "assura-check-changed-path-cli" => "hot-daemon-changed-path-cli",
        "assura-check-dirty-project-cli" => "hot-daemon-dirty-project-cli",
        "assura-check-dirty-project-session-cli" => "hot-daemon-dirty-project-session-cli",
        "assura-check-dirty-project-socket" => "hot-daemon-dirty-project-socket",
        "assura-prepared-full-check" => "prepared-full-project-check",
        "assura-prepared-five-changed-paths" => "prepared-scoped-changed-paths",
        "assura-check-status-cli" => "status-file-cli",
        "assura-rust-cli-floor" => "rust-cli-floor",
        "assura-in-process" => "in-process",
        "process-floor" => "process-floor",
        row if row.starts_with("assura:phase:") => "phase-timing",
        row if row.starts_with("traversal:") => "traversal-only",
        row if row.starts_with("strategy:") => "diagnostic-strategy-cli",
        "native:content-check-cli" => "native-cold-check-cli",
        "native:content-collections-cli"
        | "native:content-instances-cli"
        | "native:content-show-cli"
        | "native:content-expand-cli"
        | "native:content-search-cli"
        | "native:content-missing-relations-cli"
        | "native:content-references-cli"
        | "native:agent-query-keyword-search-cli"
        | "native:agent-query-missing-relations-cli" => "native-query-cli",
        "native:context-pack-cli" => "native-context-pack-cli",
        "native:markdown-safe-fix-dry-run-cli" => "native-markdown-safe-fix-cli",
        "native:session-agent-context-cli" => "native-session-query-cli",
        "native:daemon-status-cli" => "native-daemon-query-cli",
        row if row.starts_with("native:phase:") => "native-phase-timing",
        _ => "diagnostic",
    }
}
