use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
    for dir in [".assura", "scripts", "docs"] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    fs::write(
        root.join(".assura/config.yml"),
        r#"extensions:
  computed_checks:
    - id: rollup_score
      severity: high
      script: scripts/rollup.sh
      windows_script: scripts/rollup.cmd
      timeout_ms: 5000
structure:
  ./:
    required: false
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(root.join("docs/source.md"), "# Source\n").unwrap();
    write_script(
        root,
        r#"#!/bin/sh
read _request || true
printf '%s\n' '{"schema":"assura.computed-check.output.v1","findings":[]}'
"#,
    );
}

fn write_script(root: &Path, body: &str) {
    let path = root.join("scripts/rollup.sh");
    fs::write(&path, body).unwrap();
    let cmd_body = if body.contains("score_low") {
        r#"@echo off
echo {"schema":"assura.computed-check.output.v1","findings":[{"code":"score_low","message":"Rollup score is below threshold","path":"docs/source.md","severity":"high"}]}
"#
    } else {
        r#"@echo off
echo {"schema":"assura.computed-check.output.v1","findings":[]}
"#
    };
    fs::write(root.join("scripts/rollup.cmd"), cmd_body).unwrap();
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).unwrap();
    }
}

#[test]
fn compiled_config_cli_supports_computed_check_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-computed-check-project");
    let compiled_config = temp.path().join("check-config.bin");
    write_project(&project);

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--output")
        .arg(&compiled_config)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    write_script(
        &project,
        r#"#!/bin/sh
read _request || true
printf '%s\n' '{"schema":"assura.computed-check.output.v1","findings":[{"code":"score_low","message":"Rollup score is below threshold","path":"docs/source.md","severity":"high"}]}'
"#,
    );

    let invalid = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&invalid.stdout);
    let stderr = String::from_utf8_lossy(&invalid.stderr);
    assert_eq!(
        invalid.status.code(),
        Some(1),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("computed_check:rollup_score:score_low"),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("Rollup score is below threshold"),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
