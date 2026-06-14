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
    assert_eq!(relationship.providers.len(), 1);
    assert_eq!(
        relationship.providers[0].path,
        "src/components/{component}.test.tsx"
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
    assert_eq!(relationship.need, "doc");
    assert_eq!(relationship.providers.len(), 2);
    assert!(relationship.providers.iter().any(|provider| {
        provider.path == "docs/packages/{package}.md" && provider.section.is_none()
    }));
    assert!(relationship.providers.iter().any(|provider| {
        provider.path == "docs/packages.md" && provider.section.as_deref() == Some("{package}")
    }));
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
