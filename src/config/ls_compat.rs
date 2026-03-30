//! LS-Lint compatibility layer for unified config
//!
//! Provides support for LS-Lint style rules in unified config format.
//! NOTE: This is for testing purposes only. Internal backwards compatibility
//! will not be maintained until the 1.0 release.

use super::config::{DirectoryNode, Config, FileBundle};
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
    pub fn with_extension_rule(mut self, ext: impl Into<String>, convention: impl Into<String>) -> Self {
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
            .or_insert_with(HashMap::new)
            .insert(ext.into(), convention.into());
        self
    }

    /// Convert LS-Lint style rules to structure format
    pub fn to_structure_nodes(&self) -> HashMap<String, DirectoryNode> {
        let mut nodes = HashMap::new();

        // Convert extension-based rules
        if !self.rules.is_empty() {
            let mut bundle = FileBundle::new();
            // Use the first rule as the default naming convention
            // In practice, LS-Lint only has one rule per extension
            if let Some((_, convention)) = self.rules.iter().next() {
                bundle.naming = Some(convention.clone());
            }

            nodes.insert("".to_string(), DirectoryNode::new().with_files(bundle));
        }

        // Convert path-specific rules
        for (path, rules) in &self.paths {
            let mut bundle = FileBundle::new();

            // Find the most common convention or use the first one
            if let Some((_, convention)) = rules.iter().next() {
                bundle.naming = Some(convention.clone());
            }

            nodes.insert(path.clone(), DirectoryNode::new().with_files(bundle));
        }

        nodes
    }

    /// Convert a V1-style extension to a file pattern
    fn extension_to_pattern(ext: &str) -> String {
        if ext.starts_with('.') {
            format!("*{}", ext)
        } else {
            format!("*. {}", ext)
        }
    }

    /// Parse LS-Lint convention name to assura convention
    fn parse_convention(ls_convention: &str) -> String {
        match ls_convention {
            "kebab-case" => "kebab-case".to_string(),
            "snake_case" => "snake_case".to_string(),
            "PascalCase" => "PascalCase".to_string(),
            "camelCase" => "camelCase".to_string(),
            "SCREAMING_SNAKE_CASE" => "SCREAMING_SNAKE_CASE".to_string(),
            "dot.case" => "dot.case".to_string(),
            _ => ls_convention.to_string(),
        }
    }
}

impl Default for LsLintCompatibility {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert an LS-Lint YAML config to unified structure config
pub fn convert_ls_lint_to_config(ls_lint_content: &str) -> Result<Config, String> {
    // Parse the LS-Lint YAML content
    let ls_config: serde_yaml::Value = serde_yaml::from_str(ls_lint_content)
        .map_err(|e| format!("Failed to parse LS-Lint config: {}", e))?;

    let mut config = Config::new();

    // Extract the "ls" section
    if let Some(ls_section) = ls_config.get("ls") {
        if let Some(rules) = ls_section.as_mapping() {
            let mut compat = LsLintCompatibility::new();

            for (key, value) in rules {
                let key_str = key.as_str().unwrap_or("");
                let value_str = value.as_str().unwrap_or("");

                if key_str.starts_with('.') {
                    // Extension rule
                    compat = compat.with_extension_rule(key_str, value_str);
                } else if key_str.ends_with('/') {
                    // Path rule - this is a simplified conversion
                    // Real LS-Lint configs can have more complex path patterns
                    compat = compat.with_path_rule(key_str, ".*", value_str);
                }
            }

            config.ls = Some(compat);

            // Convert to structure nodes
            let nodes = config.ls.as_ref().unwrap().to_structure_nodes();
            config.structure.extend(nodes);
        }
    }

    Ok(config)
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
        assert_eq!(compat.paths.get("src/").unwrap().get(".rs"), Some(&"snake_case".to_string()));
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
        assert_eq!(root_node.files.as_ref().unwrap().naming, Some("snake_case".to_string()));
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
        assert!(config.ls.is_some());
        assert!(!config.structure.is_empty());
    }
}
