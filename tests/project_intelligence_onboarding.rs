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

fn init_project_intelligence(project: &TempDir) {
    let output = run_assura(&[
        "init",
        project.path().to_str().unwrap(),
        "--project-intelligence",
        "--no-git-hooks",
    ]);
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn project_intelligence_onboarding_starter_generates_valid_project() {
    let project = TempDir::new().unwrap();
    init_project_intelligence(&project);

    let config = fs::read_to_string(project.path().join(".assura/config.yml")).unwrap();
    assert!(config.contains("models:"));
    assert!(config.contains("collections:"));
    assert!(config.contains("relations:"));
    assert!(config.contains("docs/goals/*.md"));
    assert!(project
        .path()
        .join("schemas/project-intelligence-starter.schema.json")
        .is_file());
    assert!(project
        .path()
        .join("docs/goals/goal_project_intelligence_starter.md")
        .is_file());
    assert!(project
        .path()
        .join("docs/examples/project-intelligence-broken-goal.md")
        .is_file());

    let check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(check["success"], true);
    assert_eq!(check["violations"].as_array().unwrap().len(), 0);

    let search = json_from_success(run_assura(&[
        "content",
        "search",
        "Adopt Project Intelligence",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(search["matches"]
        .as_array()
        .expect("matches array")
        .iter()
        .any(|item| item["instance_id"] == "goal-project-intelligence-starter"));

    let expanded = json_from_success(run_assura(&[
        "content",
        "expand",
        "goals",
        "goal-project-intelligence-starter",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    let related = expanded["related"].as_array().expect("related array");
    assert!(related
        .iter()
        .any(|item| item["path"] == "specs/spec_project_intelligence_starter.json"));
    assert!(related
        .iter()
        .any(|item| item["path"] == "docs/decisions/adr_project_intelligence_starter.json"));
}

#[test]
fn project_intelligence_onboarding_starter_proves_broken_state_diagnostics() {
    let project = TempDir::new().unwrap();
    init_project_intelligence(&project);

    fs::copy(
        project
            .path()
            .join("docs/examples/project-intelligence-broken-goal.md"),
        project
            .path()
            .join("docs/goals/goal_project_intelligence_missing_context.md"),
    )
    .unwrap();

    let check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(check.status.code(), Some(1));
    let check_json: Value = serde_json::from_slice(&check.stdout).expect("check JSON");
    assert!(check_json["violations"]
        .as_array()
        .expect("violations array")
        .iter()
        .any(|item| item["rule"] == "content_runtime:missing_reference"));

    let missing = json_from_success(run_assura(&[
        "content",
        "missing-relations",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(missing["missing_relations"]
        .as_array()
        .expect("missing relations")
        .iter()
        .any(|item| item["target_instance_id"] == "missing-spec-project-context"));

    let diagnostics = json_from_success(run_assura(&[
        "content",
        "agent-query",
        "diagnostics",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(
        diagnostics["schema"],
        "assura.project-intelligence.agent-query.v1"
    );
    assert!(diagnostics["response"]["diagnostics"]
        .as_array()
        .expect("diagnostics array")
        .iter()
        .any(|item| {
            item["rule"] == "content_runtime:missing_reference" && item["severity"] == "high"
        }));
}

#[test]
fn project_intelligence_onboarding_starter_refuses_to_overwrite_files_without_force() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join("schemas")).unwrap();
    fs::write(
        project
            .path()
            .join("schemas/project-intelligence-starter.schema.json"),
        "{}\n",
    )
    .unwrap();

    let output = run_assura(&[
        "init",
        project.path().to_str().unwrap(),
        "--project-intelligence",
        "--no-git-hooks",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("already exists"));
}
