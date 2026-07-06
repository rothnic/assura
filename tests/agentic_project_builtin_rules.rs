use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn run_assura(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(args)
        .output()
        .expect("assura command runs")
}

fn json_from_success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("command emits JSON")
}

#[test]
fn agentic_project_builtin_rule_enforces_root_guidance_and_skill_tree() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join(".agents/skills/release-maintenance")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"version: "2.0"

structure:
  ./:
    use: "@agentic-project"
    extra: false

exclude:
  - ".git/**"
"#,
    )
    .unwrap();
    fs::write(project.path().join("AGENTS.md"), "# Agent Guidance\n").unwrap();
    fs::write(
        project
            .path()
            .join(".agents/skills/release-maintenance/SKILL.md"),
        "# Release Maintenance\n",
    )
    .unwrap();

    let valid_check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(valid_check["success"], true);

    fs::remove_file(project.path().join("AGENTS.md")).unwrap();
    let missing_agents = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(missing_agents.status.code(), Some(1));
    let missing_agents_json: Value = serde_json::from_slice(&missing_agents.stdout).unwrap();
    assert!(missing_agents_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| {
            item["rule"] == "exists_count"
                && item["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("AGENTS.md"))
        }));

    fs::write(project.path().join("AGENTS.md"), "# Agent Guidance\n").unwrap();
    fs::remove_file(
        project
            .path()
            .join(".agents/skills/release-maintenance/SKILL.md"),
    )
    .unwrap();
    let missing_skill = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(missing_skill.status.code(), Some(1));
    let missing_skill_json: Value = serde_json::from_slice(&missing_skill.stdout).unwrap();
    assert!(missing_skill_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| {
            item["path"] == ".agents/skills/release-maintenance"
                && item["rule"] == "exists_count"
                && item["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("SKILL.md"))
        }));
}
