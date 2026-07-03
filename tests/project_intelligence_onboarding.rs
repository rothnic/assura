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
        .join(".assura/models/project-intelligence/starter.schema.json")
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
    fs::create_dir_all(project.path().join(".assura/models/project-intelligence")).unwrap();
    fs::write(
        project
            .path()
            .join(".assura/models/project-intelligence/starter.schema.json"),
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

#[test]
fn agent_onboard_generates_broad_baseline_and_packet() {
    let project = TempDir::new().unwrap();

    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));

    assert_eq!(output["schema"], "assura.agent-onboarding.v1");
    assert_eq!(output["detected"]["project_type"], "empty");
    assert_eq!(output["detected"]["agent_harness"], "generic");
    assert_eq!(output["installed"]["config"], ".assura/config.yml");
    assert_eq!(output["content"]["template"], "none");
    assert_eq!(output["content"]["status"], "inactive");
    let lifecycle_modes = output["lifecycle_profiles"]
        .as_array()
        .expect("lifecycle profiles")
        .iter()
        .map(|item| item["mode"].as_str().expect("mode"))
        .collect::<Vec<_>>();
    assert_eq!(lifecycle_modes, ["nudge", "warn", "gate"]);
    assert!(output["lifecycle_profiles"]
        .as_array()
        .expect("lifecycle profiles")
        .iter()
        .any(|item| item["mode"] == "gate" && item["blocking"] == true));
    let first_action = &output["next_actions"][0];
    assert_eq!(first_action["priority"], 1);
    assert_eq!(first_action["action"], "Read the onboarding handoff");
    assert_eq!(
        first_action["affected_paths"][0],
        ".assura/onboarding/agent-next.md"
    );
    assert!(output["verified"]
        .as_array()
        .expect("verified array")
        .iter()
        .any(|item| item["name"] == "structure_config" && item["status"] == "pass"));
    assert!(output["inactive"]
        .as_array()
        .expect("inactive array")
        .iter()
        .any(|item| item["name"] == "content_models"));

    for path in [
        ".assura/config.yml",
        ".assura/presets.lock.yml",
        ".assura/onboarding/summary.md",
        ".assura/onboarding/questions.md",
        ".assura/onboarding/lifecycle.md",
        ".assura/onboarding/agent-next.md",
        ".assura/onboarding/doctor.json",
        "AGENTS.md",
        ".agents/skills/assura-project-maintenance/SKILL.md",
        "docs/process/agent-workflow.md",
        "docs/learnings/README.md",
    ] {
        assert!(project.path().join(path).is_file(), "missing {path}");
    }

    let agent_next =
        fs::read_to_string(project.path().join(".assura/onboarding/agent-next.md")).unwrap();
    assert!(agent_next.contains("Do not invent project conventions"));
    assert!(agent_next.contains(".assura/onboarding/lifecycle.md"));
    assert!(agent_next.contains("What primary language or stack should this project use?"));
    assert!(agent_next.contains("What test layout should the project use?"));
    let lifecycle =
        fs::read_to_string(project.path().join(".assura/onboarding/lifecycle.md")).unwrap();
    assert!(lifecycle.contains("| nudge |"));
    assert!(lifecycle.contains("| warn |"));
    assert!(lifecycle.contains("| gate |"));
    assert!(lifecycle.contains("does not silently mutate"));

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

    let check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(check["success"], true);
}

#[test]
fn agent_onboard_content_template_activates_agent_project_models() {
    let project = TempDir::new().unwrap();

    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--content-template",
        "agent-project",
        "--format",
        "json",
    ]));

    assert_eq!(output["content"]["template"], "agent-project");
    assert_eq!(output["content"]["status"], "active");
    assert!(!output["inactive"]
        .as_array()
        .expect("inactive array")
        .iter()
        .any(|item| item["name"] == "content_models"));
    assert!(project
        .path()
        .join(".assura/models/agent-project/baseline.schema.json")
        .is_file());
    assert!(project
        .path()
        .join("docs/requirements/requirement-agent-content-baseline.md")
        .is_file());

    let config = fs::read_to_string(project.path().join(".assura/config.yml")).unwrap();
    assert!(config.contains("models:"));
    assert!(config.contains("class: Decision"));
    assert!(config.contains("class: Requirement"));
    assert!(!config.contains("SourceDocument"));

    let check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(check["success"], true);

    let search = json_from_success(run_assura(&[
        "content",
        "search",
        "Agent Content Baseline Requirement",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(search["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .any(|item| item["collection"] == "requirements"
            && item["instance_id"] == "requirement-agent-content-baseline"));

    let doctor = json_from_success(run_assura(&[
        "doctor",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(doctor["configured"]
        .as_array()
        .expect("configured")
        .iter()
        .any(|item| item["name"] == "content_models" && item["status"] == "active"));
    assert!(!doctor["inactive"]
        .as_array()
        .expect("inactive")
        .iter()
        .any(|item| item["name"] == "content_models"));
    assert_eq!(doctor["gaps"].as_array().expect("gaps").len(), 0);
}

#[test]
fn agent_onboard_document_project_tracks_source_document_custody() {
    let project = TempDir::new().unwrap();
    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--content-template",
        "document-project",
        "--format",
        "json",
    ]));

    assert_eq!(output["content"]["template"], "document-project");
    assert!(project
        .path()
        .join("source-documents/manifest.md")
        .is_file());
    assert!(project
        .path()
        .join("source-documents/files/sample-source.txt")
        .is_file());
    let questions =
        fs::read_to_string(project.path().join(".assura/onboarding/questions.md")).unwrap();
    assert!(questions.contains("research-authoring project"));
    for path in [
        "library/topics/topic-document-project-baseline.md",
        "docs/drafts/draft-document-project-baseline.md",
        "docs/final/final-document-project-baseline.md",
        "docs/requirements/requirement-agent-content-baseline.md",
        "docs/evidence/evidence-agent-content-baseline.md",
        "docs/decisions/decision-agent-content-baseline.md",
        "docs/process/agent-workflow.md",
        "docs/learnings/README.md",
    ] {
        assert!(project.path().join(path).is_file(), "missing {path}");
    }

    let config = fs::read_to_string(project.path().join(".assura/config.yml")).unwrap();
    assert!(config.contains("library/:"));
    assert!(config.contains("source-documents/:"));
    assert!(config.contains("class: Topic"));
    assert!(config.contains("class: Draft"));
    assert!(config.contains("class: FinalDocument"));
    assert!(config.contains("topics.related_requirements"));
    assert!(config.contains("drafts.evidence"));
    assert!(config.contains("final_docs.evidence"));

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
    assert_eq!(doctor["binary_custody"]["status"], "active");
    assert_eq!(doctor["gaps"].as_array().expect("gaps").len(), 0);

    let topic_search = json_from_success(run_assura(&[
        "content",
        "search",
        "Research Authoring Baseline Topic",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(topic_search["matches"]
        .as_array()
        .expect("matches")
        .iter()
        .any(|item| item["collection"] == "topics"
            && item["instance_id"] == "topic-document-project-baseline"));

    let references = json_from_success(run_assura(&[
        "content",
        "references",
        project.path().to_str().unwrap(),
        "--source",
        "source-documents/manifest.md",
        "--format",
        "json",
    ]));
    let manifest_references = references["references"].as_array().expect("references");
    assert!(manifest_references.iter().any(|item| {
        item["source_path"] == "source-documents/manifest.md"
            && item["target_path"] == "source-documents/files/sample-source.txt"
            && item["target_exists"] == true
    }));

    fs::remove_file(
        project
            .path()
            .join("source-documents/files/sample-source.txt"),
    )
    .unwrap();
    let missing = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(missing.status.code(), Some(1));
    let missing_json: Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert!(missing_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| item["rule"] == "repository_reference_target"
            && item["path"] == "source-documents/manifest.md"));
}

#[test]
fn source_document_custody_does_not_read_binary_targets_as_utf8() {
    let project = TempDir::new().unwrap();
    json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--content-template",
        "document-project",
        "--format",
        "json",
    ]));

    let manifest_path = project.path().join("source-documents/manifest.md");
    let manifest = fs::read_to_string(&manifest_path).unwrap().replace(
        "source-documents/files/sample-source.txt",
        "source-documents/files/source.pdf",
    );
    fs::write(&manifest_path, manifest).unwrap();
    fs::remove_file(
        project
            .path()
            .join("source-documents/files/sample-source.txt"),
    )
    .unwrap();
    fs::write(
        project.path().join("source-documents/files/source.pdf"),
        [0xff, 0x00, 0xfe, 0xfd],
    )
    .unwrap();

    let check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(check["success"], true);
}

#[test]
fn agent_onboard_generated_config_validates_dynamic_directory_skill_contracts() {
    let project = TempDir::new().unwrap();
    json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));

    let config = fs::read_to_string(project.path().join(".assura/config.yml")).unwrap();
    assert!(config.contains("\"@assura-skill-dir\""));
    assert!(config.contains("\"{skill}/\""));
    assert!(!config.contains(".agents/skills/assura-project-maintenance/:"));

    let second_skill = project.path().join(".agents/skills/release-maintenance");
    fs::create_dir_all(second_skill.join("references")).unwrap();
    fs::create_dir_all(second_skill.join("scripts")).unwrap();
    fs::create_dir_all(second_skill.join("assets")).unwrap();
    fs::write(second_skill.join("SKILL.md"), "# Release Maintenance\n").unwrap();
    fs::write(second_skill.join("references/runbook.md"), "# Runbook\n").unwrap();
    fs::write(second_skill.join("scripts/check.sh"), "#!/bin/sh\n").unwrap();
    fs::write(second_skill.join("assets/template.txt"), "template\n").unwrap();

    let valid_check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(valid_check["success"], true);

    let missing_skill = project.path().join(".agents/skills/missing-skill-md");
    fs::create_dir_all(&missing_skill).unwrap();
    let invalid_check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(invalid_check.status.code(), Some(1));
    let invalid_json: Value = serde_json::from_slice(&invalid_check.stdout).unwrap();
    assert!(invalid_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| {
            item["path"] == ".agents/skills/missing-skill-md"
                && item["rule"] == "exists_count"
                && item["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("SKILL.md"))
        }));

    fs::write(missing_skill.join("SKILL.md"), "# Missing Fixed\n").unwrap();
    fs::create_dir_all(missing_skill.join("tmp")).unwrap();
    let invalid_child_check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(invalid_child_check.status.code(), Some(1));
    let invalid_child_json: Value = serde_json::from_slice(&invalid_child_check.stdout).unwrap();
    assert!(invalid_child_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| {
            item["path"] == ".agents/skills/missing-skill-md/tmp"
                && item["rule"] == "unexpected_directory"
        }));
}

#[test]
fn agent_project_dynamic_contracts_validate_repeated_project_structures() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"version: "2.0"

rules:
  "@package-dir":
    README.md: exists:1
    src/: exists:1
    docs/: exists:0-1
    extra: false
  "@doc-section":
    README.md: exists:1
    assets/: exists:0-1
    extra: false
  "@example-dir":
    README.md: exists:1
    fixtures/: exists:0-1
    extra: false
  "@fixture-dir":
    README.md: exists:1
    input/: exists:0-1
    expected/: exists:0-1
    extra: false

structure:
  ./:
    extra: true
    packages/: exists:1
    docs/: exists:1
    examples/: exists:1
    tests/: exists:1
  packages/:
    extra: true
    "{package}/":
      use: "@package-dir"
  docs/:
    extra: true
    "{section}/":
      use: "@doc-section"
  examples/:
    extra: true
    "{example}/":
      use: "@example-dir"
  tests/:
    fixtures/: exists:1
  tests/fixtures/:
    extra: true
    "{fixture}/":
      use: "@fixture-dir"
"#,
    )
    .unwrap();

    for package in ["core", "ui"] {
        let package_dir = project.path().join("packages").join(package);
        fs::create_dir_all(package_dir.join("src")).unwrap();
        fs::write(package_dir.join("README.md"), "# Package\n").unwrap();
        fs::write(package_dir.join("src/lib.rs"), "").unwrap();
    }
    let docs_section = project.path().join("docs/process");
    fs::create_dir_all(docs_section.join("assets")).unwrap();
    fs::write(docs_section.join("README.md"), "# Process\n").unwrap();
    fs::write(docs_section.join("assets/template.txt"), "template\n").unwrap();
    let example = project.path().join("examples/basic");
    fs::create_dir_all(example.join("fixtures")).unwrap();
    fs::write(example.join("README.md"), "# Basic Example\n").unwrap();
    fs::write(example.join("fixtures/sample.txt"), "sample\n").unwrap();
    let fixture = project.path().join("tests/fixtures/parser");
    fs::create_dir_all(fixture.join("input")).unwrap();
    fs::create_dir_all(fixture.join("expected")).unwrap();
    fs::write(fixture.join("README.md"), "# Parser Fixture\n").unwrap();

    let valid_check = json_from_success(run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(valid_check["success"], true);

    let missing_package = project.path().join("packages/missing-readme");
    fs::create_dir_all(missing_package.join("src")).unwrap();
    let missing_check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(missing_check.status.code(), Some(1));
    let missing_json: Value = serde_json::from_slice(&missing_check.stdout).unwrap();
    assert!(missing_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| {
            item["path"] == "packages/missing-readme"
                && item["rule"] == "exists_count"
                && item["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("README.md"))
        }));

    fs::write(missing_package.join("README.md"), "# Missing Fixed\n").unwrap();
    fs::create_dir_all(project.path().join("docs/process/tmp")).unwrap();
    let unexpected_check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(unexpected_check.status.code(), Some(1));
    let unexpected_json: Value = serde_json::from_slice(&unexpected_check.stdout).unwrap();
    assert!(unexpected_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| item["path"] == "docs/process/tmp" && item["rule"] == "unexpected_directory"));

    fs::remove_dir_all(project.path().join("docs/process/tmp")).unwrap();
    let missing_example = project.path().join("examples/missing-readme");
    fs::create_dir_all(missing_example.join("fixtures")).unwrap();
    let missing_example_check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(missing_example_check.status.code(), Some(1));
    let missing_example_json: Value =
        serde_json::from_slice(&missing_example_check.stdout).unwrap();
    assert!(missing_example_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| {
            item["path"] == "examples/missing-readme"
                && item["rule"] == "exists_count"
                && item["message"]
                    .as_str()
                    .is_some_and(|message| message.contains("README.md"))
        }));

    fs::write(missing_example.join("README.md"), "# Missing Fixed\n").unwrap();
    fs::create_dir_all(project.path().join("tests/fixtures/parser/tmp")).unwrap();
    let unexpected_fixture_check = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);
    assert_eq!(unexpected_fixture_check.status.code(), Some(1));
    let unexpected_fixture_json: Value =
        serde_json::from_slice(&unexpected_fixture_check.stdout).unwrap();
    assert!(unexpected_fixture_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(|item| {
            item["path"] == "tests/fixtures/parser/tmp" && item["rule"] == "unexpected_directory"
        }));
}

#[test]
fn agent_onboard_preserves_existing_user_authored_files() {
    let project = TempDir::new().unwrap();
    fs::write(project.path().join("AGENTS.md"), "# Existing Agent Notes\n").unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\n",
    )
    .unwrap();

    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--agent",
        "codex",
        "--format",
        "json",
    ]));

    assert_eq!(output["detected"]["project_type"], "rust");
    assert_eq!(output["detected"]["agent_harness"], "codex");
    assert_eq!(output["integration"]["status"], "installed");
    assert!(output["lifecycle_profiles"]
        .as_array()
        .expect("lifecycle profiles")
        .iter()
        .any(|item| item["mode"] == "nudge"
            && item["command"]
                .as_str()
                .expect("command")
                .contains("--agent codex")));
    assert!(output["lifecycle_profiles"]
        .as_array()
        .expect("lifecycle profiles")
        .iter()
        .any(|item| item["mode"] == "warn"
            && item["command"]
                .as_str()
                .expect("command")
                .contains("--agent codex --warn")));
    assert!(output["lifecycle_profiles"]
        .as_array()
        .expect("lifecycle profiles")
        .iter()
        .any(|item| item["mode"] == "gate"
            && item["blocking"] == true
            && item["command"]
                .as_str()
                .expect("command")
                .contains("--agent codex --min-severity medium")));
    assert!(output["next_actions"]
        .as_array()
        .expect("next actions")
        .iter()
        .any(
            |item| item["action"] == "Review host-agent integration bundle"
                && item["follow_up"]
                    .as_str()
                    .expect("follow_up")
                    .contains("agent integration doctor codex")
        ));
    assert_eq!(
        fs::read_to_string(project.path().join("AGENTS.md")).unwrap(),
        "# Existing Agent Notes\n"
    );
    assert!(output["files"]
        .as_array()
        .expect("files array")
        .iter()
        .any(|item| item["path"] == "AGENTS.md" && item["action"] == "existing"));
    assert!(project
        .path()
        .join(".assura/integrations/codex/manifest.json")
        .is_file());
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
        merged_config["structure"]["./"]["AGENTS.md"],
        serde_yaml::Value::String("exists:1".to_string())
    );
    assert!(merged_config["rules"]
        .as_mapping()
        .expect("rules mapping")
        .contains_key(serde_yaml::Value::String("@assura-skill-dir".to_string())));
    assert_eq!(
        merged_config["structure"][".agents/skills/"]["{skill}/"]["use"],
        serde_yaml::Value::String("@assura-skill-dir".to_string())
    );
    assert!(merged_config["exclude"]
        .as_sequence()
        .expect("exclude sequence")
        .contains(&serde_yaml::Value::String("custom/**".to_string())));
    assert!(merged_config["exclude"]
        .as_sequence()
        .expect("exclude sequence")
        .contains(&serde_yaml::Value::String(".git/**".to_string())));
}
