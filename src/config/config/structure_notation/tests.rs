//! Tests for Assura's concise structure notation normalizer.

use crate::config::config::ConfigLoader;

fn parse_config(yaml: &str) -> crate::cli::config::ConfigResult<crate::config::config::Config> {
    ConfigLoader::parse(yaml)
}

#[test]
fn normalizes_concise_file_directory_and_extension_directives() {
    let config = parse_config(
        r#"
structure:
  ./:
    extra: false
    README.md: exists:1
    "*.lock": exists:0-1
    src/: exists:1
    .rs: snake_case
"#,
    )
    .unwrap();

    let root = config.structure.get("./").unwrap();
    let files = root.files.as_ref().unwrap();
    assert_eq!(
        files.exists.as_ref().unwrap().get("README.md"),
        Some(&"1".to_string())
    );
    assert_eq!(
        files.allowed_names.as_ref().unwrap(),
        &vec!["README.md".to_string()]
    );
    assert_eq!(
        files.allowed_patterns.as_ref().unwrap(),
        &vec!["*.lock".to_string()]
    );
    assert_eq!(
        files.naming_patterns.as_ref().unwrap().get("*.rs"),
        Some(&"snake_case".to_string())
    );
    assert_eq!(files.allow_extra, Some(false));

    let directories = root.directories.as_ref().unwrap();
    assert_eq!(
        directories.exists.as_ref().unwrap().get("src"),
        Some(&"1".to_string())
    );
    assert_eq!(
        directories.allowed_names.as_ref().unwrap(),
        &vec!["src".to_string()]
    );
    assert_eq!(directories.allow_extra, Some(false));
}

#[test]
fn scalar_exists_on_captured_path_creates_required_counterpart_relationship() {
    let config = parse_config(
        r#"
structure:
  src/components/:
    "{component}.tsx": {}
    "{component}.test.tsx": exists:1
"#,
    )
    .unwrap();

    let relationships = &config.extensions.unwrap().relationships;
    assert_eq!(relationships.len(), 1);
    let relationship = &relationships[0];
    assert_eq!(relationship.source, "src/components/{component}.tsx");
    assert_eq!(
        relationship.source_declaration.as_deref(),
        Some("src/components/{component}.tsx")
    );
    assert_eq!(relationship.providers.len(), 1);
    assert_eq!(
        relationship.providers[0].path,
        "src/components/{component}.test.tsx"
    );
    assert_eq!(
        relationship.providers[0].kind.as_deref(),
        Some("counterpart")
    );
    assert_eq!(
        relationship.providers[0].declaration.as_deref(),
        Some("src/components/{component}.test.tsx")
    );
}

#[test]
fn needs_and_provides_compile_to_provider_alternatives() {
    let config = parse_config(
        r#"
structure:
  packages/:
    "{package}/":
      needs: doc
  docs/packages/:
    required: false
    "{package}.md":
      provides: doc
  docs/:
    required: false
    packages.md:
      sections:
        "{package}":
          provides: doc
"#,
    )
    .unwrap();

    let relationships = &config.extensions.unwrap().relationships;
    assert_eq!(relationships.len(), 1);
    let relationship = &relationships[0];
    assert_eq!(relationship.source, "packages/{package}");
    assert_eq!(
        relationship.source_declaration.as_deref(),
        Some("packages/{package}")
    );
    assert_eq!(relationship.need, "doc");
    assert_eq!(relationship.providers.len(), 2);
    assert!(relationship.providers.iter().any(|provider| {
        provider.path == "docs/packages/{package}.md"
            && provider.section.is_none()
            && provider.kind.as_deref() == Some("file")
            && provider.declaration.as_deref() == Some("docs/packages/{package}.md")
    }));
    assert!(relationship.providers.iter().any(|provider| {
        provider.path == "docs/packages.md"
            && provider.section.as_deref() == Some("{package}")
            && provider.kind.as_deref() == Some("section")
            && provider.declaration.as_deref() == Some("docs/packages.md")
    }));
}

#[test]
fn same_named_captures_in_separate_scopes_pair_with_local_counterparts() {
    let config = parse_config(
        r#"
structure:
  src/components/:
    "{name}.tsx": {}
    "{name}.test.tsx": exists:1
  src/hooks/:
    "{name}.ts": {}
    "{name}.test.ts": exists:1
"#,
    )
    .unwrap();

    let relationships = &config.extensions.unwrap().relationships;
    assert_eq!(relationships.len(), 2);
    assert!(relationships.iter().any(|relationship| {
        relationship.source == "src/components/{name}.tsx"
            && relationship.providers.len() == 1
            && relationship.providers[0].path == "src/components/{name}.test.tsx"
    }));
    assert!(relationships.iter().any(|relationship| {
        relationship.source == "src/hooks/{name}.ts"
            && relationship.providers.len() == 1
            && relationship.providers[0].path == "src/hooks/{name}.test.ts"
    }));
}

#[test]
fn provider_only_captured_entries_do_not_become_counterpart_producers() {
    let config = parse_config(
        r#"
structure:
  docs/packages/:
    "{package}.md":
      provides: doc
    "{package}.review.md": exists:1
"#,
    )
    .unwrap();

    let relationships_are_empty = match config.extensions.as_ref() {
        Some(extensions) => extensions.relationships.is_empty(),
        None => true,
    };
    assert!(
        relationships_are_empty,
        "provider-only captured entries should not produce counterpart relationships: {:#?}",
        config.extensions
    );
}

#[test]
fn ambiguous_captured_counterparts_are_rejected() {
    let error = parse_config(
        r#"
structure:
  src/:
    "{name}.rs": {}
  tests/:
    "{name}_test.rs": exists:1
  docs/:
    "{name}.md": exists:1
"#,
    )
    .unwrap_err()
    .to_string();

    assert!(
        error.contains("ambiguous captured counterparts"),
        "unexpected error: {error}"
    );
}

#[test]
fn rule_keys_with_at_prefix_resolve_from_use_references() {
    let config = parse_config(
        r#"
rules:
  "@readme-standard":
    exists: 1
  "@project-docs":
    README.md: "@readme-standard"
structure:
  ./:
    use: "@project-docs"
"#,
    )
    .unwrap();

    let root = config.structure.get("./").unwrap();
    let files = root.files.as_ref().unwrap();
    assert_eq!(
        files.exists.as_ref().unwrap().get("README.md"),
        Some(&"1".to_string())
    );
    assert_eq!(
        files.allowed_names.as_ref().unwrap(),
        &vec!["README.md".to_string()]
    );
}

mod built_in_rules;

#[test]
fn nested_captured_directory_use_expands_tree_rule_fragments() {
    let config = parse_config(
        r#"
rules:
  "@agent-skill-dir":
    SKILL.md: exists:1
    agents/: exists:0-1
    references/: exists:0-1
    scripts/: exists:0-1
    assets/: exists:0-1
    extra: false
structure:
  .agents/skills/:
    extra: true
    "{skill}/":
      use: "@agent-skill-dir"
"#,
    )
    .unwrap();

    let skills = config.structure.get(".agents/skills/").unwrap();
    let skill = skills
        .children
        .as_ref()
        .and_then(|children| children.get("{skill}"))
        .expect("captured skill child");
    assert_eq!(
        skill
            .files
            .as_ref()
            .and_then(|files| files.exists.as_ref())
            .and_then(|exists| exists.get("SKILL.md")),
        Some(&"1".to_string())
    );
    assert_eq!(
        skill
            .directories
            .as_ref()
            .and_then(|directories| directories.allowed_names.as_ref())
            .map(Vec::as_slice),
        Some(
            &[
                "agents".to_string(),
                "references".to_string(),
                "scripts".to_string(),
                "assets".to_string()
            ][..]
        )
    );
    assert!(
        config
            .extensions
            .as_ref()
            .map_or(true, |extensions| extensions.relationships.is_empty()),
        "exact child requirements inside captured directories should remain structural requirements"
    );
}

#[test]
fn nested_literal_directory_use_expands_tree_rule_fragments() {
    let config = parse_config(
        r#"
rules:
  "@skill-dir":
    SKILL.md: exists:1
structure:
  .agents/skills/:
    demo/:
      use: "@skill-dir"
"#,
    )
    .unwrap();

    let skills = config.structure.get(".agents/skills/").unwrap();
    let skill = skills
        .children
        .as_ref()
        .and_then(|children| children.get("demo"))
        .expect("demo skill child");
    assert_eq!(
        skill
            .files
            .as_ref()
            .and_then(|files| files.exists.as_ref())
            .and_then(|exists| exists.get("SKILL.md")),
        Some(&"1".to_string())
    );
}

#[test]
fn captured_directory_exact_child_providers_remain_explicit_relationships() {
    let config = parse_config(
        r#"
rules:
  "@package-dir":
    README.md:
      provides: doc
    src/: exists:1
structure:
  packages/:
    "{package}/":
      use: "@package-dir"
      needs: doc
"#,
    )
    .unwrap();

    let relationships = &config.extensions.unwrap().relationships;
    assert_eq!(relationships.len(), 1);
    let relationship = &relationships[0];
    assert_eq!(relationship.source, "packages/{package}");
    assert_eq!(relationship.need, "doc");
    assert_eq!(relationship.providers.len(), 1);
    assert_eq!(
        relationship.providers[0].path,
        "packages/{package}/README.md"
    );
    assert_eq!(relationship.providers[0].kind.as_deref(), Some("file"));
}

#[test]
fn detailed_file_directive_merges_markdown_attributes_in_place() {
    let config = parse_config(
        r#"
structure:
  docs/:
    "{topic}.md":
      exists: 1
      markdown:
        required_sections:
          - Summary
"#,
    )
    .unwrap();

    let docs = config.structure.get("docs/").unwrap();
    let files = docs.files.as_ref().unwrap();
    assert_eq!(
        files.allowed_patterns.as_ref().unwrap(),
        &vec!["{topic}.md".to_string()]
    );
    assert_eq!(
        docs.markdown
            .as_ref()
            .unwrap()
            .required_sections
            .as_ref()
            .unwrap(),
        &vec!["Summary".to_string()]
    );
}

#[test]
fn detailed_file_directive_preserves_markdown_outline_notation() {
    let config = parse_config(
        r#"
structure:
  docs/:
    guide.md:
      markdown:
        outline:
          - Overview
          - ?? Prerequisites
          - Quick Start:
              - Installation
              - ?? Configuration
          - Why Assura?
          - title: "?? Debug Mode"
            optional: false
"#,
    )
    .unwrap();

    let docs = config.structure.get("docs/").unwrap();
    let outline = docs.markdown.as_ref().unwrap().outline.as_ref().unwrap();
    assert_eq!(outline.len(), 5);
}

#[test]
fn section_providers_include_path_and_section_captures() {
    let config = parse_config(
        r#"
structure:
  workspaces/:
    "{workspace}/":
      "{package}/":
        needs: doc
  docs/:
    "{workspace}/":
      packages.md:
        sections:
          "{package}":
            provides: doc
"#,
    )
    .unwrap();

    let relationships = &config.extensions.unwrap().relationships;
    assert_eq!(relationships.len(), 1);
    let relationship = &relationships[0];
    assert_eq!(relationship.source, "workspaces/{workspace}/{package}");
    assert_eq!(relationship.providers.len(), 1);
    assert_eq!(
        relationship.providers[0].path,
        "docs/{workspace}/packages.md"
    );
    assert_eq!(
        relationship.providers[0].section.as_deref(),
        Some("{package}")
    );
}

#[test]
fn duplicate_provider_declarations_are_rejected_as_ambiguous() {
    let error = parse_config(
        r#"
structure:
  packages/:
    "{package}/":
      needs: doc
  docs/packages/:
    "{package}.md":
      provides:
        - doc
        - doc
"#,
    )
    .unwrap_err()
    .to_string();

    assert!(
        error.contains("ambiguous duplicate provider"),
        "unexpected error: {error}"
    );
}

#[test]
fn removed_capture_notations_are_rejected() {
    for path in ["${component}.tsx", "{{component}}.tsx"] {
        let yaml = format!(
            r#"
structure:
  src/components/:
    "{path}": {{}}
"#
        );
        let error = parse_config(&yaml).unwrap_err().to_string();
        assert!(
            error.contains("single braces"),
            "unexpected error for {path}: {error}"
        );
    }
}
