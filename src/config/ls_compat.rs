//! LS-Lint compatibility layer for unified config
//!
//! Provides support for LS-Lint style rules in unified config format.
//! NOTE: This is for testing purposes only. Internal backwards compatibility
//! will not be maintained until the 1.0 release.

#[cfg(feature = "yaml-config")]
use super::config::DirectoryBundle;
use super::config::{Config, DirectoryNode, FileBundle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod validation;

#[cfg(feature = "yaml-config")]
use validation::validate_lslint_document_shape;
#[cfg(feature = "yaml-config")]
use validation::{migration_report_for_mapping, validate_converted_config};
use validation::{normalize_lslint_naming_token, parse_exists_token, split_rule_tokens};

/// Metadata emitted by the authoritative LS-Lint converter.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LsLintMigrationReport {
    /// Extension or subextension rule entries in the merged `ls` tree.
    pub extension_rules: usize,
    /// Directory-scope mappings in the merged `ls` tree.
    pub path_rules: usize,
    /// `exists` tokens in the merged `ls` tree.
    pub exists_rules: usize,
    /// User-provided `ignore` patterns, excluding Assura's own generated config
    /// exclusion.
    pub ignored_patterns: usize,
    /// Non-fatal migration notes.
    pub warnings: Vec<String>,
}

/// Converted Assura config plus the report generated from the same conversion
/// pass.
#[derive(Debug, Clone)]
pub struct LsLintMigration {
    /// Converted structure-first Assura config.
    pub config: Config,
    /// Report counts derived from the authoritative converter.
    pub report: LsLintMigrationReport,
}

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

/// Convert an LS-Lint YAML config and return migration metadata.
#[cfg(feature = "yaml-config")]
pub fn convert_ls_lint_to_migration(ls_lint_content: &str) -> Result<LsLintMigration, String> {
    convert_ls_lint_documents_to_migration(&[ls_lint_content])
}

/// Convert one or more LS-Lint YAML configs using LS-Lint's `--config` merge shape.
#[cfg(feature = "yaml-config")]
pub fn convert_ls_lint_documents_to_config(contents: &[&str]) -> Result<Config, String> {
    Ok(convert_ls_lint_documents_to_migration(contents)?.config)
}

/// Convert one or more LS-Lint YAML configs and return migration metadata.
#[cfg(feature = "yaml-config")]
pub fn convert_ls_lint_documents_to_migration(
    contents: &[&str],
) -> Result<LsLintMigration, String> {
    let mut config = Config::new();
    let mut ls_section = serde_yaml::Mapping::new();
    let mut ignored_patterns = 0;

    for content in contents {
        let ls_config: serde_yaml::Value = serde_yaml::from_str(content)
            .map_err(|e| format!("Failed to parse LS-Lint config: {}", e))?;
        validate_lslint_document_shape(&ls_config)?;
        let ignore = parse_ignore(&ls_config)?;
        ignored_patterns += ignore.len();
        config.exclude.extend(ignore);

        if let Some(mapping) = ls_config.get("ls").and_then(|value| value.as_mapping()) {
            merge_lslint_mapping(&mut ls_section, mapping);
        }
    }

    config.exclude.sort();
    config.exclude.dedup();
    ensure_assura_config_excluded(&mut config.exclude);

    let report = migration_report_for_mapping(&ls_section, ignored_patterns)?;

    if ls_section.is_empty() {
        validate_converted_config(&config)?;
        return Ok(LsLintMigration { config, report });
    }

    let root = parse_ls_directory(&ls_section)?;
    config.structure.insert("./".to_string(), root);
    validate_converted_config(&config)?;
    Ok(LsLintMigration { config, report })
}

#[cfg(feature = "yaml-config")]
fn merge_lslint_mapping(target: &mut serde_yaml::Mapping, source: &serde_yaml::Mapping) {
    for (key, source_value) in source {
        target.insert(key.clone(), source_value.clone());
    }
}

#[cfg(feature = "yaml-config")]
fn parse_ignore(config: &serde_yaml::Value) -> Result<Vec<String>, String> {
    Ok(config
        .get("ignore")
        .and_then(|value| value.as_sequence())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default())
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
            return Err("Unsupported LS-Lint YAML shape: 'ls' keys must be strings".to_string());
        };

        if key == ".dir" {
            let rule = value.as_str().ok_or_else(|| {
                "Unsupported LS-Lint YAML shape: '.dir' rule must be a string".to_string()
            })?;
            apply_directory_rule(rule, &mut self_directory_bundle, &mut self_directory_exists)?;
            continue;
        }

        if let Some(child_mapping) = value.as_mapping() {
            children.insert(
                normalize_child_key(key),
                parse_ls_directory(child_mapping)?
                    .with_required(false)
                    .with_inherit(false),
            );
            continue;
        }

        if key.starts_with('.') {
            let rule = value.as_str().ok_or_else(|| {
                format!("Unsupported LS-Lint YAML shape: rule for '{key}' must be a string")
            })?;
            apply_file_rule(key, rule, &mut naming_patterns, &mut file_exists)?;
            continue;
        }

        if let Some(rule) = value.as_str() {
            apply_scalar_rule(key, rule, &mut file_exists, &mut directory_exists)?;
        } else {
            return Err(format!(
                "Unsupported LS-Lint YAML shape: value for '{key}' must be a rule string or mapping"
            ));
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

fn apply_scalar_rule(
    key: &str,
    rule: &str,
    file_exists: &mut HashMap<String, String>,
    directory_exists: &mut HashMap<String, String>,
) -> Result<(), String> {
    let mut exists = Vec::new();
    for token in split_rule_tokens(rule)? {
        if let Some(count) = parse_exists_token(token)? {
            exists.push(count);
        } else {
            normalize_lslint_naming_token(token)?;
        }
    }

    if !exists.is_empty() {
        let count = exists.join(" | ");
        if key.ends_with('/') {
            directory_exists.insert(normalize_child_key(key), count);
        } else {
            file_exists.insert(key.to_string(), count);
        }
    }
    Ok(())
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
    for token in split_rule_tokens(rule)? {
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
    for token in split_rule_tokens(rule)? {
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

fn ls_file_pattern_to_glob(pattern: &str) -> String {
    if pattern == ".*" {
        return "*.*".to_string();
    }

    pattern.to_string()
}
