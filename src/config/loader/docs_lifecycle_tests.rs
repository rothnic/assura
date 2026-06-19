//! Docs-lifecycle config loader tests.

use super::ConfigLoader;

#[test]
fn test_parse_with_docs_lifecycle() {
    let yaml = r#"
extensions:
  docs_lifecycles:
    - id: project_docs
      severity: high
      active:
        - docs/**/*.md
      historical:
        - docs/archive/**
      require_frontmatter_status:
        - docs/goals/*.md
      allowed_statuses:
        - planned
        - completed
        - archived
      claim_patterns:
        - id: release_assets
          pattern: "assura-*.tar.gz"
          evidence_files:
            - docs/release-notes.md
      historical_exceptions:
        - docs/archive/**
structure: {}
"#;

    let config = ConfigLoader::parse(yaml).unwrap();
    let extensions = config.extensions.unwrap();
    assert_eq!(extensions.docs_lifecycles.len(), 1);
    let policy = &extensions.docs_lifecycles[0];
    assert_eq!(policy.id, "project_docs");
    assert_eq!(
        policy.allowed_statuses,
        vec!["planned", "completed", "archived"]
    );
    assert_eq!(policy.claim_patterns[0].id, "release_assets");
}

#[test]
fn test_parse_rejects_docs_lifecycle_missing_allowed_statuses() {
    let yaml = r#"
extensions:
  docs_lifecycles:
    - id: project_docs
      active:
        - docs/**/*.md
      require_frontmatter_status:
        - docs/goals/*.md
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("extensions.docs_lifecycles.project_docs.allowed_statuses"),
        "unexpected error: {error}"
    );
}

#[test]
fn test_parse_rejects_docs_lifecycle_path_escape() {
    let yaml = r#"
extensions:
  docs_lifecycles:
    - id: project_docs
      active:
        - ../docs/**/*.md
      require_frontmatter_status:
        - docs/goals/*.md
      allowed_statuses:
        - planned
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("active") && error.contains("must be relative"),
        "unexpected error: {error}"
    );
}

#[test]
fn test_parse_rejects_docs_lifecycle_claim_without_evidence() {
    let yaml = r#"
extensions:
  docs_lifecycles:
    - id: project_docs
      active:
        - docs/**/*.md
      allowed_statuses:
        - active
      claim_patterns:
        - id: performance_current
          pattern: "2x"
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("claim_patterns.performance_current") && error.contains("evidence file"),
        "unexpected error: {error}"
    );
}
