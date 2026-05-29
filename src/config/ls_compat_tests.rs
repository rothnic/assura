//! Unit coverage for LS-Lint config conversion.

use super::ls_compat::{
    convert_ls_lint_documents_to_config, convert_ls_lint_to_config, LsLintCompatibility,
};

#[test]
fn test_ls_compat_builder() {
    let compat = LsLintCompatibility::new()
        .with_extension_rule(".rs", "snake_case")
        .with_extension_rule(".ts", "camelCase")
        .with_path_rule("src/", ".rs", "snake_case");

    assert_eq!(compat.rules.get(".rs"), Some(&"snake_case".to_string()));
    assert_eq!(
        compat.paths.get("src/").unwrap().get(".rs"),
        Some(&"snake_case".to_string())
    );
}

#[test]
fn test_to_structure_nodes() {
    let compat = LsLintCompatibility::new()
        .with_extension_rule(".rs", "snake_case")
        .with_path_rule("src/", ".rs", "snake_case");

    let nodes = compat.to_structure_nodes();

    assert!(nodes.contains_key(""));
    assert!(nodes.contains_key("src/"));

    let root_node = nodes.get("").unwrap();
    let naming_patterns = root_node
        .files
        .as_ref()
        .unwrap()
        .naming_patterns
        .as_ref()
        .unwrap();
    assert_eq!(naming_patterns.get("*.rs"), Some(&"snake_case".to_string()));
}

#[test]
fn test_convert_ls_lint_to_config() {
    let ls_lint_yaml = r#"
ls:
  .rs: snake_case
  .ts: camelCase
  src/:
    .rs: snake_case
"#;

    let config = convert_ls_lint_to_config(ls_lint_yaml).unwrap();
    assert!(!config.structure.is_empty());
    let root = config.structure.get("./").unwrap();
    let src = root.children.as_ref().unwrap().get("src").unwrap();
    let patterns = src
        .files
        .as_ref()
        .unwrap()
        .naming_patterns
        .as_ref()
        .unwrap();
    assert_eq!(patterns.get("*.rs"), Some(&"snake_case".to_string()));
}

#[test]
fn test_convert_multiple_ls_lint_configs_merges_like_config_flags() {
    let first = r#"
ignore:
  - dist/**
ls:
  .js: camelCase
  .ts: snake_case
  src:
    .js: camelCase
    components:
      .tsx: PascalCase
"#;
    let second = r#"
ignore:
  - node_modules
ls:
  .ts: kebab-case
  .rs: snake_case
  src:
    .ts: kebab-case
    components:
      .test.tsx: kebab-case
"#;

    let config = convert_ls_lint_documents_to_config(&[first, second]).unwrap();
    assert!(config.exclude.contains(&"dist/**".to_string()));
    assert!(config.exclude.contains(&"node_modules".to_string()));
    let root = config.structure.get("./").unwrap();
    let patterns = root
        .files
        .as_ref()
        .unwrap()
        .naming_patterns
        .as_ref()
        .unwrap();
    assert_eq!(patterns.get("*.js"), Some(&"camelCase".to_string()));
    assert_eq!(patterns.get("*.ts"), Some(&"kebab-case".to_string()));
    assert_eq!(patterns.get("*.rs"), Some(&"snake_case".to_string()));

    let src = root.children.as_ref().unwrap().get("src").unwrap();
    let src_patterns = src
        .files
        .as_ref()
        .unwrap()
        .naming_patterns
        .as_ref()
        .unwrap();
    assert_eq!(src_patterns.get("*.js"), Some(&"camelCase".to_string()));
    assert_eq!(src_patterns.get("*.ts"), Some(&"kebab-case".to_string()));

    let components = src.children.as_ref().unwrap().get("components").unwrap();
    let component_patterns = components
        .files
        .as_ref()
        .unwrap()
        .naming_patterns
        .as_ref()
        .unwrap();
    assert_eq!(
        component_patterns.get("*.tsx"),
        Some(&"PascalCase".to_string())
    );
    assert_eq!(
        component_patterns.get("*.test.tsx"),
        Some(&"kebab-case".to_string())
    );
}

#[test]
fn test_convert_ls_lint_dir_and_exists_rules() {
    let ls_lint_yaml = r#"
ls:
  components:
    .dir: kebab-case
    .*: exists:0
    .ts: kebab-case | exists:1
"#;

    let config = convert_ls_lint_to_config(ls_lint_yaml).unwrap();
    let root = config.structure.get("./").unwrap();
    let components = root.children.as_ref().unwrap().get("components").unwrap();
    let dirs = components.self_directory.as_ref().unwrap();
    assert_eq!(dirs.naming.as_deref(), Some("kebab-case"));
    let files = components.files.as_ref().unwrap();
    assert_eq!(
        files.exists.as_ref().unwrap().get("*.*"),
        Some(&"0".to_string())
    );
    assert_eq!(
        files.exists.as_ref().unwrap().get("*.ts"),
        Some(&"1".to_string())
    );
}

#[test]
fn test_converts_directory_glob_scopes() {
    for scope in ["packages/*", "**", "{src,tests}"] {
        let ls_lint_yaml = format!(
            r#"
ls:
  "{scope}":
    .ts: kebab-case
"#
        );

        let config = convert_ls_lint_to_config(&ls_lint_yaml).unwrap();
        let root = config.structure.get("./").unwrap();
        assert!(root.children.as_ref().unwrap().contains_key(scope));
    }
}
