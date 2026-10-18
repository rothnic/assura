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

fn count_failure_project() -> TempDir {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      exists:
        "*.tmp": "0"
        "limit-*.txt": "1-2"
        missing.rs: "1"
      allowed_patterns:
        - "keep-*.tmp"
    directories:
      exists:
        "bad-dir-*": "0"
      allowed_patterns:
        - "bad-dir-allowed"
    limit_children:
      max: 3
      message: Keep this project folder tidy.
    children:
      nested/:
        files:
          exists:
            "*.tmp": "0"
          allowed_patterns:
            - "keep-*.tmp"
        directories:
          exists:
            "bad-dir-*": "0"
          allowed_patterns:
            - "bad-dir-allowed"
"#,
    );

    fs::create_dir(project.path().join("nested")).unwrap();
    for number in (1..=7).rev() {
        fs::write(
            project.path().join(format!("bad-{number}.tmp")),
            "unexpected",
        )
        .unwrap();
        fs::create_dir(project.path().join(format!("bad-dir-{number}"))).unwrap();
        fs::write(
            project
                .path()
                .join("nested")
                .join(format!("bad-{number}.tmp")),
            "unexpected",
        )
        .unwrap();
        fs::create_dir(
            project
                .path()
                .join("nested")
                .join(format!("bad-dir-{number}")),
        )
        .unwrap();
    }
    for number in 1..=7 {
        fs::write(
            project.path().join(format!("limit-{number}.txt")),
            "too many",
        )
        .unwrap();
    }
    fs::write(project.path().join("keep-allowed.tmp"), "allowed").unwrap();
    fs::create_dir(project.path().join("bad-dir-allowed")).unwrap();
    fs::write(project.path().join("nested/keep-allowed.tmp"), "allowed").unwrap();
    fs::create_dir(project.path().join("nested/bad-dir-allowed")).unwrap();
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
fn count_findings_are_bounded_aggregated_and_consistent_across_formats() {
    let project = count_failure_project();
    let text_output = run_check(&project, None);
    let json_output = run_check(&project, Some("json"));
    let agent_output = run_check(&project, Some("agent"));

    for output in [&text_output, &json_output, &agent_output] {
        assert_eq!(output.status.code(), Some(1));
    }

    let text = String::from_utf8_lossy(&text_output.stdout);
    let json: Value = serde_json::from_slice(&json_output.stdout).unwrap();
    let agent: Value = serde_json::from_slice(&agent_output.stdout).unwrap();
    let json_messages = diagnostic_messages(&json);
    let agent_messages = agent["feedback"][0]["messages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|message| message["rule"] == "exists_count")
        .map(|message| message["problem"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(json_messages.len(), 6, "one finding per failed count rule");
    let mut sorted_json_messages = json_messages.clone();
    let mut sorted_agent_messages = agent_messages.clone();
    sorted_json_messages.sort();
    sorted_agent_messages.sort();
    assert_eq!(sorted_json_messages, sorted_agent_messages);
    assert_eq!(agent_messages, sorted_agent_messages);
    let text_messages = text
        .lines()
        .filter_map(|line| line.split_once("[medium:exists_count] "))
        .map(|(_, message)| message.to_owned())
        .collect::<Vec<_>>();
    assert_eq!(json_messages, text_messages);

    let violations = json["violations"].as_array().unwrap();
    let count_findings = violations
        .iter()
        .filter(|violation| violation["rule"] == "exists_count")
        .collect::<Vec<_>>();
    assert_eq!(
        count_findings
            .iter()
            .map(|violation| violation["path"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["", "", "", "", "nested", "nested"]
    );
    assert!(count_findings
        .iter()
        .all(|violation| violation["severity"] == "medium"));

    for message in &json_messages {
        if !message.contains("expected 0") {
            continue;
        }
        assert!(message.contains("expected 0"), "{message}");
        assert!(message.contains("Unexpected paths:"), "{message}");
        assert!(
            message.contains("bad-1") || message.contains("bad-dir-1"),
            "{message}"
        );
        assert!(
            message.contains("bad-5") || message.contains("bad-dir-5"),
            "{message}"
        );
        assert!(message.contains("2 more omitted"), "{message}");
        assert!(!message.contains("bad-6"), "{message}");
        assert!(!message.contains("bad-7"), "{message}");
        assert!(!message.contains("bad-dir-6"), "{message}");
        assert!(!message.contains("bad-dir-7"), "{message}");
        assert!(!message.contains("allowed"), "{message}");
    }
    assert!(json_messages[0].contains("./bad-1.tmp"));
    assert!(json_messages[1].contains("Matching paths (sample): ./limit-1.txt"));
    assert!(!json_messages[1].contains("Unexpected paths"));
    assert!(json_messages[1].contains("2 more omitted"));
    assert!(json_messages[2].contains("has 0 files matching 'missing.rs', expected 1"));
    assert!(!json_messages[2].contains("paths:"));
    assert!(json_messages[3].contains("./bad-dir-1"));
    assert!(json_messages[4].contains("nested/bad-1.tmp"));
    assert!(json_messages[5].contains("nested/bad-dir-1"));

    let limit = violations
        .iter()
        .find(|violation| violation["rule"] == "limit_children")
        .unwrap();
    let limit_message = limit["message"].as_str().unwrap();
    assert!(limit_message.contains("Keep this project folder tidy"));
    assert!(limit_message.contains("Actual child count: 24"));
    assert!(limit_message.contains("configured maximum: 3"));
}

#[test]
fn unrelated_forbidden_file_policy_keeps_its_blocking_exit() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      forbidden_patterns:
        - blocked.txt
"#,
    );
    fs::write(project.path().join("blocked.txt"), "blocked").unwrap();

    let output = run_check(&project, Some("json"));
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["violations"][0]["rule"], "forbidden_file");
    assert_eq!(report["violations"][0]["severity"], "medium");
}
