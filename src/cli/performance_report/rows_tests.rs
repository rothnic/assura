//! Tests for performance report row metadata and evidence labeling.

use super::rows::{is_diagnostic_row, row, validation_execution_mode, RowMeasurement};
use super::{materialize_fixture, scenarios, PerformanceEnvironment};
use std::fs;

fn test_environment() -> PerformanceEnvironment {
    PerformanceEnvironment {
        os: "test-os".to_string(),
        arch: "test-arch".to_string(),
        cpu_model: "test-cpu".to_string(),
        logical_cpu_count: 8,
        total_memory_bytes: Some(16 * 1024 * 1024 * 1024),
        rust_version: "rustc test".to_string(),
        node_version: "node test".to_string(),
        npm_version: "npm test".to_string(),
    }
}

#[test]
fn cli_row_carries_fixture_metadata_and_headline_label() {
    let scenario = scenarios(false)
        .into_iter()
        .find(|scenario| scenario.id == "simple_library")
        .unwrap();
    let fixture = materialize_fixture(scenario).unwrap();
    let binary_path = fixture.root.join("target/debug/assura");

    let result = row(
        &fixture,
        "2026-05-17T00:00:00Z",
        "commit",
        "branch",
        &test_environment(),
        "ls-lint v2.3.0",
        RowMeasurement::new("assura-cli", "assura-cli")
            .with_assura_binary(&binary_path, Some("debug")),
        vec![3.0, 1.0, 2.0],
        None,
        "baseline",
    );

    assert_eq!(result.tool_name, "assura-cli");
    assert_eq!(result.median_runtime_ms, Some(2.0));
    assert_eq!(result.p95_runtime_ms, Some(3.0));
    assert_eq!(result.distribution.p95_ms, Some(3.0));
    assert_eq!(result.row_family, "assura-cli");
    assert_eq!(result.validation_execution_mode, "cold-cli");
    assert_eq!(result.evidence_role, "headline-candidate");
    assert!(!result.diagnostic);
    assert_eq!(result.fixture_cohort, "realistic-equivalent");
    assert_eq!(result.legacy_fixture_cohort, "stable-baseline");
    assert_eq!(result.source_type, "generated");
    assert!(result.checked_file_count > 0);
    assert!(result.ignored_file_count > 0);
    assert!(result.directory_count > 0);
    assert!(result.rule_count > 0);
    assert!(result.native_ls_lint_parity);
    assert_eq!(result.assura_config_path, ".assura/config.yml");
    assert_eq!(result.ls_lint_config_path, ".ls-lint.yml");
    assert_eq!(result.config_generation_method, "ls-lint-conversion");
    assert_eq!(result.expected_assura_exit_status, 0);
    assert_eq!(result.expected_ls_lint_exit_status, 0);
    assert_eq!(result.assura_binary_profile.as_deref(), Some("debug"));
    assert!(result.assura_binary_path.is_some());
    assert!(result.ls_lint_binary_path.is_none());
    assert!(result.ls_lint_execution_mode.is_none());

    let serialized = serde_json::to_value(&result).unwrap();
    for field in [
        "row_family",
        "validation_execution_mode",
        "evidence_role",
        "source_commit_sha",
        "source_branch",
        "source_patch_id",
        "fixture_source_revision",
        "source_type",
        "checked_file_count",
        "ignored_file_count",
        "directory_count",
        "rule_count",
        "rule_surface_summary",
        "native_ls_lint_parity",
        "assura_config_path",
        "ls_lint_config_path",
        "config_generation_method",
        "shared_config_id",
        "expected_assura_exit_status",
        "expected_ls_lint_exit_status",
        "ls_lint_binary_path",
        "ls_lint_execution_mode",
        "runtime_above_process_floor_ms",
        "assura_cli_overhead_ms",
        "two_x_claim_status",
        "rust_cli_floor_runtime_ms",
        "rust_cli_floor_to_two_x_target_ratio",
        "rust_cli_floor_blocks_two_x",
        "native_regression_baseline_median_ms",
        "native_regression_baseline_report_count",
        "native_regression_baseline_sample_count",
        "native_regression_threshold_ms",
        "native_regression_delta_ms",
        "native_regression_status",
    ] {
        assert!(serialized.get(field).is_some(), "missing field {field}");
    }

    let _ = fs::remove_dir_all(&fixture.root);
}

#[test]
fn synthetic_and_diagnostic_families_are_not_headline_rows() {
    let scenario = scenarios(false)
        .into_iter()
        .find(|scenario| scenario.id == "rule_heavy")
        .unwrap();
    let fixture = materialize_fixture(scenario).unwrap();

    let result = row(
        &fixture,
        "2026-05-17T00:00:00Z",
        "commit",
        "branch",
        &test_environment(),
        "ls-lint v2.3.0",
        RowMeasurement::new("ls-lint-cli", "ls-lint-cli"),
        vec![1.0],
        None,
        "baseline",
    );

    assert_eq!(result.fixture_cohort, "synthetic-stress");
    assert_eq!(result.evidence_role, "diagnostic");
    assert!(result.diagnostic);
    assert!(!is_diagnostic_row("assura-cli", "real-repo-headline"));
    assert!(!is_diagnostic_row("ls-lint-cli", "real-repo-headline"));
    assert!(is_diagnostic_row(
        "assura-in-process",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura:phase:walk-and-validate",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "traversal:jwalk-parallel",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "strategy:jwalk-parallel-cli",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura-check-cached-cli",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura-check-compiled-cli",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura-check-hot-cli",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura-check-changed-path-cli",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura-check-dirty-project-cli",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura-check-dirty-project-session-cli",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura-check-dirty-project-socket",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "assura-check-status-cli",
        "realistic-equivalent"
    ));
    assert!(is_diagnostic_row(
        "strategy:walkdir-cli",
        "realistic-equivalent"
    ));
    assert_eq!(
        validation_execution_mode("assura-check-cached-cli"),
        "warm-cache-cli"
    );
    assert_eq!(
        validation_execution_mode("assura-check-changed-path-cli"),
        "hot-daemon-changed-path-cli"
    );
    assert_eq!(
        validation_execution_mode("assura-check-dirty-project-cli"),
        "hot-daemon-dirty-project-cli"
    );
    assert_eq!(
        validation_execution_mode("assura-check-dirty-project-session-cli"),
        "hot-daemon-dirty-project-session-cli"
    );
    assert_eq!(
        validation_execution_mode("assura-check-dirty-project-socket"),
        "hot-daemon-dirty-project-socket"
    );
    assert_eq!(validation_execution_mode("assura-check-cli"), "cold-cli");

    let _ = fs::remove_dir_all(&fixture.root);
}

#[test]
fn ls_lint_row_carries_native_binary_metadata() {
    let scenario = scenarios(false)
        .into_iter()
        .find(|scenario| scenario.id == "simple_library")
        .unwrap();
    let fixture = materialize_fixture(scenario).unwrap();
    let binary_path = fixture
        .root
        .join("node_modules/@ls-lint/ls-lint/bin/ls-lint-test");

    let result = row(
        &fixture,
        "2026-05-17T00:00:00Z",
        "commit",
        "branch",
        &test_environment(),
        "ls-lint v2.3.0",
        RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli")
            .with_ls_lint_binary(&binary_path, Some("native-binary-from-pinned-npm-package")),
        vec![1.0],
        None,
        "baseline",
    );

    assert_eq!(result.tool_name, "ls-lint-native-cli");
    assert_eq!(result.row_family, "ls-lint-cli");
    assert_eq!(
        result.ls_lint_execution_mode.as_deref(),
        Some("native-binary-from-pinned-npm-package")
    );
    assert!(result
        .ls_lint_binary_path
        .as_deref()
        .unwrap()
        .contains("@ls-lint/ls-lint/bin/ls-lint-test"));

    let _ = fs::remove_dir_all(&fixture.root);
}
