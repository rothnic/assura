//! Report and error types for structure-first checks.

use crate::cli::config::ConfigError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

/// Result of running a structure-first check.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructureCheckReport {
    /// Whether the checked path passed all configured validations.
    pub success: bool,
    /// Project root used to resolve relative config paths.
    pub project_root: PathBuf,
    /// Configuration file used for validation.
    pub config_path: PathBuf,
    /// Path that was checked.
    pub checked_path: PathBuf,
    /// Number of files checked.
    pub files_checked: usize,
    /// Number of directories checked.
    pub dirs_checked: usize,
    /// Validation violations.
    pub violations: Vec<StructureViolation>,
}

impl StructureCheckReport {
    /// Number of validation violations.
    pub fn violation_count(&self) -> usize {
        self.violations.len()
    }
}

/// A single structure validation violation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructureViolation {
    /// Path associated with the violation.
    pub path: PathBuf,
    /// Rule that produced the violation.
    pub rule: String,
    /// Human-readable violation message.
    pub message: String,
    /// Violation severity.
    pub severity: String,
}

impl StructureViolation {
    pub(in crate::cli::check) fn new(
        path: PathBuf,
        rule: impl Into<String>,
        message: impl Into<String>,
        severity: impl Into<String>,
    ) -> Self {
        Self {
            path,
            rule: rule.into(),
            message: message.into(),
            severity: severity.into(),
        }
    }
}

/// Errors produced while preparing or running a structure check.
#[derive(Debug, Error)]
pub enum CheckError {
    /// The target path does not exist.
    #[error("checked path does not exist: {0:?}")]
    MissingPath(PathBuf),
    /// No Assura configuration was found.
    #[error("no .assura/config.yml found for {0:?}")]
    NoConfig(PathBuf),
    /// The configured project root could not be determined.
    #[error("could not determine project root for config {0:?}")]
    InvalidConfigLocation(PathBuf),
    /// The checked path is outside the discovered project root.
    #[error("checked path {checked_path:?} is outside project root {project_root:?}")]
    OutsideProject {
        /// Path requested by the user.
        checked_path: PathBuf,
        /// Discovered project root.
        project_root: PathBuf,
    },
    /// Filesystem I/O failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Jwalk directory walking failed.
    #[cfg(feature = "full-cli")]
    #[error(transparent)]
    WalkDir(#[from] jwalk::Error),
    /// Walkdir directory walking failed.
    #[error(transparent)]
    Walkdir(#[from] walkdir::Error),
    /// Configuration loading failed.
    #[error(transparent)]
    Config(#[from] ConfigError),
}
