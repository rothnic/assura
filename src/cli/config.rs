//! CLI configuration loading and discovery helpers.
use std::path::{Path, PathBuf};

#[cfg(feature = "full-cli")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feature = "full-cli")]
use crate::constraints::ConstraintConfig;
#[cfg(feature = "full-cli")]
use std::collections::HashMap;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(String),

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
#[cfg(feature = "full-cli")]
pub struct CliConfig {
    #[serde(default = "default_version")]
    pub version: String,

    #[serde(default)]
    pub constraint_config: ConstraintConfig,

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
#[cfg(feature = "full-cli")]
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
#[cfg(feature = "full-cli")]
pub struct WatchConfig {
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    #[serde(default)]
    pub ignore_git: bool,

    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "full-cli")]
pub struct OutputConfig {
    #[serde(default = "default_format")]
    pub format: String,

    #[serde(default = "default_true")]
    pub colors: bool,

    #[serde(default = "default_true")]
    pub progress: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "full-cli")]
pub struct GitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub hooks: HashMap<String, HookConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "full-cli")]
pub struct HookConfig {
    pub enabled: bool,
    #[serde(default)]
    pub args: Vec<String>,
}

#[cfg(feature = "full-cli")]
impl CliConfig {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        let project_root = project_root.into();
        Self {
            version: default_version(),
            constraint_config: ConstraintConfig::default(),
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

    pub fn save(&self) -> ConfigResult<PathBuf> {
        let config_dir = self.project_root.join(ASSURA_DIR);
        std::fs::create_dir_all(&config_dir)?;

        let config_path = config_dir.join(CONFIG_FILE);
        let content =
            serde_yaml::to_string(self).map_err(|error| ConfigError::Yaml(error.to_string()))?;
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
        let mut config: CliConfig =
            serde_yaml::from_str(&content).map_err(|error| ConfigError::Yaml(error.to_string()))?;
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

#[cfg(feature = "full-cli")]
impl Default for CliConfig {
    fn default() -> Self {
        Self::new(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
    }
}

#[cfg(feature = "full-cli")]
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

#[cfg(feature = "full-cli")]
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

#[cfg(feature = "full-cli")]
impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: "text".to_string(),
            colors: true,
            progress: true,
        }
    }
}

#[cfg(feature = "full-cli")]
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

#[cfg(feature = "full-cli")]
fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[cfg(feature = "full-cli")]
fn default_true() -> bool {
    true
}

#[cfg(feature = "full-cli")]
fn default_debounce_ms() -> u64 {
    300
}

#[cfg(feature = "full-cli")]
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

            current = current.parent()?.to_path_buf();
        }
    }

    pub fn find_config_path(start: impl AsRef<Path>) -> Option<PathBuf> {
        let start = start.as_ref();
        let mut current = if start.is_dir() {
            start
        } else {
            start.parent()?
        };
        loop {
            let config = current.join(ASSURA_DIR).join(CONFIG_FILE);
            if config.exists() {
                return Some(config);
            }
            if current.join(".git").exists() {
                return None;
            }
            current = current.parent()?;
        }
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

            current = current.parent()?.to_path_buf();
        }
    }
}

#[cfg(all(test, feature = "full-cli"))]
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
    fn config_discovery_skips_cache_only_assura_marker() {
        let project = tempdir().unwrap();
        let root_config = project.path().join(".assura/config.yml");
        std::fs::create_dir_all(root_config.parent().unwrap()).unwrap();
        std::fs::write(&root_config, "structure: {}\n").unwrap();
        let nested = project.path().join("src/components");
        let cache = nested.join(".assura/cache/check");
        std::fs::create_dir_all(&cache).unwrap();
        std::fs::write(cache.join("keep.json"), "{}\n").unwrap();
        let file = nested.join("Button.tsx");
        std::fs::write(&file, "").unwrap();

        for selected in [&nested, &file] {
            assert_eq!(
                ConfigDiscovery::find_config_path(selected),
                Some(root_config.clone())
            );
        }
        assert_eq!(
            ConfigDiscovery::find_project_root(&nested),
            Some(nested.clone())
        );
        assert!(cache.join("keep.json").is_file());
    }

    #[test]
    fn config_discovery_prefers_nearest_real_policy() {
        let project = tempdir().unwrap();
        for root in [project.path().to_path_buf(), project.path().join("nested")] {
            std::fs::create_dir_all(root.join(".assura")).unwrap();
            std::fs::write(root.join(".assura/config.yml"), "structure: {}\n").unwrap();
        }
        let selected = project.path().join("nested/src");
        std::fs::create_dir_all(&selected).unwrap();
        assert_eq!(
            ConfigDiscovery::find_config_path(selected),
            Some(project.path().join("nested/.assura/config.yml"))
        );
    }

    #[test]
    fn config_discovery_does_not_replace_malformed_nearest_policy() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".assura")).unwrap();
        std::fs::write(project.path().join(".assura/config.yml"), "structure: {}\n").unwrap();
        let nested = project.path().join("nested");
        let malformed = nested.join(".assura/config.yml");
        std::fs::create_dir_all(malformed.parent().unwrap()).unwrap();
        std::fs::write(&malformed, "structure: [\n").unwrap();
        for directory in [false, true] {
            if directory {
                std::fs::remove_file(&malformed).unwrap();
                std::fs::create_dir(&malformed).unwrap();
            }
            assert_eq!(
                ConfigDiscovery::find_config_path(&nested),
                Some(malformed.clone())
            );
            let error =
                crate::cli::run_structure_check(Some(nested.clone()), None, false).unwrap_err();
            assert!(
                matches!(error, crate::cli::check::CheckError::Config(_)),
                "{error}"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn config_discovery_retains_dangling_or_non_directory_policy_candidates() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".assura")).unwrap();
        std::fs::write(project.path().join(".assura/config.yml"), "structure: {}\n").unwrap();
        let nested = project.path().join("nested");
        let marker = nested.join(".assura");
        let nearest = marker.join("config.yml");
        std::fs::create_dir_all(&marker).unwrap();
        let selected = nested.join("source.ts");
        std::fs::write(&selected, "").unwrap();
        std::os::unix::fs::symlink("missing-policy.yml", &nearest).unwrap();

        for non_directory in [false, true] {
            if non_directory {
                std::fs::remove_file(&nearest).unwrap();
                std::fs::remove_dir(&marker).unwrap();
                std::fs::write(&marker, "not a directory\n").unwrap();
            }
            assert_eq!(
                ConfigDiscovery::find_config_path(&selected),
                Some(nearest.clone())
            );
            assert!(crate::cli::run_structure_check(Some(selected.clone()), None, false).is_err());
        }
    }

    #[test]
    fn config_discovery_stops_at_git_boundary_without_borrowing_outer_policy() {
        let project = tempdir().unwrap();
        std::fs::create_dir_all(project.path().join(".assura")).unwrap();
        std::fs::write(project.path().join(".assura/config.yml"), "structure: {}\n").unwrap();
        let nested = project.path().join("nested");
        std::fs::create_dir_all(nested.join(".git")).unwrap();
        std::fs::create_dir_all(nested.join("src/.assura/cache/check")).unwrap();
        assert_eq!(ConfigDiscovery::find_config_path(nested.join("src")), None);
        std::fs::create_dir_all(nested.join(".assura")).unwrap();
        let nested_config = nested.join(".assura/config.yml");
        std::fs::write(&nested_config, "structure: {}\n").unwrap();
        assert_eq!(
            ConfigDiscovery::find_config_path(nested.join("src")),
            Some(nested_config)
        );
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
