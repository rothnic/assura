use std::fs;
use std::process::Command;

use sha2::{Digest, Sha256};
use std::path::Path;
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
    fs::write(
        &recipe_path,
        "structure:\n  ./:\n    CONTRIBUTING.md: exists:1\n",
    )
    .expect("local recipe");

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

#[test]
fn agent_onboard_preserves_a_conflicting_local_rule_and_reports_both_values() {
    let project = TempDir::new().expect("project directory");
    let patterns = TempDir::new().expect("pattern directory");
    fs::create_dir_all(project.path().join(".assura")).expect("Assura directory");
    let original_policy =
        "version: \"2.0\"\nrules:\n  project-file:\n    max_lines: 80\nstructure:\n  ./ :\n    extra: true\n";
    fs::write(project.path().join(".assura/config.yml"), original_policy).expect("project policy");
    let recipe_path = patterns.path().join("conflicting-policy.yml");
    fs::write(&recipe_path, "rules:\n  project-file:\n    max_lines: 20\n").expect("local recipe");

    let output = Command::new(assura_full_bin())
        .args(["agent", "onboard"])
        .arg(project.path())
        .arg("--recipe-file")
        .arg(&recipe_path)
        .args(["--format", "json"])
        .output()
        .expect("assura agent onboard runs");

    assert!(
        !output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("rules.project-file.max_lines"),
        "stderr:\n{stderr}"
    );
    assert!(stderr.contains("existing: 80"), "stderr:\n{stderr}");
    assert!(stderr.contains("incoming: 20"), "stderr:\n{stderr}");
    let config: serde_yaml::Value = serde_yaml::from_str(
        &fs::read_to_string(project.path().join(".assura/config.yml")).expect("project policy"),
    )
    .expect("valid config YAML");
    assert_eq!(config["rules"]["project-file"]["max_lines"], 80);
    assert_eq!(
        fs::read_to_string(project.path().join(".assura/config.yml")).expect("project policy"),
        original_policy,
        "a local-recipe conflict must not leave a partially merged project policy"
    );
}

#[test]
fn init_rejects_a_recipe_that_makes_the_complete_policy_invalid_without_writing_config() {
    let project = TempDir::new().expect("project directory");
    let patterns = TempDir::new().expect("pattern directory");
    let recipe_path = patterns.path().join("invalid-policy.yml");
    fs::write(&recipe_path, "structure: not-a-mapping\n").expect("local recipe");

    let output = Command::new(assura_full_bin())
        .arg("init")
        .arg(project.path())
        .arg("--recipe-file")
        .arg(&recipe_path)
        .output()
        .expect("assura init runs");

    assert!(
        !output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !project.path().join(".assura/config.yml").exists(),
        "invalid prospective policy must not create a config file"
    );
}

#[test]
fn init_rerun_with_the_same_explicit_recipe_is_byte_identical() {
    let project = TempDir::new().expect("project directory");
    let patterns = TempDir::new().expect("pattern directory");
    let recipe_path = patterns.path().join("stable-policy.yml");
    fs::write(&recipe_path, "structure:\n  ./:\n    README.md: exists:1\n").expect("local recipe");

    for force in [false, true] {
        let mut command = Command::new(assura_full_bin());
        command
            .arg("init")
            .arg(project.path())
            .arg("--recipe-file")
            .arg(&recipe_path);
        if force {
            command.arg("--force");
        }
        let output = command.output().expect("assura init runs");
        assert!(
            output.status.success(),
            "stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if !force {
            continue;
        }
    }

    let config = fs::read(project.path().join(".assura/config.yml")).expect("config bytes");
    let profile = fs::read(
        project
            .path()
            .join(".assura/onboarding/profile-selection.json"),
    )
    .expect("profile bytes");

    let output = Command::new(assura_full_bin())
        .arg("init")
        .arg(project.path())
        .arg("--recipe-file")
        .arg(&recipe_path)
        .arg("--force")
        .output()
        .expect("third init runs");
    assert!(output.status.success());
    assert_eq!(
        fs::read(project.path().join(".assura/config.yml")).unwrap(),
        config
    );
    assert_eq!(
        fs::read(
            project
                .path()
                .join(".assura/onboarding/profile-selection.json")
        )
        .unwrap(),
        profile
    );
}

#[test]
fn bundled_local_pattern_examples_validate_matching_minimal_projects() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    for (name, files) in [
        (
            "rust-library",
            vec!["Cargo.toml", "src/lib.rs", "tests/library.rs"],
        ),
        (
            "typescript-bun-utility",
            vec![
                "package.json",
                "src/format_value.ts",
                "test/format_value.test.ts",
            ],
        ),
        (
            "python-pytest",
            vec![
                "pyproject.toml",
                "src/sample_package/__init__.py",
                "tests/test_package.py",
            ],
        ),
    ] {
        let project = TempDir::new().expect("project directory");
        for file in files {
            let path = project.path().join(file);
            fs::create_dir_all(path.parent().expect("parent directory")).expect("parent");
            fs::write(path, "fixture\n").expect("fixture file");
        }
        let recipe = manifest.join(format!("tests/fixtures/agent_init/patterns/{name}.yml"));
        let init = Command::new(assura_full_bin())
            .arg("init")
            .arg(project.path())
            .arg("--recipe-file")
            .arg(&recipe)
            .output()
            .expect("assura init runs");
        assert!(
            init.status.success(),
            "{name} init stderr:\n{}",
            String::from_utf8_lossy(&init.stderr)
        );
        let check = Command::new(assura_full_bin())
            .args(["check", "--format", "json"])
            .arg(project.path())
            .output()
            .expect("assura check runs");
        assert!(
            check.status.success(),
            "{name} check stdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );
    }
}

#[test]
fn agent_onboard_unions_explicit_pattern_excludes_in_stable_order() {
    let project = TempDir::new().expect("project directory");
    let patterns = TempDir::new().expect("pattern directory");
    fs::create_dir_all(project.path().join(".assura")).expect("Assura directory");
    fs::write(
        project.path().join(".assura/config.yml"),
        "version: \"2.0\"\nstructure:\n  ./ :\n    extra: true\nexclude:\n  - generated/**\n",
    )
    .expect("project policy");
    let recipe_path = patterns.path().join("exclude-policy.yml");
    fs::write(&recipe_path, "exclude:\n  - vendor/**\n  - generated/**\n").expect("local recipe");

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
    assert_eq!(
        config["exclude"].as_sequence().expect("exclude sequence"),
        &vec![
            serde_yaml::Value::String("generated/**".to_string()),
            serde_yaml::Value::String("vendor/**".to_string()),
            serde_yaml::Value::String(".assura/**".to_string()),
            serde_yaml::Value::String(".git/**".to_string()),
            serde_yaml::Value::String("target/**".to_string()),
            serde_yaml::Value::String("node_modules/**".to_string()),
            serde_yaml::Value::String("dist/**".to_string()),
            serde_yaml::Value::String("**/dist/**".to_string()),
        ]
    );
}
