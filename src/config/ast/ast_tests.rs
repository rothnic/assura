//! Unit tests for the parent module.
use super::*;

#[test]
fn test_parse_simple_rule() {
    let yaml = r#"
rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase, lines:..400]
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: react
"#;

    let config = LegacyNotationConfig::from_yaml(yaml).expect("Should parse");
    assert!(config.rules.contains_key("react"));
}

#[test]
fn test_parse_violation_array() {
    let yaml = r#"[warn, ci:block, feature:warn]"#;

    // This would be part of a larger structure
    let entries: Vec<ViolationEntry> = serde_yaml::from_str(yaml).expect("Should parse");
    assert_eq!(entries.len(), 3);
}

#[test]
fn test_json_roundtrip() {
    let yaml = r#"
rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn]

policy:
  src/:
    ${name}.tsx:
      - apply: react
"#;

    let config = LegacyNotationConfig::from_yaml(yaml).expect("Should parse YAML");
    let json = config.to_json().expect("Should serialize to JSON");
    let config2: LegacyNotationConfig = serde_json::from_str(&json).expect("Should parse JSON");

    assert_eq!(config.rules.len(), config2.rules.len());
}
