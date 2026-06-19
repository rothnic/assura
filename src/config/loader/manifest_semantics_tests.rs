//! Loader tests for manifest-semantics extension config.

use super::ConfigLoader;

#[test]
fn test_parse_with_manifest_semantics() {
    let yaml = r#"
extensions:
  manifest_semantics:
    - id: cargo_workspace
      severity: high
      manifests:
        - path: Cargo.toml
          package: assura
          role: public
          version: "0.1.0"
          rust_version: "1.70.0"
          license: "MIT OR Apache-2.0"
          publish: public
          description_required_terms: ["structure-first"]
          description_forbidden_terms: ["dependency graph validation"]
          keywords: ["structure"]
          binaries: ["assura"]
structure: {}
"#;

    let config = ConfigLoader::parse(yaml).unwrap();
    let extensions = config.extensions.unwrap();
    assert_eq!(extensions.manifest_semantics.len(), 1);
    let policy = &extensions.manifest_semantics[0];
    assert_eq!(policy.id, "cargo_workspace");
    assert_eq!(policy.manifests[0].path, "Cargo.toml");
    assert_eq!(policy.manifests[0].package.as_deref(), Some("assura"));
}

#[test]
fn test_parse_rejects_manifest_semantics_invalid_publish_policy() {
    let yaml = r#"
extensions:
  manifest_semantics:
    - id: cargo_workspace
      manifests:
        - path: Cargo.toml
          publish: private
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("extensions.manifest_semantics.cargo_workspace.manifests.publish"),
        "unexpected error: {error}"
    );
}

#[test]
fn test_parse_rejects_manifest_semantics_path_escape() {
    let yaml = r#"
extensions:
  manifest_semantics:
    - id: cargo_workspace
      manifests:
        - path: ../Cargo.toml
          package: assura
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("manifests.path") && error.contains("must be relative"),
        "unexpected error: {error}"
    );
}
