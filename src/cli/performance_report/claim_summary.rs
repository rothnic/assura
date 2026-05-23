//! Headline performance-claim summary derived from measured rows.

use super::PerformanceResultRow;
use serde::Serialize;
use std::collections::BTreeSet;

const HEADLINE_COHORT: &str = "realistic-equivalent";
const ASSURA_HEADLINE_ROW: &str = "assura-cli";
const ASSURA_WARM_ROW: &str = "assura-check-dirty-project-session-cli";
const LS_LINT_HEADLINE_ROW: &str = "ls-lint-cli";
const MIN_COMPLETION_ITERATIONS: usize = 3;

/// Machine-readable verdict for the public Assura versus LS-Lint claim.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PerformanceClaimSummary {
    /// Fixture cohort used for the headline comparison.
    pub fixture_cohort: String,
    /// Assura row family accepted as headline product evidence.
    pub assura_row_family: String,
    /// Native LS-Lint row family used as the comparison baseline.
    pub ls_lint_row_family: String,
    /// Number of fixture cases in the headline cohort.
    pub fixture_count: usize,
    /// Number of headline fixtures where both rows were measured successfully.
    pub measured_fixture_count: usize,
    /// Number of measured fixtures where Assura is faster than native LS-Lint.
    pub assura_faster_count: usize,
    /// Number of measured fixtures where Assura meets the 2x target.
    pub two_x_pass_count: usize,
    /// Number of measured fixtures where Assura misses the 2x target.
    pub two_x_fail_count: usize,
    /// Misses where generic process launch alone exceeds the 2x target.
    pub blocked_by_process_floor_count: usize,
    /// Misses where the smallest measured Assura Rust CLI exceeds the 2x target.
    pub blocked_by_rust_cli_floor_count: usize,
    /// Misses that are not explained by the measured floor rows.
    pub plain_miss_count: usize,
    /// Sum of headline Assura medians across measured fixtures.
    pub total_assura_runtime_ms: Option<f64>,
    /// Sum of native LS-Lint medians across measured fixtures.
    pub total_ls_lint_runtime_ms: Option<f64>,
    /// Native LS-Lint total divided by Assura total when both are available.
    pub aggregate_speedup_ratio: Option<f64>,
    /// Whether every headline fixture has successful Assura and LS-Lint rows.
    pub all_headline_rows_available: bool,
    /// Whether every measured headline fixture satisfies the 2x target.
    pub all_two_x_targets_met: bool,
    /// Minimum measured iterations required before a complete claim is accepted.
    pub minimum_completion_iterations: usize,
    /// Iteration count used by this report.
    pub measured_iterations: usize,
    /// Whether this report has enough samples to support a completion verdict.
    pub sufficient_completion_iterations: bool,
    /// Human-safe completion status for the universal 2x claim.
    pub two_x_claim_verdict: String,
}

pub(super) fn summarize_headline_claim(
    rows: &[PerformanceResultRow],
    measured_iterations: usize,
) -> PerformanceClaimSummary {
    summarize_claim(
        rows,
        measured_iterations,
        HEADLINE_COHORT,
        ASSURA_HEADLINE_ROW,
        LS_LINT_HEADLINE_ROW,
    )
}

pub(super) fn summarize_warm_claim(
    rows: &[PerformanceResultRow],
    measured_iterations: usize,
) -> PerformanceClaimSummary {
    summarize_claim(
        rows,
        measured_iterations,
        HEADLINE_COHORT,
        ASSURA_WARM_ROW,
        LS_LINT_HEADLINE_ROW,
    )
}

fn summarize_claim(
    rows: &[PerformanceResultRow],
    measured_iterations: usize,
    fixture_cohort: &str,
    assura_row_family: &str,
    ls_lint_row_family: &str,
) -> PerformanceClaimSummary {
    let fixture_ids = rows
        .iter()
        .filter(|row| row.fixture_cohort == fixture_cohort)
        .map(|row| row.fixture_id.as_str())
        .collect::<BTreeSet<_>>();

    let mut measured_fixture_count = 0;
    let mut assura_faster_count = 0;
    let mut two_x_pass_count = 0;
    let mut two_x_fail_count = 0;
    let mut blocked_by_process_floor_count = 0;
    let mut blocked_by_rust_cli_floor_count = 0;
    let mut plain_miss_count = 0;
    let mut total_assura_runtime_ms = 0.0;
    let mut total_ls_lint_runtime_ms = 0.0;

    for fixture_id in &fixture_ids {
        let assura = find_row(rows, fixture_id, fixture_cohort, assura_row_family);
        let ls_lint = find_row(rows, fixture_id, fixture_cohort, ls_lint_row_family);
        let Some((assura, ls_lint)) = assura.zip(ls_lint) else {
            continue;
        };
        let Some(assura_ms) = assura.median_runtime_ms else {
            continue;
        };
        let Some(ls_lint_ms) = ls_lint.median_runtime_ms else {
            continue;
        };

        measured_fixture_count += 1;
        total_assura_runtime_ms += assura_ms;
        total_ls_lint_runtime_ms += ls_lint_ms;
        if assura_ms < ls_lint_ms {
            assura_faster_count += 1;
        }

        match assura.meets_two_x_target {
            Some(true) => two_x_pass_count += 1,
            Some(false) => {
                two_x_fail_count += 1;
                match assura.two_x_claim_status.as_deref() {
                    Some("blocked-by-process-floor") => blocked_by_process_floor_count += 1,
                    Some("blocked-by-rust-cli-floor") => blocked_by_rust_cli_floor_count += 1,
                    _ => plain_miss_count += 1,
                }
            }
            None => {}
        }
    }

    let all_headline_rows_available =
        measured_fixture_count == fixture_ids.len() && measured_fixture_count > 0;
    let all_two_x_targets_met =
        all_headline_rows_available && two_x_pass_count == measured_fixture_count;
    let aggregate_speedup_ratio = (total_assura_runtime_ms > 0.0 && total_ls_lint_runtime_ms > 0.0)
        .then_some(total_ls_lint_runtime_ms / total_assura_runtime_ms);
    let sufficient_completion_iterations = measured_iterations >= MIN_COMPLETION_ITERATIONS;

    PerformanceClaimSummary {
        fixture_cohort: fixture_cohort.to_string(),
        assura_row_family: assura_row_family.to_string(),
        ls_lint_row_family: ls_lint_row_family.to_string(),
        fixture_count: fixture_ids.len(),
        measured_fixture_count,
        assura_faster_count,
        two_x_pass_count,
        two_x_fail_count,
        blocked_by_process_floor_count,
        blocked_by_rust_cli_floor_count,
        plain_miss_count,
        total_assura_runtime_ms: all_headline_rows_available.then_some(total_assura_runtime_ms),
        total_ls_lint_runtime_ms: all_headline_rows_available.then_some(total_ls_lint_runtime_ms),
        aggregate_speedup_ratio,
        all_headline_rows_available,
        all_two_x_targets_met,
        minimum_completion_iterations: MIN_COMPLETION_ITERATIONS,
        measured_iterations,
        sufficient_completion_iterations,
        two_x_claim_verdict: if !sufficient_completion_iterations {
            "not-complete-low-sample"
        } else if all_two_x_targets_met {
            "complete"
        } else {
            "not-complete"
        }
        .to_string(),
    }
}

fn find_row<'a>(
    rows: &'a [PerformanceResultRow],
    fixture_id: &str,
    fixture_cohort: &str,
    row_family: &str,
) -> Option<&'a PerformanceResultRow> {
    rows.iter().find(|row| {
        row.fixture_id == fixture_id
            && row.fixture_cohort == fixture_cohort
            && row.row_family == row_family
            && row.status == "pass"
    })
}

#[cfg(test)]
mod tests {
    use super::{summarize_headline_claim, summarize_warm_claim};
    use crate::cli::performance_report::feasibility::annotate_two_x_feasibility;
    use crate::cli::performance_report::{
        materialize_fixture, row, scenarios, PerformanceEnvironment, RowMeasurement,
    };

    fn test_environment() -> PerformanceEnvironment {
        PerformanceEnvironment {
            os: "test-os".to_string(),
            arch: "test-arch".to_string(),
            rust_version: "rustc test".to_string(),
            node_version: "node test".to_string(),
            npm_version: "npm test".to_string(),
        }
    }

    #[test]
    fn headline_summary_requires_every_fixture_to_meet_two_x() {
        let scenario = scenarios(false)
            .into_iter()
            .find(|scenario| scenario.id == "simple_library")
            .unwrap();
        let fixture = materialize_fixture(scenario).unwrap();
        let mut rows = vec![
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("process-floor", "process-floor"),
                vec![2.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-check-status-cli", "assura-check-status-cli"),
                vec![2.8],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli"),
                vec![5.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-cli", "assura-cli"),
                vec![3.0],
                None,
                "baseline",
            ),
        ];

        annotate_two_x_feasibility(&mut rows);
        let summary = summarize_headline_claim(&rows, 5);

        assert_eq!(summary.fixture_count, 1);
        assert_eq!(summary.measured_fixture_count, 1);
        assert_eq!(summary.assura_faster_count, 1);
        assert_eq!(summary.two_x_pass_count, 0);
        assert_eq!(summary.two_x_fail_count, 1);
        assert_eq!(summary.blocked_by_rust_cli_floor_count, 1);
        assert!(!summary.all_two_x_targets_met);
        assert_eq!(summary.minimum_completion_iterations, 3);
        assert_eq!(summary.measured_iterations, 5);
        assert!(summary.sufficient_completion_iterations);
        assert_eq!(summary.two_x_claim_verdict, "not-complete");

        let _ = std::fs::remove_dir_all(&fixture.root);
    }

    #[test]
    fn warm_summary_uses_session_row_without_changing_headline_row() {
        let scenario = scenarios(false)
            .into_iter()
            .find(|scenario| scenario.id == "simple_library")
            .unwrap();
        let fixture = materialize_fixture(scenario).unwrap();
        let mut rows = vec![
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("process-floor", "process-floor"),
                vec![1.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-rust-cli-floor", "assura-rust-cli-floor"),
                vec![1.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli"),
                vec![10.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-cli", "assura-cli"),
                vec![8.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new(
                    "assura-check-dirty-project-session-cli",
                    "assura-check-dirty-project-session-cli",
                ),
                vec![4.0],
                None,
                "baseline",
            ),
        ];

        annotate_two_x_feasibility(&mut rows);
        let headline = summarize_headline_claim(&rows, 5);
        let warm = summarize_warm_claim(&rows, 5);

        assert_eq!(headline.assura_row_family, "assura-cli");
        assert_eq!(headline.two_x_pass_count, 0);
        assert_eq!(headline.two_x_claim_verdict, "not-complete");
        assert_eq!(
            warm.assura_row_family,
            "assura-check-dirty-project-session-cli"
        );
        assert_eq!(warm.two_x_pass_count, 1);
        assert_eq!(warm.two_x_claim_verdict, "complete");

        let _ = std::fs::remove_dir_all(&fixture.root);
    }

    #[test]
    fn headline_summary_rejects_low_sample_completion() {
        let scenario = scenarios(false)
            .into_iter()
            .find(|scenario| scenario.id == "simple_library")
            .unwrap();
        let fixture = materialize_fixture(scenario).unwrap();
        let mut rows = vec![
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("process-floor", "process-floor"),
                vec![0.1],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-check-status-cli", "assura-check-status-cli"),
                vec![0.1],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli"),
                vec![10.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-19T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-cli", "assura-cli"),
                vec![4.0],
                None,
                "baseline",
            ),
        ];

        annotate_two_x_feasibility(&mut rows);
        let summary = summarize_headline_claim(&rows, 1);

        assert!(summary.all_two_x_targets_met);
        assert_eq!(summary.two_x_pass_count, 1);
        assert!(!summary.sufficient_completion_iterations);
        assert_eq!(summary.two_x_claim_verdict, "not-complete-low-sample");

        let _ = std::fs::remove_dir_all(&fixture.root);
    }
}
