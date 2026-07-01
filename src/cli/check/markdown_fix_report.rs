//! Machine-readable Markdown safe-fix report types.

use super::markdown_fix::MarkdownFixRule;
use serde::{Serialize, Serializer};
use std::path::{Path, PathBuf};

/// Summary of a Markdown fix run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkdownFixReport {
    /// Stable report schema for agent/editor wrappers.
    pub schema: &'static str,
    /// Project root used for config discovery.
    #[serde(serialize_with = "serialize_path")]
    pub project_root: PathBuf,
    /// Path checked by the user.
    #[serde(serialize_with = "serialize_path")]
    pub checked_path: PathBuf,
    /// Whether this run only planned changes without writing files.
    pub dry_run: bool,
    /// Execution mode for this report.
    pub mode: MarkdownFixMode,
    /// Safe fix rule requested by the caller.
    pub rule: MarkdownFixRuleReport,
    /// Configured Markdown files considered for this fix rule.
    pub files_checked: usize,
    /// Files written with at least one fix.
    pub files_changed: usize,
    /// Individual fixes applied.
    pub fixes_applied: usize,
    /// Files that would change if the same run wrote changes.
    pub files_would_change: usize,
    /// Individual fixes that would apply if the same run wrote changes.
    pub fixes_would_apply: usize,
    /// Fixable findings before the run.
    pub fixes_before: usize,
    /// Fixable findings remaining in files after the run.
    pub fixes_after: usize,
    /// Relative paths written during an apply run.
    #[serde(serialize_with = "serialize_paths")]
    pub changed_paths: Vec<PathBuf>,
    /// Stable fix IDs applied during an apply run.
    pub applied_fix_ids: Vec<String>,
    /// Per-file safe-fix plan and audit records.
    pub files: Vec<MarkdownFixFileReport>,
    /// Per-fix safe-fix plan and audit records.
    pub fixes: Vec<MarkdownFixRecord>,
    /// Markdown files considered but intentionally skipped.
    pub skipped_fixes: Vec<MarkdownFixSkip>,
    /// Write failures recorded before returning an error.
    pub failures: Vec<MarkdownFixFailure>,
    /// Recovery guidance for apply runs.
    pub rollback: MarkdownFixRollback,
}

/// Execution mode for a Markdown safe-fix report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownFixMode {
    /// Preview fixes without writing.
    DryRun,
    /// Apply fixes to disk.
    Apply,
}

/// Markdown safe-fix rule name for report consumers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MarkdownFixRuleReport {
    /// Remove spaces and tabs from otherwise blank Markdown lines.
    TrailingSpaces,
    /// Append configured required Markdown section headings.
    RequiredSections,
}

impl From<MarkdownFixRule> for MarkdownFixRuleReport {
    fn from(value: MarkdownFixRule) -> Self {
        match value {
            MarkdownFixRule::TrailingSpaces => Self::TrailingSpaces,
            MarkdownFixRule::RequiredSections => Self::RequiredSections,
        }
    }
}

/// Per-file Markdown safe-fix report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkdownFixFileReport {
    /// Relative path from the project root.
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    /// File-level outcome.
    pub status: MarkdownFixFileStatus,
    /// Fixable findings before the run.
    pub fixes_before: usize,
    /// Fixable findings after the run.
    pub fixes_after: usize,
    /// Fixes planned for this file.
    pub fixes_planned: usize,
    /// Fixes applied to this file.
    pub fixes_applied: usize,
    /// Stable fix IDs planned or applied for this file.
    pub fix_ids: Vec<String>,
}

/// File-level Markdown safe-fix status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownFixFileStatus {
    /// File has fixes in a dry run.
    Planned,
    /// File was changed during apply.
    Changed,
    /// File had fixes but could not be changed.
    Failed,
    /// File was eligible and already clean.
    Unchanged,
}

/// Per-fix Markdown safe-fix plan or audit record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkdownFixRecord {
    /// Stable fix ID for correlating previews, applies, and audits.
    pub id: String,
    /// Relative path from the project root.
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    /// Safe operation name.
    pub operation: &'static str,
    /// Record status for this run.
    pub status: MarkdownFixStatus,
    /// One-based line number.
    pub line: usize,
    /// One-based column number.
    pub column: usize,
    /// Trailing whitespace characters before the fix, or zero for non-whitespace fixes.
    pub before_trailing_whitespace: usize,
    /// Trailing whitespace characters after the run, or zero for non-whitespace fixes.
    pub after_trailing_whitespace: usize,
    /// Text planned or written by this fix when the operation inserts content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inserted_text: Option<String>,
}

/// Per-fix Markdown safe-fix status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownFixStatus {
    /// Fix was planned but not written.
    Planned,
    /// Fix was written.
    Applied,
    /// Fix could not be written.
    Failed,
}

/// Markdown file skipped by the safe-fix workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkdownFixSkip {
    /// Relative path from the project root.
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    /// Skipped operation name.
    pub operation: &'static str,
    /// Stable skipped-fix ID for audit correlation.
    pub id: String,
    /// Machine-readable reason.
    pub reason: &'static str,
    /// Human-readable explanation.
    pub message: String,
}

/// Markdown safe-fix failure recorded before aborting the run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkdownFixFailure {
    /// Relative path from the project root.
    #[serde(serialize_with = "serialize_path")]
    pub path: PathBuf,
    /// Failed operation name.
    pub operation: &'static str,
    /// Stable fix IDs that could not be applied.
    pub fix_ids: Vec<String>,
    /// Human-readable failure reason.
    pub reason: String,
}

/// Recovery guidance included in every safe-fix report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MarkdownFixRollback {
    /// Whether Assura created its own backup.
    pub backup_created: bool,
    /// Preferred rollback workflow.
    pub guidance: &'static str,
}

fn serialize_path<S>(path: &Path, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&portable_path(path))
}

fn serialize_paths<S>(paths: &[PathBuf], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    paths
        .iter()
        .map(|path| portable_path(path))
        .collect::<Vec<_>>()
        .serialize(serializer)
}

fn portable_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
