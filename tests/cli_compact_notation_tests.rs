use assura::cli::run_structure_check;
use std::fs;
use tempfile::TempDir;

fn write_config(project: &TempDir, config: &str) {
    let assura_dir = project.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    fs::write(assura_dir.join("config.yml"), config).unwrap();
}

#[test]
fn compact_exists_direct_children_are_allowed_without_redundant_allow_lists() {
    let project = TempDir::new().unwrap();
    write_config(
        &project,
        r#"
structure:
  ./:
    inherit: false
    README.md: exists:0-1
    docs/: exists:0-1
    files:
      naming: kebab-case
      allow_extra: false
    directories:
      naming: kebab-case
      allow_extra: false
exclude:
  - .assura/**
"#,
    );

    fs::write(project.path().join("README.md"), "# Fixture\n").unwrap();
    fs::create_dir(project.path().join("docs")).unwrap();

    let report = run_structure_check(Some(project.path().to_path_buf()), None, false).unwrap();

    assert!(report.success, "report was: {report:#?}");
    assert_eq!(report.violations.len(), 0);
}
