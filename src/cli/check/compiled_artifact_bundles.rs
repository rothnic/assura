// Portable bundle types for compiled structure config artifacts.

use crate::config::config::MarkdownOutlineEntry;

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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
/// Portable required file and directory declarations.
pub(super) struct PortableExistsValidation {
    /// Required file names or patterns.
    pub(super) files: Option<Vec<String>>,
    /// Required directory names or patterns.
    pub(super) directories: Option<Vec<String>>,
}
