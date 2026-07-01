use std::fs;
use std::process::Command;

use assura::cli::CompiledStructureConfigArtifact;

#[test]
fn compiled_config_cli_preserves_markdown_rule_config_severity() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-markdown-severity-project");
    let compiled_config = temp.path().join("markdown-check-config.bin");
    fs::create_dir_all(project.join(".assura")).unwrap();
    fs::create_dir_all(project.join("docs")).unwrap();
    fs::write(
        project.join(".assura/config.yml"),
        r#"
structure:
  ./:
    children:
      docs/:
        markdown:
          check_links: true
          lint_common: true
          rules:
            markdown_link_target:
              severity: low
            markdown_multiple_blank_lines:
              severity: low
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(
        project.join("docs/note.md"),
        "# Note\n\n\n[Missing](missing.md)\n",
    )
    .unwrap();

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

    let check = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let stdout = String::from_utf8_lossy(&check.stdout);
    assert!(
        stdout.contains("markdown_multiple_blank_lines"),
        "stdout:\n{stdout}"
    );
}

#[test]
fn compiled_config_cli_rejects_incompatible_markdown_artifact_schema() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-markdown-schema-project");
    let compiled_config = temp.path().join("markdown-check-config.bin");
    fs::create_dir_all(project.join(".assura")).unwrap();
    fs::create_dir_all(project.join("docs")).unwrap();
    fs::write(
        project.join(".assura/config.yml"),
        r#"
structure:
  ./:
    children:
      docs/:
        markdown:
          check_links: true
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(project.join("docs/note.md"), "# Note\n").unwrap();

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

    let current_schema = CompiledStructureConfigArtifact::current_schema_version();
    let stale_schema_prefix = postcard::to_allocvec(&current_schema.saturating_sub(1)).unwrap();
    fs::write(&compiled_config, stale_schema_prefix).unwrap();

    let check = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(!check.status.success());
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert!(
        stderr.contains("incompatible Assura version"),
        "stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("invalid compiled config"),
        "stderr:\n{stderr}"
    );
}
