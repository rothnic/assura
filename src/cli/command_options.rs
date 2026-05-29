//! Option structs shared by full CLI command handlers.

use crate::cli::args::OutputFormat;
use std::path::PathBuf;

/// Options for a one-shot structure check command.
pub struct CheckCommandOptions {
    /// Path to validate.
    pub path: Option<PathBuf>,
    /// Explicit Assura configuration path.
    pub config: Option<PathBuf>,
    /// Report output format.
    pub format: OutputFormat,
    /// Minimum severity required for advice and status output.
    pub min_severity: Option<String>,
    /// Maximum advice and status items to render.
    pub max_issues: Option<usize>,
    /// Optional report output path.
    pub output: Option<PathBuf>,
    /// Stop after the first violation when supported.
    pub fail_fast: bool,
    /// Return success while still reporting violations.
    pub warn: bool,
    /// Match LS-Lint path-argument behavior for a single explicit target.
    pub ls_lint_target_semantics: bool,
}
