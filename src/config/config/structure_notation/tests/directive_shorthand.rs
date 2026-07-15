//! Equivalence coverage for concise and expanded node directives.

use super::parse_config;

#[test]
fn reusable_file_directive_shorthand_matches_expanded_attributes() {
    let shorthand = parse_config(
        r#"
rules:
  source-file:
    naming: kebab-case
    max_lines: 500
structure:
  src/:
    .ts: $source-file
    .tsx: $source-file
"#,
    )
    .unwrap();
    let expanded = parse_config(
        r#"
structure:
  src/:
    .ts:
      naming: kebab-case
      max_lines: 500
    .tsx:
      naming: kebab-case
      max_lines: 500
"#,
    )
    .unwrap();

    for config in [&shorthand, &expanded] {
        let files = config
            .structure
            .get("src/")
            .and_then(|node| node.files.as_ref())
            .unwrap();
        assert_eq!(files.max_lines, None);
        let naming = files.naming_patterns.as_ref().unwrap();
        assert_eq!(naming.get("*.ts"), Some(&"kebab-case".to_string()));
        assert_eq!(naming.get("*.tsx"), Some(&"kebab-case".to_string()));
        let line_limits = files.max_lines_patterns.as_ref().unwrap();
        assert_eq!(line_limits.get("*.ts"), Some(&500));
        assert_eq!(line_limits.get("*.tsx"), Some(&500));
    }
}

#[test]
fn pattern_scoped_line_limits_are_semantically_validated() {
    let error = parse_config(
        r#"
structure:
  src/:
    .ts:
      max_lines: 0
"#,
    )
    .unwrap_err()
    .to_string();

    assert!(
        error.contains("max_lines_patterns.*.ts"),
        "unexpected error: {error}"
    );
}

#[test]
fn recursive_exists_file_globs_are_rejected_until_recursive_counting_is_supported() {
    let error = parse_config(
        r#"
structure:
  ./:
    "./**/*.ts": exists:1
"#,
    )
    .unwrap_err()
    .to_string();

    assert!(
        error.contains("cannot use exists across directories"),
        "unexpected error: {error}"
    );

    let direct = parse_config(
        r#"
structure:
  ./:
    "./*.ts": exists:1
"#,
    )
    .unwrap();
    assert_eq!(
        direct
            .structure
            .get("./")
            .and_then(|node| node.files.as_ref())
            .and_then(|files| files.exists.as_ref())
            .and_then(|exists| exists.get("*.ts")),
        Some(&"1".to_string())
    );
}

#[test]
fn literal_hierarchy_uses_exists_cardinality_and_scalar_tree_rules() {
    let config = parse_config(
        r#"
rules:
  web-app:
    package.json: exists:1
    src/: exists:1
structure:
  ./:
    extra: false
    docs/:
      exists: 0-1
      README.md: exists:0-1
    apps/:
      web/: $web-app
"#,
    )
    .unwrap();

    let root = config.structure.get("./").unwrap();
    assert_eq!(
        root.directories
            .as_ref()
            .and_then(|directories| directories.exists.as_ref())
            .and_then(|exists| exists.get("apps")),
        Some(&"1".to_string())
    );
    assert_eq!(
        root.directories
            .as_ref()
            .and_then(|directories| directories.exists.as_ref())
            .and_then(|exists| exists.get("docs")),
        Some(&"0-1".to_string())
    );

    let apps = root
        .children
        .as_ref()
        .and_then(|children| children.get("apps"))
        .unwrap();
    assert!(!apps.required);
    assert_eq!(
        apps.directories
            .as_ref()
            .and_then(|directories| directories.exists.as_ref())
            .and_then(|exists| exists.get("web")),
        Some(&"1".to_string())
    );

    let web = apps
        .children
        .as_ref()
        .and_then(|children| children.get("web"))
        .unwrap();
    assert!(!web.required);
    assert_eq!(
        web.files
            .as_ref()
            .and_then(|files| files.exists.as_ref())
            .and_then(|exists| exists.get("package.json")),
        Some(&"1".to_string())
    );
    assert_eq!(
        web.directories
            .as_ref()
            .and_then(|directories| directories.exists.as_ref())
            .and_then(|exists| exists.get("src")),
        Some(&"1".to_string())
    );
}

#[test]
fn scalar_directory_rule_matches_expanded_use_form() {
    let shorthand = parse_config(
        r#"
rules:
  package:
    AGENTS.md: exists:1
structure:
  packages/:
    core/: $package
"#,
    )
    .unwrap();
    let expanded = parse_config(
        r#"
rules:
  package:
    AGENTS.md: exists:1
structure:
  packages/:
    core/:
      use: $package
"#,
    )
    .unwrap();

    let shorthand_core = shorthand
        .structure
        .get("packages/")
        .and_then(|node| node.children.as_ref())
        .and_then(|children| children.get("core"))
        .unwrap();
    let expanded_core = expanded
        .structure
        .get("packages/")
        .and_then(|node| node.children.as_ref())
        .and_then(|children| children.get("core"))
        .unwrap();
    assert_eq!(
        serde_yaml::to_value(shorthand_core).unwrap(),
        serde_yaml::to_value(expanded_core).unwrap()
    );
}

#[test]
fn reusable_rule_references_reject_unknown_mismatched_and_cyclic_fragments() {
    for (index, (yaml, expected)) in [
        (
            r#"
structure:
  packages/:
    core/: $missing
"#,
            "unknown Assura config rule '$missing'",
        ),
        (
            r#"
rules:
  source-file:
    naming: kebab-case
structure:
  packages/:
    core/: $source-file
"#,
            "node fragment but a tree fragment is required",
        ),
        (
            r#"
rules:
  package:
    AGENTS.md: exists:1
structure:
  src/:
    .ts: $package
"#,
            "tree fragment but a node fragment is required",
        ),
        (
            r#"
rules:
  a:
    child/:
      use: $b
  b:
    child/:
      use: $a
structure:
  ./:
    use: $a
"#,
            "Assura config rule cycle detected",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let error = parse_config(yaml)
            .err()
            .unwrap_or_else(|| panic!("reference case {index} unexpectedly parsed"))
            .to_string();
        assert!(
            error.contains(expected),
            "reference case {index} returned an unexpected error: {error}"
        );
    }
}

#[test]
fn rule_reference_validation_ignores_relationship_labels() {
    let config = parse_config(
        r#"
rules:
  document:
    needs: $docs
    provides: $document
    max_lines: 500
structure:
  docs/:
    "{document}.md": $document
"#,
    )
    .unwrap();

    assert!(config.structure.contains_key("docs/"));
}

#[test]
fn dollar_references_compose_in_order_before_local_overrides() {
    let config = parse_config(
        r#"
rules:
  repository-files:
    extra: false
    AGENTS.md: exists:1
  source-layout:
    src/: exists:1
structure:
  ./:
    use:
      - $repository-files
      - $source-layout
    extra: true
    README.md: exists:0-1
"#,
    )
    .unwrap();

    let root = config.structure.get("./").unwrap();
    let files = root.files.as_ref().unwrap();
    assert_eq!(files.allow_extra, Some(true));
    assert_eq!(
        files.exists.as_ref().unwrap().get("AGENTS.md"),
        Some(&"1".to_string())
    );
    assert_eq!(
        files.exists.as_ref().unwrap().get("README.md"),
        Some(&"0-1".to_string())
    );
    assert_eq!(
        root.directories
            .as_ref()
            .and_then(|directories| directories.exists.as_ref())
            .and_then(|exists| exists.get("src")),
        Some(&"1".to_string())
    );
}

#[test]
fn removed_rule_sigils_have_migration_guidance() {
    for (index, (yaml, expected)) in [
        (
            r#"
rules:
  "@source-file":
    naming: kebab-case
structure: {}
"#,
            "replace '@source-file' with 'source-file'",
        ),
        (
            r#"
rules:
  source-file:
    naming: kebab-case
structure:
  ./:
    .ts: "@source-file"
"#,
            "replace '@source-file' with '$source-file'",
        ),
        (
            r#"
rules:
  $source-file:
    naming: kebab-case
structure: {}
"#,
            "replace '$source-file' with 'source-file'",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let error = parse_config(yaml)
            .err()
            .unwrap_or_else(|| panic!("migration case {index} unexpectedly parsed"))
            .to_string();
        assert!(
            error.contains(expected),
            "migration case {index} returned an unexpected error: {error}"
        );
    }
}

#[test]
fn exact_file_mapping_defaults_to_exists_one() {
    let config = parse_config(
        r#"
structure:
  ./:
    AGENTS.md:
      max_lines: 300
"#,
    )
    .unwrap();

    assert_eq!(
        config
            .structure
            .get("./")
            .and_then(|node| node.files.as_ref())
            .and_then(|files| files.exists.as_ref())
            .and_then(|exists| exists.get("AGENTS.md")),
        Some(&"1".to_string())
    );
}

#[test]
fn direct_child_directory_patterns_accept_explicit_ranges() {
    let config = parse_config(
        r#"
structure:
  packages/:
    "{package}/":
      exists: 1-20
      package.json: exists:1
"#,
    )
    .unwrap();

    assert_eq!(
        config
            .structure
            .get("packages/")
            .and_then(|node| node.directories.as_ref())
            .and_then(|directories| directories.exists.as_ref())
            .and_then(|exists| exists.get("{package}")),
        Some(&"1-20".to_string())
    );
}

#[test]
fn removed_required_notation_has_cardinality_migration_guidance() {
    let error = parse_config(
        r#"
structure:
  ./:
    docs/:
      required: false
"#,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("no longer use 'required'"), "{error}");
    assert!(error.contains("exists:0-1"), "{error}");
}

#[test]
fn ambiguous_or_impossible_directory_cardinality_is_rejected() {
    for (yaml, expected) in [
        (
            r#"
structure:
  ./:
    packages/*/src/: exists:1
"#,
            "nested direct-child scopes",
        ),
        (
            r#"
structure:
  ./:
    docs/:
      exists: 0
      README.md: exists:1
"#,
            "cannot combine exists:0 with child policy",
        ),
        (
            r#"
structure:
  ./:
    docs/: exists:2
"#,
            "can only use exists:0",
        ),
    ] {
        let error = parse_config(yaml).unwrap_err().to_string();
        assert!(error.contains(expected), "unexpected error: {error}");
    }
}
