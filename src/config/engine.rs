//! Policy resolution engine for the unified config format
//!
//! This module provides:
//! - Policy resolution for file paths
//! - Specificity calculation and rule precedence
//! - Rule merging and inheritance

use crate::config::types::{
    ApplyEntry, Case, Config, Directive, InlineRule, NamingConvention, PolicyEntry, PolicyNode,
    Rule, Severity,
};
use std::collections::HashMap;
use std::path::Path;

/// Resolved rules for a specific file path
#[derive(Debug, Clone, Default)]
pub struct ResolvedRules {
    /// File extensions that apply
    pub extensions: Option<Vec<String>>,
    /// Naming convention(s)
    pub naming: Option<NamingConvention>,
    /// Maximum lines
    pub max_lines: Option<usize>,
    /// Maximum size
    pub max_size: Option<String>,
    /// Whether documentation is required
    pub require_docs: Option<bool>,
    /// Test file pattern
    pub require_test: Option<String>,
    /// Custom message
    pub message: Option<crate::config::types::Message>,
    /// Severity level
    pub severity: Option<Severity>,
}

/// Policy engine for resolving rules from the config
#[derive(Debug)]
pub struct PolicyEngine {
    config: Config,
}

/// Match result for policy entries
#[derive(Debug, Clone)]
struct PolicyMatch {
    /// The specificity score (higher = more specific)
    specificity: usize,
    /// The matched rule properties
    rule_properties: RuleProperties,
}

/// Intermediate structure for collecting rule properties
#[derive(Debug, Clone, Default)]
struct RuleProperties {
    extensions: Option<Vec<String>>,
    naming: Option<NamingConvention>,
    max_lines: Option<usize>,
    max_size: Option<String>,
    require_docs: Option<bool>,
    require_test: Option<String>,
    message: Option<crate::config::types::Message>,
    severity: Option<Severity>,
}

impl PolicyEngine {
    /// Create a new policy engine from a config
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Resolve rules for a specific file path
    ///
    /// This walks the policy tree and applies rules based on:
    /// 1. Exact path matches beat wildcards
    /// 2. Deeper paths beat shallower paths
    /// 3. More segments beat fewer
    /// 4. Extensions evaluated after paths
    pub fn resolve(&self, path: &Path) -> ResolvedRules {
        let mut matches: Vec<PolicyMatch> = Vec::new();

        // Collect all matching rules from the policy tree
        if let Some(ref policy) = self.config.policy {
            self.collect_matches(path, policy, &mut matches, &self.config.rules, 0);
        }

        // Sort by specificity (most specific first)
        matches.sort_by(|a, b| b.specificity.cmp(&a.specificity));

        // Merge rules from most specific to least specific
        // (child rules override parent rules)
        let mut result = ResolvedRules::default();
        for m in matches.iter().rev() {
            result.merge(&m.rule_properties);
        }

        result
    }

    /// Collect all matching policy entries for a path
    fn collect_matches(
        &self,
        path: &Path,
        node: &PolicyNode,
        matches: &mut Vec<PolicyMatch>,
        rules: &HashMap<String, Rule>,
        depth: usize,
    ) {
        let path_str = path.to_string_lossy();

        for (key, entry) in &node.entries {
            let specificity = self.calculate_specificity(key, depth, path);

            if specificity > 0 {
                let props = self.entry_to_properties(entry, rules);
                matches.push(PolicyMatch {
                    specificity,
                    rule_properties: props,
                });

                // If this is a nested node, recurse into it
                if let PolicyEntry::Nested(ref nested) = entry {
                    self.collect_matches(path, nested, matches, rules, depth + 1);
                }
            }
        }
    }

    /// Convert a policy entry to rule properties
    fn entry_to_properties(
        &self,
        entry: &PolicyEntry,
        rules: &HashMap<String, Rule>,
    ) -> RuleProperties {
        match entry {
            PolicyEntry::RuleRef(rule_name) => {
                // Strip @ prefix if present
                let name = if rule_name.starts_with('@') {
                    &rule_name[1..]
                } else {
                    rule_name
                };

                if let Some(rule) = rules.get(name) {
                    rule_to_properties(rule)
                } else {
                    RuleProperties::default()
                }
            }
            PolicyEntry::Convention(conv) => RuleProperties {
                naming: Some(conv.clone()),
                ..Default::default()
            },
            PolicyEntry::Directive(directive) => self.directive_to_properties(directive, rules),
            PolicyEntry::InlineRule(inline) => inline_to_properties(inline),
            PolicyEntry::Nested(_) => RuleProperties::default(), // Nested nodes are handled separately
        }
    }

    /// Convert a directive to rule properties
    fn directive_to_properties(
        &self,
        directive: &Directive,
        rules: &HashMap<String, Rule>,
    ) -> RuleProperties {
        match directive {
            Directive::Apply(entries) => {
                // Merge all applied rules
                let mut merged = RuleProperties::default();
                for entry in entries {
                    let name = if entry.rule.starts_with('@') {
                        &entry.rule[1..]
                    } else {
                        &entry.rule
                    };

                    if let Some(rule) = rules.get(name) {
                        let props = rule_to_properties(rule);
                        merged.merge(&props);

                        // Apply overrides if present
                        if let Some(ref overrides) = entry.overrides {
                            let override_props = rule_to_properties(overrides);
                            merged.merge(&override_props);
                        }
                    }
                }
                merged
            }
            Directive::Require(_) | Directive::Exists(_) => {
                // Existence checks don't produce rule properties directly
                RuleProperties::default()
            }
            Directive::LimitChildren(_) => {
                // Children limit is validated separately, not as rule properties
                RuleProperties::default()
            }
            Directive::Message(msg) => RuleProperties {
                message: Some(msg.clone()),
                ..Default::default()
            },
            Directive::Severity(sev) => RuleProperties {
                severity: Some(*sev),
                ..Default::default()
            },
        }
    }

    /// Calculate specificity score for a policy key
    ///
    /// Returns 0 if the key doesn't match the path
    fn calculate_specificity(&self, key: &str, depth: usize, path: &Path) -> usize {
        let path_str = path.to_string_lossy();

        // Check if key is an extension pattern (e.g., ".rs" or "rs")
        if key.starts_with('.') || !key.contains('/') && !key.contains('*') {
            let ext = if key.starts_with('.') { &key[1..] } else { key };

            if let Some(file_ext) = path.extension().and_then(|e| e.to_str()) {
                if file_ext == ext {
                    // Extension match gets lower priority than path matches
                    return 1;
                }
            }
            return 0;
        }

        // Handle glob patterns
        if key.contains('*') || key.contains('{') || key.contains('[') {
            match glob::Pattern::new(key) {
                Ok(pattern) => {
                    if pattern.matches(&path_str) {
                        // Calculate specificity based on pattern complexity
                        let mut score = 10 + depth * 10;
                        // Fewer wildcards = more specific
                        let wildcards = key.matches('*').count();
                        score += 10_usize.saturating_sub(wildcards * 2);
                        // Longer patterns are more specific
                        score += key.len();
                        return score;
                    }
                }
                Err(_) => return 0,
            }
            return 0;
        }

        // Handle directory patterns (ending with /)
        if key.ends_with('/') {
            let prefix = &key[..key.len() - 1];
            if path_str.starts_with(prefix)
                && (path_str.len() == prefix.len() || path_str[prefix.len()..].starts_with('/'))
            {
                // Exact directory match
                let mut score = 20 + depth * 10;
                // Longer paths are more specific
                score += key.len();
                return score;
            }
            return 0;
        }

        // Handle exact file paths
        if path_str == key {
            return 30 + depth * 10 + key.len();
        }

        0
    }
}

/// Convert a Rule to RuleProperties
fn rule_to_properties(rule: &Rule) -> RuleProperties {
    RuleProperties {
        extensions: rule.extensions.clone(),
        naming: rule.naming.clone(),
        max_lines: rule.max_lines,
        max_size: rule.max_size.clone(),
        require_docs: rule.require_docs,
        require_test: rule.require_test.clone(),
        message: rule.message.clone(),
        severity: None, // Rules don't define severity directly
    }
}

/// Convert an InlineRule to RuleProperties
fn inline_to_properties(inline: &InlineRule) -> RuleProperties {
    RuleProperties {
        extensions: inline.extensions.clone(),
        naming: inline.naming.clone(),
        max_lines: inline.max_lines,
        max_size: inline.max_size.clone(),
        require_docs: inline.require_docs,
        require_test: inline.require_test.clone(),
        message: inline.message.clone(),
        severity: inline.severity,
    }
}

impl ResolvedRules {
    /// Merge another set of rules into this one
    /// (new values override existing ones)
    fn merge(&mut self, props: &RuleProperties) {
        if props.extensions.is_some() {
            self.extensions = props.extensions.clone();
        }
        if props.naming.is_some() {
            self.naming = props.naming.clone();
        }
        if props.max_lines.is_some() {
            self.max_lines = props.max_lines;
        }
        if props.max_size.is_some() {
            self.max_size = props.max_size.clone();
        }
        if props.require_docs.is_some() {
            self.require_docs = props.require_docs;
        }
        if props.require_test.is_some() {
            self.require_test = props.require_test.clone();
        }
        if props.message.is_some() {
            self.message = props.message.clone();
        }
        if props.severity.is_some() {
            self.severity = props.severity;
        }
    }
}

impl RuleProperties {
    /// Merge another RuleProperties into this one
    fn merge(&mut self, other: &RuleProperties) {
        if other.extensions.is_some() {
            self.extensions = other.extensions.clone();
        }
        if other.naming.is_some() {
            self.naming = other.naming.clone();
        }
        if other.max_lines.is_some() {
            self.max_lines = other.max_lines;
        }
        if other.max_size.is_some() {
            self.max_size = other.max_size.clone();
        }
        if other.require_docs.is_some() {
            self.require_docs = other.require_docs;
        }
        if other.require_test.is_some() {
            self.require_test = other.require_test.clone();
        }
        if other.message.is_some() {
            self.message = other.message.clone();
        }
        if other.severity.is_some() {
            self.severity = other.severity;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> Config {
        Config::new()
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
        let config = Config::new().with_policy(
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
        let config = Config::new()
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
        let config = Config::new().with_policy(
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
}
