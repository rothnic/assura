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
    assert!(output.status.success(), "onboard command succeeds");
    serde_json::from_slice(&output.stdout).expect("command emits JSON")
}

#[test]
fn agent_onboard_uses_evidence_first_specialization_for_a_recognizable_cargo_project() {
    let project = TempDir::new().unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"sample\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(project.path().join("src/lib.rs"), "pub fn sample() {}\n").unwrap();
    fs::create_dir_all(project.path().join("tests")).unwrap();
    fs::write(
        project.path().join("tests/sample.rs"),
        "#[test]\nfn sample() {}\n",
    )
    .unwrap();

    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));

    assert!(output["inactive"]
        .as_array()
        .expect("specialization state")
        .iter()
        .any(|item| item["name"] == "project_specialization"
            && item["status"] == "configured_unverified"));
    let handoff =
        fs::read_to_string(project.path().join(".assura/onboarding/agent-next.md")).unwrap();
    for step in [
        "1. Inspect explicit repository instructions, manifests, and established layout.",
        "2. Preserve explicit local policy intent before selecting a pattern.",
        "3. Select the smallest matching pattern from project evidence.",
        "4. Record source, test, generated-output boundaries, and native tools.",
        "5. Apply and validate the project-owned policy.",
        "6. Configure integration and gate guidance.",
        "7. Prove one negative policy case.",
        "8. Report only unresolved exceptions.",
    ] {
        assert!(handoff.contains(step), "missing handoff step: {step}");
    }
    assert!(!handoff.contains("What primary language or stack should this project use?"));
    let exceptions =
        fs::read_to_string(project.path().join(".assura/onboarding/questions.md")).unwrap();
    assert!(exceptions.contains("Unresolved Specialization Exceptions"));
    assert!(!exceptions.contains("What primary language or stack should this project use?"));

    let profile: Value = serde_json::from_str(
        &fs::read_to_string(
            project
                .path()
                .join(".assura/onboarding/profile-selection.json"),
        )
        .expect("specialization profile"),
    )
    .expect("valid specialization profile JSON");
    assert_eq!(profile["schema"], "assura.profile-selection.v1");
    assert_eq!(profile["profile"], "rust-library");
    assert_eq!(profile["source"], "Cargo.toml");
    assert!(profile["source_hash"]
        .as_str()
        .is_some_and(|hash| !hash.is_empty()));
    assert!(profile["decisions"]
        .as_array()
        .expect("decisions")
        .iter()
        .any(|decision| {
            decision["key"] == "stack"
                && decision["value"] == "rust"
                && decision["evidence"] == "Cargo.toml"
        }));
    assert_eq!(profile["conflicts"], serde_json::json!([]));
    assert_eq!(profile["verification"]["config"], "pass");
}

#[test]
fn agent_onboard_recommends_profiles_for_bun_and_pytest_repositories() {
    for (manifest, contents, profile_name, stack) in [
        (
            "package.json",
            r#"{"scripts":{"test":"bun test"}}"#,
            "typescript-bun-utility",
            "node",
        ),
        (
            "pyproject.toml",
            "[project]\nname = \"sample\"\nversion = \"0.1.0\"\n",
            "python-pytest",
            "python",
        ),
    ] {
        let project = TempDir::new().unwrap();
        fs::write(project.path().join(manifest), contents).unwrap();

        let output = json_from_success(run_assura(&[
            "agent",
            "onboard",
            project.path().to_str().unwrap(),
            "--format",
            "json",
        ]));
        assert!(output["inactive"]
            .as_array()
            .expect("specialization state")
            .iter()
            .any(|item| item["name"] == "project_specialization"
                && item["status"] == "configured_unverified"));

        let profile: Value = serde_json::from_str(
            &fs::read_to_string(
                project
                    .path()
                    .join(".assura/onboarding/profile-selection.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(profile["profile"], profile_name);
        assert_eq!(profile["source"], manifest);
        assert_eq!(profile["decisions"][0]["value"], stack);
    }
}

#[test]
fn agent_onboard_keeps_ambiguous_repositories_reversible_and_repeatable() {
    let project = TempDir::new().unwrap();
    fs::write(project.path().join("README.md"), "# Sample\n").unwrap();

    let first = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert!(first["inactive"]
        .as_array()
        .expect("specialization state")
        .iter()
        .any(|item| item["name"] == "project_specialization"
            && item["status"] == "needs_agent_specialization"));
    assert!(!project.path().join("Cargo.toml").exists());
    assert!(!project.path().join("package.json").exists());
    assert!(!project.path().join("pyproject.toml").exists());

    let profile_path = project
        .path()
        .join(".assura/onboarding/profile-selection.json");
    let first_profile = fs::read(&profile_path).unwrap();
    let second = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(second["detected"]["project_type"], "unknown");
    assert_eq!(fs::read(profile_path).unwrap(), first_profile);
}

#[test]
fn agent_onboard_requires_user_authority_for_conflicting_manifests() {
    let project = TempDir::new().unwrap();
    fs::write(
        project.path().join("Cargo.toml"),
        "[package]\nname = \"sample\"\n",
    )
    .unwrap();
    fs::write(
        project.path().join("package.json"),
        "{\"name\":\"sample\"}\n",
    )
    .unwrap();

    let output = json_from_success(run_assura(&[
        "agent",
        "onboard",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(output["detected"]["project_type"], "ambiguous");
    assert!(output["inactive"]
        .as_array()
        .expect("specialization state")
        .iter()
        .any(|item| item["name"] == "project_specialization"
            && item["status"] == "conflict_requires_user"));
    let profile: Value = serde_json::from_str(
        &fs::read_to_string(
            project
                .path()
                .join(".assura/onboarding/profile-selection.json"),
        )
        .unwrap(),
    )
    .unwrap();
    assert!(profile["conflicts"]
        .as_array()
        .expect("manifest conflicts")
        .iter()
        .any(|conflict| conflict["kind"] == "manifest" && conflict["source"] == "Cargo.toml"));
}

#[test]
fn public_onboarding_guide_documents_the_v2_specialization_contract() {
    let guide = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/website/src/content/docs/guides/agent-ready-onboarding.md"
    ));

    assert!(guide.contains("\"schema\": \"assura.agent-onboarding.v2\""));
    assert!(guide.contains("## Agent-Next Procedure"));
    assert!(guide.contains("Exceptions that still require user authority"));
    assert!(!guide.contains("## Agent-Next Questions"));
    assert!(!guide.contains("Record the answers in project notes"));
}
