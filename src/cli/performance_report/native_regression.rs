//! Baseline-calibrated native performance regression metadata.

use super::PerformanceResultRow;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct NativeBaselineEnvironmentKey {
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
    annotate_native_regressions_with_baseline(rows, baseline.as_ref());
}

fn annotate_native_regressions_with_baseline(
    rows: &mut [PerformanceResultRow],
    baseline: Option<&BTreeMap<NativeBaselineKey, NativeBaselineRow>>,
) {
    let baseline_environments = baseline.map(baseline_environment_keys);
    for row in rows.iter_mut() {
        if !row.row_family.starts_with("native:") {
            continue;
        }

        let Some(baseline) = baseline else {
            row.native_regression_status = Some(STATUS_BASELINE_MISSING.to_string());
            continue;
        };

        let key = NativeBaselineKey::from_performance_row(row);
        if let Some(baseline_row) = baseline.get(&key).copied() {
            apply_native_baseline(row, baseline_row);
            continue;
        }

        let environment_key = NativeBaselineEnvironmentKey::from_performance_row(row);
        if baseline_environments
            .as_ref()
            .is_some_and(|environments| environments.contains(&environment_key))
        {
            row.native_regression_status = Some(STATUS_BASELINE_ROW_MISSING.to_string());
            continue;
        }

        match provisional_baseline_from_current_row(row) {
            Some(baseline_row) => apply_native_baseline(row, baseline_row),
            None => row.native_regression_status = Some(STATUS_BASELINE_ROW_UNUSABLE.to_string()),
        }
    }
}

fn baseline_environment_keys(
    baseline: &BTreeMap<NativeBaselineKey, NativeBaselineRow>,
) -> BTreeSet<NativeBaselineEnvironmentKey> {
    baseline
        .keys()
        .map(NativeBaselineEnvironmentKey::from_baseline_key)
        .collect()
}

fn apply_native_baseline(row: &mut PerformanceResultRow, baseline_row: NativeBaselineRow) {
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

fn provisional_baseline_from_current_row(row: &PerformanceResultRow) -> Option<NativeBaselineRow> {
    let median_ms = row.median_runtime_ms?;
    let max_ms = row.distribution.max_ms?;
    let min_ms = row.distribution.min_ms.unwrap_or(median_ms);
    Some(NativeBaselineRow {
        median_ms,
        threshold_ms: derive_threshold_ms(
            1,
            median_ms,
            min_ms,
            max_ms,
            &row.distribution.samples_ms,
        ),
        report_count: 1,
        sample_count: row.distribution.samples,
    })
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

impl NativeBaselineEnvironmentKey {
    fn from_performance_row(row: &PerformanceResultRow) -> Self {
        Self {
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

    fn from_baseline_key(key: &NativeBaselineKey) -> Self {
        Self {
            os: key.os.clone(),
            arch: key.arch.clone(),
            rust_version: key.rust_version.clone(),
            node_version: key.node_version.clone(),
            npm_version: key.npm_version.clone(),
            assura_version: key.assura_version.clone(),
            assura_binary_profile: key.assura_binary_profile.clone(),
        }
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
#[path = "native_regression_tests.rs"]
mod tests;
