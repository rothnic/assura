//! Loader tests for test-relationship extension config.

use super::ConfigLoader;

#[test]
fn test_parse_with_test_relationship() {
    let yaml = r#"
extensions:
  test_relationships:
    - id: supported_tests
      severity: high
      relationships:
        - source: "src/cli/check/*.rs"
          required_tests:
            - "tests/custom_constraints_tests.rs"
      fixture_roots:
        - tests/fixtures
      fixture_families:
        - path: tests/fixtures/test-relationship
          owner: validation-tests
          purpose: reusable test relationship rule coverage
      allowed_ignore_reasons:
        - manual_performance_audit
      ignored_tests:
        - path: tests/manual_performance.rs
          test: manual_performance_audit
          reason: manual_performance_audit
structure: {}
"#;

    let config = ConfigLoader::parse(yaml).unwrap();
    let extensions = config.extensions.unwrap();
    assert_eq!(extensions.test_relationships.len(), 1);
    let policy = &extensions.test_relationships[0];
    assert_eq!(policy.id, "supported_tests");
    assert_eq!(policy.relationships[0].source, "src/cli/check/*.rs");
    assert_eq!(policy.fixture_families[0].owner, "validation-tests");
    assert_eq!(policy.ignored_tests[0].test, "manual_performance_audit");
    assert_eq!(policy.ignored_tests[0].reason, "manual_performance_audit");
}

#[test]
fn test_parse_rejects_test_relationship_unknown_ignore_reason() {
    let yaml = r#"
extensions:
  test_relationships:
    - id: supported_tests
      allowed_ignore_reasons:
        - manual_performance_audit
      ignored_tests:
        - path: tests/manual.rs
          test: manual_test
          reason: temporary
structure: {}
"#;

    let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
    assert!(
        error.contains("extensions.test_relationships.supported_tests.ignored_tests")
            && error.contains("not listed in allowed_ignore_reasons"),
        "unexpected error: {error}"
    );
}
