use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn assura_full_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura-full")
}

#[test]
fn missing_config_json_diagnostics_offer_read_only_init_recovery() {
    let scratch = TempDir::new().unwrap();
    let project = scratch.path().join("project with 'quotes'");
    fs::create_dir_all(&project).unwrap();

    let cases = [
        (
            "lightweight check",
            assura_bin(),
            ["check", "--format", "json"],
        ),
        (
            "full check",
            assura_full_bin(),
            ["check", "--format", "json"],
        ),
        ("doctor", assura_full_bin(), ["doctor", "--format", "json"]),
        ("review", assura_full_bin(), ["review", "--format", "json"]),
    ];
    for (name, binary, args) in cases {
        let output = Command::new(binary)
            .args(args)
            .arg(&project)
            .output()
            .unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(4), "{name} stderr:\n{stderr}");
        assert!(output.stdout.is_empty(), "{name} stdout was not empty");
        assert!(
            stderr.contains(&project.display().to_string()),
            "{name} did not identify the project path: {stderr}"
        );
        assert!(
            stderr.contains("assura init"),
            "{name} did not offer init recovery: {stderr}"
        );
        assert!(
            stderr.contains("project directory") && stderr.contains("there"),
            "{name} must explain where to run the command: {stderr}"
        );
        assert!(
            !project.join(".assura").exists() && !project.join(".codex").exists(),
            "{name} changed project files during diagnosis"
        );
    }

    let file = project.join("example.ts");
    fs::write(&file, "export {};\n").unwrap();
    let file_check = Command::new(assura_full_bin())
        .args(["check", "--format", "json"])
        .arg(&file)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&file_check.stderr);
    assert_eq!(file_check.status.code(), Some(4), "stderr:\n{stderr}");
    assert!(file_check.stdout.is_empty());
    assert!(
        stderr.contains(&project.display().to_string()),
        "file target recovery must identify its parent project: {stderr}"
    );
    assert!(
        stderr.contains(&file.display().to_string()),
        "file target diagnostic must retain the checked path: {stderr}"
    );
    assert!(stderr.contains("assura init"));
    assert!(
        stderr.contains("project directory"),
        "recovery must identify the directory where init should run: {stderr}"
    );

    let malformed = TempDir::new().unwrap();
    fs::create_dir(malformed.path().join(".assura")).unwrap();
    let malformed_config = malformed.path().join(".assura/config.yml");
    fs::write(&malformed_config, "structure: [\n").unwrap();
    let malformed_output = Command::new(assura_full_bin())
        .arg("--config")
        .arg(&malformed_config)
        .args(["doctor", "--format", "json"])
        .arg(malformed.path())
        .output()
        .unwrap();
    let malformed_stderr = String::from_utf8_lossy(&malformed_output.stderr);
    assert_eq!(malformed_output.status.code(), Some(2));
    assert!(malformed_output.stdout.is_empty());
    assert!(
        !malformed_stderr.contains("assura init"),
        "malformed config must not be mislabeled as missing: {malformed_stderr}"
    );

    let missing_target = project.join("does-not-exist");
    let missing_output = Command::new(assura_full_bin())
        .args(["check", "--format", "json"])
        .arg(&missing_target)
        .output()
        .unwrap();
    let missing_stderr = String::from_utf8_lossy(&missing_output.stderr);
    assert_eq!(missing_output.status.code(), Some(3));
    assert!(missing_output.stdout.is_empty());
    assert!(
        !missing_stderr.contains("assura init"),
        "missing targets must not be mislabeled as missing config: {missing_stderr}"
    );
}
