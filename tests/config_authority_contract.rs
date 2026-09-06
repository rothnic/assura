use serde_json::Value;
use std::fs;
use std::process::{Command, Output};
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn run_assura(cwd: &std::path::Path, args: &[&str]) -> Output {
    Command::new(assura_bin())
        .current_dir(cwd)
        .args(args)
        .output()
        .expect("assura command runs")
}

fn json_from(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "command emits JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn normalized_config_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn assert_json_path_matches_path(json_path: &Value, expected_path: &str) {
    assert_eq!(
        normalized_config_path(json_path.as_str().expect("JSON path")),
        normalized_config_path(expected_path)
    );
}

#[test]
fn windows_extended_config_path_representation_normalizes_for_comparison() {
    let json_path = "//?/C:/fixture/project/.assura/structure.yml";
    let native_path = r"\\?\C:\fixture\project\.assura\structure.yml";

    assert_ne!(json_path, native_path, "raw Windows representations differ");
    assert_eq!(
        normalized_config_path(json_path),
        normalized_config_path(native_path)
    );
}

#[test]
fn current_commands_share_explicit_structure_config_authority() {
    let fixture = TempDir::new().expect("fixture root");
    let project = fixture.path().join("project");
    let config_dir = project.join(".assura");
    let config_path = config_dir.join("structure.yml");
    fs::create_dir_all(project.join("src")).expect("source directory");
    fs::create_dir_all(&config_dir).expect("config directory");
    fs::write(project.join("src/BadName.rs"), "fn main() {}\n").expect("invalid name");
    fs::write(
        &config_path,
        r#"
rules:
  rust-source:
    naming: snake_case
structure:
  src/:
    .rs: $rust-source
"#,
    )
    .expect("config");

    let canonical_config = config_path.canonicalize().expect("canonical config path");
    let config = canonical_config.to_str().expect("utf-8 config path");
    let project_path = project.to_str().expect("utf-8 project path");

    let check = run_assura(
        fixture.path(),
        &[
            "--config",
            config,
            "check",
            project_path,
            "--format",
            "json",
        ],
    );
    assert_eq!(check.status.code(), Some(1));
    let check_json = json_from(&check);
    assert_json_path_matches_path(&check_json["config_path"], config);
    assert!(check_json["violations"]
        .as_array()
        .expect("violations")
        .iter()
        .any(
            |violation| violation["path"] == "src/BadName.rs" && violation["rule"] == "file_naming"
        ));

    let status = run_assura(
        fixture.path(),
        &[
            "--config",
            config,
            "status",
            project_path,
            "--format",
            "json",
        ],
    );
    assert_success(&status);
    let status_json = json_from(&status);
    assert_json_path_matches_path(&status_json["config_path"], config);
    assert_eq!(status_json["reusable_rules"], 1);

    let explain = run_assura(
        fixture.path(),
        &[
            "--config",
            config,
            "explain",
            "project/src/BadName.rs",
            "--format",
            "json",
        ],
    );
    assert_success(&explain);
    let explain_json = json_from(&explain);
    assert_json_path_matches_path(&explain_json["config_path"], config);
    assert!(explain_json["source_rules"]
        .as_array()
        .expect("source rules")
        .iter()
        .any(|rule| rule["rule"] == "rust-source"));

    let info = run_assura(fixture.path(), &["info", config]);
    assert_success(&info);
    let info_text = String::from_utf8_lossy(&info.stdout);
    assert!(info_text.contains(&format!("Config: {config}")));
    assert!(info_text.contains("Structure roots: 1"));
    assert!(info_text.contains("Reusable rules: 1"));
}

#[test]
fn malformed_current_structure_config_fails_clearly() {
    let fixture = TempDir::new().expect("fixture root");
    let config_path = fixture.path().join("invalid.yml");
    fs::write(&config_path, "structure: [\n").expect("invalid config");

    let output = run_assura(
        fixture.path(),
        &[
            "--config",
            config_path.to_str().expect("utf-8 config path"),
            "status",
            fixture.path().to_str().expect("utf-8 project path"),
            "--format",
            "json",
        ],
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Error:"),
        "stderr:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
