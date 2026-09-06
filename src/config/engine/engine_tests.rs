//! Unit tests for the parent module.
use super::*;
use crate::config::types::Case;

fn create_test_config() -> LegacyPolicyConfig {
    LegacyPolicyConfig::new()
        .with_rule(
            "rust-source",
            Rule::new()
                .with_extensions(vec!["rs".to_string()])
                .with_naming(NamingConvention::Single(Case::SnakeCase))
                .with_max_lines(500),
        )
        .with_rule(
            "rust-test",
            Rule::new()
                .with_extensions(vec!["rs".to_string()])
                .with_naming(NamingConvention::Single(Case::SnakeCase))
                .with_max_lines(1000),
        )
        .with_policy(
            PolicyNode::new()
                .with_entry(
                    "src/",
                    PolicyEntry::InlineRule(InlineRule {
                        extensions: Some(vec!["rs".to_string()]),
                        naming: Some(NamingConvention::Single(Case::SnakeCase)),
                        max_lines: Some(500),
                        max_size: None,
                        require_docs: None,
                        require_test: None,
                        message: None,
                        severity: None,
                    }),
                )
                .with_entry(
                    "src/components/",
                    PolicyEntry::InlineRule(InlineRule {
                        extensions: Some(vec!["rs".to_string()]),
                        naming: Some(NamingConvention::Single(Case::PascalCase)),
                        max_lines: Some(300),
                        max_size: None,
                        require_docs: None,
                        require_test: None,
                        message: None,
                        severity: None,
                    }),
                ),
        )
}

#[test]
fn test_resolve_simple_path() {
    let config = create_test_config();
    let engine = PolicyEngine::new(config);

    let rules = engine.resolve(Path::new("src/main.rs"));
    assert_eq!(rules.max_lines, Some(500));
    assert!(matches!(
        rules.naming,
        Some(NamingConvention::Single(Case::SnakeCase))
    ));
}

#[test]
fn test_resolve_nested_path() {
    let config = create_test_config();
    let engine = PolicyEngine::new(config);

    // src/components/ has more specific rules that override src/
    let rules = engine.resolve(Path::new("src/components/Button.rs"));
    assert_eq!(rules.max_lines, Some(300));
    assert!(matches!(
        rules.naming,
        Some(NamingConvention::Single(Case::PascalCase))
    ));
}

#[test]
fn test_resolve_non_matching_path() {
    let config = create_test_config();
    let engine = PolicyEngine::new(config);

    let rules = engine.resolve(Path::new("tests/test.rs"));
    // No matching rules
    assert_eq!(rules.max_lines, None);
    assert_eq!(rules.naming, None);
}

#[test]
fn test_specificity_rules() {
    let config = LegacyPolicyConfig::new().with_policy(
        PolicyNode::new()
            .with_entry(
                "src/",
                PolicyEntry::InlineRule(InlineRule {
                    extensions: None,
                    naming: Some(NamingConvention::Single(Case::SnakeCase)),
                    max_lines: Some(500),
                    max_size: None,
                    require_docs: None,
                    require_test: None,
                    message: None,
                    severity: None,
                }),
            )
            .with_entry(
                "src/deep/nested/",
                PolicyEntry::InlineRule(InlineRule {
                    extensions: None,
                    naming: Some(NamingConvention::Single(Case::PascalCase)),
                    max_lines: Some(100),
                    max_size: None,
                    require_docs: None,
                    require_test: None,
                    message: None,
                    severity: None,
                }),
            ),
    );

    let engine = PolicyEngine::new(config);

    let rules = engine.resolve(Path::new("src/deep/nested/file.rs"));
    assert_eq!(rules.max_lines, Some(100));
    assert!(matches!(
        rules.naming,
        Some(NamingConvention::Single(Case::PascalCase))
    ));
}

#[test]
fn test_rule_ref_resolution() {
    let config = LegacyPolicyConfig::new()
        .with_rule(
            "my-rule",
            Rule::new()
                .with_naming(NamingConvention::Single(Case::SnakeCase))
                .with_max_lines(500),
        )
        .with_policy(
            PolicyNode::new().with_entry("src/", PolicyEntry::RuleRef("@my-rule".to_string())),
        );

    let engine = PolicyEngine::new(config);
    let rules = engine.resolve(Path::new("src/main.rs"));

    assert_eq!(rules.max_lines, Some(500));
}

#[test]
fn test_extension_specificity() {
    let config = LegacyPolicyConfig::new().with_policy(
        PolicyNode::new()
            .with_entry(
                "src/",
                PolicyEntry::InlineRule(InlineRule {
                    extensions: None,
                    naming: Some(NamingConvention::Single(Case::SnakeCase)),
                    max_lines: Some(500),
                    max_size: None,
                    require_docs: None,
                    require_test: None,
                    message: None,
                    severity: None,
                }),
            )
            .with_entry(
                "rs", // Extension
                PolicyEntry::InlineRule(InlineRule {
                    extensions: None,
                    naming: Some(NamingConvention::Single(Case::PascalCase)),
                    max_lines: Some(1000),
                    max_size: None,
                    require_docs: None,
                    require_test: None,
                    message: None,
                    severity: None,
                }),
            ),
    );

    let engine = PolicyEngine::new(config);

    // Path match should win over extension match
    let rules = engine.resolve(Path::new("src/main.rs"));
    assert_eq!(rules.max_lines, Some(500));
    assert!(matches!(
        rules.naming,
        Some(NamingConvention::Single(Case::SnakeCase))
    ));
}
