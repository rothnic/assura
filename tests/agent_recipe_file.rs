use std::fs;
use std::process::Command;

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
