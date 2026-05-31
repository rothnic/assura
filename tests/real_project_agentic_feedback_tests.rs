use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/real-project-agentic-feedback")
        .join(name)
}

fn run_check(project: &Path) -> (std::process::ExitStatus, serde_json::Value) {
    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    let report = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse check output as json: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status, report)
}

#[test]
fn real_project_policy_valid_fixture_passes() {
    let project = fixture_path("valid");

    let (status, report) = run_check(&project);

    assert!(status.success(), "report was:\n{report:#}");
    assert_eq!(report["success"], true, "report was:\n{report:#}");
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn real_project_policy_invalid_fixture_reports_intended_drift() {
    let project = fixture_path("invalid");

    let (status, report) = run_check(&project);
    let violations = report["violations"].as_array().unwrap();
    let rules = violations
        .iter()
        .map(|violation| violation["rule"].as_str().unwrap())
        .collect::<HashSet<_>>();
    let paths = violations
        .iter()
        .map(|violation| violation["path"].as_str().unwrap())
        .collect::<HashSet<_>>();

    assert_eq!(status.code(), Some(1), "report was:\n{report:#}");
    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert!(rules.contains("unexpected_file"), "report was:\n{report:#}");
    assert!(rules.contains("file_naming"), "report was:\n{report:#}");
    assert!(rules.contains("exists_count"), "report was:\n{report:#}");
    assert!(paths.contains("scratch.md"), "report was:\n{report:#}");
    assert!(
        paths.contains("apps/web/src/BadName.tsx"),
        "report was:\n{report:#}"
    );
    assert!(paths.contains("packages/ui"), "report was:\n{report:#}");
}

#[test]
fn check_advice_format_renders_guided_output_in_one_command() {
    let project = fixture_path("invalid");

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(&project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("advice")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "stdout was:\n{stdout}");
    assert!(stdout.contains("Assura found 3 structural violation(s)"));
    assert!(stdout.contains("Next:"));
    assert!(stdout.contains("References: AGENTS.md, .agents/skills/, .assura/config.yml"));
}

#[test]
fn check_status_format_supports_general_display_limits() {
    let project = fixture_path("invalid");

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(&project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("status")
        .arg("--min-severity")
        .arg("medium")
        .arg("--max-issues")
        .arg("1")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "stdout was:\n{stdout}");
    assert!(stdout.contains("Assura: 3 violation(s); showing 1 guided item(s)"));
    assert!(stdout.contains("medium+ severity"));
}

#[test]
fn check_agent_format_emits_stable_feedback_for_real_project_fixture() {
    let project = fixture_path("invalid");

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(&project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("agent")
        .arg("--warn")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["schema"], "assura.agent-feedback.v1");
    assert_eq!(json["source"]["command"], "assura check --format agent");
    assert_eq!(json["blocking"], false);
    assert_eq!(json["feedback"][0]["status"], "fail");
    assert_eq!(json["feedback"][0]["violation_count"], 3);
    assert_eq!(json["feedback"][0]["metrics"]["feedback_count"], 3);
    let first_paths = json["feedback"][0]["metrics"]["affected_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(|path| path.as_str().unwrap())
        .collect::<HashSet<_>>();
    assert!(first_paths.contains("apps/web/src/BadName.tsx"));
    assert!(first_paths.contains("packages/ui"));
    assert!(first_paths.contains("scratch.md"));
}

#[test]
fn check_agent_codex_adapter_wraps_real_project_feedback_for_user_prompt_submit() {
    let project = fixture_path("invalid");

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(&project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("agent")
        .arg("--agent")
        .arg("codex")
        .arg("--warn")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        json["hookSpecificOutput"]["hookEventName"],
        "UserPromptSubmit"
    );
    let context = json["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("<assura-feedback>"), "{context}");
    assert!(
        context.contains("Check state: ran assura check --format agent --agent codex"),
        "{context}"
    );
    assert!(context.contains("Blocking: no (--warn)"), "{context}");
    assert!(context.contains("apps/web/src/BadName.tsx"), "{context}");
    assert!(context.contains("packages/ui"), "{context}");
    assert!(context.contains("scratch.md"), "{context}");
    assert!(context.contains("References: AGENTS.md, .agents/skills/, .assura/config.yml"));
}

#[test]
fn check_guided_output_rejects_unknown_minimum_severity() {
    let project = fixture_path("invalid");

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(&project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("advice")
        .arg("--min-severity")
        .arg("urgent")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported minimum severity 'urgent'"),
        "stderr was:\n{stderr}"
    );
}
