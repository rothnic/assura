//! Rule inheritance resolver for hierarchical configuration
//!
//! This module resolves hierarchical rules into flat (path_pattern, bundle) pairs
//! by walking the structure tree and applying inheritance.

use super::config::{Config, DirectoryNode, FileBundle, ResolvedFileBundle};
use std::collections::HashMap;
use std::path::Path;

/// Resolves hierarchical rules into flat (path_pattern, bundle) pairs
#[derive(Debug)]
pub struct RuleResolver<'a> {
    config: &'a Config,
}

/// A resolved rule with specificity information
#[derive(Debug, Clone)]
pub struct ResolvedRule {
    /// The path pattern this rule applies to
    pub path_pattern: String,
    /// The file validation bundle
    pub bundle: FileBundle,
    /// Specificity score (higher = more specific, wins in conflicts)
    pub specificity: usize,
}

impl<'a> RuleResolver<'a> {
    /// Create a new resolver for the given config
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    /// Resolve all rules in the config
    pub fn resolve(&self) -> Vec<ResolvedRule> {
        let mut rules = Vec::new();

        // First, add top-level pattern rules (lowest specificity)
        for (pattern, bundle) in &self.config.patterns {
            rules.push(ResolvedRule {
                path_pattern: pattern.clone(),
                bundle: bundle.clone(),
                specificity: Self::calculate_pattern_specificity(pattern),
            });
        }

        // Then resolve structure hierarchy rules
        for (path, node) in &self.config.structure {
            self.resolve_node(path, node, None, 0, &mut rules);
        }

        // Sort by specificity (most specific first)
        rules.sort_by_key(|rule| std::cmp::Reverse(rule.specificity));

        rules
    }

    /// Resolve rules for a specific path
    pub fn resolve_for_path(&self, path: &Path) -> Option<ResolvedFileBundle> {
        let rules = self.resolve();

        // Find the most specific matching rule
        for rule in rules {
            if Self::path_matches_pattern(path, &rule.path_pattern) {
                return Some(ResolvedFileBundle {
                    path_pattern: rule.path_pattern,
                    naming: rule.bundle.naming.clone(),
                    naming_patterns: rule.bundle.naming_patterns.clone(),
                    max_lines: rule.bundle.max_lines,
                    max_lines_patterns: rule.bundle.max_lines_patterns.clone(),
                    max_size: rule.bundle.max_size.clone(),
                    max_size_patterns: rule.bundle.max_size_patterns.clone(),
                    require_docs: rule.bundle.require_docs,
                    extensions: rule.bundle.extensions.clone(),
                    severity: rule.bundle.severity.clone(),
                    required: rule.bundle.required.clone(),
                    allowed_names: rule.bundle.allowed_names.clone(),
                    allowed_patterns: rule.bundle.allowed_patterns.clone(),
                    forbidden_patterns: rule.bundle.forbidden_patterns.clone(),
                    allow_extra: rule.bundle.allow_extra,
                    exists: rule.bundle.exists.clone(),
                });
            }
        }

        None
    }

    /// Recursively resolve a node and its children
    fn resolve_node(
        &self,
        path: &str,
        node: &DirectoryNode,
        parent_bundle: Option<&FileBundle>,
        depth: usize,
        rules: &mut Vec<ResolvedRule>,
    ) {
        // Merge with parent bundle if inheritance is enabled
        let merged_bundle = if node.inherit {
            parent_bundle.map(|parent| Self::merge_bundles(parent, node.files.as_ref()))
        } else {
            node.files.clone()
        };

        // Add rule for this node if it has file validations
        if let Some(ref bundle) = node.files {
            let merged = merged_bundle.as_ref().unwrap_or(bundle);
            rules.push(ResolvedRule {
                path_pattern: path.to_string(),
                bundle: merged.clone(),
                specificity: Self::calculate_specificity(path, depth),
            });
        }

        // Recurse into children
        if let Some(ref children) = node.children {
            for (child_name, child_node) in children {
                let child_path = if path.ends_with('/') {
                    format!("{}{}", path, child_name)
                } else {
                    format!("{}/{}", path, child_name)
                };

                self.resolve_node(
                    &child_path,
                    child_node,
                    merged_bundle.as_ref().or(node.files.as_ref()),
                    depth + 1,
                    rules,
                );
            }
        }
    }

    /// Merge parent and child bundles (child values override parent)
    fn merge_bundles(parent: &FileBundle, child: Option<&FileBundle>) -> FileBundle {
        let child = match child {
            Some(c) => c,
            None => {
                return FileBundle {
                    required: None,
                    allowed_names: None,
                    allowed_patterns: None,
                    forbidden_patterns: None,
                    allow_extra: None,
                    exists: None,
                    ..parent.clone()
                }
            }
        };

        FileBundle {
            naming: child.naming.clone().or_else(|| parent.naming.clone()),
            naming_patterns: merge_pattern_maps(
                parent.naming_patterns.as_ref(),
                child.naming_patterns.as_ref(),
            ),
            max_lines: child.max_lines.or(parent.max_lines),
            max_lines_patterns: merge_pattern_maps(
                parent.max_lines_patterns.as_ref(),
                child.max_lines_patterns.as_ref(),
            ),
            max_size: child.max_size.clone().or_else(|| parent.max_size.clone()),
            max_size_patterns: merge_pattern_maps(
                parent.max_size_patterns.as_ref(),
                child.max_size_patterns.as_ref(),
            ),
            require_docs: child.require_docs.or(parent.require_docs),
            extensions: child
                .extensions
                .clone()
                .or_else(|| parent.extensions.clone()),
            severity: child.severity.clone().or_else(|| parent.severity.clone()),
            required: child.required.clone(),
            allowed_names: child.allowed_names.clone(),
            allowed_patterns: child.allowed_patterns.clone(),
            forbidden_patterns: child.forbidden_patterns.clone(),
            allow_extra: child.allow_extra,
            exists: child.exists.clone(),
        }
    }

    /// Calculate specificity score for a path
    ///
    /// More specific paths (deeper nesting, exact matches) get higher scores
    fn calculate_specificity(path: &str, depth: usize) -> usize {
        let mut score = depth * 10;

        // Exact paths are more specific than globs
        if !path.contains('*') && !path.contains('{') {
            score += 5;
        }

        // Longer paths are more specific
        score += path.len();

        score
    }

    /// Calculate specificity score for a pattern
    fn calculate_pattern_specificity(pattern: &str) -> usize {
        let mut score = 0;

        // More specific patterns (fewer wildcards) get higher scores
        let wildcards = pattern.matches('*').count();
        score += 10_usize.saturating_sub(wildcards * 2);

        // Longer patterns are more specific
        score += pattern.len();

        score
    }

    /// Check if a path matches a pattern
    fn path_matches_pattern(path: &Path, pattern: &str) -> bool {
        let path_str = path.to_string_lossy();

        // Handle glob patterns
        if pattern.contains('*') || pattern.contains('{') || pattern.contains('[') {
            match glob::Pattern::new(pattern) {
                Ok(p) => return p.matches(&path_str),
                Err(_) => return false,
            }
        }

        // Handle directory prefix patterns (e.g., "src/" matches "src/main.rs")
        if let Some(prefix) = pattern.strip_suffix('/') {
            return path_str.starts_with(prefix)
                && (path_str.len() == prefix.len() || path_str[prefix.len()..].starts_with('/'));
        }

        // Exact match
        path_str == pattern
    }
}

fn merge_pattern_maps<T: Clone>(
    parent: Option<&HashMap<String, T>>,
    child: Option<&HashMap<String, T>>,
) -> Option<HashMap<String, T>> {
    match (parent, child) {
        (None, None) => None,
        (Some(parent), None) => Some(parent.clone()),
        (None, Some(child)) => Some(child.clone()),
        (Some(parent), Some(child)) => {
            let mut merged = parent.clone();
            merged.extend(child.clone());
            Some(merged)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::config::{Config, DirectoryNode, FileBundle};
    use super::*;

    fn create_test_config() -> Config {
        Config::new()
            .with_node(
                "src/",
                DirectoryNode::new()
                    .with_files(
                        FileBundle::new()
                            .with_naming("snake_case")
                            .with_max_lines(500),
                    )
                    .with_child(
                        "components/",
                        DirectoryNode::new()
                            .with_files(
                                FileBundle::new()
                                    .with_naming("PascalCase")
                                    .with_max_lines(300),
                            )
                            .with_inherit(true),
                    )
                    .with_child(
                        "utils/",
                        DirectoryNode::new()
                            .with_files(FileBundle::new().with_naming("kebab-case"))
                            .with_inherit(false), // Don't inherit from src/
                    ),
            )
            .with_node(
                "tests/",
                DirectoryNode::new().with_files(FileBundle::new().with_naming("snake_case")),
            )
    }

    #[test]
    fn test_resolve_all_rules() {
        let config = create_test_config();
        let resolver = RuleResolver::new(&config);
        let rules = resolver.resolve();

        // Should have rules for: src/, src/components/, src/utils/, tests/
        assert_eq!(rules.len(), 4);

        // Check that more specific paths have higher specificity
        let src_rule = rules.iter().find(|r| r.path_pattern == "src/").unwrap();
        let components_rule = rules
            .iter()
            .find(|r| r.path_pattern == "src/components/")
            .unwrap();

        assert!(components_rule.specificity > src_rule.specificity);
    }

    #[test]
    fn test_inheritance_enabled() {
        let config = create_test_config();
        let resolver = RuleResolver::new(&config);
        let rules = resolver.resolve();

        // src/components/ inherits from src/ and overrides naming
        let components_rule = rules
            .iter()
            .find(|r| r.path_pattern == "src/components/")
            .unwrap();

        assert_eq!(
            components_rule.bundle.naming,
            Some("PascalCase".to_string())
        );
        // max_lines should be inherited from parent (300 from child, not 500 from parent)
        assert_eq!(components_rule.bundle.max_lines, Some(300));
    }

    #[test]
    fn test_inheritance_disabled() {
        let config = create_test_config();
        let resolver = RuleResolver::new(&config);
        let rules = resolver.resolve();

        // src/utils/ has inherit: false, so it should not inherit from src/
        let utils_rule = rules
            .iter()
            .find(|r| r.path_pattern == "src/utils/")
            .unwrap();

        assert_eq!(utils_rule.bundle.naming, Some("kebab-case".to_string()));
        // max_lines should not be inherited
        assert_eq!(utils_rule.bundle.max_lines, None);
    }

    #[test]
    fn test_resolve_for_path() {
        let config = create_test_config();
        let resolver = RuleResolver::new(&config);

        // src/main.rs should match src/ rule
        let bundle = resolver.resolve_for_path(Path::new("src/main.rs"));
        assert!(bundle.is_some());
        assert_eq!(bundle.unwrap().naming, Some("snake_case".to_string()));

        // src/components/Button.rs should match src/components/ rule
        let bundle = resolver.resolve_for_path(Path::new("src/components/Button.rs"));
        assert!(bundle.is_some());
        assert_eq!(bundle.unwrap().naming, Some("PascalCase".to_string()));

        // tests/test.rs should match tests/ rule
        let bundle = resolver.resolve_for_path(Path::new("tests/test.rs"));
        assert!(bundle.is_some());
        assert_eq!(bundle.unwrap().naming, Some("snake_case".to_string()));

        // nonexistent/path should not match anything
        let bundle = resolver.resolve_for_path(Path::new("nonexistent/file.rs"));
        assert!(bundle.is_none());
    }

    #[test]
    fn test_path_matches_pattern() {
        // Exact match
        assert!(RuleResolver::path_matches_pattern(
            Path::new("src/main.rs"),
            "src/main.rs"
        ));

        // Directory prefix
        assert!(RuleResolver::path_matches_pattern(
            Path::new("src/main.rs"),
            "src/"
        ));

        // Glob pattern
        assert!(RuleResolver::path_matches_pattern(
            Path::new("src/main.rs"),
            "src/*.rs"
        ));

        // No match
        assert!(!RuleResolver::path_matches_pattern(
            Path::new("tests/test.rs"),
            "src/"
        ));
    }

    #[test]
    fn test_specificity_ordering() {
        let config = create_test_config();
        let resolver = RuleResolver::new(&config);
        let rules = resolver.resolve();

        // Rules should be sorted by specificity (most specific first)
        for i in 1..rules.len() {
            assert!(
                rules[i - 1].specificity >= rules[i].specificity,
                "Rules should be sorted by specificity (most specific first)"
            );
        }
    }

    #[test]
    fn test_merge_bundles() {
        let parent = FileBundle::new()
            .with_naming("snake_case")
            .with_max_lines(500)
            .with_max_size("1MB");

        let child = FileBundle::new().with_naming("PascalCase");

        let merged = RuleResolver::merge_bundles(&parent, Some(&child));

        // Child values should override parent
        assert_eq!(merged.naming, Some("PascalCase".to_string()));
        // Parent values should be inherited when child doesn't specify
        assert_eq!(merged.max_lines, Some(500));
        assert_eq!(merged.max_size, Some("1MB".to_string()));
    }

    #[test]
    fn test_pattern_rules() {
        let config = Config::new()
            .with_pattern(
                "**/*.rs",
                FileBundle::new()
                    .with_naming("snake_case")
                    .with_max_lines(500),
            )
            .with_pattern("src/**/*.rs", FileBundle::new().with_max_lines(300));

        let resolver = RuleResolver::new(&config);
        let rules = resolver.resolve();

        // Should have 2 pattern rules
        assert_eq!(rules.len(), 2);

        // src/**/*.rs is more specific than **/*.rs, so it should have higher specificity
        let global_rule = rules.iter().find(|r| r.path_pattern == "**/*.rs").unwrap();
        let src_rule = rules
            .iter()
            .find(|r| r.path_pattern == "src/**/*.rs")
            .unwrap();

        assert!(src_rule.specificity > global_rule.specificity);
    }

    #[test]
    fn test_directory_local_file_exceptions_do_not_inherit() {
        let parent = FileBundle::new()
            .with_naming("snake_case")
            .with_allowed_names(vec!["README.md".to_string(), "LICENSE".to_string()]);

        let child = FileBundle::new().with_naming("PascalCase");

        let merged = RuleResolver::merge_bundles(&parent, Some(&child));

        assert!(merged.allowed_names.is_none());

        let inherited_without_child_bundle = RuleResolver::merge_bundles(&parent, None);
        assert!(inherited_without_child_bundle.allowed_names.is_none());
    }
}
