//! Unit coverage for native performance regression baseline annotation.

use super::{
    annotate_native_regressions_with_baseline, baseline_is_calibrated, classify_regression_status,
    derive_threshold_ms, load_native_baseline_rows_from_paths, load_native_history_rows,
    load_native_history_rows_excluding, median, representative_provisional_max_ms,
    NativeBaselineKey, NativeBaselineRow, NativeExecutionKey, STATUS_BASELINE_ROW_MISSING,
    STATUS_WITHIN_PROVISIONAL,
};
use crate::cli::performance_report::{PerformanceResultRow, RuntimeDistribution};
use serde_json::json;
use std::collections::BTreeMap;
use std::io::Write;

#[test]
fn native_history_duplicate_execution_counts_once_and_stays_provisional() {
    let row = measured_native_row("run-a", "commit-a", &[9.0, 10.0, 11.0, 12.0, 13.0]);
    let mut replay = row.clone();
    replay.native_regression_status = Some("replayed-annotation".to_string());
    let history = temporary_history(&[&row, &replay]);

    let baseline = load_native_history_rows(history.path()).expect("history loads");
    assert_eq!(baseline.len(), 1);
    let baseline_row = baseline.values().next().expect("native row admitted");
    assert_eq!(baseline_row.report_count, 1);
    assert_eq!(baseline_row.sample_count, 5);
    let expected_threshold =
        derive_threshold_ms(1, 11.0, 9.0, 13.0, &[9.0, 10.0, 11.0, 12.0, 13.0]);
    assert_eq!(baseline_row.threshold_ms, expected_threshold);

    let reversed_history = temporary_history(&[&replay, &row]);
    let reversed_baseline =
        load_native_history_rows(reversed_history.path()).expect("reordered history loads");
    assert_eq!(baseline, reversed_baseline);

    // A separate execution with the same baseline key exercises the real
    // loader-to-annotation path without excluding the duplicate history row.
    let target = measured_native_row(
        "run-target",
        "commit-target",
        &[10.0, 11.0, 12.0, 13.0, 14.0],
    );
    let annotated = annotate_from_history(&[&row, &replay], target.clone());
    let reversed_annotated = annotate_from_history(&[&replay, &row], target);
    assert_eq!(
        annotated.native_regression_status,
        Some(STATUS_WITHIN_PROVISIONAL.into())
    );
    assert_eq!(annotated.native_regression_baseline_report_count, Some(1));
    assert_eq!(annotated.native_regression_baseline_sample_count, Some(5));
    assert_eq!(
        annotated.native_regression_threshold_ms,
        Some(expected_threshold)
    );
    assert_eq!(
        annotated.native_regression_status,
        reversed_annotated.native_regression_status
    );
    assert_eq!(
        annotated.native_regression_baseline_report_count,
        reversed_annotated.native_regression_baseline_report_count
    );
    assert_eq!(
        annotated.native_regression_baseline_sample_count,
        reversed_annotated.native_regression_baseline_sample_count
    );
    assert_eq!(
        annotated.native_regression_threshold_ms,
        reversed_annotated.native_regression_threshold_ms
    );
}

#[test]
fn native_history_conflicting_duplicate_execution_fails_closed() {
    let first = measured_native_row("run-a", "commit-a", &[9.0, 10.0, 11.0, 12.0, 13.0]);
    let second = measured_native_row("run-a", "commit-a", &[19.0, 20.0, 21.0, 22.0, 23.0]);
    let history = temporary_history(&[&first, &second]);

    assert!(load_native_history_rows(history.path()).is_err());
}

#[test]
fn native_history_same_commit_different_timestamps_count_as_distinct_runs() {
    let first = measured_native_row("run-a", "commit-a", &[9.0, 10.0, 11.0, 12.0, 13.0]);
    let second = measured_native_row("run-b", "commit-a", &[10.0, 11.0, 12.0, 13.0, 14.0]);
    let history = temporary_history(&[&first, &second]);

    let baseline = load_native_history_rows(history.path()).expect("history loads");
    let baseline = baseline.values().next().expect("native row admitted");

    assert_eq!(baseline.report_count, 2);
    assert_eq!(baseline.sample_count, 10);
    assert_eq!(baseline.threshold_ms, 14.0);
}

#[test]
fn native_history_without_usable_execution_identity_gets_no_report_credit() {
    let mut row = measured_native_row("run-a", "commit-a", &[9.0, 10.0, 11.0, 12.0, 13.0]);
    row.timestamp.clear();
    let history = temporary_history(&[&row]);

    assert!(load_native_history_rows(history.path())
        .expect("history remains usable")
        .is_empty());
}

#[test]
fn native_history_excludes_the_slow_current_execution_from_its_own_baseline() {
    let first = measured_native_row("run-a", "commit-a", &[9.0, 10.0, 11.0, 12.0, 13.0]);
    let second = measured_native_row("run-b", "commit-a", &[10.0, 11.0, 12.0, 13.0, 14.0]);
    let target = measured_native_row("run-c", "commit-b", &[99.0, 100.0, 101.0, 102.0, 103.0]);
    let unrelated_key_row = measured_native_row("run-c", "commit-b", &[5.0, 6.0, 7.0, 8.0, 9.0]);
    let mut unrelated_key_row = unrelated_key_row;
    unrelated_key_row.fixture_id = "native_other".to_string();
    let history = temporary_history(&[&first, &second, &target, &unrelated_key_row]);

    let target_key = NativeExecutionKey::from_performance_row(&target).expect("target identity");
    let baseline = load_native_history_rows_excluding(
        history.path(),
        &std::collections::BTreeSet::from([target_key]),
    )
    .expect("history loads");
    let unrelated_key = NativeBaselineKey::from_performance_row(&unrelated_key_row);
    assert_eq!(
        baseline.get(&unrelated_key).map(|row| row.report_count),
        Some(1)
    );
    let mut annotated = vec![target.clone()];
    annotate_native_regressions_with_baseline(&mut annotated, Some(&baseline));

    assert_eq!(
        annotated[0].native_regression_baseline_report_count,
        Some(2)
    );
    assert_eq!(
        annotated[0].native_regression_baseline_sample_count,
        Some(10)
    );
    assert_eq!(annotated[0].native_regression_threshold_ms, Some(14.0));
    assert_eq!(
        annotated[0].native_regression_status.as_deref(),
        Some("regressed-vs-calibrated-baseline")
    );
    write_gate_row_if_requested("seeded-slowdown-row.json", &annotated[0]);

    let unchanged = measured_native_row("run-d", "commit-d", &[10.0, 11.0, 12.0, 13.0, 14.0]);
    let unchanged_history = unchanged.clone();
    let unchanged = annotate_from_history(&[&first, &second, &unchanged_history], unchanged);
    assert_eq!(
        unchanged.native_regression_status.as_deref(),
        Some("within-calibrated-baseline")
    );
    assert_eq!(unchanged.native_regression_baseline_report_count, Some(2));
    assert_eq!(unchanged.native_regression_baseline_sample_count, Some(10));
    assert_eq!(unchanged.native_regression_threshold_ms, Some(14.0));
    write_gate_row_if_requested("unchanged-row.json", &unchanged);
}

#[test]
fn native_current_fallback_excludes_target_but_keeps_unseen_row_provisional() {
    let target = measured_native_row("run-c", "commit-c", &[99.0, 100.0, 101.0, 102.0, 103.0]);
    let mut unrelated = native_row("linux", "native_other", "native:content-check-cli");
    unrelated.timestamp = target.timestamp.clone();
    unrelated.commit_sha = target.commit_sha.clone();
    let empty_history = temporary_history(&[]);
    let checked_current = temporary_current(&[&target, &unrelated]);
    let target_key = NativeExecutionKey::from_performance_row(&target).expect("target identity");
    let targets = std::collections::BTreeSet::from([target_key.clone()]);

    let baseline = load_native_baseline_rows_from_paths(
        empty_history.path(),
        checked_current.path(),
        &targets,
    )
    .expect("checked-current fallback loads");
    assert_eq!(baseline.len(), 1, "unrelated environment remains available");
    assert!(
        !baseline.contains_key(&NativeBaselineKey::from_performance_row(&target)),
        "the exact target cannot reenter through fallback"
    );

    let mut annotated = vec![target];
    annotate_native_regressions_with_baseline(&mut annotated, Some(&baseline));
    assert_eq!(
        annotated[0].native_regression_baseline_report_count,
        Some(1)
    );
    assert_eq!(
        annotated[0].native_regression_baseline_sample_count,
        Some(5)
    );
    assert_eq!(
        annotated[0].native_regression_status.as_deref(),
        Some(STATUS_WITHIN_PROVISIONAL)
    );
}

#[test]
fn native_current_fallback_rejects_conflicting_rows_for_one_execution() {
    let first = measured_native_row("run-a", "commit-a", &[9.0, 10.0, 11.0, 12.0, 13.0]);
    let second = measured_native_row("run-a", "commit-a", &[19.0, 20.0, 21.0, 22.0, 23.0]);
    let empty_history = temporary_history(&[]);
    let checked_current = temporary_current(&[&first, &second]);

    assert!(load_native_baseline_rows_from_paths(
        empty_history.path(),
        checked_current.path(),
        &std::collections::BTreeSet::new(),
    )
    .is_err());
}

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
        measurement_order: None,
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

fn measured_native_row(timestamp: &str, commit_sha: &str, samples: &[f64]) -> PerformanceResultRow {
    let mut row = native_row("macos", "native_small", "native:content-check-cli");
    row.rust_version = "rustc 1.94.1 (e408947bf 2026-03-25)".to_string();
    row.timestamp = timestamp.to_string();
    row.commit_sha = commit_sha.to_string();
    row.median_runtime_ms = median(samples);
    row.distribution.samples = samples.len();
    row.distribution.samples_ms = samples.to_vec();
    row.distribution.min_ms = samples.iter().copied().min_by(f64::total_cmp);
    row.distribution.max_ms = samples.iter().copied().max_by(f64::total_cmp);
    row.distribution.p95_ms = row.distribution.max_ms;
    row
}

fn temporary_history(rows: &[&PerformanceResultRow]) -> tempfile::NamedTempFile {
    let mut file = tempfile::NamedTempFile::new().expect("temporary history file");
    for row in rows {
        serde_json::to_writer(&mut file, row).expect("serialize history row");
        writeln!(file).expect("separate history rows");
    }
    file.flush().expect("flush history file");
    file
}

fn temporary_current(rows: &[&PerformanceResultRow]) -> tempfile::NamedTempFile {
    let file = tempfile::NamedTempFile::new().expect("temporary current report");
    serde_json::to_writer(&file, &json!({ "results": rows })).expect("serialize current rows");
    file
}

fn annotate_from_history(
    history_rows: &[&PerformanceResultRow],
    target: PerformanceResultRow,
) -> PerformanceResultRow {
    let history = temporary_history(history_rows);
    let target_key = NativeExecutionKey::from_performance_row(&target).expect("target identity");
    let baseline = load_native_history_rows_excluding(
        history.path(),
        &std::collections::BTreeSet::from([target_key]),
    )
    .expect("history loads");
    let mut annotated = vec![target];
    annotate_native_regressions_with_baseline(&mut annotated, Some(&baseline));
    annotated.remove(0)
}

fn write_gate_row_if_requested(filename: &str, row: &PerformanceResultRow) {
    let Some(directory) = std::env::var_os("SP04_GATE_ROW_DIR") else {
        return;
    };
    let path = std::path::Path::new(&directory).join(filename);
    std::fs::write(
        path,
        serde_json::to_vec(row).expect("serialize annotated gate row"),
    )
    .expect("write annotated gate row");
}
