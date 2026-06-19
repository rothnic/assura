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
    /// Stable corrective context for fixing the violation or policy.
    pub corrective_context: String,
}

impl StructureViolation {
    pub(in crate::cli::check) fn new(
        path: PathBuf,
        rule: impl Into<String>,
        message: impl Into<String>,
        severity: impl Into<String>,
    ) -> Self {
        let rule = rule.into();
        Self {
            path,
            corrective_context: corrective_context_for_rule(&rule).to_string(),
            rule,
            message: message.into(),
            severity: severity.into(),
        }
    }
}

fn corrective_context_for_rule(rule: &str) -> &'static str {
    if rule.starts_with("custom:") {
        return "Fix the configured custom constraint target, or update extensions.custom_constraints in .assura/config.yml when the project policy changed.";
    }
    if rule.starts_with("release_contract:") {
        return "Update the release artifact contract, workflow uploads, installer URLs, or release documentation so configured release assets agree.";
    }
    if rule.starts_with("support_matrix:") {
        return "Add the public surface to the configured support matrix, or remove/rename the exposed command/API surface when it is not intentional.";
    }
    if rule.starts_with("relationship:") {
        return "Create one of the expected counterpart/provider artifacts named in the relationship message, or update the declaring structure entry in .assura/config.yml.";
    }

    match rule {
        "file_naming" => {
            "Rename the file to match the effective naming rule, or update files.naming/naming_patterns when the policy is stale."
        }
        "directory_naming" => {
            "Rename the directory to match the effective directory naming rule, or update directories.naming/self_directory.naming when the policy is stale."
        }
        "required_file" => {
            "Create the required file at the reported path, or remove it from files.required/exists.files when it is no longer required."
        }
        "required_directory" => {
            "Create the required directory at the reported path, or remove it from directories.required/exists.directories/children when it is no longer required."
        }
        "unexpected_file" => {
            "Remove or move the file, or declare it with files.allowed_names, files.allowed_patterns, extensions, or allow_extra."
        }
        "forbidden_file" => {
            "Remove or rename the file, or narrow files.forbidden_patterns if this file should be allowed."
        }
        "unexpected_directory" => {
            "Remove or move the directory, or declare it with children, directories.allowed_names, directories.allowed_patterns, or allow_extra."
        }
        "forbidden_directory" => {
            "Remove or rename the directory, or narrow directories.forbidden_patterns if this directory should be allowed."
        }
        "exists_count" => {
            "Adjust direct child files/directories until the count matches the configured exists range, or update the range in .assura/config.yml."
        }
        "markdown_frontmatter" | "markdown_frontmatter_field" | "markdown_frontmatter_parse" => {
            "Add valid YAML frontmatter with the required fields, or relax markdown.require_frontmatter/required_fields for this scope."
        }
        "markdown_heading_depth" => {
            "Promote deep headings or increase markdown.max_heading_depth when the deeper outline is intentional."
        }
        "markdown_required_section" => {
            "Add the missing heading text or update markdown.required_sections when the section is no longer required."
        }
        "markdown_outline" => {
            "Add or reorder headings to match markdown.outline, or update the configured outline when the documented structure changed."
        }
        "extension" => {
            "Rename the file to an allowed extension or update files.extensions for this scope."
        }
        "max_lines" => {
            "Split or shorten the file, or raise files.max_lines when the size is intentional."
        }
        "max_size" => {
            "Reduce the file size or raise files.max_size when the larger file is intentional."
        }
        "require_docs" => {
            "Add module or item rustdoc, or disable files.require_docs for this scope."
        }
        _ => {
            "Inspect the reported path and effective rule, then update the file tree or .assura/config.yml so they agree."
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
