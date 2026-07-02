use std::collections::BTreeSet;

const CURRENT_REPORT: &str = include_str!("../benches/history/current.json");
const WEBSITE_CURRENT_REPORT: &str =
    include_str!("../website/public/data/performance/current.json");
const HISTORY_REPORT: &str = include_str!("../benches/history/ls-lint-comparison-history.jsonl");
const WEBSITE_HISTORY_REPORT: &str =
    include_str!("../website/public/data/performance/ls-lint-comparison-history.jsonl");

#[test]
fn current_report_claim_summary_matches_headline_rows() {
    let report: serde_json::Value =
        serde_json::from_str(CURRENT_REPORT).expect("current performance report parses as JSON");
    let summary = report
        .get("claim_summary")
        .expect("current report includes claim_summary");
    let rows = report
        .get("results")
        .and_then(serde_json::Value::as_array)
        .expect("current report includes result rows");
    assert_eq!(
        report["source_worktree_dirty"].as_bool(),
        Some(false),
        "checked current report must be generated from a clean source worktree"
    );
    assert!(
        report["command_line"]
            .as_str()
            .is_some_and(|value| !value.is_empty()),
        "current report records the generating command line"
    );
    if let Some(warm_summary) = report.get("warm_claim_summary") {
        assert_summary_matches_rows(
            warm_summary,
            rows,
            "assura-check-dirty-project-session-cli",
            "ls-lint-cli",
        );
    }

    assert_eq!(
        summary["fixture_cohort"],
        serde_json::Value::String("realistic-equivalent".to_string())
    );
    assert_eq!(
        summary["assura_row_family"],
        serde_json::Value::String("assura-cli".to_string())
    );
    assert_eq!(
        summary["ls_lint_row_family"],
        serde_json::Value::String("ls-lint-cli".to_string())
    );

    for row in rows {
        let row_family = row["row_family"]
            .as_str()
            .expect("current report row includes row_family");
        let fixture_acceptance = row["fixture_acceptance"]
            .as_str()
            .expect("current report row includes fixture_acceptance");
        assert!(
            matches!(
                fixture_acceptance,
                "accepted-ls-lint-equivalent"
                    | "diagnostic"
                    | "experimental"
                    | "retired"
                    | "assura-native-diagnostic"
            ),
            "unexpected fixture_acceptance {fixture_acceptance:?}"
        );
        if row["native_ls_lint_parity"].as_bool() == Some(true)
            && matches!(
                row["fixture_cohort"].as_str(),
                Some("realistic-equivalent" | "real-repo-headline")
            )
        {
            assert_eq!(
                fixture_acceptance, "accepted-ls-lint-equivalent",
                "LS-Lint-equivalent fixture rows must be accepted for fixture-floor gates"
            );
        }
        assert_eq!(
            row["validation_execution_mode"].as_str(),
            Some(expected_execution_mode(row_family)),
            "unexpected execution mode for {row_family}"
        );
    }

    assert_summary_matches_rows(summary, rows, "assura-cli", "ls-lint-cli");
}

#[test]
fn checked_current_reports_match_and_cover_accepted_fixture_targets() {
    assert_eq!(
        CURRENT_REPORT, WEBSITE_CURRENT_REPORT,
        "website current performance data must match checked benchmark data"
    );

    let report: serde_json::Value =
        serde_json::from_str(CURRENT_REPORT).expect("current performance report parses as JSON");
    let rows = report
        .get("results")
        .and_then(serde_json::Value::as_array)
        .expect("current report includes result rows");
    let cohort = report
        .pointer("/claim_summary/fixture_cohort")
        .and_then(serde_json::Value::as_str)
        .expect("current report records claim summary fixture cohort");

    let accepted_fixtures = rows
        .iter()
        .filter(|row| row["fixture_cohort"] == cohort)
        .filter(|row| row["fixture_acceptance"] == "accepted-ls-lint-equivalent")
        .filter_map(|row| row["fixture_id"].as_str())
        .collect::<BTreeSet<_>>();
    assert!(
        !accepted_fixtures.is_empty(),
        "current report must include accepted fixture rows"
    );

    for fixture_id in accepted_fixtures {
        let assura = find_headline_row(rows, cohort, fixture_id, "assura-cli")
            .unwrap_or_else(|| panic!("{fixture_id}: missing accepted assura-cli row"));
        let ls_lint = find_headline_row(rows, cohort, fixture_id, "ls-lint-cli")
            .unwrap_or_else(|| panic!("{fixture_id}: missing accepted ls-lint-cli row"));
        assert_eq!(
            assura["fixture_acceptance"], "accepted-ls-lint-equivalent",
            "{fixture_id}: assura-cli row must be accepted"
        );
        assert_eq!(
            ls_lint["fixture_acceptance"], "accepted-ls-lint-equivalent",
            "{fixture_id}: ls-lint-cli row must be accepted"
        );
        assert_eq!(
            ls_lint["tool_name"], "ls-lint-native-cli",
            "{fixture_id}: accepted LS-Lint row must use native LS-Lint"
        );
    }
}

#[test]
fn history_rows_include_execution_mode_metadata() {
    for (name, content) in [
        ("bench history", HISTORY_REPORT),
        ("website history", WEBSITE_HISTORY_REPORT),
    ] {
        for (index, line) in content.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let row: serde_json::Value = serde_json::from_str(line)
                .unwrap_or_else(|error| panic!("{name} line {} parses: {error}", index + 1));
            let row_family = row["row_family"]
                .as_str()
                .unwrap_or_else(|| panic!("{name} line {} includes row_family", index + 1));
            assert_eq!(
                row["validation_execution_mode"].as_str(),
                Some(expected_execution_mode(row_family)),
                "{name} line {} has unexpected execution mode for {row_family}",
                index + 1
            );
        }
    }
}

fn find_headline_row<'a>(
    rows: &'a [serde_json::Value],
    fixture_cohort: &str,
    fixture_id: &str,
    row_family: &str,
) -> Option<&'a serde_json::Value> {
    rows.iter().find(|row| {
        row["fixture_cohort"] == fixture_cohort
            && row["fixture_id"] == fixture_id
            && row["row_family"] == row_family
            && row["status"] == "pass"
    })
}

fn assert_summary_matches_rows(
    summary: &serde_json::Value,
    rows: &[serde_json::Value],
    assura_row_family: &str,
    ls_lint_row_family: &str,
) {
    let fixture_cohort = summary["fixture_cohort"]
        .as_str()
        .expect("claim summary includes fixture_cohort");
    assert_eq!(
        summary["fixture_cohort"],
        serde_json::Value::String(fixture_cohort.to_string())
    );
    assert_eq!(
        summary["assura_row_family"],
        serde_json::Value::String(assura_row_family.to_string())
    );
    assert_eq!(
        summary["ls_lint_row_family"],
        serde_json::Value::String(ls_lint_row_family.to_string())
    );

    let fixture_ids = rows
        .iter()
        .filter(|row| row["fixture_cohort"] == fixture_cohort)
        .filter_map(|row| row["fixture_id"].as_str())
        .collect::<BTreeSet<_>>();

    let mut measured_fixture_count = 0_u64;
    let mut assura_faster_count = 0_u64;
    let mut two_x_pass_count = 0_u64;
    let mut two_x_fail_count = 0_u64;
    let mut blocked_by_process_floor_count = 0_u64;
    let mut blocked_by_rust_cli_floor_count = 0_u64;
    let mut plain_miss_count = 0_u64;
    let mut total_assura_runtime_ms = 0.0;
    let mut total_ls_lint_runtime_ms = 0.0;

    for fixture_id in &fixture_ids {
        let assura = find_headline_row(rows, fixture_cohort, fixture_id, assura_row_family);
        let ls_lint = find_headline_row(rows, fixture_cohort, fixture_id, ls_lint_row_family);
        let Some((assura, ls_lint)) = assura.zip(ls_lint) else {
            continue;
        };
        let Some(assura_ms) = assura["median_runtime_ms"].as_f64() else {
            continue;
        };
        let Some(ls_lint_ms) = ls_lint["median_runtime_ms"].as_f64() else {
            continue;
        };

        measured_fixture_count += 1;
        total_assura_runtime_ms += assura_ms;
        total_ls_lint_runtime_ms += ls_lint_ms;
        if assura_ms < ls_lint_ms {
            assura_faster_count += 1;
        }

        match assura["meets_two_x_target"].as_bool() {
            Some(true) => two_x_pass_count += 1,
            Some(false) => {
                two_x_fail_count += 1;
                match assura["two_x_claim_status"].as_str() {
                    Some("blocked-by-process-floor") => blocked_by_process_floor_count += 1,
                    Some("blocked-by-rust-cli-floor") => blocked_by_rust_cli_floor_count += 1,
                    _ => plain_miss_count += 1,
                }
            }
            None => {}
        }
    }

    let all_headline_rows_available =
        measured_fixture_count == fixture_ids.len() as u64 && measured_fixture_count > 0;
    let all_two_x_targets_met =
        all_headline_rows_available && two_x_pass_count == measured_fixture_count;

    assert_eq!(
        summary["fixture_count"].as_u64(),
        Some(fixture_ids.len() as u64)
    );
    assert_eq!(
        summary["measured_fixture_count"].as_u64(),
        Some(measured_fixture_count)
    );
    assert_eq!(
        summary["assura_faster_count"].as_u64(),
        Some(assura_faster_count)
    );
    assert_eq!(summary["two_x_pass_count"].as_u64(), Some(two_x_pass_count));
    assert_eq!(summary["two_x_fail_count"].as_u64(), Some(two_x_fail_count));
    assert_eq!(
        summary["blocked_by_process_floor_count"].as_u64(),
        Some(blocked_by_process_floor_count)
    );
    assert_eq!(
        summary["blocked_by_rust_cli_floor_count"].as_u64(),
        Some(blocked_by_rust_cli_floor_count)
    );
    assert_eq!(summary["plain_miss_count"].as_u64(), Some(plain_miss_count));
    assert_eq!(
        summary["all_headline_rows_available"].as_bool(),
        Some(all_headline_rows_available)
    );
    assert_eq!(
        summary["all_two_x_targets_met"].as_bool(),
        Some(all_two_x_targets_met)
    );
    if let Some(measured_iterations) = summary["measured_iterations"].as_u64() {
        let minimum_iterations = summary["minimum_completion_iterations"]
            .as_u64()
            .expect("claim summary includes minimum_completion_iterations");
        let sufficient_iterations = measured_iterations >= minimum_iterations;
        assert_eq!(
            summary["sufficient_completion_iterations"].as_bool(),
            Some(sufficient_iterations)
        );
        assert_eq!(
            summary["two_x_claim_verdict"].as_str(),
            Some(if !sufficient_iterations {
                "not-complete-low-sample"
            } else if all_two_x_targets_met {
                "complete"
            } else {
                "not-complete"
            })
        );
    } else {
        assert!(
            summary.get("minimum_completion_iterations").is_none(),
            "legacy claim summaries should omit all low-sample fields together"
        );
        assert!(
            summary.get("sufficient_completion_iterations").is_none(),
            "legacy claim summaries should omit all low-sample fields together"
        );
        assert_eq!(
            summary["two_x_claim_verdict"].as_str(),
            Some(if all_two_x_targets_met {
                "complete"
            } else {
                "not-complete"
            })
        );
    }

    let expected_ratio = total_ls_lint_runtime_ms / total_assura_runtime_ms;
    let actual_ratio = summary["aggregate_speedup_ratio"]
        .as_f64()
        .expect("claim summary includes aggregate speedup ratio");
    assert!(
        (actual_ratio - expected_ratio).abs() < 1e-12,
        "aggregate ratio differs for {assura_row_family}: expected {expected_ratio}, got {actual_ratio}"
    );
}

fn expected_execution_mode(row_family: &str) -> &'static str {
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
        _ => "diagnostic",
    }
}
