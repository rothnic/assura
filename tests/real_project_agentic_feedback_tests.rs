use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn normalized_json_path(value: &Value) -> String {
    value.as_str().unwrap().replace('\\', "/")
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

fn run_agent_check(project: &Path, args: &[&str]) -> (std::process::ExitStatus, serde_json::Value) {
    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("agent")
        .args(args)
        .output()
        .unwrap();

    let report = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "failed to parse agent output as json: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    });
    (output.status, report)
}

fn copy_dir_all(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let source = entry.path();
        let target = to.join(entry.file_name());
        if source.is_dir() {
            copy_dir_all(&source, &target);
        } else {
            fs::copy(&source, &target).unwrap();
        }
    }
}

fn apply_scripted_same_turn_fix(project: &Path) {
    for path in [
        "scratch.md",
        "draft-plan.md",
        "apps/web/notes.txt",
        "apps/web/src/legacy.js",
        "apps/web/src/old-helper.js",
        "apps/web/tests/BadSpec.ts",
        "apps/web/tests/HomePage.test.ts",
    ] {
        fs::remove_file(project.join(path)).unwrap();
    }
    fs::rename(
        project.join("apps/web/src/BadName.tsx"),
        project.join("apps/web/src/bad-name.tsx"),
    )
    .unwrap();
    fs::rename(
        project.join("apps/web/src/AnotherBad.tsx"),
        project.join("apps/web/src/another-bad.tsx"),
    )
    .unwrap();
    fs::write(
        project.join("packages/ui/AGENTS.md"),
        "# UI Agent Guidance\n",
    )
    .unwrap();
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
        .map(|violation| normalized_json_path(&violation["path"]))
        .collect::<HashSet<_>>();

    assert_eq!(status.code(), Some(1), "report was:\n{report:#}");
    assert_eq!(report["success"], false, "report was:\n{report:#}");
    assert_eq!(violations.len(), 12, "report was:\n{report:#}");
    assert!(rules.contains("unexpected_file"), "report was:\n{report:#}");
    assert!(rules.contains("file_naming"), "report was:\n{report:#}");
    assert!(rules.contains("exists_count"), "report was:\n{report:#}");
    assert!(rules.contains("extension"), "report was:\n{report:#}");
    assert!(rules.contains("forbidden_file"), "report was:\n{report:#}");
    assert!(paths.contains("scratch.md"), "report was:\n{report:#}");
    assert!(paths.contains("draft-plan.md"), "report was:\n{report:#}");
    assert!(
        paths.contains("apps/web/notes.txt"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("apps/web/src/BadName.tsx"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("apps/web/src/AnotherBad.tsx"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("apps/web/src/legacy.js"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("apps/web/src/old-helper.js"),
        "report was:\n{report:#}"
    );
    assert!(
        paths.contains("apps/web/tests/BadSpec.ts"),
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
        .arg("--warn")
        .arg("--min-severity")
        .arg("medium")
        .arg("--max-issues")
        .arg("11")
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    let normalized_stdout = stdout.replace('\\', "/");
    assert!(output.status.success(), "stdout was:\n{stdout}");
    assert!(normalized_stdout.contains("Assura found 12 structural violation(s)"));
    assert!(normalized_stdout.contains("showing 11 guided item(s)"));
    assert!(normalized_stdout.contains("draft-plan.md [unexpected_file:critical]"));
    assert!(normalized_stdout.contains("packages/ui [exists_count:high]"));
    assert!(normalized_stdout.contains("Next:"));
    assert!(
        normalized_stdout.contains("References: AGENTS.md, .agents/skills/, .assura/config.yml")
    );
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
    assert!(stdout.contains("Assura: 12 violation(s); showing 1 guided item(s)"));
    assert!(stdout.contains("medium+ severity"));
}

#[test]
fn check_agent_format_emits_stable_priority_feedback_for_real_project_fixture() {
    let project = fixture_path("invalid");

    let (status, json) = run_agent_check(
        &project,
        &["--warn", "--min-severity", "medium", "--max-issues", "11"],
    );

    assert!(status.success(), "feedback was:\n{json:#}");
    assert_eq!(json["schema"], "assura.agent-feedback.v1");
    assert_eq!(json["source"]["command"], "assura check --format agent");
    assert_eq!(json["blocking"], false);
    let feedback = json["feedback"].as_array().unwrap();
    assert_eq!(feedback.len(), 1, "feedback was:\n{json:#}");
    assert_eq!(feedback[0]["status"], "fail");
    assert_eq!(feedback[0]["violation_count"], 12);
    assert_eq!(feedback[0]["suppressed_violation_count"], 1);
    assert_eq!(feedback[0]["metrics"]["feedback_count"], 11);

    let messages = feedback[0]["messages"].as_array().unwrap();
    let shown_paths = messages
        .iter()
        .map(|message| normalized_json_path(&message["path"]))
        .collect::<HashSet<_>>();
    let severities = messages
        .iter()
        .map(|message| message["severity"].as_str().unwrap())
        .collect::<Vec<_>>();
    let priority_keys = messages
        .iter()
        .map(|message| {
            (
                message["severity"].as_str().unwrap(),
                normalized_json_path(&message["path"]),
                message["rule"].as_str().unwrap(),
            )
        })
        .collect::<Vec<_>>();
    let medium_count = severities
        .iter()
        .filter(|severity| **severity == "medium")
        .count();

    assert_eq!(&severities[0..4], ["critical", "critical", "high", "high"]);
    assert_eq!(
        priority_keys,
        [
            ("critical", "draft-plan.md", "unexpected_file"),
            ("critical", "scratch.md", "unexpected_file"),
            ("high", "apps/web/notes.txt", "unexpected_file"),
            ("high", "packages/ui", "exists_count"),
            ("medium", "apps/web/src/AnotherBad.tsx", "file_naming"),
            ("medium", "apps/web/src/BadName.tsx", "file_naming"),
            ("medium", "apps/web/src/legacy.js", "extension"),
            ("medium", "apps/web/src/legacy.js", "forbidden_file"),
            ("medium", "apps/web/src/old-helper.js", "extension"),
            ("medium", "apps/web/src/old-helper.js", "forbidden_file"),
            ("medium", "apps/web/tests/BadSpec.ts", "file_naming"),
        ]
        .map(|(severity, path, rule)| (severity, path.to_string(), rule))
        .to_vec(),
        "feedback was:\n{json:#}"
    );
    assert_eq!(medium_count, 7, "feedback was:\n{json:#}");
    assert!(shown_paths.contains("draft-plan.md"));
    assert!(shown_paths.contains("scratch.md"));
    assert!(shown_paths.contains("apps/web/notes.txt"));
    assert!(shown_paths.contains("packages/ui"));
    assert!(shown_paths.contains("apps/web/src/BadName.tsx"));
    for message in messages {
        assert!(
            message["corrective_context"]
                .as_str()
                .is_some_and(|context| !context.is_empty()),
            "message should include corrective context: {message:#}"
        );
        assert!(
            message["guidance"]
                .as_array()
                .is_some_and(|guidance| !guidance.is_empty()),
            "message should include actionable guidance: {message:#}"
        );
    }

    let first_paths = feedback[0]["metrics"]["affected_paths"]
        .as_array()
        .unwrap()
        .iter()
        .map(normalized_json_path)
        .collect::<HashSet<_>>();
    assert!(first_paths.contains("draft-plan.md"));
    assert!(first_paths.contains("scratch.md"));
    assert!(first_paths.contains("apps/web/notes.txt"));
    assert!(first_paths.contains("apps/web/src/BadName.tsx"));
    assert!(first_paths.contains("packages/ui"));
}

#[test]
fn check_agent_warn_mode_is_advisory_and_gate_mode_blocks() {
    let project = fixture_path("invalid");

    let (warn_status, warn_json) =
        run_agent_check(&project, &["--warn", "--min-severity", "medium"]);
    assert!(
        warn_status.success(),
        "warn mode should report without blocking:\n{warn_json:#}"
    );
    assert_eq!(warn_json["blocking"], false);
    assert_eq!(warn_json["feedback"][0]["status"], "fail");

    let (gate_status, gate_json) = run_agent_check(&project, &["--min-severity", "medium"]);
    assert_eq!(
        gate_status.code(),
        Some(1),
        "gate mode should block on configured medium+ findings:\n{gate_json:#}"
    );
    assert_eq!(gate_json["blocking"], true);
    assert_eq!(gate_json["feedback"][0]["status"], "fail");
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
        .arg("--min-severity")
        .arg("medium")
        .arg("--max-issues")
        .arg("11")
        .output()
        .unwrap();
    let repeat = Command::new(assura_bin())
        .arg("check")
        .arg(&project)
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--format")
        .arg("agent")
        .arg("--agent")
        .arg("codex")
        .arg("--warn")
        .arg("--min-severity")
        .arg("medium")
        .arg("--max-issues")
        .arg("11")
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
    let normalized_context = context.replace('\\', "/");
    assert_eq!(
        output.stdout, repeat.stdout,
        "Codex output should be deterministic"
    );
    assert!(
        context.len() < 24 * 1024,
        "Codex additionalContext should stay under 24 KiB; got {} bytes",
        context.len()
    );
    assert!(
        normalized_context.contains("<assura-feedback>"),
        "{context}"
    );
    assert!(
        normalized_context.contains("Check state: ran assura check --format agent --agent codex"),
        "{context}"
    );
    assert!(
        normalized_context.contains("Blocking: no (--warn)"),
        "{context}"
    );
    assert!(normalized_context.contains("draft-plan.md"), "{context}");
    assert!(
        normalized_context.contains("apps/web/notes.txt"),
        "{context}"
    );
    assert!(
        normalized_context.contains("apps/web/src/BadName.tsx"),
        "{context}"
    );
    assert!(normalized_context.contains("packages/ui"), "{context}");
    assert!(normalized_context.contains("scratch.md"), "{context}");
    assert!(
        normalized_context.contains("References: AGENTS.md, .agents/skills/, .assura/config.yml")
    );
}

#[test]
fn scripted_same_turn_fix_corrects_at_least_ten_seeded_violations() {
    let temp = TempDir::new().unwrap();
    let work = temp.path().join("work");
    copy_dir_all(&fixture_path("invalid"), &work);

    let (before_status, before_report) = run_check(&work);
    assert_eq!(
        before_status.code(),
        Some(1),
        "report was:\n{before_report:#}"
    );
    let before_count = before_report["violations"].as_array().unwrap().len();
    assert_eq!(before_count, 12, "report was:\n{before_report:#}");

    apply_scripted_same_turn_fix(&work);

    let (after_status, after_report) = run_check(&work);
    assert!(after_status.success(), "report was:\n{after_report:#}");
    let after_count = after_report["violations"].as_array().unwrap().len();
    assert_eq!(after_count, 0);
    assert!(
        before_count.saturating_sub(after_count) >= 10,
        "scripted same-turn fixer should correct at least 10 seeded violations"
    );
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
