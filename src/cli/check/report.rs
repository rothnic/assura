//! Report and error types for structure-first checks.

use crate::cli::config::ConfigError;
use serde::{Deserialize, Serialize, Serializer};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Result of running a structure-first check.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructureCheckReport {
    /// Whether the checked path passed all configured validations.
    pub success: bool,
    /// Project root used to resolve relative config paths.
    #[serde(serialize_with = "serialize_path")]
    pub project_root: PathBuf,
    /// Configuration file used for validation.
    #[serde(serialize_with = "serialize_path")]
    pub config_path: PathBuf,
    /// Path that was checked.
    #[serde(serialize_with = "serialize_path")]
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

    /// Whether any validation violation should fail the check.
    pub fn has_blocking_violations(&self) -> bool {
        self.violations.iter().any(StructureViolation::is_blocking)
    }

    /// Refresh the success flag from the shared severity contract.
    pub fn refresh_success(&mut self) {
        self.success = !self.has_blocking_violations();
    }
}

/// A single structure validation violation.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StructureViolation {
    /// Path associated with the violation.
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    /// Rule that produced the violation.
    pub rule: String,
    /// Human-readable violation message.
    pub message: String,
    /// Violation severity.
    pub severity: String,
    /// Stable display label for the severity.
    pub severity_label: String,
    /// Whether this violation should make the check fail.
    pub blocking: bool,
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
        let severity_value = severity.into();
        let severity = FindingSeverity::parse_or_default(&severity_value);
        Self {
            path,
            corrective_context: corrective_context_for_rule(&rule).to_string(),
            rule,
            message: message.into(),
            severity: severity.as_str().to_string(),
            severity_label: severity.label().to_string(),
            blocking: severity.is_blocking(),
        }
    }

    /// Whether this violation should make the check fail.
    pub fn is_blocking(&self) -> bool {
        self.blocking
    }

    /// Numeric rank for deterministic prioritization.
    pub fn severity_rank(&self) -> u8 {
        FindingSeverity::parse_or_default(&self.severity).rank()
    }
}

/// Stable beta severity contract for structure findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingSeverity {
    /// Advisory finding that should be shown but should not fail the check.
    Low,
    /// Default blocking finding.
    Medium,
    /// High-priority blocking finding.
    High,
    /// Critical blocking finding.
    Critical,
}

impl FindingSeverity {
    /// Parse a configured severity, defaulting unknown values to `medium`.
    pub fn parse_or_default(value: &str) -> Self {
        match value.to_ascii_lowercase().as_str() {
            "low" => Self::Low,
            "high" => Self::High,
            "critical" => Self::Critical,
            _ => Self::Medium,
        }
    }

    /// Stable lowercase identifier.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    /// Stable display label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low",
            Self::Medium => "Medium",
            Self::High => "High",
            Self::Critical => "Critical",
        }
    }

    /// Stable ordering for prioritization.
    pub fn rank(self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }

    /// Whether this severity should fail `assura check`.
    pub fn is_blocking(self) -> bool {
        !matches!(self, Self::Low)
    }
}

fn corrective_context_for_rule(rule: &str) -> &'static str {
    if rule.starts_with("content_runtime:") {
        return "Update the configured content model, runtime schema artifact, content file, or relation so repo-native object validation passes.";
    }
    if rule.starts_with("custom:") {
        return "Fix the configured custom constraint target, or update extensions.custom_constraints in .assura/config.yml when the project policy changed.";
    }
    if rule.starts_with("release_contract:") {
        return "Update the release artifact contract, workflow uploads, installer URLs, or release documentation so configured release assets agree.";
    }
    if rule.starts_with("support_matrix:") {
        return "Add the public surface to the configured support matrix, or remove/rename the exposed command/API surface when it is not intentional.";
    }
    if rule.starts_with("manifest_semantics:") {
        return "Update the Cargo manifest metadata or the configured manifest semantics policy so package fields, publish status, and binary declarations agree.";
    }
    if rule.starts_with("test_relationship:") {
        return "Add the missing test evidence, classify the ignored/manual test or fixture family, or update extensions.test_relationships when the project policy changed.";
    }
    if rule.starts_with("module_topology:") {
        return "Classify the public module/export, fix the configured module root, or update extensions.module_topologies when module ownership changed.";
    }
    if rule.starts_with("docs_lifecycle:") {
        return "Add lifecycle metadata, declare current evidence for the claim, add an explicit historical exception, or update extensions.docs_lifecycles when docs policy changed.";
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
        "markdown_frontmatter" => {
            "Add YAML frontmatter, or relax markdown.require_frontmatter for this scope. Use content runtime models and collections for typed frontmatter fields."
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
        "markdown_trailing_spaces" => {
            "Run `assura fix markdown --dry-run` to preview safe blank-line trailing-space fixes, then rerun with `--apply` to write them, or disable markdown.lint_trailing_spaces for this scope."
        }
        "markdown_heading_increment" => {
            "Promote intermediate headings or demote the skipped heading so heading levels increase one level at a time."
        }
        "markdown_heading_marker_spacing" => {
            "Use exactly one space between the heading marker and heading text, such as `## Heading`."
        }
        "markdown_duplicate_heading" => {
            "Rename one repeated heading so Markdown anchors remain stable and unambiguous."
        }
        "markdown_multiple_blank_lines" => {
            "Collapse consecutive blank lines to a single blank line."
        }
        "markdown_suppression" => {
            "Use `<!-- assura-ignore <markdown_rule>: <reason> -->` with a supported Markdown rule ID and a non-empty reason."
        }
        "markdown_link_format" => {
            "Rewrite the reference as a relative Markdown link so GitHub can render it in branches, forks, and pull requests."
        }
        "markdown_link_target" => {
            "Create the linked file, fix the relative path, or remove the stale Markdown link."
        }
        "markdown_link_heading_anchor" => {
            "Update the heading slug in the Markdown link or rename the target heading so the GitHub-rendered anchor exists."
        }
        "markdown_link_line_anchor" => {
            "Update the GitHub-style line or line-range anchor so it points at existing target lines."
        }
        "repository_reference_target" => {
            "Create the referenced file, fix the local path in the comment/string/docstring, or remove the stale reference."
        }
        "repository_reference_anchor" => {
            "Update the referenced Markdown heading anchor or rename the target heading so the local reference points at an existing section."
        }
        "repository_reference_line_anchor" => {
            "Update the referenced line or line range so it points at existing target lines."
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

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&path.to_string_lossy().replace('\\', "/"))
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
