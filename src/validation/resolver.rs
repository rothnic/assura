//! Rule Resolution Engine
//!
//! Resolves rules from the policy tree and merges constraints.
//! Follows Constitution: structure-first resolution.

use crate::config::ast::{ApplyValue, Config, Constraint, ConstraintItem, FileItem, Rule};
use std::collections::HashMap;
use std::path::Path;

/// Resolved constraints for a specific file
#[derive(Debug, Clone, Default)]
pub struct ResolvedConstraints {
    pub constraints: Vec<Constraint>,
    pub violation_entries: Vec<crate::config::ast::ViolationEntry>,
    pub messages: HashMap<String, String>, // context -> message
}

/// Resolves rules from the policy tree
pub struct RuleResolver;

impl RuleResolver {
    /// Resolve all constraints for a file at a given path
    pub fn resolve(config: &Config, file_path: &Path) -> ResolvedConstraints {
        let mut result = ResolvedConstraints::default();

        // Traverse policy tree and collect applicable constraints
        Self::traverse_and_resolve(
            config,
            &config.policy,
            file_path,
            Path::new(""),
            &mut result,
        );

        result
    }

    /// Recursively traverse policy tree
    fn traverse_and_resolve(
        config: &Config,
        node: &crate::config::ast::PolicyNode,
        file_path: &Path,
        current_path: &Path,
        result: &mut ResolvedConstraints,
    ) {
        use crate::config::ast::PolicyEntry;

        for (raw_key, entry) in &node.entries {
            // Strip quotes from key (preprocessor adds them for special characters)
            let key = raw_key.trim_matches('"');
            let entry_path = current_path.join(key);

            match entry {
                PolicyEntry::Directory(subdir) if file_path.starts_with(&entry_path) => {
                    // Check if this directory is in the file path
                    Self::traverse_and_resolve(config, subdir, file_path, &entry_path, result);
                }
                PolicyEntry::File(items) if Self::pattern_matches(key, file_path) => {
                    // Check if this file pattern matches
                    Self::resolve_file_items(config, items, file_path, result);
                }
                _ => {} // Other entry types handled at directory level
            }
        }
    }

    /// Check if a pattern matches a file path
    fn pattern_matches(pattern: &str, file_path: &Path) -> bool {
        let file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        // Handle ${name} pattern
        if pattern.contains("${") {
            // Extract the pattern structure
            // e.g., "${name}.tsx" should match "Button.tsx"
            let parts: Vec<&str> = pattern.split("${name}").collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];

                if file_name.starts_with(prefix) && file_name.ends_with(suffix) {
                    let middle = &file_name[prefix.len()..file_name.len() - suffix.len()];
                    // Middle part should be valid identifier (alphanumeric or underscore)
                    return !middle.is_empty()
                        && middle.chars().all(|c| c.is_alphanumeric() || c == '_');
                }
            }
            return false;
        }

        // Handle glob patterns
        if pattern.contains('*') {
            return Self::glob_matches(pattern, file_name);
        }

        // Exact match
        file_name == pattern
    }

    /// Simple glob matching
    fn glob_matches(pattern: &str, text: &str) -> bool {
        // Very simple glob: * matches any characters
        if pattern == "*" {
            return true;
        }

        if pattern.starts_with("*") && pattern.ends_with("*") {
            let middle = &pattern[1..pattern.len() - 1];
            return text.contains(middle);
        }

        if let Some(suffix) = pattern.strip_prefix("*") {
            return text.ends_with(suffix);
        }

        if let Some(prefix) = pattern.strip_suffix("*") {
            return text.starts_with(prefix);
        }

        pattern == text
    }

    /// Resolve items from a file entry
    fn resolve_file_items(
        config: &Config,
        items: &Vec<FileItem>,
        file_path: &Path,
        result: &mut ResolvedConstraints,
    ) {
        for item in items {
            match item {
                FileItem::Apply { apply } => {
                    let rule_names = match apply {
                        ApplyValue::Single(name) => vec![name.clone()],
                        ApplyValue::Multiple(names) => names.clone(),
                    };

                    for rule_name in rule_names {
                        if let Some(rule) = config.rules.get(&rule_name) {
                            Self::merge_rule_constraints(rule, file_path, result);
                        }
                    }
                }
                FileItem::Constraints { constraints } => {
                    result.constraints.extend(constraints.clone());
                }
                FileItem::Violation { violation } => {
                    result.violation_entries.extend(violation.clone());
                }
                FileItem::Message(message) => {
                    // Merge messages
                    for (context, msg) in &message.contexts {
                        result.messages.insert(context.clone(), msg.clone());
                    }
                }
                _ => {} // Other items handled elsewhere
            }
        }
    }

    /// Merge constraints from a rule
    fn merge_rule_constraints(rule: &Rule, file_path: &Path, result: &mut ResolvedConstraints) {
        let _file_name = file_path.file_name().and_then(|s| s.to_str()).unwrap_or("");

        for (raw_pattern, items) in &rule.patterns {
            // Strip quotes from pattern (preprocessor adds them for special characters)
            let pattern = raw_pattern.trim_matches('"');
            if Self::pattern_matches(pattern, file_path) {
                for item in items {
                    match item {
                        ConstraintItem::Constraint(constraint) => {
                            result.constraints.push(constraint.clone());
                        }
                        ConstraintItem::Constraints { constraints } => {
                            result.constraints.extend(constraints.clone());
                        }
                        ConstraintItem::Violation { violation } => {
                            result.violation_entries.extend(violation.clone());
                        }
                        ConstraintItem::Message { message } => {
                            for (context, msg) in &message.contexts {
                                result.messages.insert(context.clone(), msg.clone());
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ast::{PolicyEntry, PolicyNode};
    use std::collections::HashMap;

    #[test]
    fn test_pattern_matching_variable() {
        assert!(RuleResolver::pattern_matches(
            "${name}.tsx",
            Path::new("/src/Button.tsx")
        ));
        assert!(!RuleResolver::pattern_matches(
            "${name}.tsx",
            Path::new("/src/Button.test.tsx")
        ));
    }

    #[test]
    fn test_pattern_matching_glob() {
        assert!(RuleResolver::pattern_matches(
            "*.tsx",
            Path::new("/src/Button.tsx")
        ));
        assert!(RuleResolver::pattern_matches(
            "Button*",
            Path::new("/src/Button.tsx")
        ));
        assert!(!RuleResolver::pattern_matches(
            "*.rs",
            Path::new("/src/Button.tsx")
        ));
    }

    #[test]
    fn test_resolve_simple_rule() {
        use crate::config::ast::{Constraint, NamingConvention};

        let mut rules = HashMap::new();
        let mut patterns = HashMap::new();
        patterns.insert(
            "${name}.tsx".to_string(),
            vec![ConstraintItem::Constraint(Constraint::Naming(
                NamingConvention::PascalCase,
            ))],
        );
        rules.insert("react".to_string(), Rule { patterns });

        let mut policy_entries = HashMap::new();
        policy_entries.insert(
            "${name}.tsx".to_string(),
            PolicyEntry::File(vec![FileItem::Apply {
                apply: ApplyValue::Single("react".to_string()),
            }]),
        );

        let config = Config {
            rules,
            contexts: HashMap::new(),
            messages: HashMap::new(),
            policy: PolicyNode {
                entries: policy_entries,
            },
        };

        let result = RuleResolver::resolve(&config, Path::new("/src/Button.tsx"));

        assert_eq!(result.constraints.len(), 1);
    }
}
