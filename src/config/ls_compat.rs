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
                if apply_file_rule(pattern, rule, &mut naming_patterns, &mut exists).is_err() {
                    continue;
                }
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
                if apply_file_rule(pattern, rule, &mut naming_patterns, &mut exists).is_err() {
                    continue;
                }
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
    convert_ls_lint_documents_to_config(&[ls_lint_content])
}

/// Convert one or more LS-Lint YAML configs using LS-Lint's `--config` merge shape.
#[cfg(feature = "yaml-config")]
pub fn convert_ls_lint_documents_to_config(contents: &[&str]) -> Result<Config, String> {
    let mut config = Config::new();
    let mut ls_section = serde_yaml::Mapping::new();

    for content in contents {
        let ls_config: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| format!("Failed to parse LS-Lint config: {}", e))?;
        config.exclude.extend(parse_ignore(&ls_config));

        if let Some(mapping) = ls_config.get("ls").and_then(|value| value.as_mapping()) {
            for (key, value) in mapping {
                ls_section.insert(key.clone(), value.clone());
            }
        }
    }

    config.exclude.sort();
    config.exclude.dedup();
    ensure_assura_config_excluded(&mut config.exclude);

    if ls_section.is_empty() {
        return Ok(config);
    }

    let root = parse_ls_directory(&ls_section)?;
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
    let mut self_directory_bundle = DirectoryBundle::new();
    let mut naming_patterns = HashMap::new();
    let mut file_exists = HashMap::new();
    let mut directory_exists = HashMap::new();
    let mut self_directory_exists = HashMap::new();
    let mut children = HashMap::new();

    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            continue;
        };

        if key == ".dir" {
            let rule = value.as_str().unwrap_or("");
            apply_directory_rule(rule, &mut self_directory_bundle, &mut self_directory_exists)?;
            continue;
        }

        if key.starts_with('.') {
            let rule = value.as_str().unwrap_or("");
            apply_file_rule(key, rule, &mut naming_patterns, &mut file_exists)?;
            continue;
        }

        if let Some(child_mapping) = value.as_mapping() {
            children.insert(
                normalize_child_key(key),
                parse_ls_directory(child_mapping)?
                    .with_required(false)
                    .with_inherit(false),
            );
        } else if let Some(rule) = value.as_str() {
            if apply_direct_child_exists_rule(key, rule, &mut file_exists, &mut directory_exists)? {
                continue;
            }
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

    if !self_directory_exists.is_empty() {
        self_directory_bundle.exists = Some(self_directory_exists);
    }
    if self_directory_bundle.naming.is_some()
        || self_directory_bundle.exists.is_some()
        || self_directory_bundle.allow_extra.is_some()
    {
        node.self_directory = Some(self_directory_bundle);
    }

    if !children.is_empty() {
        node.children = Some(children);
    }

    Ok(node)
}

#[cfg(feature = "yaml-config")]
fn apply_direct_child_exists_rule(
    key: &str,
    rule: &str,
    file_exists: &mut HashMap<String, String>,
    directory_exists: &mut HashMap<String, String>,
) -> Result<bool, String> {
    let tokens = split_rule_tokens(rule);
    if tokens.is_empty() {
        return Ok(false);
    }

    let mut exists = Vec::new();
    for token in tokens {
        let Some(count) = parse_exists_token(token)? else {
            return Ok(false);
        };
        exists.push(count);
    }

    let count = exists.join(" | ");
    if key.ends_with('/') {
        directory_exists.insert(normalize_child_key(key), count);
    } else {
        file_exists.insert(key.to_string(), count);
    }
    Ok(true)
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
) -> Result<(), String> {
    let mut naming = Vec::new();
    for token in split_rule_tokens(rule) {
        if let Some(count) = parse_exists_token(token)? {
            exists.insert(ls_file_pattern_to_glob(pattern), count);
        } else {
            naming.push(normalize_lslint_naming_token(token)?);
        }
    }

    if !naming.is_empty() {
        naming_patterns.insert(ls_file_pattern_to_glob(pattern), naming.join(" | "));
    }
    Ok(())
}

#[cfg(feature = "yaml-config")]
fn apply_directory_rule(
    rule: &str,
    directories: &mut DirectoryBundle,
    exists: &mut HashMap<String, String>,
) -> Result<(), String> {
    let mut naming = Vec::new();
    for token in split_rule_tokens(rule) {
        if let Some(count) = parse_exists_token(token)? {
            exists.insert("*".to_string(), count);
        } else {
            naming.push(normalize_lslint_naming_token(token)?);
        }
    }

    if !naming.is_empty() {
        directories.naming = Some(naming.join(" | "));
    }
    Ok(())
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

fn parse_exists_token(token: &str) -> Result<Option<String>, String> {
    if token == "exists" {
        Ok(Some("exists".to_string()))
    } else if let Some(raw) = token.strip_prefix("exists:") {
        Ok(Some(parse_exists_count(raw)?))
    } else {
        Ok(None)
    }
}

fn parse_exists_count(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Invalid LS-Lint exists rule: exists value is empty".to_string());
    }

    if let Some((min, max)) = raw.split_once('-') {
        parse_exists_bound(min, raw)?;
        parse_exists_bound(max, raw)?;
        return Ok(raw.to_string());
    }

    parse_exists_bound(raw, raw)?;
    Ok(raw.to_string())
}

fn parse_exists_bound(bound: &str, raw: &str) -> Result<(), String> {
    let bound = bound.trim();
    if bound.is_empty() {
        return Err(format!(
            "Invalid LS-Lint exists rule 'exists:{raw}': range bounds must be non-empty"
        ));
    }
    bound
        .parse::<u16>()
        .map(|_| ())
        .map_err(|error| format!("Invalid LS-Lint exists rule 'exists:{raw}': {error}"))
}

fn normalize_lslint_naming_token(token: &str) -> Result<String, String> {
    let Some(pattern) = token.strip_prefix("regex:") else {
        return Ok(token.to_string());
    };

    if pattern.is_empty() {
        return Err("Unsupported LS-Lint regex rule: pattern is empty".to_string());
    }

    if let Some(pattern) = pattern.strip_prefix('!') {
        return Ok(format!("regex:!^{pattern}$"));
    }

    Ok(format!("regex:^{pattern}$"))
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
