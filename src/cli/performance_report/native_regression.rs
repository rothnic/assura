//! Baseline-calibrated native performance regression metadata.

use super::PerformanceResultRow;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const CHECKED_NATIVE_BASELINE_PATH: &str = "benches/history/native-current.json";
const CHECKED_NATIVE_HISTORY_PATH: &str = "benches/history/native-history.jsonl";
const MIN_CALIBRATED_BASELINE_REPORTS: usize = 2;
const PROVISIONAL_JITTER_FLOOR_MS: f64 = 0.025;
const STATUS_WITHIN_BASELINE: &str = "within-calibrated-baseline";
const STATUS_REGRESSED: &str = "regressed-vs-calibrated-baseline";
const STATUS_WITHIN_PROVISIONAL: &str = "within-provisional-baseline";
const STATUS_REGRESSED_PROVISIONAL: &str = "regressed-vs-provisional-baseline";
const STATUS_BASELINE_MISSING: &str = "baseline-missing";
const STATUS_BASELINE_ROW_MISSING: &str = "baseline-row-missing";
const STATUS_BASELINE_ROW_UNUSABLE: &str = "baseline-row-unusable";

#[derive(Debug, Clone, Copy, PartialEq)]
struct NativeBaselineRow {
    median_ms: f64,
    threshold_ms: f64,
    report_count: usize,
    sample_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NativeBaselineKey {
    fixture_id: String,
    row_family: String,
    os: String,
    arch: String,
    rust_version: String,
    node_version: String,
    npm_version: String,
    assura_version: String,
    assura_binary_profile: String,
}

pub(super) fn annotate_native_regressions(rows: &mut [PerformanceResultRow]) {
    let baseline = load_native_baseline_rows().ok();

    for row in rows.iter_mut() {
        if !row.row_family.starts_with("native:") {
            continue;
        }

        let Some(baseline) = baseline.as_ref() else {
            row.native_regression_status = Some(STATUS_BASELINE_MISSING.to_string());
            continue;
        };

        let key = NativeBaselineKey::from_performance_row(row);
        let Some(baseline_row) = baseline.get(&key) else {
            row.native_regression_status = Some(STATUS_BASELINE_ROW_MISSING.to_string());
            continue;
        };

        row.native_regression_baseline_median_ms = Some(baseline_row.median_ms);
        row.native_regression_baseline_report_count = Some(baseline_row.report_count);
        row.native_regression_baseline_sample_count = Some(baseline_row.sample_count);
        row.native_regression_threshold_ms = Some(baseline_row.threshold_ms);
        row.native_regression_delta_ms = row
            .median_runtime_ms
            .map(|median_ms| median_ms - baseline_row.median_ms);
        row.native_regression_status = Some(classify_regression_status(
            row.median_runtime_ms,
            baseline_row.threshold_ms,
            baseline_row.report_count,
        ));
    }
}

impl NativeBaselineKey {
    fn from_performance_row(row: &PerformanceResultRow) -> Self {
        Self {
            fixture_id: row.fixture_id.clone(),
            row_family: row.row_family.clone(),
            os: row.os.clone(),
            arch: row.arch.clone(),
            rust_version: row.rust_version.clone(),
            node_version: row.node_version.clone(),
            npm_version: row.npm_version.clone(),
            assura_version: row.assura_version.clone(),
            assura_binary_profile: row
                .assura_binary_profile
                .clone()
                .unwrap_or_else(|| "not-applicable".to_string()),
        }
    }

    fn from_value(row: &Value) -> Option<Self> {
        Some(Self {
            fixture_id: row.get("fixture_id")?.as_str()?.to_string(),
            row_family: row.get("row_family")?.as_str()?.to_string(),
            os: row.get("os")?.as_str()?.to_string(),
            arch: row.get("arch")?.as_str()?.to_string(),
            rust_version: row.get("rust_version")?.as_str()?.to_string(),
            node_version: row.get("node_version")?.as_str()?.to_string(),
            npm_version: row.get("npm_version")?.as_str()?.to_string(),
            assura_version: row.get("assura_version")?.as_str()?.to_string(),
            assura_binary_profile: row
                .get("assura_binary_profile")
                .and_then(Value::as_str)
                .unwrap_or("not-applicable")
                .to_string(),
        })
    }
}

fn classify_regression_status(
    median_runtime_ms: Option<f64>,
    threshold_ms: f64,
    report_count: usize,
) -> String {
    let (within_status, regressed_status) = if baseline_is_calibrated(report_count) {
        (STATUS_WITHIN_BASELINE, STATUS_REGRESSED)
    } else {
        (STATUS_WITHIN_PROVISIONAL, STATUS_REGRESSED_PROVISIONAL)
    };

    match median_runtime_ms {
        Some(median_runtime_ms) if median_runtime_ms <= threshold_ms => within_status,
        Some(_) => regressed_status,
        None => STATUS_BASELINE_ROW_UNUSABLE,
    }
    .to_string()
}

fn load_native_baseline_rows() -> Result<BTreeMap<NativeBaselineKey, NativeBaselineRow>, String> {
    let history_path = repo_root().join(CHECKED_NATIVE_HISTORY_PATH);
    if history_path.is_file() {
        let history_rows = load_native_history_rows(&history_path)?;
        if !history_rows.is_empty() {
            return Ok(history_rows);
        }
    }

    let baseline_path = repo_root().join(CHECKED_NATIVE_BASELINE_PATH);
    let baseline_text = fs::read_to_string(&baseline_path)
        .map_err(|error| format!("{}: {error}", baseline_path.display()))?;
    let baseline = serde_json::from_str::<Value>(&baseline_text)
        .map_err(|error| format!("{}: {error}", baseline_path.display()))?;
    let baseline_rows = baseline
        .get("results")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}: missing results array", baseline_path.display()))?;

    let mut rows = BTreeMap::new();
    for row in baseline_rows {
        let Some(key) = NativeBaselineKey::from_value(row) else {
            continue;
        };
        if !key.row_family.starts_with("native:") {
            continue;
        }
        let Some(median_ms) = row.get("median_runtime_ms").and_then(Value::as_f64) else {
            continue;
        };
        let Some(max_ms) = row.pointer("/distribution/max_ms").and_then(Value::as_f64) else {
            continue;
        };
        let min_ms = row
            .pointer("/distribution/min_ms")
            .and_then(Value::as_f64)
            .unwrap_or(median_ms);
        let sample_values = sample_values(row);
        rows.insert(
            key,
            NativeBaselineRow {
                median_ms,
                threshold_ms: derive_threshold_ms(1, median_ms, min_ms, max_ms, &sample_values),
                report_count: 1,
                sample_count: row
                    .pointer("/distribution/samples")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize,
            },
        );
    }
    Ok(rows)
}

fn load_native_history_rows(
    history_path: &Path,
) -> Result<BTreeMap<NativeBaselineKey, NativeBaselineRow>, String> {
    let history_text = fs::read_to_string(history_path)
        .map_err(|error| format!("{}: {error}", history_path.display()))?;

    let mut grouped = BTreeMap::<NativeBaselineKey, Vec<(f64, f64, f64, Vec<f64>, usize)>>::new();
    for line in history_text.lines().filter(|line| !line.trim().is_empty()) {
        let row = serde_json::from_str::<Value>(line)
            .map_err(|error| format!("{}: {error}", history_path.display()))?;
        let Some(key) = NativeBaselineKey::from_value(&row) else {
            continue;
        };
        if !key.row_family.starts_with("native:") {
            continue;
        }
        let Some(median_ms) = row.get("median_runtime_ms").and_then(Value::as_f64) else {
            continue;
        };
        let Some(max_ms) = row.pointer("/distribution/max_ms").and_then(Value::as_f64) else {
            continue;
        };
        let min_ms = row
            .pointer("/distribution/min_ms")
            .and_then(Value::as_f64)
            .unwrap_or(median_ms);
        let sample_values = sample_values(&row);
        let sample_count = row
            .pointer("/distribution/samples")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        grouped.entry(key).or_default().push((
            median_ms,
            min_ms,
            max_ms,
            sample_values,
            sample_count,
        ));
    }

    let mut rows = BTreeMap::new();
    for (key, samples) in grouped {
        let medians = samples
            .iter()
            .map(|(median_ms, _, _, _, _)| *median_ms)
            .collect::<Vec<_>>();
        let mins = samples
            .iter()
            .map(|(_, min_ms, _, _, _)| *min_ms)
            .collect::<Vec<_>>();
        let maxes = samples
            .iter()
            .map(|(_, _, max_ms, _, _)| *max_ms)
            .collect::<Vec<_>>();
        let sample_count = samples
            .iter()
            .map(|(_, _, _, _, sample_count)| *sample_count)
            .sum();
        let median_ms = median(&medians).unwrap_or(0.0);
        let min_ms = mins.into_iter().min_by(f64::total_cmp).unwrap_or(median_ms);
        let max_ms = maxes
            .into_iter()
            .max_by(f64::total_cmp)
            .unwrap_or(median_ms);
        let report_count = samples.len();
        let sample_values = if report_count == 1 {
            samples[0].3.clone()
        } else {
            Vec::new()
        };
        rows.insert(
            key,
            NativeBaselineRow {
                median_ms,
                threshold_ms: derive_threshold_ms(
                    report_count,
                    median_ms,
                    min_ms,
                    max_ms,
                    &sample_values,
                ),
                report_count,
                sample_count,
            },
        );
    }

    Ok(rows)
}

fn baseline_is_calibrated(report_count: usize) -> bool {
    report_count >= MIN_CALIBRATED_BASELINE_REPORTS
}

fn derive_threshold_ms(
    report_count: usize,
    median_ms: f64,
    min_ms: f64,
    max_ms: f64,
    sample_values: &[f64],
) -> f64 {
    if baseline_is_calibrated(report_count) {
        return median_ms.max(max_ms);
    }

    provisional_threshold_ms(sample_values, min_ms, max_ms)
}

fn provisional_threshold_ms(sample_values: &[f64], min_ms: f64, max_ms: f64) -> f64 {
    let representative_max_ms = representative_provisional_max_ms(sample_values).unwrap_or(max_ms);
    let representative_min_ms = sample_values
        .iter()
        .copied()
        .min_by(f64::total_cmp)
        .unwrap_or(min_ms);
    let observed_spread_ms = (representative_max_ms - representative_min_ms).max(0.0);
    representative_max_ms + observed_spread_ms + PROVISIONAL_JITTER_FLOOR_MS
}

fn representative_provisional_max_ms(sample_values: &[f64]) -> Option<f64> {
    if sample_values.is_empty() {
        return None;
    }

    let mut sorted = sample_values.to_vec();
    sorted.sort_by(f64::total_cmp);
    if sorted.len() < 2 || !highest_sample_is_extreme_outlier(&sorted) {
        return sorted.last().copied();
    }

    sorted
        .get(sorted.len() - 2)
        .copied()
        .or_else(|| sorted.last().copied())
}

fn highest_sample_is_extreme_outlier(sorted_values: &[f64]) -> bool {
    if sorted_values.len() < 3 {
        return false;
    }

    let max_value = *sorted_values.last().unwrap_or(&0.0);
    let second_highest = sorted_values[sorted_values.len() - 2];
    let lower_spread = (second_highest - sorted_values[0]).max(0.0);
    if lower_spread == 0.0 {
        return max_value > second_highest + PROVISIONAL_JITTER_FLOOR_MS;
    }

    max_value > second_highest + (5.0 * lower_spread)
}

fn sample_values(row: &Value) -> Vec<f64> {
    row.pointer("/distribution/samples_ms")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_f64)
        .collect()
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[middle - 1] + sorted[middle]) / 2.0)
    } else {
        Some(sorted[middle])
    }
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::{
        baseline_is_calibrated, classify_regression_status, derive_threshold_ms, median,
        representative_provisional_max_ms, NativeBaselineKey,
    };
    use serde_json::json;

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
}
