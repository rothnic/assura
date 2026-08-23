//! Configuration loader for unified structure-first config
//!
//! Handles parsing and validation of configuration files.

use super::config::{AuthoredRuleUse, Config, ConfigQualityDiagnostic};
use crate::cli::config::{ConfigError, ConfigResult};
use crate::config::config::validate_config_semantics;
use std::path::Path;

mod notation;
use notation::{has_top_level_rules, parse_normalized, parse_normalized_value, parse_yaml_value};

/// Loader for structure configs
#[derive(Debug)]
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load a config from a file path
    pub fn load(path: &Path) -> ConfigResult<Config> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        Self::parse(&content)
    }

    /// Load and semantically validate a config from a file path.
    pub fn load_validated(path: &Path) -> ConfigResult<Config> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        Self::parse_validated(&content)
    }

    /// Parse config from YAML string
    pub fn parse(content: &str) -> ConfigResult<Config> {
        let parsed_value = if content.contains("rules") {
            let value = parse_yaml_value(content)?;
            if has_top_level_rules(&value) {
                return parse_normalized_value(value);
            }
            Some(value)
        } else {
            None
        };
        if let Ok(config) = Self::parse_canonical(content) {
            return Ok(config);
        }
        match parsed_value {
            Some(value) => parse_normalized_value(value),
            None => parse_normalized(content),
        }
    }

    /// Parse and semantically validate config from a YAML string.
    pub fn parse_validated(content: &str) -> ConfigResult<Config> {
        Self::parse(content)
    }

    /// Return non-blocking quality diagnostics for authored shorthand rules.
    pub fn diagnostics(content: &str) -> ConfigResult<Vec<ConfigQualityDiagnostic>> {
        let value = parse_yaml_value(content)?;
        super::config::structure_config_diagnostics(&value).map_err(ConfigError::Invalid)
    }

    /// Expand authored rule references into effective rebased selectors.
    pub fn provenance(content: &str) -> ConfigResult<Vec<AuthoredRuleUse>> {
        let value = parse_yaml_value(content)?;
        super::config::structure_rule_provenance(value).map_err(ConfigError::Invalid)
    }

    fn parse_canonical(content: &str) -> ConfigResult<Config> {
        let config: Config =
            serde_yaml::from_str(content).map_err(|error| ConfigError::Yaml(error.to_string()))?;
        validate_config_semantics(&config).map_err(ConfigError::Invalid)?;
        Ok(config)
    }

    /// Save config to file
    pub fn save(config: &Config, path: &Path) -> ConfigResult<()> {
        let content = serde_yaml::to_string(config)
            .map_err(|e| ConfigError::Invalid(format!("Failed to serialize config: {}", e)))?;
        std::fs::write(path, content).map_err(ConfigError::Io)?;
        Ok(())
    }
}

#[cfg(test)]
mod docs_lifecycle_tests;
#[cfg(test)]
mod manifest_semantics_tests;
#[cfg(test)]
mod module_topology_tests;
#[cfg(test)]
mod test_relationship_tests;

#[cfg(test)]
mod tests {
    use super::super::config::{DirectoryNode, FileBundle};
    use super::*;

    #[test]
    fn test_parse_valid_config() {
        let yaml = r#"
structure:
  src/:
    files:
      naming: snake_case
      max_lines: 500
  tests/:
    files:
      naming: snake_case
exclude:
  - "target/**"
"#;

        let config = ConfigLoader::parse(yaml).unwrap();
        assert!(config.structure.contains_key("src/"));
        assert!(config.structure.contains_key("tests/"));
        assert_eq!(config.exclude.len(), 1);
    }

    #[test]
    fn test_parse_invalid_naming() {
        let yaml = r#"
structure:
  src/:
    files:
      naming: invalid_case
"#;

        let result = ConfigLoader::parse(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_load() {
        let config = Config::new()
            .with_node(
                "src/",
                DirectoryNode::new().with_files(FileBundle::new().with_naming("snake_case")),
            )
            .with_exclude("target/**");

        let temp_dir = tempfile::tempdir().unwrap();
        let config_path = temp_dir.path().join("config.yml");

        ConfigLoader::save(&config, &config_path).unwrap();

        let loaded = ConfigLoader::load(&config_path).unwrap();
        assert!(loaded.structure.contains_key("src/"));
    }

    #[test]
    fn test_parse_with_patterns() {
        let yaml = r#"
patterns:
  "**/*.rs":
    naming: snake_case
    max_lines: 500
structure:
  src/:
    files:
      naming: snake_case
"#;

        let config = ConfigLoader::parse(yaml).unwrap();
        assert!(config.patterns.contains_key("**/*.rs"));
        assert!(config.structure.contains_key("src/"));
    }

    #[test]
    fn test_parse_with_exists() {
        let yaml = r#"
structure:
  ./:
    exists:
      files: ["README.md", "LICENSE"]
      directories: ["src", "tests"]
"#;

        let config = ConfigLoader::parse(yaml).unwrap();
        let root_node = config.structure.get("./").unwrap();
        assert!(root_node.exists.is_some());

        let exists = root_node.exists.as_ref().unwrap();
        assert_eq!(exists.files.as_ref().unwrap().len(), 2);
        assert_eq!(exists.directories.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_parse_with_custom_constraint() {
        let yaml = r#"
extensions:
  custom_constraints:
    - id: source_test_pair
      type: paired_file_exists
      source: "src/*.rs"
      target: "tests/{stem}_test.rs"
structure: {}
"#;

        let config = ConfigLoader::parse(yaml).unwrap();
        let extensions = config.extensions.unwrap();
        assert_eq!(extensions.custom_constraints.len(), 1);
        assert_eq!(extensions.custom_constraints[0].id, "source_test_pair");
    }

    #[test]
    fn test_parse_with_command_surface_docs_constraint() {
        let yaml = r#"
extensions:
  custom_constraints:
    - id: command_surface_docs
      type: command_surface_docs
      source: "docs/*.md"
      target: ".assura/command-surface.yml"
structure: {}
"#;

        let config = ConfigLoader::parse(yaml).unwrap();
        let extensions = config.extensions.unwrap();
        assert_eq!(extensions.custom_constraints.len(), 1);
        assert_eq!(
            extensions.custom_constraints[0].kind,
            "command_surface_docs"
        );
    }

    #[test]
    fn test_parse_with_release_contract() {
        let yaml = r#"
extensions:
  release_contracts:
    - id: cli_release
      severity: high
      artifacts:
        - name: example-linux-x86_64.tar.gz
          checksum_sidecar: true
      workflow_files:
        - .github/workflows/release.yml
      docs_files:
        - docs/install.md
      installer_files:
        - scripts/install.sh
      allowed_url_branches:
        - main
structure: {}
"#;

        let config = ConfigLoader::parse(yaml).unwrap();
        let extensions = config.extensions.unwrap();
        assert_eq!(extensions.release_contracts.len(), 1);
        let contract = &extensions.release_contracts[0];
        assert_eq!(contract.id, "cli_release");
        assert_eq!(contract.artifacts[0].name, "example-linux-x86_64.tar.gz");
        assert!(contract.artifacts[0].checksum_sidecar);
    }

    #[test]
    fn test_parse_rejects_release_contract_invalid_severity() {
        let yaml = r#"
extensions:
  release_contracts:
    - id: cli_release
      severity: urgent
      artifacts:
        - name: example-linux-x86_64.tar.gz
      workflow_files:
        - .github/workflows/release.yml
      docs_files:
        - docs/install.md
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains("extensions.release_contracts.cli_release.severity"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_parse_rejects_duplicate_release_contract_id() {
        let yaml = r#"
extensions:
  release_contracts:
    - id: cli_release
      artifacts:
        - name: example-linux-x86_64.tar.gz
      workflow_files:
        - .github/workflows/release.yml
      docs_files:
        - docs/install.md
    - id: cli_release
      artifacts:
        - name: example-darwin-aarch64.tar.gz
      workflow_files:
        - .github/workflows/release.yml
      docs_files:
        - docs/install.md
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains("duplicate release contract id"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_parse_rejects_release_contract_path_escape() {
        let yaml = r#"
extensions:
  release_contracts:
    - id: cli_release
      artifacts:
        - name: example-linux-x86_64.tar.gz
      workflow_files:
        - ../release.yml
      docs_files:
        - docs/install.md
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains("workflow_files") && error.contains("must be relative"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_parse_with_support_matrix() {
        let yaml = r#"
extensions:
  support_matrices:
    - id: public_surface
      severity: high
      command_contracts:
        - .assura/command-surface.yml
      rust_exports:
        - src/lib.rs
      entries:
        - surface: "command:assura check"
          status: supported
        - surface: "rust:intelligence"
          status: internal
structure: {}
"#;

        let config = ConfigLoader::parse(yaml).unwrap();
        let extensions = config.extensions.unwrap();
        assert_eq!(extensions.support_matrices.len(), 1);
        let matrix = &extensions.support_matrices[0];
        assert_eq!(matrix.id, "public_surface");
        assert_eq!(matrix.entries[0].surface, "command:assura check");
        assert_eq!(matrix.entries[0].status, "supported");
    }

    #[test]
    fn test_parse_rejects_support_matrix_invalid_status() {
        let yaml = r#"
extensions:
  support_matrices:
    - id: public_surface
      command_contracts:
        - .assura/command-surface.yml
      entries:
        - surface: "command:assura check"
          status: stable
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains(
                "extensions.support_matrices.public_surface.entries.command:assura check.status"
            ),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_parse_rejects_support_matrix_path_escape() {
        let yaml = r#"
extensions:
  support_matrices:
    - id: public_surface
      command_contracts:
        - ../command-surface.yml
      entries:
        - surface: "command:assura check"
          status: supported
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains("command_contracts") && error.contains("must be relative"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_parse_with_quality_scopes() {
        let yaml = r#"
quality:
  scopes:
    rust:
      paths:
        - "src/**"
      frequent:
        - "cargo xtask check"
      pr:
        - "cargo xtask pr"
structure: {}
"#;

        let config = ConfigLoader::parse(yaml).unwrap();
        let quality = config.quality.unwrap();
        let rust = quality.scopes.get("rust").unwrap();
        assert_eq!(rust.paths, vec!["src/**"]);
        assert_eq!(rust.frequent, vec!["cargo xtask check"]);
        assert_eq!(rust.pr, vec!["cargo xtask pr"]);
    }

    #[test]
    fn test_parse_rejects_invalid_quality_scope_pattern() {
        let yaml = r#"
quality:
  scopes:
    rust:
      paths:
        - "../src/**"
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains("must be relative"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_parse_rejects_unknown_quality_scope_keys() {
        let yaml = r#"
quality:
  scopes:
    rust:
      paths:
        - "src/**"
      pre-push:
        - "cargo xtask test"
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains("unknown field") && error.contains("pre-push"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_parse_rejects_unsupported_custom_constraint_type() {
        let yaml = r#"
extensions:
  custom_constraints:
    - id: shell_plugin
      type: shell
      source: "src/*.rs"
      target: "tests/{stem}_test.rs"
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains("unsupported custom constraint"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn test_parse_rejects_custom_constraint_path_escape() {
        let yaml = r#"
extensions:
  custom_constraints:
    - id: source_test_pair
      type: paired_file_exists
      source: "src/*.rs"
      target: "../tests/{stem}_test.rs"
structure: {}
"#;

        let error = ConfigLoader::parse(yaml).unwrap_err().to_string();
        assert!(
            error.contains("must be relative"),
            "unexpected error: {error}"
        );
    }
}
