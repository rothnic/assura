//! LS-Lint compatibility layer for unified config
//!
//! Provides support for LS-Lint style rules in unified config format.
//! NOTE: This is for testing purposes only. Internal backwards compatibility
//! will not be maintained until the 1.0 release.

use super::config::{split_naming_conventions, DirectoryNode, FileBundle};
#[cfg(feature = "yaml-config")]
use super::config::{Config, DirectoryBundle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// LS-Lint compatibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LsLintCompatibility {
    /// Extension-based rules (e.g., ".rs" -> "snake_case")
    #[serde(default)]
    pub rules: HashMap<String, String>,

    /// Path-specific rules (e.g., "src/" -> {".rs" -> "snake_case"})
    #[serde(default)]
    pub paths: HashMap<String, HashMap<String, String>>,
}

impl LsLintCompatibility {
    /// Create a new empty compatibility layer
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            paths: HashMap::new(),
        }
    }

    /// Add an extension rule
    pub fn with_extension_rule(
        mut self,
        ext: impl Into<String>,
        convention: impl Into<String>,
    ) -> Self {
        self.rules.insert(ext.into(), convention.into());
        self
    }

    /// Add a path-specific rule
    pub fn with_path_rule(
        mut self,
        path: impl Into<String>,
        ext: impl Into<String>,
        convention: impl Into<String>,
    ) -> Self {
        self.paths
            .entry(path.into())
            .or_default()
            .insert(ext.into(), convention.into());
        self
    }

    /// Convert LS-Lint style rules to structure format
    pub fn to_structure_nodes(&self) -> HashMap<String, DirectoryNode> {
        let mut nodes = HashMap::new();

        // Convert extension-based rules
        if !self.rules.is_empty() {
            let mut bundle = FileBundle::new();
            let mut naming_patterns = HashMap::new();
            let mut exists = HashMap::new();
            for (pattern, rule) in &self.rules {
                apply_file_rule(pattern, rule, &mut naming_patterns, &mut exists);
            }
            if !naming_patterns.is_empty() {
                bundle.naming_patterns = Some(naming_patterns);
            }
            if !exists.is_empty() {
                bundle.exists = Some(exists);
            }

            nodes.insert("".to_string(), DirectoryNode::new().with_files(bundle));
        }

        // Convert path-specific rules
        for (path, rules) in &self.paths {
            let mut bundle = FileBundle::new();

            let mut naming_patterns = HashMap::new();
            let mut exists = HashMap::new();
            for (pattern, rule) in rules {
                apply_file_rule(pattern, rule, &mut naming_patterns, &mut exists);
            }
            if !naming_patterns.is_empty() {
                bundle.naming_patterns = Some(naming_patterns);
            }
            if !exists.is_empty() {
                bundle.exists = Some(exists);
            }

            nodes.insert(path.clone(), DirectoryNode::new().with_files(bundle));
        }

        nodes
    }
}

impl Default for LsLintCompatibility {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert an LS-Lint YAML config to unified structure config
#[cfg(feature = "yaml-config")]
pub fn convert_ls_lint_to_config(ls_lint_content: &str) -> Result<Config, String> {
    let ls_config: serde_yaml::Value = serde_yaml::from_str(ls_lint_content)
        .map_err(|e| format!("Failed to parse LS-Lint config: {}", e))?;

    let mut config = Config::new();
    config.exclude = parse_ignore(&ls_config);
    ensure_assura_config_excluded(&mut config.exclude);

    let Some(ls_section) = ls_config.get("ls").and_then(|value| value.as_mapping()) else {
        return Ok(config);
    };

    let root = parse_ls_directory(ls_section)?;
    config.structure.insert("./".to_string(), root);
    Ok(config)
}

#[cfg(feature = "yaml-config")]
fn parse_ignore(config: &serde_yaml::Value) -> Vec<String> {
    config
        .get("ignore")
        .and_then(|value| value.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(feature = "yaml-config")]
fn ensure_assura_config_excluded(exclude: &mut Vec<String>) {
    if !exclude.iter().any(|pattern| pattern == ".assura/**") {
        exclude.push(".assura/**".to_string());
    }
}

#[cfg(feature = "yaml-config")]
fn parse_ls_directory(mapping: &serde_yaml::Mapping) -> Result<DirectoryNode, String> {
    let mut node = DirectoryNode::new();
    let mut file_bundle = FileBundle::new();
    let mut directory_bundle = DirectoryBundle::new();
    let mut naming_patterns = HashMap::new();
    let mut file_exists = HashMap::new();
    let mut directory_exists = HashMap::new();
    let mut children = HashMap::new();

    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            continue;
        };

        if key == ".dir" {
            let rule = value.as_str().unwrap_or("");
            apply_directory_rule(rule, &mut directory_bundle, &mut directory_exists);
            continue;
        }

        if key.starts_with('.') {
            let rule = value.as_str().unwrap_or("");
            apply_file_rule(key, rule, &mut naming_patterns, &mut file_exists);
            continue;
        }

        if let Some(child_mapping) = value.as_mapping() {
            reject_unsupported_directory_scope(key)?;
            children.insert(
                normalize_child_key(key),
                parse_ls_directory(child_mapping)?.with_required(false),
            );
        } else if let Some(rule) = value.as_str() {
            if apply_direct_child_exists_rule(key, rule, &mut file_exists, &mut directory_exists) {
                continue;
            }

            reject_unsupported_directory_scope(key)?;

            let mut child = DirectoryNode::new();
            let mut child_files = FileBundle::new();
            let mut child_naming_patterns = HashMap::new();
            let mut child_exists = HashMap::new();
            apply_file_rule(".*", rule, &mut child_naming_patterns, &mut child_exists);
            if !child_naming_patterns.is_empty() {
                child_files.naming_patterns = Some(child_naming_patterns);
            }
            if !child_exists.is_empty() {
                child_files.exists = Some(child_exists);
            }
            child.files = Some(child_files);
            children.insert(normalize_child_key(key), child.with_required(false));
        }
    }

    if !naming_patterns.is_empty() {
        file_bundle.naming_patterns = Some(naming_patterns);
    }
    if !file_exists.is_empty() {
        file_bundle.exists = Some(file_exists);
    }
    if file_bundle.naming_patterns.is_some() || file_bundle.exists.is_some() {
        node.files = Some(file_bundle);
    }

    if !directory_exists.is_empty() {
        directory_bundle.exists = Some(directory_exists);
    }
    if directory_bundle.naming.is_some()
        || directory_bundle.exists.is_some()
        || directory_bundle.allow_extra.is_some()
    {
        node.directories = Some(directory_bundle);
    }

    if !children.is_empty() {
        node.children = Some(children);
    }

    Ok(node)
}

#[cfg(feature = "yaml-config")]
fn reject_unsupported_directory_scope(key: &str) -> Result<(), String> {
    if key.contains('*') || key.contains('{') || key.contains('}') {
        return Err(format!(
            "Unsupported LS-Lint directory scope '{key}'. Assura migrate currently supports explicit directory scopes only; glob and brace scopes such as packages/*, **, and {{src,tests}} are not converted yet."
        ));
    }

    Ok(())
}

#[cfg(feature = "yaml-config")]
fn apply_direct_child_exists_rule(
    key: &str,
    rule: &str,
    file_exists: &mut HashMap<String, String>,
    directory_exists: &mut HashMap<String, String>,
) -> bool {
    let tokens = split_rule_tokens(rule);
    if tokens.is_empty() {
        return false;
    }

    let mut exists = Vec::new();
    for token in tokens {
        let Some(count) = parse_exists_token(token) else {
            return false;
        };
        exists.push(count);
    }

    let count = exists.join(" | ");
    if key.ends_with('/') {
        directory_exists.insert(normalize_child_key(key), count);
    } else {
        file_exists.insert(key.to_string(), count);
    }
    true
}

#[cfg(feature = "yaml-config")]
fn normalize_child_key(key: &str) -> String {
    key.trim_end_matches('/').to_string()
}

fn apply_file_rule(
    pattern: &str,
    rule: &str,
    naming_patterns: &mut HashMap<String, String>,
    exists: &mut HashMap<String, String>,
) {
    let mut naming = Vec::new();
    for token in split_rule_tokens(rule) {
        if let Some(count) = parse_exists_token(token) {
            exists.insert(ls_file_pattern_to_glob(pattern), count);
        } else {
            naming.push(token.to_string());
        }
    }

    if !naming.is_empty() {
        naming_patterns.insert(ls_file_pattern_to_glob(pattern), naming.join(" | "));
    }
}

#[cfg(feature = "yaml-config")]
fn apply_directory_rule(
    rule: &str,
    directories: &mut DirectoryBundle,
    exists: &mut HashMap<String, String>,
) {
    let mut naming = Vec::new();
    for token in split_rule_tokens(rule) {
        if let Some(count) = parse_exists_token(token) {
            if count == "0" {
                directories.allow_extra = Some(false);
            }
            exists.insert("*".to_string(), count);
        } else if token == "exists" {
            exists.insert("*".to_string(), "exists".to_string());
        } else {
            naming.push(token.to_string());
        }
    }

    if !naming.is_empty() {
        directories.naming = Some(naming.join(" | "));
    }
}

fn split_rule_tokens(rule: &str) -> Vec<&str> {
    if rule.contains("exists") && rule.contains(" | ") {
        return rule
            .split(" | ")
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .collect();
    }

    split_naming_conventions(rule)
}

fn parse_exists_token(token: &str) -> Option<String> {
    if token == "exists" {
        Some("exists".to_string())
    } else {
        token
            .strip_prefix("exists:")
            .map(str::trim)
            .map(ToOwned::to_owned)
    }
}

fn ls_file_pattern_to_glob(pattern: &str) -> String {
    if pattern == ".*" {
        return "*.*".to_string();
    }

    if pattern.starts_with('.') {
        return format!("*{}", pattern);
    }

    pattern.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ls_compat_builder() {
        let compat = LsLintCompatibility::new()
            .with_extension_rule(".rs", "snake_case")
            .with_extension_rule(".ts", "camelCase")
            .with_path_rule("src/", ".rs", "snake_case");

        assert_eq!(compat.rules.get(".rs"), Some(&"snake_case".to_string()));
        assert_eq!(
            compat.paths.get("src/").unwrap().get(".rs"),
            Some(&"snake_case".to_string())
        );
    }

    #[test]
    fn test_to_structure_nodes() {
        let compat = LsLintCompatibility::new()
            .with_extension_rule(".rs", "snake_case")
            .with_path_rule("src/", ".rs", "snake_case");

        let nodes = compat.to_structure_nodes();

        assert!(nodes.contains_key(""));
        assert!(nodes.contains_key("src/"));

        let root_node = nodes.get("").unwrap();
        let naming_patterns = root_node
            .files
            .as_ref()
            .unwrap()
            .naming_patterns
            .as_ref()
            .unwrap();
        assert_eq!(naming_patterns.get("*.rs"), Some(&"snake_case".to_string()));
    }

    #[test]
    fn test_convert_ls_lint_to_config() {
        let ls_lint_yaml = r#"
ls:
  .rs: snake_case
  .ts: camelCase
  src/:
    .rs: snake_case
"#;

        let config = convert_ls_lint_to_config(ls_lint_yaml).unwrap();
        assert!(!config.structure.is_empty());
        let root = config.structure.get("./").unwrap();
        let src = root.children.as_ref().unwrap().get("src").unwrap();
        let patterns = src
            .files
            .as_ref()
            .unwrap()
            .naming_patterns
            .as_ref()
            .unwrap();
        assert_eq!(patterns.get("*.rs"), Some(&"snake_case".to_string()));
    }

    #[test]
    fn test_convert_ls_lint_dir_and_exists_rules() {
        let ls_lint_yaml = r#"
ls:
  components:
    .dir: kebab-case
    .*: exists:0
    .ts: kebab-case | exists:1
"#;

        let config = convert_ls_lint_to_config(ls_lint_yaml).unwrap();
        let root = config.structure.get("./").unwrap();
        let components = root.children.as_ref().unwrap().get("components").unwrap();
        let dirs = components.directories.as_ref().unwrap();
        assert_eq!(dirs.naming.as_deref(), Some("kebab-case"));
        let files = components.files.as_ref().unwrap();
        assert_eq!(
            files.exists.as_ref().unwrap().get("*.*"),
            Some(&"0".to_string())
        );
        assert_eq!(
            files.exists.as_ref().unwrap().get("*.ts"),
            Some(&"1".to_string())
        );
    }

    #[test]
    fn test_rejects_unsupported_directory_glob_scopes() {
        for scope in ["packages/*", "**", "{src,tests}"] {
            let ls_lint_yaml = format!(
                r#"
ls:
  "{scope}":
    .ts: kebab-case
"#
            );

            let error = convert_ls_lint_to_config(&ls_lint_yaml).unwrap_err();
            assert!(
                error.contains("Unsupported LS-Lint directory scope"),
                "unexpected error for {scope}: {error}"
            );
        }
    }
}
