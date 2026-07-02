use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

const FIXTURE_ROOT: &str = "tests/fixtures/markdown_engine_candidates";

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn fixture_path(name: &str) -> PathBuf {
    Path::new(FIXTURE_ROOT).join(name)
}

fn matrix() -> Value {
    serde_json::from_str(
        &fs::read_to_string(fixture_path("matrix.json")).expect("fixture matrix is readable"),
    )
    .expect("fixture matrix parses as JSON")
}

fn run_assura(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(args)
        .output()
        .expect("assura command runs")
}

fn check_fixture(name: &str) -> (std::process::ExitStatus, Value) {
    let path = fixture_path(name);
    let output = run_assura(&[
        "check",
        path.to_str().expect("fixture path is UTF-8"),
        "--format",
        "json",
    ]);
    let json = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse check stdout as JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status, json)
}

fn rule_set(report: &Value) -> BTreeSet<String> {
    report["violations"]
        .as_array()
        .expect("violations is an array")
        .iter()
        .map(|violation| {
            violation["rule"]
                .as_str()
                .expect("violation has rule")
                .to_string()
        })
        .collect()
}

fn expected_rules(matrix: &Value, variant: &str, key: &str) -> BTreeSet<String> {
    matrix["variants"][variant][key]
        .as_array()
        .expect("expected rules are an array")
        .iter()
        .map(|rule| rule.as_str().expect("rule is string").to_string())
        .collect()
}

fn first_rule_index(report: &Value, expected_rules: &[&str]) -> Option<usize> {
    report["violations"]
        .as_array()
        .expect("violations is an array")
        .iter()
        .position(|violation| {
            violation["rule"]
                .as_str()
                .is_some_and(|rule| expected_rules.contains(&rule))
        })
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination");
    for entry in fs::read_dir(source).expect("read source directory") {
        let entry = entry.expect("read directory entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("copy fixture file");
        }
    }
}

fn normalize_newlines(text: &str) -> String {
    text.replace("\r\n", "\n")
}

#[test]
fn markdown_engine_candidate_matrix_describes_current_contract() {
    let matrix = matrix();

    assert_eq!(
        matrix["schema_version"],
        "assura.markdown-engine-candidate-fixtures.v1"
    );
    for variant in ["valid", "invalid"] {
        let path = matrix["variants"][variant]["path"]
            .as_str()
            .expect("variant path");
        assert!(
            fixture_path(path).join(".assura/config.yml").exists(),
            "{variant} fixture config should exist"
        );
    }

    let config = normalize_newlines(
        &fs::read_to_string(fixture_path("invalid/.assura/config.yml"))
            .expect("invalid fixture config"),
    );
    assert!(
        config.contains("rules:\n            markdown_link_target:\n              severity: low")
    );
    assert!(config.contains("markdown_trailing_spaces:\n              severity: low"));
    for forbidden in matrix["config_constraints"]["forbidden_keys"]
        .as_array()
        .expect("forbidden keys")
    {
        let forbidden = forbidden.as_str().expect("forbidden key");
        if forbidden == "severity" {
            assert!(
                !config.lines().any(|line| line.starts_with("  severity:")),
                "fixture must not use a top-level severity map"
            );
        } else {
            assert!(
                !config.contains(&format!("{forbidden}:")),
                "fixture must keep severity under markdown.rules, not {forbidden}"
            );
        }
    }

    let mappings = matrix["rule_mappings"]
        .as_array()
        .expect("rule mappings array");
    for rule in [
        "markdown_link_target",
        "markdown_link_heading_anchor",
        "markdown_link_line_anchor",
        "markdown_link_format",
    ] {
        let mapping = mappings
            .iter()
            .find(|mapping| mapping["stable_rule_id"] == rule)
            .unwrap_or_else(|| panic!("missing mapping for {rule}"));
        assert_eq!(mapping["stage"], "assura_markdown_reference");
        assert_eq!(mapping["markdownlint_aliases"].as_array().unwrap().len(), 0);
        assert_eq!(mapping["assura_owned"], true);
    }
}

#[test]
fn markdown_engine_candidate_valid_fixture_passes_current_assura() {
    let (status, json) = check_fixture("valid");

    assert!(status.success(), "report: {json:#}");
    assert_eq!(json["success"], true);
    assert_eq!(json["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn markdown_engine_candidate_invalid_fixture_preserves_assura_contracts() {
    let matrix = matrix();
    let (status, json) = check_fixture("invalid");

    assert_eq!(status.code(), Some(1), "report: {json:#}");
    assert_eq!(json["success"], false);

    let actual_rules = rule_set(&json);
    let expected = expected_rules(&matrix, "invalid", "expected_assura_rules");
    for rule in &expected {
        assert!(
            actual_rules.contains(rule),
            "missing expected rule {rule}; report: {json:#}"
        );
    }

    let structure_index =
        first_rule_index(&json, &["unexpected_file"]).expect("structure finding exists");
    let markdown_index = first_rule_index(
        &json,
        &[
            "markdown_suppression",
            "markdown_required_section",
            "markdown_heading_increment",
            "markdown_link_target",
        ],
    )
    .expect("markdown finding exists");
    assert!(
        structure_index < markdown_index,
        "structure findings must precede markdown internals: {json:#}"
    );

    for advisory_rule in expected_rules(&matrix, "invalid", "expected_advisory_rules") {
        let violation = json["violations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|violation| violation["rule"] == advisory_rule)
            .unwrap_or_else(|| panic!("missing advisory rule {advisory_rule}"));
        assert_eq!(
            violation["blocking"], false,
            "{advisory_rule} should be advisory through rule-owned severity"
        );
    }

    for blocking_rule in expected_rules(&matrix, "invalid", "expected_blocking_rules") {
        let violation = json["violations"]
            .as_array()
            .unwrap()
            .iter()
            .find(|violation| violation["rule"] == blocking_rule)
            .unwrap_or_else(|| panic!("missing blocking rule {blocking_rule}"));
        assert_eq!(violation["blocking"], true, "{blocking_rule} should block");
    }
}

#[test]
fn markdown_engine_candidate_fixture_safe_fix_previews_are_bounded() {
    let temp = TempDir::new().expect("tempdir");
    copy_dir(&fixture_path("invalid"), temp.path());
    let guide = temp.path().join("docs/guide.md");
    let before = fs::read_to_string(&guide).expect("read guide before fix");

    let trailing = run_assura(&[
        "fix",
        "markdown",
        temp.path().to_str().expect("temp path is UTF-8"),
        "--rule",
        "trailing-spaces",
        "--dry-run",
        "--format",
        "json",
    ]);
    assert_eq!(trailing.status.code(), Some(0));
    let trailing_json: Value =
        serde_json::from_slice(&trailing.stdout).expect("trailing-spaces fix JSON");
    assert_eq!(trailing_json["rule"], "trailing-spaces");
    assert_eq!(trailing_json["files_would_change"], 1);
    assert_eq!(trailing_json["fixes_would_apply"], 1);
    assert_eq!(
        trailing_json["fixes"][0]["operation"],
        "remove_blank_line_trailing_spaces"
    );

    let required = run_assura(&[
        "fix",
        "markdown",
        temp.path().to_str().expect("temp path is UTF-8"),
        "--rule",
        "required-sections",
        "--dry-run",
        "--format",
        "json",
    ]);
    assert_eq!(required.status.code(), Some(0));
    let required_json: Value =
        serde_json::from_slice(&required.stdout).expect("required-section fix JSON");
    assert_eq!(required_json["rule"], "required-sections");
    assert_eq!(required_json["files_would_change"], 1);
    assert_eq!(required_json["fixes_would_apply"], 1);
    assert_eq!(
        required_json["fixes"][0]["operation"],
        "insert_required_section_heading"
    );
    assert_eq!(required_json["fixes"][0]["inserted_text"], "## API");

    let after = fs::read_to_string(&guide).expect("read guide after fix");
    assert_eq!(after, before, "dry-run safe fixes must not write");
    assert!(normalize_newlines(&after).starts_with("---\ntitle: Guide\n---\n"));
}
