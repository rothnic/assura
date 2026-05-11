//! LS-Lint Parser Bridge
//!
//! Parses .ls-lint.yml files and converts to Assura configuration.
//! Ensures feature parity with LS-Lint.

use crate::config::ast::{
    ApplyValue, Config, Constraint, ConstraintItem, FileItem, NamingConvention, PolicyEntry,
    PolicyNode, Rule, ViolationEntry,
};
use std::collections::HashMap;

/// LS-Lint configuration structure
#[derive(Debug, Clone, Default)]
pub struct LsLintConfig {
    /// Extension to convention mappings
    pub extensions: HashMap<String, Vec<String>>,
    /// Path-specific rules
    pub paths: HashMap<String, HashMap<String, Vec<String>>>,
    /// Ignore patterns
    pub ignore: Vec<String>,
    /// Exists directives (pattern -> count range)
    pub exists: HashMap<String, String>,
}

/// Parser for LS-Lint configuration
pub struct LsLintParser;

impl LsLintParser {
    /// Parse LS-Lint YAML configuration
    pub fn parse(yaml: &str) -> Result<LsLintConfig, LsLintParseError> {
        let mut config = LsLintConfig::default();
        let mut in_ignore_section = false;

        // Simple YAML parsing for LS-Lint format
        // LS-Lint uses a flat structure with ls: key
        for line in yaml.lines() {
            let trimmed = line.trim();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Check for ignore directive - LS-Lint allows this anywhere in the file
            // Format: ignore:\n  - pattern1\n  - pattern2
            if trimmed.starts_with("ignore:") || (trimmed.starts_with("- ") && in_ignore_section) {
                if trimmed.starts_with("ignore:") {
                    in_ignore_section = true;
                }
                // Parse ignore pattern from list item
                if let Some(stripped) = trimmed.strip_prefix("- ") {
                    let pattern = stripped.trim();
                    // Remove surrounding quotes if present
                    let pattern = pattern.trim_matches('"').trim_matches('\'');
                    config.ignore.push(pattern.to_string());
                }
                continue;
            } else if in_ignore_section && !trimmed.starts_with("#") {
                // End of ignore section when we hit a non-comment, non-list line
                in_ignore_section = false;
            }

            // Parse exists directive (e.g., .rs: exists:0, README.md: exists:1)
            if trimmed.contains("exists:") {
                if let Some(colon_pos) = trimmed.find(':') {
                    let pattern = &trimmed[..colon_pos].trim();
                    let value = &trimmed[colon_pos + 1..].trim();

                    // Extract exists value (e.g., "exists:0" -> "0")
                    if let Some(exists_val) = value.strip_prefix("exists:") {
                        config
                            .exists
                            .insert(pattern.to_string(), exists_val.trim().to_string());
                    }
                }
                continue;
            }

            // Parse extension rules (e.g., .rs: snake_case)
            if trimmed.starts_with('.') {
                if let Some(colon_pos) = trimmed.find(':') {
                    let ext = &trimmed[..colon_pos].trim();
                    let value = &trimmed[colon_pos + 1..].trim();

                    // Handle multi-part extensions (.test.tsx, .d.ts)
                    let conventions = Self::parse_conventions(value);
                    config.extensions.insert(ext.to_string(), conventions);
                }
            }

            // Parse path rules (e.g., src/components/*: PascalCase)
            if trimmed.contains('/') && !trimmed.starts_with("ignore:") {
                if let Some(colon_pos) = trimmed.find(':') {
                    let path = &trimmed[..colon_pos].trim();
                    let value = &trimmed[colon_pos + 1..].trim();

                    let conventions = Self::parse_conventions(value);
                    let path_map = config.paths.entry(path.to_string()).or_default();

                    // In LS-Lint, path rules often don't specify extension
                    // We'll use "*" as the pattern for all files
                    path_map.insert("*".to_string(), conventions);
                }
            }
        }

        Ok(config)
    }

    /// Parse convention string (handles OR with |)
    fn parse_conventions(value: &str) -> Vec<String> {
        value
            .split('|')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Convert LS-Lint convention to Assura NamingConvention
    fn convert_convention(ls_convention: &str) -> Option<NamingConvention> {
        match ls_convention.to_lowercase().as_str() {
            "snake_case" | "snakecase" => Some(NamingConvention::SnakeCase),
            "camelcase" | "camel_case" => Some(NamingConvention::CamelCase),
            "pascalcase" | "pascal_case" => Some(NamingConvention::PascalCase),
            "kebab-case" | "kebabcase" => Some(NamingConvention::KebabCase),
            "screaming_snake_case" | "screamingsnakecase" => {
                Some(NamingConvention::ScreamingSnakeCase)
            }
            "lowercase" => Some(NamingConvention::Lowercase),
            "uppercase" => Some(NamingConvention::Uppercase),
            _ => None,
        }
    }

    /// Convert LS-Lint config to Assura config
    pub fn convert_to_assura(ls_config: &LsLintConfig) -> Config {
        let mut rules = HashMap::new();
        let mut policy_entries = HashMap::new();

        // Convert extension-based rules
        for (ext, conventions) in &ls_config.extensions {
            let rule_name = format!("{}_files", ext.trim_start_matches('.'));

            // Build pattern (e.g., "*.rs")
            let pattern = format!("*{}", ext);

            // Build constraints
            let constraint_items = Self::build_constraints(conventions);

            let rule = Rule {
                patterns: {
                    let mut map = HashMap::new();
                    map.insert(pattern, constraint_items);
                    map
                },
            };

            rules.insert(rule_name.clone(), rule);

            // Add to policy root
            let file_entry = PolicyEntry::File(vec![FileItem::Apply {
                apply: ApplyValue::Single(rule_name),
            }]);

            policy_entries.insert(format!("*{}", ext), file_entry);
        }

        // Convert path-specific rules
        for (path, file_patterns) in &ls_config.paths {
            let mut subdir_entries = HashMap::new();

            for conventions in file_patterns.values() {
                // Build constraints for this path
                let constraint_items = Self::build_constraints(conventions);

                // Use a generic pattern since LS-Lint paths apply to all files
                let file_entry = PolicyEntry::File(vec![FileItem::Constraints {
                    constraints: constraint_items
                        .iter()
                        .filter_map(|item| {
                            if let ConstraintItem::Constraint(c) = item {
                                Some(c.clone())
                            } else {
                                None
                            }
                        })
                        .collect(),
                }]);

                subdir_entries.insert("*".to_string(), file_entry);
            }

            // Add subdirectory to policy
            let subdir = PolicyNode {
                entries: subdir_entries,
            };
            policy_entries.insert(path.to_string(), PolicyEntry::Directory(subdir));
        }

        // Convert exists directives
        for (pattern, exists_val) in &ls_config.exists {
            // Parse exists value (e.g., "0", "1", "1..10")
            let _exists_constraint = if exists_val == "0" {
                // exists:0 means no files allowed
                // Use strict mode or specific handling
                ConstraintItem::Constraint(Constraint::Exists {
                    exists: crate::config::ast::Range::Exact(0),
                })
            } else if exists_val == "1" {
                // exists:1 means exactly one file
                ConstraintItem::Constraint(Constraint::Exists {
                    exists: crate::config::ast::Range::Exact(1),
                })
            } else {
                // Range like "1..10"
                ConstraintItem::Constraint(Constraint::Exists {
                    exists: crate::config::ast::Range::RangeString(exists_val.clone()),
                })
            };

            // Create file entry with exists constraint
            let file_entry = PolicyEntry::File(vec![
                FileItem::Constraints {
                    constraints: vec![Constraint::Exists {
                        exists: if exists_val == "0" {
                            crate::config::ast::Range::Exact(0)
                        } else if exists_val == "1" {
                            crate::config::ast::Range::Exact(1)
                        } else {
                            crate::config::ast::Range::RangeString(exists_val.clone())
                        },
                    }],
                },
                FileItem::Violation {
                    violation: vec![ViolationEntry::Level("block".to_string())],
                },
            ]);

            policy_entries.insert(pattern.clone(), file_entry);
        }

        Config {
            rules,
            contexts: HashMap::new(),
            messages: HashMap::new(),
            policy: PolicyNode {
                entries: policy_entries,
            },
        }
    }

    /// Build constraint items from LS-Lint conventions
    fn build_constraints(conventions: &[String]) -> Vec<ConstraintItem> {
        let mut items = Vec::new();

        // Convert conventions to OR array if multiple
        if conventions.len() == 1 {
            if let Some(conv) = Self::convert_convention(&conventions[0]) {
                items.push(ConstraintItem::Constraint(Constraint::Naming(conv)));
            }
        } else {
            // Multiple conventions = OR logic
            // In Assura, we represent this as constraints: [PascalCase, camelCase]
            // which means either one is acceptable
            let naming_constraints: Vec<Constraint> = conventions
                .iter()
                .filter_map(|c| Self::convert_convention(c))
                .map(Constraint::Naming)
                .collect();

            if !naming_constraints.is_empty() {
                items.push(ConstraintItem::Constraint(Constraint::ConstraintsArray(
                    naming_constraints,
                )));
            }
        }

        // Add default violation level (warn)
        items.push(ConstraintItem::Violation {
            violation: vec![ViolationEntry::Level("warn".to_string())],
        });

        items
    }
}

/// Errors during LS-Lint parsing
#[derive(Debug, thiserror::Error)]
pub enum LsLintParseError {
    #[error("Invalid LS-Lint syntax: {0}")]
    InvalidSyntax(String),

    #[error("Unknown convention: {0}")]
    UnknownConvention(String),
}

/// Migration tool from LS-Lint to Assura
pub struct MigrationTool;

impl MigrationTool {
    /// Migrate an LS-Lint config file to Assura format
    pub fn migrate(ls_lint_yaml: &str) -> Result<String, LsLintParseError> {
        let ls_config = LsLintParser::parse(ls_lint_yaml)?;
        let assura_config = LsLintParser::convert_to_assura(&ls_config);

        // Serialize to YAML
        let yaml = assura_config
            .to_yaml()
            .map_err(|e| LsLintParseError::InvalidSyntax(e.to_string()))?;

        Ok(yaml)
    }

    /// Generate migration report
    pub fn generate_report(ls_lint_yaml: &str) -> Result<MigrationReport, LsLintParseError> {
        let ls_config = LsLintParser::parse(ls_lint_yaml)?;

        Ok(MigrationReport {
            extension_rules: ls_config.extensions.len(),
            path_rules: ls_config.paths.len(),
            exists_rules: ls_config.exists.len(),
            ignored_patterns: ls_config.ignore.len(),
            warnings: vec![],
        })
    }
}

/// Migration report
#[derive(Debug, Clone)]
pub struct MigrationReport {
    pub extension_rules: usize,
    pub path_rules: usize,
    pub exists_rules: usize,
    pub ignored_patterns: usize,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod parser_tests;
