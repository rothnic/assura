use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

fn fast_count_route_project(force_general_route: bool) -> TempDir {
    let project = TempDir::new().unwrap();
    let general_route_guard = if force_general_route {
        "    limit_children:\n      max: 100\n"
    } else {
        ""
    };
    let config = format!(
        r#"
structure:
  ./:
    files:
      exists:
        "bad-*.tmp": "0"
        "limit-*.txt": "1-2"
        missing.rs: "1"
    directories:
      exists:
        "bad-dir-*": "0"
{general_route_guard}    children:
      .agents/:
        children:
          skills/:
            children:
              missing-skill-md/:
                files:
                  exists:
                    "bad-*.tmp": "0"
                directories:
                  exists:
                    "bad-dir-*": "0"
"#
    );
    write_config(&project, &config);

    let nested_parent = project
        .path()
        .join(".agents")
        .join("skills")
        .join("missing-skill-md");
    fs::create_dir_all(&nested_parent).unwrap();
    for number in 1..=7 {
        fs::write(
            project.path().join(format!("bad-{number}.tmp")),
            "unexpected",
        )
        .unwrap();
        fs::write(
            project.path().join(format!("limit-{number}.txt")),
            "too many",
        )
        .unwrap();
        fs::create_dir(project.path().join(format!("bad-dir-{number}"))).unwrap();

        fs::write(
            nested_parent.join(format!("bad-{number}.tmp")),
            "unexpected",
        )
        .unwrap();
        fs::create_dir(nested_parent.join(format!("bad-dir-{number}"))).unwrap();
    }
    project
}

fn fast_zero_count_project(force_general_route: bool, use_allowed_pattern: bool) -> TempDir {
    let project = TempDir::new().unwrap();
    let file_count_rules = if use_allowed_pattern {
        r#"                  exists:
                    "*.tmp": "0"
                  allowed_patterns:
                    - "keep-*.tmp"
"#
    } else {
        r#"                  exists:
                    "*.tmp": "0"
                    "keep-*.tmp": "1"
"#
    };
    let general_route_guard = if force_general_route {
        "                limit_children:\n                  max: 100\n"
    } else {
        ""
    };
    let config = format!(
        r#"
structure:
  ./:
    children:
      .agents/:
        children:
          skills/:
            children:
              missing-skill-md/:
                files:
{file_count_rules}{general_route_guard}"#
    );
    write_config(&project, &config);

    let nested_parent = project
        .path()
        .join(".agents")
        .join("skills")
        .join("missing-skill-md");
    fs::create_dir_all(&nested_parent).unwrap();
    fs::write(nested_parent.join("keep-one.tmp"), "allowed or refined").unwrap();
    project
}

fn run_check(project: &TempDir, format: Option<&str>) -> Output {
    let mut command = Command::new(assura_bin());
    command.arg("check").arg(project.path());
    if let Some(format) = format {
        command.arg("--format").arg(format);
    }
    command.output().unwrap()
}

fn run_lslint_target_check(project: &TempDir, target: &str) -> Output {
    Command::new(assura_bin())
        .arg("check")
        .arg(project.path().join(target))
        .arg("--ls-lint-target-semantics")
        .arg("--format")
        .arg("json")
        .output()
        .unwrap()
}

fn diagnostic_messages(json: &Value) -> Vec<String> {
    json["violations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|violation| violation["rule"] == "exists_count")
        .map(|violation| violation["message"].as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn fast_eligible_counts_match_general_route_bounded_diagnostics() {
    let fast_project = fast_count_route_project(false);
    let general_project = fast_count_route_project(true);
    let fast_output = run_check(&fast_project, Some("json"));
    let fast_text_output = run_check(&fast_project, None);
    let fast_agent_output = run_check(&fast_project, Some("agent"));
    let general_output = run_check(&general_project, Some("json"));
    assert_eq!(fast_output.status.code(), Some(1));
    assert_eq!(fast_text_output.status.code(), Some(1));
    assert_eq!(fast_agent_output.status.code(), Some(1));
    assert_eq!(general_output.status.code(), Some(1));

    let fast_report: Value = serde_json::from_slice(&fast_output.stdout).unwrap();
    let fast_agent: Value = serde_json::from_slice(&fast_agent_output.stdout).unwrap();
    let general_report: Value = serde_json::from_slice(&general_output.stdout).unwrap();
    let fast_findings = fast_report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "exists_count")
        .collect::<Vec<_>>();
    let general_findings = general_report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "exists_count")
        .collect::<Vec<_>>();
    assert_eq!(fast_findings.len(), 6);
    assert_eq!(general_findings.len(), 6);

    let mut fast_messages = diagnostic_messages(&fast_report);
    let mut fast_agent_messages = fast_agent["feedback"][0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|message| message["rule"] == "exists_count")
        .map(|message| message["problem"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    let mut fast_text_messages = String::from_utf8_lossy(&fast_text_output.stdout)
        .lines()
        .filter_map(|line| line.split_once("[medium:exists_count] "))
        .map(|(_, message)| message.to_owned())
        .collect::<Vec<_>>();
    fast_messages.sort();
    fast_agent_messages.sort();
    fast_text_messages.sort();
    assert_eq!(fast_messages, fast_text_messages);
    assert_eq!(fast_messages, fast_agent_messages);

    let signatures = |findings: &[&Value]| {
        let mut signatures = findings
            .iter()
            .map(|finding| {
                (
                    finding["path"].as_str().unwrap().to_owned(),
                    finding["severity"].as_str().unwrap().to_owned(),
                )
            })
            .collect::<Vec<_>>();
        signatures.sort();
        signatures
    };
    assert_eq!(signatures(&fast_findings), signatures(&general_findings));
    assert_eq!(
        signatures(&fast_findings),
        [
            (String::new(), "medium".to_owned()),
            (String::new(), "medium".to_owned()),
            (String::new(), "medium".to_owned()),
            (String::new(), "medium".to_owned()),
            (
                ".agents/skills/missing-skill-md".to_owned(),
                "medium".to_owned()
            ),
            (
                ".agents/skills/missing-skill-md".to_owned(),
                "medium".to_owned()
            ),
        ]
    );

    let messages = |findings: &[&Value]| {
        let mut messages = findings
            .iter()
            .map(|finding| finding["message"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>();
        messages.sort();
        messages
    };
    let general_messages = messages(&general_findings);
    let missing_minimum = general_messages
        .iter()
        .find(|message| message.contains("missing.rs"))
        .unwrap();
    assert!(missing_minimum.contains("expected 1"));
    assert!(!missing_minimum.contains("paths:"));
    let over_maximum = general_messages
        .iter()
        .find(|message| message.contains("limit-1.txt"))
        .unwrap();
    assert!(over_maximum.contains("Matching paths (sample): ./limit-1.txt"));
    assert!(over_maximum.contains("2 more omitted"));
    for expected_sample in [
        "Unexpected paths: ./bad-1.tmp, ./bad-2.tmp, ./bad-3.tmp, ./bad-4.tmp, ./bad-5.tmp; 2 more omitted",
        "Unexpected paths: ./bad-dir-1, ./bad-dir-2, ./bad-dir-3, ./bad-dir-4, ./bad-dir-5; 2 more omitted",
        "Unexpected paths: .agents/skills/missing-skill-md/bad-1.tmp, .agents/skills/missing-skill-md/bad-2.tmp, .agents/skills/missing-skill-md/bad-3.tmp, .agents/skills/missing-skill-md/bad-4.tmp, .agents/skills/missing-skill-md/bad-5.tmp; 2 more omitted",
        "Unexpected paths: .agents/skills/missing-skill-md/bad-dir-1, .agents/skills/missing-skill-md/bad-dir-2, .agents/skills/missing-skill-md/bad-dir-3, .agents/skills/missing-skill-md/bad-dir-4, .agents/skills/missing-skill-md/bad-dir-5; 2 more omitted",
    ] {
        assert!(
            general_messages
                .iter()
                .any(|message| message.contains(expected_sample)),
            "missing expected general-route sample: {expected_sample}"
        );
    }

    assert_eq!(
        messages(&fast_findings),
        general_messages,
        "fast route must preserve full bounded diagnostics from the equivalent general route"
    );
}

fn assert_fast_zero_count_project_uses_general_route_semantics(use_allowed_pattern: bool) {
    let fast_project = fast_zero_count_project(false, use_allowed_pattern);
    let general_project = fast_zero_count_project(true, use_allowed_pattern);
    let fast_output = run_check(&fast_project, Some("json"));
    let general_output = run_check(&general_project, Some("json"));
    assert_eq!(general_output.status.code(), Some(0));
    assert_eq!(fast_output.status.code(), Some(0));
    let general_report: Value = serde_json::from_slice(&general_output.stdout).unwrap();
    assert!(!general_report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["rule"] == "exists_count"));

    let fast_report: Value = serde_json::from_slice(&fast_output.stdout).unwrap();
    let fast_count_findings = fast_report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "exists_count")
        .collect::<Vec<_>>();
    assert!(fast_count_findings.is_empty());

    assert_eq!(
        fast_output.status.code(),
        general_output.status.code(),
        "fast zero-count matching must honor {} as a refinement; fast report: {}",
        if use_allowed_pattern {
            "allowed_patterns"
        } else {
            "a more-specific positive-count rule"
        },
        String::from_utf8_lossy(&fast_output.stdout)
    );
}

#[test]
fn fast_zero_count_allowed_patterns_follow_general_route() {
    assert_fast_zero_count_project_uses_general_route_semantics(true);
}

#[test]
fn fast_zero_count_positive_refinement_follows_general_route() {
    assert_fast_zero_count_project_uses_general_route_semantics(false);
}

#[test]
fn explicit_file_target_suffix_count_includes_bounded_sibling_paths() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      exists:
        "*.ts": "1"
"#,
    );
    for number in (1..=7).rev() {
        fs::write(project.path().join(format!("file-{number}.ts")), "").unwrap();
    }

    let output = run_lslint_target_check(&project, "file-1.ts");
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let findings = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["rule"] == "exists_count")
        .collect::<Vec<_>>();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0]["path"], "");
    let message = findings[0]["message"].as_str().unwrap();
    assert!(
        message.contains("7 files matching '*.ts', expected 1"),
        "{message}"
    );
    assert!(
        message.contains(
            "Matching paths (sample): ./file-1.ts, ./file-2.ts, ./file-3.ts, ./file-4.ts, ./file-5.ts; 2 more omitted"
        ),
        "{message}"
    );
}
