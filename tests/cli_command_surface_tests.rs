use std::fs;
use std::process::Command;

use serde_json::Value;
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn assura_full_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura-full")
}

#[test]
fn check_help_uses_lightweight_primary_path() {
    let output = Command::new(assura_bin())
        .arg("check")
        .arg("--help")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Fast structure validation entrypoint."),
        "stdout was:\n{stdout}"
    );
    assert!(
        stdout.contains("--format <FORMAT>"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn companion_help_can_render_primary_command_name() {
    let output = Command::new(assura_full_bin())
        .env("ASSURA_CLI_BIN_NAME", "assura")
        .arg("--help")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage: assura "), "stdout was:\n{stdout}");
    assert!(
        !stdout.contains("Usage: assura-full"),
        "stdout was:\n{stdout}"
    );
}

#[test]
fn init_creates_supported_structure_config() {
    let project = TempDir::new().unwrap();

    let output = Command::new(assura_bin())
        .arg("init")
        .arg(project.path())
        .arg("--no-git-hooks")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let config_path = project.path().join(".assura/config.yml");
    assert!(config_path.is_file());
    let config = fs::read_to_string(config_path).unwrap();
    assert!(config.contains("structure:"));

    fs::write(project.path().join("README.md"), "# Example\n").unwrap();
    let check = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("json")
        .output()
        .unwrap();

    assert!(
        check.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn init_refuses_to_overwrite_without_force() {
    let project = TempDir::new().unwrap();
    let assura_dir = project.path().join(".assura");
    fs::create_dir(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), "structure: {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("init")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
}

#[test]
fn migrate_is_exposed_as_clap_command() {
    let project = TempDir::new().unwrap();
    let ls_lint_path = project.path().join(".ls-lint.yml");
    let assura_path = project.path().join(".assura/config.yml");
    fs::write(
        &ls_lint_path,
        r#"
ls:
  .rs: snake_case
"#,
    )
    .unwrap();

    let output = Command::new(assura_bin())
        .arg("migrate")
        .arg(&ls_lint_path)
        .arg("--output")
        .arg(&assura_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(assura_path.is_file());
    let generated = fs::read_to_string(&assura_path).unwrap();
    assert!(generated.contains("structure:"));
    assert!(generated.contains(".assura/**"));

    fs::write(project.path().join("good_name.rs"), "fn main() {}\n").unwrap();
    let check = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert!(
        check.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}

#[test]
fn info_is_exposed_as_clap_command() {
    let project = TempDir::new().unwrap();
    let assura_dir = project.path().join(".assura");
    fs::create_dir(&assura_dir).unwrap();
    let config_path = assura_dir.join("config.yml");
    fs::write(
        &config_path,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
"#,
    )
    .unwrap();

    let output = Command::new(assura_bin())
        .arg("info")
        .arg(&config_path)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("Structure roots: 1"));
}

#[test]
fn watch_returns_check_failure_for_invalid_project() {
    let project = TempDir::new().unwrap();
    let assura_dir = project.path().join(".assura");
    fs::create_dir(&assura_dir).unwrap();
    fs::write(
        assura_dir.join("config.yml"),
        r#"
structure:
  ./:
    files:
      naming: kebab-case
"#,
    )
    .unwrap();
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();

    let output = Command::new(assura_bin())
        .arg("watch")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("one-shot validation"),
        "stdout was:\n{stdout}"
    );
    assert!(stdout.contains("file_naming"), "stdout was:\n{stdout}");
}

#[test]
fn check_agent_format_with_codex_adapter_emits_user_prompt_submit_json() {
    let project = bounded_structure_project();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("agent")
        .arg("--agent")
        .arg("codex")
        .arg("--warn")
        .arg("--min-severity")
        .arg("medium")
        .arg("--max-issues")
        .arg("1")
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
    assert!(context.contains("file_naming"), "{context}");
    assert!(context.contains("showing 1 guided item(s)"), "{context}");
    assert!(
        context.contains("violation(s) were hidden by display filters"),
        "{context}"
    );
}

#[test]
fn check_agent_format_emits_generic_feedback_json() {
    let project = passing_structure_project();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("agent")
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
    assert_eq!(json["feedback"][0]["status"], "pass");
    assert_eq!(json["blocking"], true);
}

#[test]
fn check_agent_format_with_codex_adapter_applies_min_severity_filter() {
    let project = mixed_severity_project();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("agent")
        .arg("--agent")
        .arg("codex")
        .arg("--warn")
        .arg("--min-severity")
        .arg("high")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let context = json["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.contains("required_file"), "{context}");
    assert!(!context.contains("file_naming"), "{context}");
    assert!(context.contains("severity >= high"), "{context}");
}

#[test]
fn check_agent_format_with_codex_adapter_blocks_without_warn() {
    let project = invalid_structure_project();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("agent")
        .arg("--agent")
        .arg("codex")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    let context = json["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        context.contains("Blocking: yes (validation failures exit 1)"),
        "{context}"
    );
    assert!(context.contains("</assura-feedback>"), "{context}");
}

#[test]
fn check_rejects_old_codex_hook_format() {
    let project = passing_structure_project();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("codex-hook")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("unsupported format 'codex-hook'"),
        "stderr was:\n{stderr}"
    );
}

#[test]
fn status_rejects_agent_format() {
    let project = passing_structure_project();

    let output = Command::new(assura_bin())
        .arg("status")
        .arg(project.path())
        .arg("--format")
        .arg("agent")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid value"), "stderr was:\n{stderr}");
}

#[test]
fn agent_adapter_requires_agent_format() {
    let project = passing_structure_project();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .arg("--format")
        .arg("status")
        .arg("--agent")
        .arg("codex")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--agent requires --format agent"),
        "stderr was:\n{stderr}"
    );
}

fn passing_structure_project() -> TempDir {
    let project = structure_project(
        r#"
structure:
  ./:
    files:
      naming: kebab-case
"#,
    );
    fs::write(project.path().join("good-name.rs"), "fn main() {}\n").unwrap();
    project
}

fn bounded_structure_project() -> TempDir {
    let project = invalid_structure_project();
    fs::write(project.path().join("AnotherBad.rs"), "fn main() {}\n").unwrap();
    project
}

fn mixed_severity_project() -> TempDir {
    let project = structure_project(
        r#"
structure:
  ./:
    files:
      naming: kebab-case
      severity: low
    children:
      required-dir/:
        files:
          required:
            - must-exist.md
          severity: high
"#,
    );
    fs::create_dir(project.path().join("required-dir")).unwrap();
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();
    project
}

fn invalid_structure_project() -> TempDir {
    let project = structure_project(
        r#"
structure:
  ./:
    files:
      naming: kebab-case
"#,
    );
    fs::write(project.path().join("BadName.rs"), "fn main() {}\n").unwrap();
    project
}

fn structure_project(config: &str) -> TempDir {
    let project = TempDir::new().unwrap();
    let assura_dir = project.path().join(".assura");
    fs::create_dir(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
    project
}
