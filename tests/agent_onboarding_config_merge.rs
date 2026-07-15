use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn run_assura(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_assura"))
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
fn agent_onboard_merges_existing_config_and_accepts_config_flag() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    let config_path = project.path().join(".assura/config.yml");
    fs::write(
        &config_path,
        r#"version: "2.0"

structure:
  ./:
    extra: true
    CUSTOM.md: exists:0-1

exclude:
  - "custom/**"
"#,
    )
    .unwrap();

    let output = json_from_success(run_assura(&[
        "--config",
        config_path.to_str().unwrap(),
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(output["files"]
        .as_array()
        .expect("files array")
        .iter()
        .any(|item| item["path"] == ".assura/config.yml" && item["action"] == "merge"));

    let merged_config: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(
        merged_config["structure"]["./"]["CUSTOM.md"],
        serde_yaml::Value::String("exists:0-1".to_string())
    );
    assert_eq!(
        merged_config["structure"]["./"]["use"],
        serde_yaml::Value::String("$project-agentic-baseline".to_string())
    );
    assert_eq!(
        merged_config["rules"]["project-agentic-baseline"]["use"],
        serde_yaml::Value::String("$agentic-project".to_string())
    );
    let exclude = merged_config["exclude"]
        .as_sequence()
        .expect("exclude sequence");
    assert!(exclude.contains(&serde_yaml::Value::String("custom/**".to_string())));
    assert!(exclude.contains(&serde_yaml::Value::String(".git/**".to_string())));
}

#[test]
fn agent_onboard_preserves_existing_root_rule_and_reports_wrapper_available() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    let config_path = project.path().join(".assura/config.yml");
    fs::write(
        &config_path,
        r#"version: "2.0"

rules:
  existing-root:
    extra: true

structure:
  ./:
    use:
      - $existing-root
"#,
    )
    .unwrap();

    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(output["rule_recommendations"][0]["status"], "available");
    assert!(output["rule_recommendations"][0]["reason"]
        .as_str()
        .expect("recommendation reason")
        .contains("without replacing the existing root rule"));
    assert!(
        fs::read_to_string(project.path().join(".assura/onboarding/rules.md"))
            .unwrap()
            .contains("Recommendation status: `available`")
    );

    let merged: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(merged["structure"]["./"]["use"][0], "$existing-root");
    assert_eq!(
        merged["rules"]["project-agentic-baseline"]["use"],
        "$agentic-project"
    );
}

#[test]
fn agent_onboard_preserves_colliding_local_wrapper_and_reports_conflict() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura/onboarding")).unwrap();
    let config_path = project.path().join(".assura/config.yml");
    fs::write(
        project.path().join(".assura/onboarding/rules.md"),
        "stale recommendation status",
    )
    .unwrap();
    fs::write(
        &config_path,
        r#"version: "2.0"

rules:
  project-agentic-baseline:
    extra: false

structure:
  ./:
    use: $project-agentic-baseline
"#,
    )
    .unwrap();

    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(output["rule_recommendations"][0]["status"], "conflict");
    assert!(output["rule_recommendations"][0]["reason"]
        .as_str()
        .expect("recommendation reason")
        .contains("preserved for manual review"));
    let rules = fs::read_to_string(project.path().join(".assura/onboarding/rules.md")).unwrap();
    assert!(rules.contains("Recommendation status: `conflict`"));
    assert!(!rules.contains("stale recommendation status"));

    let merged: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(merged["rules"]["project-agentic-baseline"]["extra"], false);
    assert!(merged["rules"]["project-agentic-baseline"]["use"].is_null());
}

#[test]
fn agent_onboard_recognizes_rule_lists_as_applied() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"version: "2.0"

rules:
  project-agentic-baseline:
    use:
      - $agentic-project

structure:
  ./:
    use:
      - $project-agentic-baseline
"#,
    )
    .unwrap();

    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(output["rule_recommendations"][0]["status"], "applied");
}

#[test]
fn agent_onboard_reports_selected_config_without_rewriting_it() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    let selected = project.path().join(".assura/custom.yml");
    fs::write(
        &selected,
        r#"version: "2.0"
structure:
  ./:
    extra: true
"#,
    )
    .unwrap();

    let output = json_from_success(run_assura(&[
        "--config",
        selected.to_str().unwrap(),
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(output["rule_recommendations"][0]["status"], "not-applied");
    assert!(output["rule_recommendations"][0]["reason"]
        .as_str()
        .expect("recommendation reason")
        .contains("selected config does not define"));
    assert!(!fs::read_to_string(&selected)
        .unwrap()
        .contains("$project-agentic-baseline"));
}
