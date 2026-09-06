use std::fs;
use std::process::Command;

use sha2::{Digest, Sha256};
use tempfile::TempDir;

fn assura_full_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura-full")
}

#[test]
fn init_applies_an_explicit_local_recipe_file_with_spaces_in_its_path() {
    let project = TempDir::new().expect("project directory");
    let patterns = TempDir::new().expect("pattern directory");
    let recipe_dir = patterns.path().join("team patterns");
    fs::create_dir_all(&recipe_dir).expect("recipe directory");
    let recipe_path = recipe_dir.join("rust library.yml");
    fs::write(&recipe_path, "structure:\n  ./:\n    README.md: exists:1\n").expect("local recipe");

    let output = Command::new(assura_full_bin())
        .arg("init")
        .arg(project.path())
        .arg("--recipe-file")
        .arg(&recipe_path)
        .output()
        .expect("assura init runs");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let config: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml"))
            .expect("materialized config"),
    )
    .expect("valid config YAML");
    assert_eq!(config["structure"]["./"]["README.md"], "exists:1");
}

#[test]
fn init_records_the_explicit_local_recipe_origin_and_content_hash() {
    let project = TempDir::new().expect("project directory");
    let patterns = TempDir::new().expect("pattern directory");
    let recipe_path = patterns.path().join("team-policy.yml");
    let recipe = "structure:\n  ./:\n    CONTRIBUTING.md: exists:1\n";
    fs::write(&recipe_path, recipe).expect("local recipe");

    let output = Command::new(assura_full_bin())
        .arg("init")
        .arg(project.path())
        .arg("--recipe-file")
        .arg(&recipe_path)
        .output()
        .expect("assura init runs");

    assert!(
        output.status.success(),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let profile: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            project
                .path()
                .join(".assura/onboarding/profile-selection.json"),
        )
        .expect("profile selection record"),
    )
    .expect("valid profile selection JSON");
    assert_eq!(profile["source"], recipe_path.display().to_string());
    assert_eq!(
        profile["source_hash"],
        format!("{:x}", Sha256::digest(recipe.as_bytes()))
    );
}

#[test]
fn agent_onboard_applies_an_explicit_local_recipe_to_existing_project_policy() {
    let project = TempDir::new().expect("project directory");
    let patterns = TempDir::new().expect("pattern directory");
    fs::create_dir_all(project.path().join(".assura")).expect("Assura directory");
    fs::write(
        project.path().join(".assura/config.yml"),
        "version: \"2.0\"\nstructure:\n  ./:\n    extra: true\n",
    )
    .expect("project policy");
    fs::write(project.path().join("CONTRIBUTING.md"), "# Contributing\n")
        .expect("required project file");
    let recipe_path = patterns.path().join("team policy.yml");
    fs::write(&recipe_path, "structure:\n  CONTRIBUTING.md: exists:1\n").expect("local recipe");

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .arg("--recipe-file")
        .arg(&recipe_path)
        .args(["--format", "json"])
        .output()
        .expect("assura agent onboard runs");

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let config: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml")).expect("project policy"),
    )
    .expect("valid config YAML");
    assert_eq!(config["structure"]["CONTRIBUTING.md"], "exists:1");
}
