use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::constraints::ConstraintConfig;
use crate::maturity::MaturityConfig;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Configuration not found at: {0}")]
    NotFound(PathBuf),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Project not initialized (no .assura directory found)")]
    NotInitialized,
}

pub type ConfigResult<T> = Result<T, ConfigError>;

const ASSURA_DIR: &str = ".assura";
const CONFIG_FILE: &str = "config.yml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    #[serde(default = "default_version")]
    pub version: String,

    #[serde(default)]
    pub constraint_config: ConstraintConfig,

    #[serde(default)]
    pub maturity_config: MaturityConfig,

    #[serde(default)]
    pub check: CheckConfig,

    #[serde(default)]
    pub watch: WatchConfig,

    #[serde(default)]
    pub output: OutputConfig,

    #[serde(default)]
    pub git: GitConfig,

    #[serde(skip)]
    pub project_root: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckConfig {
    #[serde(default = "default_true")]
    pub parallel: bool,

    #[serde(default)]
    pub fail_fast: bool,

    #[serde(default)]
    pub include_patterns: Vec<String>,

    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_file_size_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    #[serde(default)]
    pub ignore_git: bool,

    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default = "default_true")]
    pub colors: bool,

    #[serde(default = "default_true")]
    pub progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub hooks: HashMap<String, HookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub enabled: bool,
    #[serde(default)]
    pub args: Vec<String>,
}

impl CliConfig {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        Self {
            version: default_version(),
            constraint_config: ConstraintConfig::default(),
            maturity_config: MaturityConfig::default(),
            check: CheckConfig::default(),
            watch: WatchConfig::default(),
            output: OutputConfig::default(),
            git: GitConfig::default(),
            project_root,
        }
    }

    pub fn with_constraint_config(mut self, config: ConstraintConfig) -> Self {
        self.constraint_config = config;
        self
    }

    pub fn with_maturity_config(mut self, config: MaturityConfig) -> Self {
        self.maturity_config = config;
        self
    }

    pub fn save(&self) -> ConfigResult<PathBuf> {
        let config_dir = self.project_root.join(ASSURA_DIR);
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join(CONFIG_FILE);
        let content = serde_yaml::to_string(self)?;
        std::fs::write(&config_path, content)?;

        Ok(config_path)
    }

    pub fn load(project_root: impl AsRef<Path>) -> ConfigResult<Self> {
        let project_root = project_root.as_ref();
        let config_path = project_root.join(ASSURA_DIR).join(CONFIG_FILE);

        if !config_path.exists() {
            return Err(ConfigError::NotInitialized);
        }

        let content = std::fs::read_to_string(&config_path)?;
        let mut config: CliConfig = serde_yaml::from_str(&content)?;
        config.project_root = project_root.to_path_buf();

        Ok(config)
    }

    pub fn exists(project_root: impl AsRef<Path>) -> bool {
        project_root
            .as_ref()
            .join(ASSURA_DIR)
            .join(CONFIG_FILE)
            .exists()
    }

    pub fn assura_dir(&self) -> PathBuf {
        self.project_root.join(ASSURA_DIR)
    }

    pub fn hooks_dir(&self) -> PathBuf {
        self.assura_dir().join("hooks")
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.assura_dir().join("cache")
    }
}

impl Default for CliConfig {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}

impl Default for CheckConfig {
    fn default() -> Self {
        Self {
            parallel: true,
            fail_fast: false,
            include_patterns: Vec::new(),
            exclude_patterns: vec![
                "node_modules/**".to_string(),
                ".git/**".to_string(),
                "target/**".to_string(),
            ],
            max_file_size_mb: None,
        }
    }
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 300,
            ignore_git: false,
            exclude_patterns: vec![
                ".git/**".to_string(),
                "node_modules/**".to_string(),
                "target/**".to_string(),
            ],
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "text".to_string(),
            colors: true,
            progress: true,
        }
    }
}

impl Default for GitConfig {
    fn default() -> Self {
        let mut hooks = HashMap::new();
        hooks.insert(
            "pre-commit".to_string(),
            HookConfig {
                enabled: true,
                args: vec!["--fail-fast".to_string()],
            },
        );
        hooks.insert(
            "pre-push".to_string(),
            HookConfig {
                enabled: true,
                args: vec![],
            },
        );
        hooks.insert(
            "post-checkout".to_string(),
            HookConfig {
                enabled: false,
                args: vec!["--format".to_string(), "json".to_string()],
            },
        );

        Self {
            enabled: true,
            hooks,
        }
    }
}

fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_true() -> bool {
    true
}

fn default_debounce_ms() -> u64 {
    300
}

fn default_format() -> String {
    "text".to_string()
}

pub struct ConfigDiscovery;

impl ConfigDiscovery {
    pub fn find_project_root(start: impl AsRef<Path>) -> Option<PathBuf> {
        let start = start.as_ref();
        let mut current = if start.is_dir() {
            start.to_path_buf()
        } else {
            start.parent()?.to_path_buf()
        };

        loop {
            if current.join(ASSURA_DIR).exists() {
                return Some(current);
            }

            if current.join(".git").exists() {
                return Some(current);
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return None,
            }
        }
    }

    pub fn find_config_path(start: impl AsRef<Path>) -> Option<PathBuf> {
        Self::find_project_root(start).map(|root| root.join(ASSURA_DIR).join(CONFIG_FILE))
    }

    pub fn find_git_dir(start: impl AsRef<Path>) -> Option<PathBuf> {
        let start = start.as_ref();
        let mut current = if start.is_dir() {
            start.to_path_buf()
        } else {
            start.parent()?.to_path_buf()
        };

        loop {
            let git_dir = current.join(".git");
            if git_dir.exists() {
                return Some(git_dir);
            }

            match current.parent() {
                Some(parent) => current = parent.to_path_buf(),
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = CliConfig::default();
        assert!(!config.project_root.as_os_str().is_empty());
    }

    #[test]
    fn test_config_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config = CliConfig::new(temp_dir.path());

        let path = config.save().unwrap();
        assert!(path.exists());

        let loaded = CliConfig::load(temp_dir.path()).unwrap();
        assert_eq!(loaded.version, config.version);
    }

    #[test]
    fn test_config_discovery_find_project_root() {
        let temp_dir = tempdir().unwrap();
        let assura_dir = temp_dir.path().join(ASSURA_DIR);
        std::fs::create_dir_all(&assura_dir).unwrap();

        let found = ConfigDiscovery::find_project_root(temp_dir.path());
        assert_eq!(found, Some(temp_dir.path().to_path_buf()));
    }

    #[test]
    fn test_config_discovery_find_git_root() {
        let temp_dir = tempdir().unwrap();
        let git_dir = temp_dir.path().join(".git");
        std::fs::create_dir_all(&git_dir).unwrap();

        let found = ConfigDiscovery::find_project_root(temp_dir.path());
        assert_eq!(found, Some(temp_dir.path().to_path_buf()));
    }

    #[test]
    fn test_config_not_found() {
        let temp_dir = tempdir().unwrap();
        let result = CliConfig::load(temp_dir.path());
        assert!(matches!(result, Err(ConfigError::NotInitialized)));
    }

    #[test]
    fn test_yaml_serialization() {
        let config = CliConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("version:"));
        assert!(yaml.contains("parallel:"));
    }
}
