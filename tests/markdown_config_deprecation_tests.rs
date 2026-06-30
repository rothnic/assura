use std::fs;
use std::process::Command;

use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

#[test]
fn check_rejects_legacy_markdown_required_fields_with_model_guidance() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    children:
      docs/:
        markdown:
          require_frontmatter: true
          required_fields:
            - title
"#,
    );

    fs::create_dir(project.path().join("docs")).unwrap();
    fs::write(
        project.path().join("docs/project-note.md"),
        "---\ntitle: Project Note\n---\n# Project Note\n",
    )
    .unwrap();

    let output = Command::new(assura_bin())
        .arg("check")
        .arg(project.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("markdown.required_fields") || stderr.contains(".markdown.required_fields"),
        "stderr was:\n{stderr}"
    );
    assert!(stderr.contains("models"), "stderr was:\n{stderr}");
    assert!(stderr.contains("collections"), "stderr was:\n{stderr}");
}
