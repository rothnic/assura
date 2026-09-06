//! Unit tests for the parent module.
use super::*;

#[test]
fn test_rule_builder() {
    let rule = Rule::new()
        .with_extensions(vec!["rs".to_string()])
        .with_naming(NamingConvention::Single(Case::SnakeCase))
        .with_max_lines(500);

    assert_eq!(rule.extensions.unwrap(), vec!["rs"]);
    assert!(rule.max_lines.is_some());
}

#[test]
fn test_naming_convention_single() {
    let yaml = "snake_case";
    let conv: NamingConvention = serde_yaml::from_str(yaml).unwrap();
    match conv {
        NamingConvention::Single(Case::SnakeCase) => {}
        _ => panic!("Expected single snake_case"),
    }
}

#[test]
fn test_naming_convention_multiple() {
    let yaml = "[snake_case, camelCase]";
    let conv: NamingConvention = serde_yaml::from_str(yaml).unwrap();
    match conv {
        NamingConvention::Multiple(cases) => {
            assert_eq!(cases.len(), 2);
        }
        _ => panic!("Expected multiple cases"),
    }
}

#[test]
fn test_config_builder() {
    let config = LegacyPolicyConfig::new()
        .with_rule("rust", Rule::new().with_max_lines(500))
        .with_exclude("target/**");

    assert!(config.rules.contains_key("rust"));
    assert_eq!(config.exclude.len(), 1);
}

#[test]
fn test_yaml_serialization() {
    let config = LegacyPolicyConfig::new().with_rule(
        "rust-source",
        Rule::new()
            .with_extensions(vec!["rs".to_string()])
            .with_naming(NamingConvention::Single(Case::SnakeCase))
            .with_max_lines(500),
    );

    let yaml = serde_yaml::to_string(&config).unwrap();
    assert!(yaml.contains("rules:"));
    assert!(yaml.contains("rust-source:"));
}

#[test]
fn test_case_deserialization() {
    let cases = vec![
        ("snake_case", Case::SnakeCase),
        ("camelCase", Case::CamelCase),
        ("PascalCase", Case::PascalCase),
        ("kebab-case", Case::KebabCase),
        ("SCREAMING_SNAKE_CASE", Case::ScreamingSnakeCase),
    ];

    for (yaml, expected) in cases {
        let parsed: Case = serde_yaml::from_str(yaml).unwrap();
        assert!(
            std::mem::discriminant(&parsed) == std::mem::discriminant(&expected),
            "Failed for {}",
            yaml
        );
    }
}
