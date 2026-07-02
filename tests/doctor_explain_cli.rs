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

fn json_from_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "command emits JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn write_config(project: &TempDir, config: &str) {
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(project.path().join(".assura/config.yml"), config).unwrap();
}

#[test]
fn doctor_reports_clean_check_with_inactive_and_unwired_model_gap() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
exclude:
  - .assura/**
"#,
    );
    fs::create_dir_all(project.path().join(".assura/models")).unwrap();
    fs::write(
        project.path().join(".assura/models/draft.schema.json"),
        "{}\n",
    )
    .unwrap();

    let check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(check["success"], true);

    let doctor = json_from_success(run_assura(&[
        "doctor",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(doctor["schema"], "assura.project-doctor.v1");
    assert_eq!(doctor["check"]["status"], "pass");
    assert!(doctor["inactive"]
        .as_array()
        .expect("inactive array")
        .iter()
        .any(|item| item["name"] == "content_models" && item["status"] == "inactive"));
    assert!(doctor["inactive"]
        .as_array()
        .expect("inactive array")
        .iter()
        .any(|item| item["name"] == "search_chunks" && item["status"] == "unchecked"));
    assert!(doctor["gaps"]
        .as_array()
        .expect("gaps array")
        .iter()
        .any(|item| item["name"] == "draft_models_unwired"));

    let agent = json_from_success(run_assura(&[
        "doctor",
        project.path().to_str().unwrap(),
        "--format",
        "agent",
    ]));
    assert_eq!(agent["schema"], "assura.project-doctor.agent.v1");
    assert!(agent["next_actions"]
        .as_array()
        .expect("next actions")
        .iter()
        .any(|item| item["follow_up"] == "assura explain <path> --format json"));
}

#[test]
fn doctor_exits_nonzero_and_reports_blocking_violation_context() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      exists:
        README.md: 1
exclude:
  - .assura/**
"#,
    );

    let output = run_assura(&[
        "doctor",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let doctor = json_from_output(&output);
    assert_eq!(doctor["check"]["status"], "fail");
    assert!(doctor["blocking_violations"]
        .as_array()
        .expect("blocking violations")
        .iter()
        .any(|violation| violation["rule"] == "exists_count"
            && violation["path"] == "."
            && violation["blocking"] == true));

    let agent_output = run_assura(&[
        "doctor",
        project.path().to_str().unwrap(),
        "--format",
        "agent",
    ]);
    assert_eq!(agent_output.status.code(), Some(1));
    let agent = json_from_output(&agent_output);
    assert_eq!(agent["check_status"], "fail");
    assert!(agent["blocking_violations"]
        .as_array()
        .expect("blocking violations")
        .iter()
        .any(|violation| violation["rule"] == "exists_count"));
}

#[test]
fn explain_reports_inherited_scope_and_source_markdown_skips() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      naming: kebab-case
    markdown:
      require_frontmatter: true
    children:
      src/:
        files:
          naming_patterns:
            "*.rs": snake_case
      isolated/:
        inherit: false
        files:
          naming: kebab-case
          severity: low
        markdown:
          rules:
            markdown_engine:
              severity: high
"#,
    );
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::create_dir_all(project.path().join("isolated")).unwrap();
    fs::write(project.path().join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        project.path().join("isolated/readme.md"),
        "---\n---\n# Readme\n",
    )
    .unwrap();
    fs::write(project.path().join("README.md"), "---\n---\n# Readme\n").unwrap();

    let source = json_from_success(run_assura(&[
        "explain",
        project.path().join("src/main.rs").to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(source["schema"], "assura.path-explain.v1");
    assert_eq!(source["relative_path"], "src/main.rs");
    assert!(source["applied_scopes"]
        .as_array()
        .expect("applied scopes")
        .iter()
        .any(|scope| scope["match_kind"] == "inherited"));
    assert!(source["skipped_checks"]
        .as_array()
        .expect("skipped checks")
        .iter()
        .any(|skip| skip["name"] == "markdown_checks" && skip["status"] == "not_applicable"));

    let markdown = json_from_success(run_assura(&[
        "explain",
        project.path().join("README.md").to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(
        markdown["effective_rules"]["markdown_require_frontmatter"],
        true
    );
    assert!(markdown["skipped_checks"]
        .as_array()
        .expect("skipped checks")
        .iter()
        .any(|skip| skip["name"] == "binary_read"
            && skip["status"] == "read_as_text_when_markdown_policy_requires"));

    let isolated = json_from_success(run_assura(&[
        "explain",
        project.path().join("isolated/readme.md").to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(isolated["applied_scopes"]
        .as_array()
        .expect("applied scopes")
        .iter()
        .any(|scope| scope["scope"] == "isolated"
            && scope["inheritance_reset"] == true
            && scope["inherits_parent"] == false));
    assert_eq!(isolated["effective_rules"]["file_severity"], "low");
    assert!(isolated["effective_rules"]["markdown_rule_severities"]
        .as_array()
        .expect("markdown severities")
        .iter()
        .any(|entry| entry == "markdown_engine:high"));

    let isolated_agent = json_from_success(run_assura(&[
        "explain",
        project.path().join("isolated/readme.md").to_str().unwrap(),
        "--format",
        "agent",
    ]));
    assert!(isolated_agent["applied_scopes"]
        .as_array()
        .expect("applied scopes")
        .iter()
        .any(|scope| scope["scope"] == "isolated"
            && scope["inheritance_reset"] == true
            && scope["inherits_parent"] == false));
    assert_eq!(isolated_agent["effective_rules"]["file_severity"], "low");
}

#[test]
fn explain_reports_skill_path_scope_without_special_cases() {
    let project = TempDir::new().unwrap();
    json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));

    let skill_path = project
        .path()
        .join(".agents/skills/assura-project-maintenance/SKILL.md");
    let explain = json_from_success(run_assura(&[
        "explain",
        skill_path.to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(
        explain["relative_path"],
        ".agents/skills/assura-project-maintenance/SKILL.md"
    );
    assert!(explain["applied_scopes"]
        .as_array()
        .expect("applied scopes")
        .iter()
        .any(|scope| scope["scope"] == ".agents/skills/{skill}" && scope["match_kind"] == "exact"));
    assert!(explain["next_actions"]
        .as_array()
        .expect("next actions")
        .iter()
        .any(|item| item["follow_up"] == "assura check --format json ."));
}

#[test]
fn explain_reports_excluded_generated_binary_path() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
exclude:
  - generated/**
"#,
    );
    fs::create_dir_all(project.path().join("generated")).unwrap();
    fs::write(project.path().join("generated/output.bin"), [0_u8, 1, 2, 3]).unwrap();

    let explain = json_from_success(run_assura(&[
        "explain",
        project
            .path()
            .join("generated/output.bin")
            .to_str()
            .unwrap(),
        "--format",
        "agent",
    ]));
    assert_eq!(explain["schema"], "assura.path-explain.agent.v1");
    assert_eq!(explain["excluded"], true);
    assert!(explain["skipped_checks"]
        .as_array()
        .expect("skipped checks")
        .iter()
        .any(|skip| skip["name"] == "all_structure_checks" && skip["status"] == "skipped"));
}

#[test]
fn onboarding_packet_uses_project_doctor_schema() {
    let project = TempDir::new().unwrap();
    json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));

    let doctor: Value = serde_json::from_str(
        &fs::read_to_string(project.path().join(".assura/onboarding/doctor.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(doctor["schema"], "assura.project-doctor.v1");
    assert!(doctor["inactive"]
        .as_array()
        .expect("inactive array")
        .iter()
        .any(|item| item["name"] == "content_models"));
    assert!(doctor["next_actions"]
        .as_array()
        .expect("next actions")
        .iter()
        .any(|item| item["follow_up"] == "assura explain <path> --format json"));
}
