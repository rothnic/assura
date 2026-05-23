//! Feasibility annotations for two-times-faster CLI claims.

use super::PerformanceResultRow;
use std::collections::HashMap;

pub(super) fn annotate_two_x_feasibility(rows: &mut [PerformanceResultRow]) {
    let mut ls_lint_by_fixture = HashMap::new();
    let mut process_floor_by_fixture = HashMap::new();
    let mut rust_cli_floor_by_fixture = HashMap::new();
    let mut assura_in_process_by_fixture = HashMap::new();

    for row in rows.iter() {
        match row.tool_name.as_str() {
            "ls-lint-native-cli" | "ls-lint-cli" => {
                if let Some(median) = row.median_runtime_ms {
                    ls_lint_by_fixture.insert(row.fixture_id.clone(), median);
                }
            }
            "process-floor" => {
                if let Some(median) = row.median_runtime_ms {
                    process_floor_by_fixture.insert(row.fixture_id.clone(), median);
                }
            }
            "assura-rust-cli-floor" => {
                if let Some(median) = row.median_runtime_ms {
                    rust_cli_floor_by_fixture.insert(row.fixture_id.clone(), median);
                }
            }
            "assura-check-status-cli" => {
                if let Some(median) = row.median_runtime_ms {
                    rust_cli_floor_by_fixture
                        .entry(row.fixture_id.clone())
                        .or_insert(median);
                }
            }
            "assura-in-process" => {
                if let Some(median) = row.median_runtime_ms {
                    assura_in_process_by_fixture.insert(row.fixture_id.clone(), median);
                }
            }
            _ => {}
        }
    }

    for row in rows {
        let Some(ls_lint_median) = ls_lint_by_fixture.get(&row.fixture_id).copied() else {
            continue;
        };
        let target = ls_lint_median / 2.0;
        row.two_x_target_runtime_ms = Some(target);
        if let Some(runtime) = row.median_runtime_ms {
            row.runtime_to_two_x_target_ratio = (target > 0.0).then_some(runtime / target);
            row.meets_two_x_target = Some(runtime <= target);
        }

        if let Some(rust_cli_floor) = rust_cli_floor_by_fixture.get(&row.fixture_id).copied() {
            row.rust_cli_floor_runtime_ms = Some(rust_cli_floor);
            row.rust_cli_floor_to_two_x_target_ratio =
                (target > 0.0).then_some(rust_cli_floor / target);
            row.rust_cli_floor_blocks_two_x = Some(rust_cli_floor > target);
        }

        let Some(process_floor) = process_floor_by_fixture.get(&row.fixture_id).copied() else {
            continue;
        };
        row.process_floor_runtime_ms = Some(process_floor);
        row.process_floor_to_two_x_target_ratio = (target > 0.0).then_some(process_floor / target);
        row.process_floor_blocks_two_x = Some(process_floor > target);
        if let Some(runtime) = row.median_runtime_ms {
            row.runtime_above_process_floor_ms = Some(runtime - process_floor);
            if is_assura_cli_row(&row.row_family) {
                if let Some(in_process) = assura_in_process_by_fixture.get(&row.fixture_id) {
                    row.assura_cli_overhead_ms = Some(runtime - process_floor - in_process);
                }
            }
        }

        row.two_x_claim_status = classify_two_x_claim(row);
    }
}

fn is_assura_cli_row(row_family: &str) -> bool {
    matches!(
        row_family,
        "assura-cli"
            | "assura-check-cli"
            | "assura-check-compiled-cli"
            | "assura-check-hot-cli"
            | "assura-check-changed-path-cli"
            | "assura-check-dirty-project-cli"
            | "assura-check-status-cli"
            | "assura-rust-cli-floor"
    )
}

fn classify_two_x_claim(row: &PerformanceResultRow) -> Option<String> {
    if !is_assura_cli_row(&row.row_family) {
        return None;
    }

    match row.meets_two_x_target {
        Some(true) => Some("meets-target".to_string()),
        Some(false) if row.process_floor_blocks_two_x == Some(true) => {
            Some("blocked-by-process-floor".to_string())
        }
        Some(false) if row.rust_cli_floor_blocks_two_x == Some(true) => {
            Some("blocked-by-rust-cli-floor".to_string())
        }
        Some(false) => Some("misses-target".to_string()),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::annotate_two_x_feasibility;
    use crate::cli::performance_report::{
        materialize_fixture, row, scenarios, PerformanceEnvironment, RowMeasurement,
    };
    use std::fs;

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
    fn feasibility_annotation_marks_process_floor_blocker() {
        let scenario = scenarios(false)
            .into_iter()
            .find(|scenario| scenario.id == "simple_library")
            .unwrap();
        let fixture = materialize_fixture(scenario).unwrap();
        let mut rows = vec![
            row(
                &fixture,
                "2026-05-17T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("process-floor", "process-floor"),
                vec![5.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-17T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-check-status-cli", "assura-check-status-cli"),
                vec![4.5],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-17T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli"),
                vec![8.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-17T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-in-process", "assura-in-process"),
                vec![1.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-17T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-check-cli", "assura-check-cli"),
                vec![7.0],
                None,
                "baseline",
            ),
        ];

        annotate_two_x_feasibility(&mut rows);

        assert_eq!(rows[0].two_x_target_runtime_ms, Some(4.0));
        assert_eq!(rows[0].process_floor_runtime_ms, Some(5.0));
        assert_eq!(rows[0].process_floor_to_two_x_target_ratio, Some(1.25));
        assert_eq!(rows[0].process_floor_blocks_two_x, Some(true));
        assert_eq!(rows[0].rust_cli_floor_runtime_ms, Some(4.5));
        assert_eq!(rows[0].rust_cli_floor_to_two_x_target_ratio, Some(1.125));
        assert_eq!(rows[0].rust_cli_floor_blocks_two_x, Some(true));
        assert_eq!(rows[0].runtime_above_process_floor_ms, Some(0.0));
        assert_eq!(rows[0].assura_cli_overhead_ms, None);
        assert_eq!(rows[0].runtime_to_two_x_target_ratio, Some(1.25));
        assert_eq!(rows[0].meets_two_x_target, Some(false));
        assert_eq!(rows[1].runtime_to_two_x_target_ratio, Some(1.125));
        assert_eq!(rows[1].meets_two_x_target, Some(false));
        assert_eq!(
            rows[1].two_x_claim_status.as_deref(),
            Some("blocked-by-process-floor")
        );
        assert_eq!(rows[2].runtime_to_two_x_target_ratio, Some(2.0));
        assert_eq!(rows[2].meets_two_x_target, Some(false));
        assert_eq!(rows[2].two_x_claim_status, None);
        assert_eq!(rows[4].runtime_above_process_floor_ms, Some(2.0));
        assert_eq!(rows[4].assura_cli_overhead_ms, Some(1.0));
        assert_eq!(
            rows[4].two_x_claim_status.as_deref(),
            Some("blocked-by-process-floor")
        );

        let _ = fs::remove_dir_all(&fixture.root);
    }

    #[test]
    fn feasibility_annotation_separates_rust_cli_floor_from_plain_miss() {
        let scenario = scenarios(false)
            .into_iter()
            .find(|scenario| scenario.id == "web_app")
            .unwrap();
        let fixture = materialize_fixture(scenario).unwrap();
        let mut rows = vec![
            row(
                &fixture,
                "2026-05-17T00:00:00Z",
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
                "2026-05-17T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-check-status-cli", "assura-check-status-cli"),
                vec![4.5],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-17T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("ls-lint-native-cli", "ls-lint-cli"),
                vec![8.0],
                None,
                "baseline",
            ),
            row(
                &fixture,
                "2026-05-17T00:00:00Z",
                "commit",
                "branch",
                &test_environment(),
                "ls-lint v2.3.0",
                RowMeasurement::new("assura-check-cli", "assura-check-cli"),
                vec![4.2],
                None,
                "baseline",
            ),
        ];

        annotate_two_x_feasibility(&mut rows);

        assert_eq!(rows[3].process_floor_blocks_two_x, Some(false));
        assert_eq!(rows[3].rust_cli_floor_blocks_two_x, Some(true));
        assert_eq!(
            rows[3].two_x_claim_status.as_deref(),
            Some("blocked-by-rust-cli-floor")
        );

        let _ = fs::remove_dir_all(&fixture.root);
    }
}
