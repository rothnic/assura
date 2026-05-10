//! Configuration loader for unified structure-first config
//!
//! Handles parsing and validation of configuration files.

use super::config::Config;
use crate::cli::config::{ConfigError, ConfigResult};
use std::path::Path;
use validator::Validate;

/// Loader for structure configs
#[derive(Debug)]
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load a config from a file path
    pub fn load(path: &Path) -> ConfigResult<Config> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        Self::parse(&content)
    }

    /// Parse config from YAML string
    pub fn parse(content: &str) -> ConfigResult<Config> {
        let config: Config = serde_yaml::from_str(content).map_err(ConfigError::Yaml)?;
        config
            .validate()
            .map_err(|e| ConfigError::Invalid(format!("Configuration validation failed: {}", e)))?;
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
}
