//! Public operation contracts for repo-native content mutations.

use super::model::RepositoryValidation;
use serde_json::{Map, Value};
use std::path::PathBuf;

/// Payload for creating one file-backed content object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRecordRequest {
    /// Configured collection name to create within.
    pub collection: String,
    /// Stable object identifier. If the record data omits the collection ID
    /// field, this value is inserted before validation.
    pub id: String,
    /// Project-relative destination path for the new record.
    pub path: PathBuf,
    /// Frontmatter or JSON object data to validate and write.
    pub data: Map<String, Value>,
    /// Markdown body for `markdown_frontmatter` records. Ignored for JSON.
    pub body: Option<String>,
}

/// Result of a successful create operation.
#[derive(Debug, Clone)]
pub struct CreateRecordResult {
    /// Project-relative path written by the operation.
    pub path: PathBuf,
    /// Repository validation after the new record is written.
    pub validation: RepositoryValidation,
}

/// Payload for updating one existing file-backed content object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRecordRequest {
    /// Configured collection name to update within.
    pub collection: String,
    /// Stable object identifier to update.
    pub id: String,
    /// Optional project-relative path that must match the existing record.
    pub path: Option<PathBuf>,
    /// Field-level data changes to merge into the existing record data.
    pub changes: Map<String, Value>,
    /// Validate and return proposed bytes without mutating the repository.
    pub dry_run: bool,
}

/// Deterministic dry-run preview for an update operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateRecordDryRun {
    /// Project-relative path that would be written.
    pub path: PathBuf,
    /// Full deterministic record contents that would be written.
    pub content: String,
}

/// Result of a successful update operation.
#[derive(Debug, Clone)]
pub struct UpdateRecordResult {
    /// Project-relative path targeted by the operation.
    pub path: PathBuf,
    /// Repository validation after the proposed or written update.
    pub validation: RepositoryValidation,
    /// Proposed contents when the operation was a dry run.
    pub dry_run: Option<UpdateRecordDryRun>,
}
