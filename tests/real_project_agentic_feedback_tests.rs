use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

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

#[test]
fn hooks_install_status_and_verify_are_agent_runnable() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".git/hooks")).unwrap();

    let install = Command::new(assura_bin())
        .arg("hooks")
        .arg("install")
        .arg(project.path())
        .output()
        .unwrap();
    assert!(
        install.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    let second_install = Command::new(assura_bin())
        .arg("hooks")
        .arg("install")
        .arg(project.path())
        .output()
        .unwrap();
    assert!(
        second_install.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&second_install.stdout),
        String::from_utf8_lossy(&second_install.stderr)
    );
    assert!(
        String::from_utf8_lossy(&second_install.stdout).contains("already exist"),
        "stdout was:\n{}",
        String::from_utf8_lossy(&second_install.stdout)
    );

    let status = Command::new(assura_bin())
        .arg("hooks")
        .arg("status")
        .arg(project.path())
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );
    assert!(
        String::from_utf8_lossy(&status.stdout).contains("runnable"),
        "stdout was:\n{}",
        String::from_utf8_lossy(&status.stdout)
    );

    let verify = Command::new(assura_bin())
        .arg("hooks")
        .arg("verify")
        .arg(project.path())
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&verify.stdout),
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(
        String::from_utf8_lossy(&verify.stdout).contains("All Assura hooks"),
        "stdout was:\n{}",
        String::from_utf8_lossy(&verify.stdout)
    );
}

#[test]
fn hooks_verify_fails_for_custom_or_broken_hook_state() {
    let project = TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    fs::create_dir_all(&hooks_dir).unwrap();
    fs::write(hooks_dir.join("pre-commit"), "#!/bin/sh\necho custom\n").unwrap();

    let install = Command::new(assura_bin())
        .arg("hooks")
        .arg("install")
        .arg(project.path())
        .output()
        .unwrap();
    assert!(
        install.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&install.stdout),
        String::from_utf8_lossy(&install.stderr)
    );

    let verify = Command::new(assura_bin())
        .arg("hooks")
        .arg("verify")
        .arg(project.path())
        .output()
        .unwrap();
    assert_eq!(verify.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&verify.stdout).contains("not managed by assura"),
        "stdout was:\n{}",
        String::from_utf8_lossy(&verify.stdout)
    );
}
