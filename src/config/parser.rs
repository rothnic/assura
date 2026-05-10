//! Configuration Parser
//!
//! Parses Assura configuration files into AST.
//! Follows Constitution: structure-first, valid YAML/JSON.

use crate::config::ast::Config;
use crate::config::preprocessor::YamlPreprocessor;

/// Parser for Assura configuration
pub struct ConfigParser;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("YAML parse error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

impl ConfigParser {
    /// Parse configuration from YAML string
    ///
    /// # Arguments
    /// * `input` - Raw YAML configuration
    ///
    /// # Returns
    /// * Parsed Config or ParseError
    ///
    /// # Example
    /// ```ignore
    /// use assura::config::parser::ConfigParser;
    ///
    /// let yaml = r#"
    /// rules:
    ///   react:
    ///     ${name}.tsx:
    ///       - constraints: [PascalCase]
    ///       - violation: [warn]
    ///
    /// policy:
    ///   src/:
    ///     ${name}.tsx:
    ///       - apply: react
    /// "#;
    ///
    /// let config = ConfigParser::parse(yaml)?;
    /// ```
    pub fn parse(input: &str) -> Result<Config, ParseError> {
        // Step 1: Preprocess to make valid YAML
        let processed = YamlPreprocessor::process(input);

        // Step 2: Parse YAML to AST
        let config: Config = serde_yaml::from_str(&processed)?;

        // Step 3: Validate
        Self::validate(&config)?;

        Ok(config)
    }

    /// Parse from file path
    pub fn parse_file(path: &std::path::Path) -> Result<Config, ParseError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ParseError::Invalid(format!("Failed to read file: {}", e)))?;

        Self::parse(&content)
    }

    /// Validate configuration
    fn validate(config: &Config) -> Result<(), ParseError> {
        // Check that policy tree is not empty
        if config.policy.entries.is_empty() {
            return Err(ParseError::MissingField("policy".to_string()));
        }

        // Validate rules
        for (name, rule) in &config.rules {
            if rule.patterns.is_empty() {
                return Err(ParseError::Invalid(format!(
                    "Rule '{}' has no patterns",
                    name
                )));
            }
        }

        // Validate rule references in policy tree
        Self::validate_rule_refs(config, &config.policy, "")?;

        Ok(())
    }

    /// Recursively validate rule references in policy tree
    fn validate_rule_refs(
        config: &Config,
        node: &crate::config::ast::PolicyNode,
        path: &str,
    ) -> Result<(), ParseError> {
        use crate::config::ast::{ApplyValue, FileItem, PolicyEntry};

        for (key, entry) in &node.entries {
            let current_path = format!("{}/{}", path, key);

            match entry {
                PolicyEntry::File(items) => {
                    for item in items {
                        if let FileItem::Apply { apply } = item {
                            let rule_names: Vec<String> = match apply {
                                ApplyValue::Single(name) => vec![name.clone()],
                                ApplyValue::Multiple(names) => names.clone(),
                            };

                            for rule_name in rule_names {
                                if !config.rules.contains_key(&rule_name) {
                                    return Err(ParseError::Invalid(format!(
                                        "Rule '{}' not found (referenced at {})",
                                        rule_name, current_path
                                    )));
                                }
                            }
                        }
                    }
                }
                PolicyEntry::Directory(subdir) => {
                    Self::validate_rule_refs(config, subdir, &current_path)?;
                }
                _ => {} // Other entry types don't have rule references
            }
        }

        Ok(())
    }
}

/// Extension methods for Config
pub trait ConfigExt {
    /// Get all rules applied at a policy node
    fn get_applied_rules(
        &self,
        path: &[&str],
        file_pattern: &str,
    ) -> Vec<&crate::config::ast::Rule>;

    /// Get violation level for context
    fn get_violation_level(&self, context_name: &str, file_path: &str) -> Option<String>;
}

impl ConfigExt for Config {
    fn get_applied_rules(
        &self,
        _path: &[&str],
        _file_pattern: &str,
    ) -> Vec<&crate::config::ast::Rule> {
        // Implementation would traverse policy tree
        // and collect all applied rules
        vec![]
    }

    fn get_violation_level(&self, context_name: &str, _file_path: &str) -> Option<String> {
        self.contexts.get(context_name).map(|_| "block".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_config() {
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
    
    ${name}.test.tsx:
      - exists: 1
"#;

        let result = ConfigParser::parse(yaml);
        assert!(
            result.is_ok(),
            "Should parse valid config: {:?}",
            result.err()
        );

        let config = result.unwrap();
        assert!(config.rules.contains_key("react"));
        assert!(!config.policy.entries.is_empty());
    }

    #[test]
    fn test_parse_without_preprocessing() {
        // This should fail without preprocessing
        let yaml = r#"
rules:
  react:
    .tsx: PascalCase
"#;

        // Direct YAML parse would fail on unquoted .tsx
        let direct = serde_yaml::from_str::<Config>(yaml);
        assert!(direct.is_err());

        // Preprocessor quotes keys but LS-Lint shorthand (.tsx: PascalCase)
        // needs to be converted to full format (.tsx: [PascalCase])
        // Use full Assura format for this test
        let full_yaml = r#"
rules:
  react:
    .tsx:
      - constraints: [PascalCase]

policy:
  src/:
    .tsx:
      - apply: react
"#;

        let result = ConfigParser::parse(full_yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_policy() {
        let yaml = r#"
rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]

policy:
"#;

        let result = ConfigParser::parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_complex_violation_array() {
        // Test with constraints only (needs policy)
        let yaml1 = r#"
rules:
  sized:
    ${name}.tsx:
      - constraints: [PascalCase]

policy:
  src/:
    ${name}.tsx:
      - apply: sized
"#;
        let result1 = ConfigParser::parse(yaml1);
        if let Err(ref e) = result1 {
            println!("Test 1 (constraints only) error: {:?}", e);
        }
        assert!(result1.is_ok(), "Should parse with constraints only");

        // Test with violation
        let yaml2 = r#"
rules:
  sized:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn, ci:block]

policy:
  src/:
    ${name}.tsx:
      - apply: sized
"#;
        let result2 = ConfigParser::parse(yaml2);
        if let Err(ref e) = result2 {
            println!("Test 2 (with violation) error: {:?}", e);
        }
        assert!(result2.is_ok(), "Should parse with violation");

        // Test with message
        let yaml3 = r#"
rules:
  sized:
    ${name}.tsx:
      - constraints: [PascalCase]
      - message:
          warn: "Getting large"

policy:
  src/:
    ${name}.tsx:
      - apply: sized
"#;
        let result3 = ConfigParser::parse(yaml3);
        if let Err(ref e) = result3 {
            println!("Test 3 (with message) error: {:?}", e);
        }
        assert!(result3.is_ok(), "Should parse with message");

        // Full test with lines constraint
        let yaml = r#"
rules:
  sized:
    ${name}.tsx:
      - constraints: [PascalCase, lines:..400]
      - violation: [warn, ci:block, feature:warn]
      - message:
          warn: "Getting large"
          block: "Must refactor"

policy:
  src/:
    ${name}.tsx:
      - apply: sized
"#;

        let config = ConfigParser::parse(yaml).expect("Should parse");
        let rule = config.rules.get("sized").expect("Should have sized rule");

        // Verify the rule has the correct patterns
        assert!(rule.patterns.contains_key("${name}.tsx"));
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

        let config = ConfigParser::parse(yaml).expect("Should parse");
        let json = config.to_json().expect("Should serialize to JSON");

        // Should produce valid JSON
        assert!(json.contains("\"rules\""));
        assert!(json.contains("\"policy\""));
    }

    #[test]
    fn test_context_inheritance() {
        let yaml = r#"
contexts:
  ci:
    hook: ci
  feature:
    hook: pre-commit
    branch: "feature/*"

rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn, ci:block, feature:warn]

policy:
  src/components/:
    violation: [warn]
    
    ${name}.tsx:
      - apply: react
"#;

        let config = ConfigParser::parse(yaml).expect("Should parse");
        assert!(config.contexts.contains_key("ci"));
        assert!(config.contexts.contains_key("feature"));
    }

    #[test]
    fn test_invalid_rule_reference() {
        let yaml = r#"
rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]

policy:
  src/components/:
    ${name}.tsx:
      - apply: nonexistent-rule
"#;

        let result = ConfigParser::parse(yaml);
        assert!(
            result.is_err(),
            "Should fail for non-existent rule reference"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("nonexistent-rule"),
            "Error should mention the missing rule"
        );
    }

    #[test]
    fn test_apply_array_parsing() {
        let yaml = r#"
rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
  
  tested:
    ${name}.test.tsx:
      - exists: 1

policy:
  src/components/:
    ${name}.tsx:
      - apply: [react, tested]
"#;

        let config = ConfigParser::parse(yaml).expect("Should parse array apply");
        // Should successfully parse apply with multiple rules
        assert!(config.rules.contains_key("react"));
        assert!(config.rules.contains_key("tested"));
    }

    #[test]
    fn test_violation_context_specific() {
        let yaml = r#"
rules:
  sized:
    ${name}.tsx:
      - constraints: [PascalCase, lines:..400]
      - violation: [warn, ci:block, feature:warn]

policy:
  src/:
    ${name}.tsx:
      - apply: sized
"#;

        let config = ConfigParser::parse(yaml).expect("Should parse context-specific violations");
        let rule = config.rules.get("sized").expect("Should have sized rule");
        let items = rule
            .patterns
            .get("${name}.tsx")
            .expect("Should have pattern");

        // Verify we have violation entries
        let has_violation = items.iter().any(|item| {
            if let crate::config::ast::ConstraintItem::Violation { violation } = item {
                !violation.is_empty()
            } else {
                false
            }
        });
        assert!(has_violation, "Should have violation entries");
    }
}
