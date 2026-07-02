// Portable bundle types for compiled structure config artifacts.

use crate::config::config::{
    MarkdownOutlineEntry, MarkdownRuleConfig, MarkdownlintCandidateConfig,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Portable Markdown rule bundle inside compiled config artifacts.
pub(super) struct PortableMarkdownBundle {
    /// Whether Markdown frontmatter is required.
    pub(super) require_frontmatter: Option<bool>,
    /// Maximum allowed Markdown heading depth.
    pub(super) max_heading_depth: Option<u8>,
    /// Whether Markdown links are checked.
    pub(super) check_links: Option<bool>,
    /// Required Markdown section headings.
    pub(super) required_sections: Option<Vec<String>>,
    /// Structured Markdown outline policy.
    pub(super) outline: Option<Vec<MarkdownOutlineEntry>>,
    /// Whether to lint blank-line trailing spaces.
    pub(super) lint_trailing_spaces: Option<bool>,
    /// Whether to run common Markdown lint checks.
    pub(super) lint_common: Option<bool>,
    /// Optional markdownlint-compatible candidate engine settings.
    pub(super) markdownlint_candidate: Option<MarkdownlintCandidateConfig>,
    /// Per-rule configuration for Markdown findings.
    pub(super) rules: Option<std::collections::HashMap<String, MarkdownRuleConfig>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Portable required file and directory declarations.
pub(super) struct PortableExistsValidation {
    /// Required file names or patterns.
    pub(super) files: Option<Vec<String>>,
    /// Required directory names or patterns.
    pub(super) directories: Option<Vec<String>>,
}
