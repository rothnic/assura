use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::maturity::engine::{MaturityLevel, MaturityReport};

/// Configuration file for maturity overrides
const CONFIG_FILENAME: &str = "maturity.yml";
const CONFIG_DIR: &str = ".assura";

/// Configuration for maturity detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaturityConfig {
    /// Manual override for maturity level
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<MaturityLevel>,

    /// Override the overall score
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,

    /// Override confidence
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,

    /// Custom weights for signal categories
    #[serde(default)]
    pub weights: CategoryWeights,

    /// Custom thresholds for levels
    #[serde(default)]
    pub thresholds: LevelThresholds,

    /// Whether this is a manual override
    #[serde(default)]
    pub manual_override: bool,

    /// Reason for manual override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_reason: Option<String>,

    /// Ignore specific signals
    #[serde(default)]
    pub ignore_signals: Vec<String>,

    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl MaturityConfig {
    /// Create an empty config
    pub fn new() -> Self {
        Self {
            level: None,
            score: None,
            confidence: None,
            weights: CategoryWeights::default(),
            thresholds: LevelThresholds::default(),
            manual_override: false,
            override_reason: None,
            ignore_signals: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Load config from a directory (looks for .assura/maturity.yml)
    pub fn from_directory<P: AsRef<Path>>(path: P) -> anyhow::Result<Option<Self>> {
        let config_path = path.as_ref().join(CONFIG_DIR).join(CONFIG_FILENAME);
        
        if !config_path.exists() {
            return Ok(None);
        }

        Self::from_file(&config_path)
    }

    /// Load config from a specific file
    pub fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Option<Self>> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(path)?;

        let config: MaturityConfig = serde_yaml::from_str(&content)?;

        Ok(Some(config))
    }

    /// Save config to a directory
    pub fn save_to_directory<P: AsRef<Path>>(&self,
        path: P,
    ) -> anyhow::Result<PathBuf> {
        let config_dir = path.as_ref().join(CONFIG_DIR);
        
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        let config_path = config_dir.join(CONFIG_FILENAME);
        self.save_to_file(&config_path)?;

        Ok(config_path)
    }

    /// Save config to a specific file
    pub fn save_to_file<P: AsRef<Path>>(&self,
        path: P,
    ) -> anyhow::Result<()> {
        let content = serde_yaml::to_string(self)?;

        std::fs::write(path, content)?;

        Ok(())
    }

    /// Check if this config represents a manual override
    pub fn is_manual_override(&self) -> bool {
        self.manual_override || self.level.is_some() || self.score.is_some()
    }

    /// Convert config to a maturity report (for manual overrides)
    pub fn to_report(&self) -> MaturityReport {
        use super::engine::CategoryScores;
        use std::time::{SystemTime, UNIX_EPOCH};

        let level = self.level.unwrap_or(MaturityLevel::Raw);
        let score = self.score.unwrap_or_else(|| level.threshold());
        let confidence = self.confidence.unwrap_or(1.0);

        MaturityReport {
            level,
            score,
            confidence,
            signals: Vec::new(),
            category_scores: CategoryScores::default(),
            recommendations: Vec::new(),
            assessed_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Adjust a report based on this config
    pub fn adjust_report(&self,
        mut report: MaturityReport,
    ) -> MaturityReport {
        // Apply level override
        if let Some(level) = self.level {
            report.level = level;
        }

        // Apply score override
        if let Some(score) = self.score {
            report.score = score;
        }

        // Apply confidence override
        if let Some(confidence) = self.confidence {
            report.confidence = confidence;
        }

        // Filter out ignored signals
        if !self.ignore_signals.is_empty() {
            report.signals.retain(|s| {
                !self.ignore_signals.contains(&s.name)
            });
        }

        report
    }

    /// Create a builder for manual override
    pub fn manual_override(level: MaturityLevel, reason: impl Into<String>) -> Self {
        Self {
            level: Some(level),
            score: None,
            confidence: Some(1.0),
            weights: CategoryWeights::default(),
            thresholds: LevelThresholds::default(),
            manual_override: true,
            override_reason: Some(reason.into()),
            ignore_signals: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        // Validate score is in valid range [0.0, 1.0]
        if let Some(score) = self.score {
            if score < 0.0 || score > 1.0 {
                anyhow::bail!("Score must be between 0.0 and 1.0, got {}", score);
            }
        }

        // Validate confidence is in valid range [0.0, 1.0]
        if let Some(confidence) = self.confidence {
            if confidence < 0.0 || confidence > 1.0 {
                anyhow::bail!("Confidence must be between 0.0 and 1.0, got {}", confidence);
            }
        }

        // Validate manual override has a reason
        if self.manual_override && self.override_reason.is_none() {
            anyhow::bail!("Manual override must have a reason");
        }

        Ok(())
    }
}

impl Default for MaturityConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom weights for signal categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryWeights {
    #[serde(default = "default_weight")]
    pub git: f64,
    #[serde(default = "default_weight")]
    pub filesystem: f64,
    #[serde(default = "default_weight")]
    pub environment: f64,
}

impl CategoryWeights {
    pub fn new(git: f64, filesystem: f64, environment: f64) -> Self {
        Self {
            git,
            filesystem,
            environment,
        }
    }
}

impl Default for CategoryWeights {
    fn default() -> Self {
        Self {
            git: 1.0,
            filesystem: 1.0,
            environment: 1.0,
        }
    }
}

fn default_weight() -> f64 {
    1.0
}

/// Custom thresholds for maturity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelThresholds {
    #[serde(default = "default_developing")]
    pub developing: f64,
    #[serde(default = "default_mature")]
    pub mature: f64,
    #[serde(default = "default_established")]
    pub established: f64,
}

impl LevelThresholds {
    pub fn new(developing: f64, mature: f64, established: f64) -> Self {
        Self {
            developing,
            mature,
            established,
        }
    }

    /// Get threshold for a specific level
    pub fn threshold_for(&self,
        level: MaturityLevel,
    ) -> f64 {
        match level {
            MaturityLevel::Raw => 0.0,
            MaturityLevel::Developing => self.developing,
            MaturityLevel::Mature => self.mature,
            MaturityLevel::Established => self.established,
        }
    }
}

impl Default for LevelThresholds {
    fn default() -> Self {
        Self {
            developing: 0.3,
            mature: 0.6,
            established: 0.85,
        }
    }
}

fn default_developing() -> f64 {
    0.3
}

fn default_mature() -> f64 {
    0.6
}

fn default_established() -> f64 {
    0.85
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_config_default() {
        let config = MaturityConfig::new();
        assert!(!config.is_manual_override());
        assert!(config.level.is_none());
    }

    #[test]
    fn test_manual_override_creation() {
        let config = MaturityConfig::manual_override(
            MaturityLevel::Mature,
            "Project is well-established"
        );

        assert!(config.is_manual_override());
        assert_eq!(config.level, Some(MaturityLevel::Mature));
        assert_eq!(config.override_reason, Some("Project is well-established".to_string()));
    }

    #[test]
    fn test_config_to_report() {
        let config = MaturityConfig::manual_override(
            MaturityLevel::Established,
            "Test override"
        );

        let report = config.to_report();
        assert_eq!(report.level, MaturityLevel::Established);
        assert_eq!(report.confidence, 1.0);
    }

    #[test]
    fn test_adjust_report() {
        let config = MaturityConfig {
            level: Some(MaturityLevel::Mature),
            score: Some(0.75),
            ..Default::default()
        };

        let report = MaturityReport::new(
            MaturityLevel::Raw,
            0.1,
            0.5,
            Vec::new(),
        );

        let adjusted = config.adjust_report(report);
        assert_eq!(adjusted.level, MaturityLevel::Mature);
        assert_eq!(adjusted.score, 0.75);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let config = MaturityConfig::manual_override(
            MaturityLevel::Mature,
            "Test save/load"
        );

        // Save
        let path = config.save_to_directory(temp_dir.path()).unwrap();
        assert!(path.exists());

        // Load
        let loaded = MaturityConfig::from_directory(temp_dir.path())
            .unwrap()
            .expect("Should load config");

        assert_eq!(loaded.level, Some(MaturityLevel::Mature));
        assert_eq!(loaded.override_reason, Some("Test save/load".to_string()));
    }

    #[test]
    fn test_yaml_serialization() {
        let config = MaturityConfig {
            level: Some(MaturityLevel::Developing),
            score: Some(0.45),
            weights: CategoryWeights::new(1.5, 1.0, 0.5),
            manual_override: true,
            override_reason: Some("Testing serialization".to_string()),
            ..Default::default()
        };

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("level: Developing"));
        assert!(yaml.contains("manual_override: true"));

        let deserialized: MaturityConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.level, Some(MaturityLevel::Developing));
    }
}
