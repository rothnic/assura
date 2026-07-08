//! Unit coverage for native performance regression baseline annotation.

use super::{
    annotate_native_regressions_with_baseline, baseline_is_calibrated, classify_regression_status,
    derive_threshold_ms, median, representative_provisional_max_ms, NativeBaselineKey,
    NativeBaselineRow, STATUS_BASELINE_ROW_MISSING, STATUS_WITHIN_PROVISIONAL,
};
use crate::cli::performance_report::{PerformanceResultRow, RuntimeDistribution};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn threshold_uses_highest_observed_sample_once_baseline_is_calibrated() {
    let threshold = derive_threshold_ms(2, 100.0, 95.0, 115.0, &[95.0, 100.0, 115.0]);
    assert_eq!(threshold, 115.0);
}

#[test]
fn provisional_threshold_adds_single_report_slack() {
    let threshold = derive_threshold_ms(
        1,
        0.501_056,
        0.498_84,
        0.508_271,
        &[0.498_84, 0.500_311, 0.501_056, 0.502_117, 0.508_271],
    );
    assert!((threshold - 0.542_702).abs() < f64::EPSILON);
}

#[test]
fn provisional_threshold_ignores_single_extreme_outlier() {
    let threshold = derive_threshold_ms(
        1,
        0.486,
        0.482,
        24.909,
        &[0.482, 0.484, 0.486, 0.491, 24.909],
    );
    assert!(threshold < 1.0);
    assert_eq!(
        representative_provisional_max_ms(&[0.482, 0.484, 0.486, 0.491, 24.909]),
        Some(0.491)
    );
}

#[test]
fn regression_status_uses_median_against_calibrated_threshold() {
    assert_eq!(
        classify_regression_status(Some(19.9), 20.0, 2),
        "within-calibrated-baseline"
    );
    assert_eq!(
        classify_regression_status(Some(20.1), 20.0, 2),
        "regressed-vs-calibrated-baseline"
    );
    assert_eq!(
        classify_regression_status(None, 20.0, 2),
        "baseline-row-unusable"
    );
}

#[test]
fn regression_status_reports_provisional_baseline_until_second_report() {
    assert_eq!(
        classify_regression_status(Some(19.9), 20.0, 1),
        "within-provisional-baseline"
    );
    assert_eq!(
        classify_regression_status(Some(20.1), 20.0, 1),
        "regressed-vs-provisional-baseline"
    );
}

#[test]
fn baseline_becomes_calibrated_after_second_report() {
    assert!(!baseline_is_calibrated(1));
    assert!(baseline_is_calibrated(2));
}

#[test]
fn median_uses_sorted_middle_values() {
    assert_eq!(median(&[]), None);
    assert_eq!(median(&[3.0, 1.0, 2.0]), Some(2.0));
    assert_eq!(median(&[4.0, 1.0, 3.0, 2.0]), Some(2.5));
}

#[test]
fn baseline_key_scopes_by_environment_and_profile() {
    let base = json!({
        "fixture_id": "native_small",
        "row_family": "native:content-check-cli",
        "os": "macos",
        "arch": "x86_64",
        "rust_version": "rustc 1.94.1",
        "node_version": "v25.6.0",
        "npm_version": "11.8.0",
        "assura_version": "0.3.0",
        "assura_binary_profile": "release"
    });
    let different_profile = json!({
        "fixture_id": "native_small",
        "row_family": "native:content-check-cli",
        "os": "macos",
        "arch": "x86_64",
        "rust_version": "rustc 1.94.1",
        "node_version": "v25.6.0",
        "npm_version": "11.8.0",
        "assura_version": "0.3.0",
        "assura_binary_profile": "release-static-crt"
    });
    let different_os = json!({
        "fixture_id": "native_small",
        "row_family": "native:content-check-cli",
        "os": "linux",
        "arch": "x86_64",
        "rust_version": "rustc 1.94.1",
        "node_version": "v25.6.0",
        "npm_version": "11.8.0",
        "assura_version": "0.3.0",
        "assura_binary_profile": "release"
    });

    let base_key = NativeBaselineKey::from_value(&base).unwrap();
    assert_ne!(
        base_key,
        NativeBaselineKey::from_value(&different_profile).unwrap()
    );
    assert_ne!(
        base_key,
        NativeBaselineKey::from_value(&different_os).unwrap()
    );
}

#[test]
fn annotate_native_regressions_uses_provisional_for_unseen_environment() {
    let mut baseline = BTreeMap::new();
    baseline.insert(
        NativeBaselineKey {
            fixture_id: "native_small".to_string(),
            row_family: "native:content-check-cli".to_string(),
            os: "macos".to_string(),
            arch: "x86_64".to_string(),
            rust_version: "rustc 1.94.1".to_string(),
            node_version: "v25.6.0".to_string(),
            npm_version: "11.8.0".to_string(),
            assura_version: "0.3.0".to_string(),
            assura_binary_profile: "release".to_string(),
        },
        NativeBaselineRow {
            median_ms: 1.0,
            threshold_ms: 1.0,
            report_count: 2,
            sample_count: 10,
        },
    );
    let mut rows = vec![native_row(
        "linux",
        "native_small",
        "native:content-check-cli",
    )];

    annotate_native_regressions_with_baseline(&mut rows, Some(&baseline));

    assert_eq!(
        rows[0].native_regression_status.as_deref(),
        Some(STATUS_WITHIN_PROVISIONAL)
    );
    assert_eq!(rows[0].native_regression_baseline_report_count, Some(1));
    assert_eq!(rows[0].native_regression_baseline_sample_count, Some(3));
    assert_eq!(rows[0].native_regression_baseline_median_ms, Some(10.0));
}

#[test]
fn annotate_native_regressions_keeps_same_environment_missing_row_strict() {
    let mut baseline = BTreeMap::new();
    baseline.insert(
        NativeBaselineKey {
            fixture_id: "native_small".to_string(),
            row_family: "native:content-check-cli".to_string(),
            os: "linux".to_string(),
            arch: "x86_64".to_string(),
            rust_version: "rustc 1.94.1".to_string(),
            node_version: "v25.6.0".to_string(),
            npm_version: "11.8.0".to_string(),
            assura_version: "0.3.0".to_string(),
            assura_binary_profile: "release".to_string(),
        },
        NativeBaselineRow {
            median_ms: 1.0,
            threshold_ms: 1.0,
            report_count: 2,
            sample_count: 10,
        },
    );
    let mut rows = vec![native_row(
        "linux",
        "native_small",
        "native:content-search-cli",
    )];

    annotate_native_regressions_with_baseline(&mut rows, Some(&baseline));

    assert_eq!(
        rows[0].native_regression_status.as_deref(),
        Some(STATUS_BASELINE_ROW_MISSING)
    );
}

fn native_row(os: &str, fixture_id: &str, row_family: &str) -> PerformanceResultRow {
    PerformanceResultRow {
        schema_version: "assura.performance.v1".to_string(),
        timestamp: "2026-07-08T00:00:00Z".to_string(),
        commit_sha: "0".repeat(40),
        branch: "test".to_string(),
        source_commit_sha: None,
        source_branch: None,
        source_patch_id: None,
        os: os.to_string(),
        arch: "x86_64".to_string(),
        rust_version: "rustc 1.94.1".to_string(),
        node_version: "v25.6.0".to_string(),
        npm_version: "11.8.0".to_string(),
        assura_version: "0.3.0".to_string(),
        ls_lint_version: "not-applicable".to_string(),
        fixture_id: fixture_id.to_string(),
        fixture_source_revision: "native-fixtures-v1".to_string(),
        fixture_cohort: "assura-native".to_string(),
        legacy_fixture_cohort: "stable-baseline".to_string(),
        rule_cohort: "native-small-content-authoring".to_string(),
        row_family: row_family.to_string(),
        validation_execution_mode: "native-query-cli".to_string(),
        evidence_role: "diagnostic".to_string(),
        diagnostic: true,
        fixture_acceptance: "assura-native-diagnostic".to_string(),
        source_type: "generated-native".to_string(),
        checked_file_count: 1,
        ignored_file_count: 0,
        directory_count: 1,
        rule_count: 1,
        rule_surface_summary: "test".to_string(),
        native_ls_lint_parity: false,
        assura_config_path: ".assura/config.yml".to_string(),
        ls_lint_config_path: "not-applicable".to_string(),
        config_generation_method: "assura-native-generated-matrix".to_string(),
        shared_config_id: "native-fixtures-v1:native_small".to_string(),
        expected_assura_exit_status: 0,
        expected_ls_lint_exit_status: 1,
        assura_binary_profile: Some("release".to_string()),
        assura_binary_path: Some("target/release/assura-full".to_string()),
        ls_lint_binary_path: None,
        ls_lint_execution_mode: None,
        tool_name: "assura-full content search".to_string(),
        median_runtime_ms: Some(10.0),
        p95_runtime_ms: Some(11.0),
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
        proves_whole_project_success: true,
        changed_path_count: None,
        latency_threshold_ms: None,
        latency_threshold_met: None,
        native_regression_baseline_median_ms: None,
        native_regression_baseline_report_count: None,
        native_regression_baseline_sample_count: None,
        native_regression_threshold_ms: None,
        native_regression_delta_ms: None,
        native_regression_status: None,
        distribution: RuntimeDistribution {
            samples: 3,
            p95_ms: Some(12.0),
            samples_ms: vec![9.0, 10.0, 12.0],
            min_ms: Some(9.0),
            max_ms: Some(12.0),
        },
        success: true,
        status: "pass".to_string(),
        details: None,
        comparison_baseline_id: "stable-baseline-v1".to_string(),
    }
}
