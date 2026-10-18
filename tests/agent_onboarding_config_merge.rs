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
        merged_config["structure"]["CUSTOM.md"],
        serde_yaml::Value::String("exists:0-1".to_string())
    );
    assert_eq!(
        merged_config["structure"]["AGENTS.md"],
        serde_yaml::Value::String("exists:1 | $agent-entrypoint".to_string())
    );
    assert_eq!(
        merged_config["rules"]["agent-entrypoint"]["max_lines"],
        serde_yaml::Value::Number(160.into())
    );
    let exclude = merged_config["exclude"]
        .as_sequence()
        .expect("exclude sequence");
    assert!(exclude.contains(&serde_yaml::Value::String("custom/**".to_string())));
    assert!(exclude.contains(&serde_yaml::Value::String(".git/**".to_string())));
    assert!(!fs::read_to_string(&config_path)
        .unwrap()
        .contains("$agentic-project"));
}

#[test]
fn agent_onboard_preserves_existing_root_rule_and_applies_project_owned_recipes() {
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
    assert_eq!(output["rule_recommendations"][0]["status"], "applied");
    assert!(output["rule_recommendations"][0]["reason"]
        .as_str()
        .expect("recommendation reason")
        .contains("editable agentic-core"));
    assert!(
        fs::read_to_string(project.path().join(".assura/onboarding/rules.md"))
            .unwrap()
            .contains("Recommendation status: `applied`")
    );

    let merged: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(merged["structure"]["use"][0], "$existing-root");
    assert_eq!(
        merged["structure"]["AGENTS.md"],
        "exists:1 | $agent-entrypoint"
    );
}

#[test]
fn agent_onboard_preserves_colliding_recipe_value_and_reports_conflict() {
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
  agent-entrypoint:
    max_lines: 999

structure:
  AGENTS.md: $agent-entrypoint
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
    let profile: Value = serde_json::from_str(
        &fs::read_to_string(
            project
                .path()
                .join(".assura/onboarding/profile-selection.json"),
        )
        .expect("specialization profile"),
    )
    .expect("valid specialization profile JSON");
    assert!(profile["conflicts"]
        .as_array()
        .expect("specialization conflicts")
        .iter()
        .any(|conflict| conflict["kind"] == "recommended_rule"));
    assert!(output["inactive"]
        .as_array()
        .expect("specialization state")
        .iter()
        .any(|item| item["name"] == "project_specialization"
            && item["status"] == "conflict_requires_user"));

    let merged: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(merged["rules"]["agent-entrypoint"]["max_lines"], 999);
    assert!(merged["rules"]["skill-entrypoint"].is_mapping());
}

#[test]
fn agent_onboard_reports_existing_agents_closure_conflicts_without_relaxing_it() {
    let project = TempDir::new().unwrap();
    let project_policy = project.path().join(".agents/project-policy.json");
    let custom_skill = project.path().join(".agents/skills/custom-ops/SKILL.md");
    let colliding_assura_skill = project
        .path()
        .join(".agents/skills/assura-structure-fit/SKILL.md");
    fs::create_dir_all(project_policy.parent().unwrap()).unwrap();
    fs::create_dir_all(custom_skill.parent().unwrap()).unwrap();
    fs::create_dir_all(colliding_assura_skill.parent().unwrap()).unwrap();
    fs::write(&project_policy, "{\"policy\":\"project-owned\"}\n").unwrap();
    fs::write(
        &custom_skill,
        "# Existing custom skill\n\n## Local workflow\n",
    )
    .unwrap();
    fs::write(
        &colliding_assura_skill,
        "# Project-owned skill at an Assura path\n",
    )
    .unwrap();

    let config_path = project.path().join(".assura/config.yml");
    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
    fs::write(
        &config_path,
        r#"version: "2.0"

rules:
  closed-entry:
    exists: 0
    severity: medium
  closed:
    ./*/: $closed-entry
    ./*: $closed-entry

structure:
  .agents/:
    ./: exists:0-1 | $closed
"#,
    )
    .unwrap();

    let output = run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "onboarding must not claim a pass"
    );
    let report: Value = serde_json::from_slice(&output.stdout).expect("JSON onboarding report");
    assert_eq!(report["rule_recommendations"][0]["status"], "conflict");
    let reason = report["rule_recommendations"][0]["reason"]
        .as_str()
        .expect("conflict reason");
    assert!(
        reason.contains(".agents"),
        "reason should identify the affected policy scope: {reason}"
    );
    assert!(
        reason.contains("assura check"),
        "reason should give a recovery action: {reason}"
    );
    assert_eq!(report["verified"][0]["status"], "fail");
    assert_eq!(
        fs::read_to_string(&project_policy).unwrap(),
        "{\"policy\":\"project-owned\"}\n"
    );
    assert_eq!(
        fs::read_to_string(&custom_skill).unwrap(),
        "# Existing custom skill\n\n## Local workflow\n"
    );
    assert_eq!(
        fs::read_to_string(&colliding_assura_skill).unwrap(),
        "# Project-owned skill at an Assura path\n"
    );
    assert!(report["files"]
        .as_array()
        .expect("file actions")
        .iter()
        .any(
            |item| item["path"] == ".agents/skills/assura-structure-fit/SKILL.md"
                && item["action"] == "conflict"
        ));

    let merged: serde_yaml::Value =
        serde_yaml::from_str(&fs::read_to_string(config_path).unwrap()).unwrap();
    assert_eq!(merged["rules"]["closed-entry"]["exists"], 0);
    assert_eq!(
        merged["structure"][".agents/"]["./"],
        "exists:0-1 | $closed"
    );
}

#[test]
fn agent_onboard_preserves_unmanaged_guidance_conventions() {
    let project = TempDir::new().unwrap();
    let agents = project.path().join("AGENTS.md");
    let custom_skill = project.path().join(".agents/skills/ProjectX/SKILL.md");
    fs::create_dir_all(custom_skill.parent().unwrap()).unwrap();
    let agents_contents = "# Team Handbook\n\nOur headings and workflow are project-specific.\n";
    let skill_contents =
        "# Deployment Notes\n\n## Team-owned steps\n\nNo Assura frontmatter here.\n";
    fs::write(&agents, agents_contents).unwrap();
    fs::write(&custom_skill, skill_contents).unwrap();

    let onboarding = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(onboarding["verified"][0]["status"], "pass");
    assert_eq!(fs::read_to_string(&agents).unwrap(), agents_contents);
    assert_eq!(fs::read_to_string(&custom_skill).unwrap(), skill_contents);

    let check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(check["success"], true);
    assert!(!check["violations"]
        .as_array()
        .expect("check violations")
        .iter()
        .any(|item| item["path"] == "AGENTS.md"
            || item["path"] == ".agents/skills/ProjectX/SKILL.md"));

    let before = fs::read_to_string(&custom_skill).unwrap();
    let second = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(second["verified"][0]["status"], "pass");
    assert_eq!(fs::read_to_string(custom_skill).unwrap(), before);
}

#[test]
fn agent_onboard_keeps_strict_defaults_for_an_empty_project() {
    let project = TempDir::new().unwrap();
    let onboarding = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(onboarding["verified"][0]["status"], "pass");
    assert_eq!(onboarding["rule_recommendations"][0]["status"], "applied");

    let config: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        config["structure"][".agents/"]["./"],
        "exists:0-1 | $closed"
    );
    assert_eq!(
        config["extensions"]["agent_guidance"][0]["skill_paths"][0],
        ".agents/skills/*/SKILL.md"
    );
}

#[test]
fn agent_onboard_keeps_strict_defaults_for_empty_expected_agent_directories() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".agents/skills")).unwrap();

    let onboarding = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(onboarding["rule_recommendations"][0]["status"], "applied");
    let config: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        config["structure"][".agents/"]["./"],
        "exists:0-1 | $closed"
    );
    assert_eq!(
        config["structure"][".agents/"]["skills/"]["./"],
        "exists:0-1"
    );
    assert_eq!(
        config["extensions"]["agent_guidance"][0]["skill_paths"][0],
        ".agents/skills/*/SKILL.md"
    );
}

#[cfg(unix)]
#[test]
fn agent_onboard_does_not_follow_dangling_assura_skill_symlink() {
    use std::os::unix::fs::symlink;

    let project = TempDir::new().unwrap();
    let destination = project
        .path()
        .join(".agents/skills/assura-project-maintenance/SKILL.md");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    let target = project
        .path()
        .parent()
        .unwrap()
        .join(format!("nu02-dangling-target-{}", std::process::id()));
    let _ = fs::remove_file(&target);
    let link_text = target.to_string_lossy().into_owned();
    symlink(&target, &destination).unwrap();

    let output = run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rule_recommendations"][0]["status"], "conflict");
    assert!(report["files"].as_array().unwrap().iter().any(|item| {
        item["path"] == ".agents/skills/assura-project-maintenance/SKILL.md"
            && item["action"] == "conflict"
    }));
    assert!(!target.exists(), "dangling link target must remain absent");
    assert!(fs::symlink_metadata(&destination)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(
        fs::read_link(destination).unwrap().to_string_lossy(),
        link_text
    );
}

#[cfg(unix)]
#[test]
fn agent_onboard_preserves_bytes_behind_assura_skill_symlink() {
    use std::os::unix::fs::symlink;

    let project = TempDir::new().unwrap();
    let destination = project
        .path()
        .join(".agents/skills/assura-project-maintenance/SKILL.md");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    let external = TempDir::new().unwrap();
    let target = external.path().join("user-skill.md");
    fs::write(&target, b"user-owned bytes\n").unwrap();
    symlink(&target, &destination).unwrap();

    let output = run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rule_recommendations"][0]["status"], "conflict");
    assert_eq!(fs::read(&target).unwrap(), b"user-owned bytes\n");
    assert!(fs::symlink_metadata(destination)
        .unwrap()
        .file_type()
        .is_symlink());
}

#[cfg(unix)]
#[test]
fn agent_onboard_does_not_write_through_assura_skill_parent_symlink() {
    use std::os::unix::fs::symlink;

    let project = TempDir::new().unwrap();
    let agents = project.path().join(".agents");
    fs::create_dir_all(&agents).unwrap();
    let external = TempDir::new().unwrap();
    let skills = agents.join("skills");
    symlink(external.path(), &skills).unwrap();

    let output = run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["rule_recommendations"][0]["status"], "conflict");
    assert!(fs::symlink_metadata(&skills)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(fs::read_dir(external.path()).unwrap().count(), 0);
}

#[test]
fn agent_onboard_recognizes_composed_entrypoint_as_applied() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"version: "2.0"

structure:
  AGENTS.md: exists:1 | $agent-entrypoint
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

#[test]
fn agent_onboard_accepts_canonical_selected_config_path() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    let selected = project.path().join(".assura/custom.yml");
    let original = b"version: \"2.0\"\nstructure:\n  ./:\n    extra: true\n";
    fs::write(&selected, original).unwrap();
    let canonical_selected = selected.canonicalize().unwrap();

    let output = json_from_success(run_assura(&[
        "--config",
        canonical_selected.to_str().unwrap(),
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));

    assert_eq!(output["rule_recommendations"][0]["status"], "not-applied");
    assert_eq!(fs::read(selected).unwrap(), original);
}

#[cfg(unix)]
#[test]
fn agent_onboard_accepts_config_through_alias_of_trusted_project_root() {
    use std::os::unix::fs::symlink;

    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    let selected = project.path().join(".assura/custom.yml");
    fs::write(
        &selected,
        "version: \"2.0\"\nstructure:\n  ./:\n    extra: true\n",
    )
    .unwrap();
    let project_name = project.path().file_name().unwrap().to_string_lossy();
    let alias = project
        .path()
        .with_file_name(format!("{project_name}-alias"));
    symlink(project.path(), &alias).unwrap();
    let aliased_selected = alias.join(".assura/custom.yml");

    let output = json_from_success(run_assura(&[
        "--config",
        aliased_selected.to_str().unwrap(),
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    fs::remove_file(&alias).unwrap();

    assert_eq!(output["rule_recommendations"][0]["status"], "not-applied");
    assert!(fs::read_to_string(selected).unwrap().contains("extra: true"));
}

#[cfg(unix)]
#[test]
fn agent_onboard_refuses_config_leaf_symlink_without_changing_external_bytes() {
    use std::os::unix::fs::symlink;

    let project = TempDir::new().unwrap();
    let external = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    let target = external.path().join("user-config.yml");
    let original = b"version: \"2.0\"\nuser_owned: true\n";
    fs::write(&target, original).unwrap();
    let selected = project.path().join(".assura/custom.yml");
    symlink(&target, &selected).unwrap();

    let output = run_assura(&[
        "--config",
        selected.to_str().unwrap(),
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("is a symlink"));
    assert_eq!(fs::read(target).unwrap(), original);
    assert!(fs::symlink_metadata(selected)
        .unwrap()
        .file_type()
        .is_symlink());
}

#[cfg(unix)]
#[test]
fn agent_onboard_refuses_config_parent_symlink_without_changing_external_bytes() {
    use std::os::unix::fs::symlink;

    let project = TempDir::new().unwrap();
    let external = TempDir::new().unwrap();
    let target = external.path().join("custom.yml");
    let original = b"version: \"2.0\"\nuser_owned: true\n";
    fs::write(&target, original).unwrap();
    let assura_dir = project.path().join(".assura");
    symlink(external.path(), &assura_dir).unwrap();
    let selected = assura_dir.join("custom.yml");

    let output = run_assura(&[
        "--config",
        selected.to_str().unwrap(),
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("is a symlink"));
    assert_eq!(fs::read(target).unwrap(), original);
    assert!(fs::symlink_metadata(assura_dir)
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn agent_onboard_keeps_regular_external_selected_config_valid() {
    let project = TempDir::new().unwrap();
    let external = TempDir::new().unwrap();
    let external_root = external.path().canonicalize().unwrap();
    let selected = external_root.join("custom.yml");
    let original = b"version: \"2.0\"\nstructure:\n  ./:\n    extra: true\n";
    fs::write(&selected, original).unwrap();

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
    assert_eq!(fs::read(selected).unwrap(), original);
}
