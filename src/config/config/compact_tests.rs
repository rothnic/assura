//! Unit coverage for compact config normalization.

use serde_yaml::Value;

use super::normalize_compact_config_value;

fn normalize(yaml: &str) -> Value {
    let value: Value = serde_yaml::from_str(yaml).unwrap();
    normalize_compact_config_value(value).unwrap()
}

fn key(value: &str) -> Value {
    Value::String(value.to_string())
}

#[test]
fn normalizes_file_directory_extension_and_extra_shorthand() {
    let value = normalize(
        r#"
structure:
  ./:
    extra: false
    README.md: exists:1
    .gitignore: exists:1
    docs/: exists:0-1
    .md: kebab-case
    .graphql: kebab-case
"#,
    );
    let root = value
        .as_mapping()
        .unwrap()
        .get(key("structure"))
        .unwrap()
        .as_mapping()
        .unwrap()
        .get(key("./"))
        .unwrap()
        .as_mapping()
        .unwrap();

    let files = root.get(key("files")).unwrap().as_mapping().unwrap();
    assert_eq!(
        files
            .get(key("exists"))
            .unwrap()
            .as_mapping()
            .unwrap()
            .get(key("README.md"))
            .unwrap()
            .as_str(),
        Some("1")
    );
    assert_eq!(
        files
            .get(key("exists"))
            .unwrap()
            .as_mapping()
            .unwrap()
            .get(key(".gitignore"))
            .unwrap()
            .as_str(),
        Some("1")
    );
    assert_eq!(
        files
            .get(key("naming_patterns"))
            .unwrap()
            .as_mapping()
            .unwrap()
            .get(key("*.md"))
            .unwrap()
            .as_str(),
        Some("kebab-case")
    );
    assert_eq!(
        files
            .get(key("naming_patterns"))
            .unwrap()
            .as_mapping()
            .unwrap()
            .get(key("*.graphql"))
            .unwrap()
            .as_str(),
        Some("kebab-case")
    );
    assert_eq!(
        files.get(key("allow_extra")).unwrap().as_bool(),
        Some(false)
    );

    let directories = root.get(key("directories")).unwrap().as_mapping().unwrap();
    assert_eq!(
        directories
            .get(key("exists"))
            .unwrap()
            .as_mapping()
            .unwrap()
            .get(key("docs"))
            .unwrap()
            .as_str(),
        Some("0-1")
    );
    assert_eq!(
        directories
            .get(key("allowed_names"))
            .unwrap()
            .as_sequence()
            .unwrap()
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>(),
        vec!["docs"]
    );
}

#[test]
fn normalizes_combined_extension_exists_and_naming_as_allowed_pattern() {
    let value = normalize(
        r#"
structure:
  ./:
    .md:
      exists: 1
      naming: kebab-case
"#,
    );
    let files = value["structure"]["./"]["files"].as_mapping().unwrap();
    assert_eq!(
        files["exists"]
            .as_mapping()
            .unwrap()
            .get(key("*.md"))
            .unwrap(),
        "1"
    );
    assert_eq!(
        files["naming_patterns"]
            .as_mapping()
            .unwrap()
            .get(key("*.md"))
            .unwrap(),
        "kebab-case"
    );
    assert!(files.get(key("allowed_patterns")).is_none());
}

#[test]
fn normalizes_self_directory_rule() {
    let value = normalize(
        r#"
structure:
  packages/*/:
    .dir: kebab-case
"#,
    );
    let package = value["structure"]["packages/*/"].as_mapping().unwrap();
    assert_eq!(
        package["self_directory"]["naming"].as_str(),
        Some("kebab-case")
    );
    assert_eq!(package["required"].as_bool(), Some(false));
}

#[test]
fn use_fragments_merge_left_to_right_then_local_wins() {
    let value = normalize(
        r#"
rules:
  base:
    README.md: exists:1
  override:
    README.md: exists:0-1
    AGENTS.md: exists:1
structure:
  ./:
    use: ["@base", "@override"]
    README.md: exists:0
"#,
    );
    let exists = value["structure"]["./"]["files"]["exists"]
        .as_mapping()
        .unwrap();
    assert_eq!(exists.get(key("README.md")).unwrap().as_str(), Some("0"));
    assert_eq!(exists.get(key("AGENTS.md")).unwrap().as_str(), Some("1"));
}

#[test]
fn tree_fragments_support_multiple_required_package_files() {
    let value = normalize(
        r#"
rules:
  package-standard:
    README.md: exists:1
    AGENTS.md: exists:1
    package.json: exists:1
    src/: exists:1
structure:
  packages/*/:
    use: "@package-standard"
"#,
    );
    let package = &value["structure"]["packages/*/"];
    let file_exists = package["files"]["exists"].as_mapping().unwrap();
    assert_eq!(
        file_exists.get(key("README.md")).unwrap().as_str(),
        Some("1")
    );
    assert_eq!(
        file_exists.get(key("AGENTS.md")).unwrap().as_str(),
        Some("1")
    );
    assert_eq!(
        file_exists.get(key("package.json")).unwrap().as_str(),
        Some("1")
    );
    assert_eq!(package["directories"]["exists"]["src"].as_str(), Some("1"));
}

#[test]
fn pattern_structure_scopes_are_not_required_by_default() {
    let value = normalize(
        r#"
structure:
  packages/*/:
    README.md: exists:1
"#,
    );
    assert_eq!(
        value["structure"]["packages/*/"]["required"].as_bool(),
        Some(false)
    );
}

#[test]
fn rejects_unknown_rule_reference() {
    let value: Value = serde_yaml::from_str(
        r#"
structure:
  ./:
    use: "@missing"
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("unknown compact config rule '@missing'"));
}

#[test]
fn rejects_wrong_fragment_kind() {
    let value: Value = serde_yaml::from_str(
        r#"
rules:
  readme:
    exists: 1
structure:
  ./:
    use: "@readme"
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("node fragment but a tree fragment is required"));
}

#[test]
fn rejects_tree_fragment_used_as_node_rule() {
    let value: Value = serde_yaml::from_str(
        r#"
rules:
  project-docs:
    README.md: exists:1
structure:
  ./:
    README.md: "@project-docs"
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("tree fragment but a node fragment is required"));
}

#[test]
fn rejects_unknown_node_rule_reference() {
    let value: Value = serde_yaml::from_str(
        r#"
structure:
  ./:
    README.md: "@missing"
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("unknown compact config rule '@missing'"));
}

#[test]
fn rejects_inverted_exists_ranges() {
    let value: Value = serde_yaml::from_str(
        r#"
structure:
  ./:
    README.md: exists:2-1
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("lower bound greater than its upper bound"));
}

#[test]
fn rejects_bare_exists_node_directive() {
    let value: Value = serde_yaml::from_str(
        r#"
structure:
  ./:
    README.md: exists
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("exists shorthand must include cardinality"));
}

#[test]
fn rejects_numeric_node_directive() {
    let value: Value = serde_yaml::from_str(
        r#"
structure:
  ./:
    README.md: 1
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("node directive numbers are not supported"));
}

#[test]
fn rejects_lossy_exact_file_attributes() {
    let value: Value = serde_yaml::from_str(
        r#"
structure:
  ./:
    README.md: kebab-case
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("exact file key 'README.md' only supports exists"));
}

#[test]
fn rejects_lossy_exact_directory_attributes() {
    let value: Value = serde_yaml::from_str(
        r#"
structure:
  ./:
    docs/: kebab-case
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("exact directory key 'docs/' only supports exists"));
}

#[test]
fn rejects_compact_severity_attribute() {
    let value: Value = serde_yaml::from_str(
        r#"
structure:
  ./:
    README.md:
      exists: 1
      severity: high
"#,
    )
    .unwrap();
    let error = normalize_compact_config_value(value).unwrap_err();
    assert!(error.contains("attribute 'severity' is not supported"));
}
