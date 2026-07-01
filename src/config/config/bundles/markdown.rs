use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
#[cfg(feature = "full-cli")]
use validator::Validate;

/// Bundle of markdown validations for a directory node.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "full-cli", derive(Validate))]
#[serde(rename_all = "snake_case")]
pub struct MarkdownBundle {
    /// Whether frontmatter is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_frontmatter: Option<bool>,

    /// Legacy typed frontmatter field requirement.
    ///
    /// Assura-authored config rejects this field during semantic validation.
    /// Keep it deserializable so users get a migration diagnostic pointing to
    /// content runtime models and collections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_fields: Option<Vec<String>>,

    /// Maximum heading level depth.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(feature = "full-cli", validate(range(min = 1, max = 6)))]
    pub max_heading_depth: Option<u8>,

    /// Whether to check for dead links.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_links: Option<bool>,

    /// Required sections in markdown files.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_sections: Option<Vec<String>>,

    /// Ordered Markdown heading outline.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outline: Option<Vec<MarkdownOutlineEntry>>,

    /// Whether to report blank Markdown lines that contain trailing spaces or tabs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lint_trailing_spaces: Option<bool>,

    /// Per-rule configuration for Markdown findings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<HashMap<String, MarkdownRuleConfig>>,
}

/// Configuration for one Markdown rule.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct MarkdownRuleConfig {
    /// Optional severity override for this rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}

impl MarkdownRuleConfig {
    /// Merge a child rule object over an inherited parent rule object.
    pub(crate) fn merged_with(&self, child: &Self) -> Self {
        Self {
            severity: child.severity.clone().or_else(|| self.severity.clone()),
        }
    }
}

pub(crate) fn merge_markdown_rule_configs(
    parent: Option<&HashMap<String, MarkdownRuleConfig>>,
    child: Option<&HashMap<String, MarkdownRuleConfig>>,
) -> Option<HashMap<String, MarkdownRuleConfig>> {
    match (parent, child) {
        (None, None) => None,
        (Some(parent), None) => Some(parent.clone()),
        (None, Some(child)) => Some(child.clone()),
        (Some(parent), Some(child)) => {
            let mut merged = parent.clone();
            for (rule, child_config) in child {
                let config = merged.get(rule).map_or_else(
                    || child_config.clone(),
                    |parent| parent.merged_with(child_config),
                );
                merged.insert(rule.clone(), config);
            }
            Some(merged)
        }
    }
}

/// Markdown outline entry accepted by config shorthand and object notation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MarkdownOutlineEntry {
    /// Plain shorthand heading text.
    Text(String),
    /// Expanded object node.
    Node(MarkdownOutlineNode),
    /// Shorthand parent node keyed by title with nested children.
    Parent(BTreeMap<String, Vec<MarkdownOutlineEntry>>),
}

/// Expanded Markdown outline node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MarkdownOutlineNode {
    /// Heading title to match.
    pub title: String,
    /// Whether the heading is optional.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optional: Option<bool>,
    /// Optional match mode. `exact` and `regex` are currently supported.
    #[serde(rename = "match", skip_serializing_if = "Option::is_none")]
    pub match_mode: Option<String>,
    /// Future validators carried by the config surface.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate: Option<Vec<String>>,
    /// Nested outline entries.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<MarkdownOutlineEntry>,
}

/// Borrowed normalized view of a Markdown outline entry.
#[derive(Clone, Copy)]
pub(crate) struct MarkdownOutlineView<'a> {
    /// Heading title after shorthand normalization.
    pub title: &'a str,
    /// Whether this entry is optional.
    pub optional: bool,
    /// Match mode for the title.
    pub match_mode: &'a str,
    /// Nested child entries.
    pub children: &'a [MarkdownOutlineEntry],
}

impl MarkdownOutlineEntry {
    /// Normalize shorthand and object forms into a common borrowed view.
    pub(crate) fn view(&self, context: &str) -> Result<MarkdownOutlineView<'_>, String> {
        match self {
            Self::Text(text) => Ok(view_from_shorthand(text, &[])),
            Self::Node(node) => Ok(MarkdownOutlineView {
                title: node.title.as_str(),
                optional: node.optional.unwrap_or(false),
                match_mode: node.match_mode.as_deref().unwrap_or("exact"),
                children: &node.children,
            }),
            Self::Parent(parent) => {
                if parent.len() != 1 {
                    return Err(format!(
                        "{context}: shorthand parent outline nodes must contain exactly one heading key"
                    ));
                }
                let (title, children) = parent.iter().next().expect("checked non-empty");
                Ok(view_from_shorthand(title, children))
            }
        }
    }
}

impl MarkdownBundle {
    /// Create a new empty bundle.
    pub fn new() -> Self {
        Self {
            require_frontmatter: None,
            required_fields: None,
            max_heading_depth: None,
            check_links: None,
            required_sections: None,
            outline: None,
            lint_trailing_spaces: None,
            rules: None,
        }
    }

    /// Set frontmatter requirement.
    pub fn with_require_frontmatter(mut self, require: bool) -> Self {
        self.require_frontmatter = Some(require);
        self
    }

    /// Set legacy typed frontmatter fields.
    ///
    /// Configs using this value are rejected during semantic validation.
    pub fn with_required_fields(mut self, fields: Vec<String>) -> Self {
        self.required_fields = Some(fields);
        self
    }

    /// Set maximum heading depth.
    pub fn with_max_heading_depth(mut self, depth: u8) -> Self {
        self.max_heading_depth = Some(depth);
        self
    }

    /// Set link checking.
    pub fn with_check_links(mut self, check: bool) -> Self {
        self.check_links = Some(check);
        self
    }

    /// Set required sections.
    pub fn with_required_sections(mut self, sections: Vec<String>) -> Self {
        self.required_sections = Some(sections);
        self
    }

    /// Set required outline.
    pub fn with_outline(mut self, outline: Vec<MarkdownOutlineEntry>) -> Self {
        self.outline = Some(outline);
        self
    }

    /// Set blank-line trailing whitespace linting.
    pub fn with_lint_trailing_spaces(mut self, lint: bool) -> Self {
        self.lint_trailing_spaces = Some(lint);
        self
    }

    /// Set per-rule Markdown configuration.
    pub fn with_rules(mut self, rules: HashMap<String, MarkdownRuleConfig>) -> Self {
        self.rules = Some(rules);
        self
    }

    pub(crate) fn validate_outline_semantics(&self, context: &str) -> Result<(), String> {
        if let Some(outline) = &self.outline {
            validate_outline_entries(outline, &format!("{context}.outline"))?;
        }
        Ok(())
    }

    pub(crate) fn validate_required_sections_semantics(&self, context: &str) -> Result<(), String> {
        let Some(required_sections) = &self.required_sections else {
            return Ok(());
        };
        let mut seen = std::collections::HashSet::new();
        for (index, section) in required_sections.iter().enumerate() {
            if section.trim().is_empty() {
                return Err(format!(
                    "{context}.required_sections[{index}]: heading text cannot be empty"
                ));
            }
            if section.trim() != section || section.contains('\n') || section.contains('\r') {
                return Err(format!(
                    "{context}.required_sections[{index}]: heading text must be a single trimmed line"
                ));
            }
            if section.starts_with('#') || section.ends_with('#') {
                return Err(format!(
                    "{context}.required_sections[{index}]: heading text must not include Markdown heading markers"
                ));
            }
            if !seen.insert(section.as_str()) {
                return Err(format!(
                    "{context}.required_sections[{index}]: duplicate heading text '{section}'"
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn validate_rule_config_semantics(&self, context: &str) -> Result<(), String> {
        let Some(rules) = &self.rules else {
            return Ok(());
        };
        for (rule, config) in rules {
            validate_markdown_rule_id(rule)
                .map_err(|error| format!("{context}.rules.{rule}: {error}"))?;
            if let Some(severity) = &config.severity {
                validate_markdown_severity(severity)
                    .map_err(|error| format!("{context}.rules.{rule}.severity: {error}"))?;
            }
        }
        Ok(())
    }
}

impl Default for MarkdownBundle {
    fn default() -> Self {
        Self::new()
    }
}

fn view_from_shorthand<'a>(
    title: &'a str,
    children: &'a [MarkdownOutlineEntry],
) -> MarkdownOutlineView<'a> {
    if let Some(required_title) = title.strip_prefix("?? ") {
        MarkdownOutlineView {
            title: required_title,
            optional: true,
            match_mode: "exact",
            children,
        }
    } else {
        MarkdownOutlineView {
            title,
            optional: false,
            match_mode: "exact",
            children,
        }
    }
}

fn validate_outline_entries(entries: &[MarkdownOutlineEntry], context: &str) -> Result<(), String> {
    for (index, entry) in entries.iter().enumerate() {
        let entry_context = format!("{context}[{index}]");
        let view = entry.view(&entry_context)?;
        if view.title.trim().is_empty() {
            return Err(format!(
                "{entry_context}: markdown outline title cannot be empty"
            ));
        }
        match view.match_mode {
            "exact" | "regex" => {}
            mode => {
                return Err(format!(
                    "{entry_context}.match: unsupported markdown outline match mode '{mode}'"
                ));
            }
        }
        validate_outline_entries(view.children, &format!("{entry_context}.children"))?;
    }
    Ok(())
}

fn validate_markdown_rule_id(value: &str) -> Result<(), String> {
    match value {
        "markdown_frontmatter"
        | "markdown_heading_depth"
        | "markdown_required_section"
        | "markdown_outline"
        | "markdown_trailing_spaces"
        | "markdown_link_format"
        | "markdown_link_target"
        | "markdown_link_heading_anchor"
        | "markdown_link_line_anchor" => Ok(()),
        _ => Err("expected a supported markdown_* rule id".to_string()),
    }
}

fn validate_markdown_severity(value: &str) -> Result<(), String> {
    match value {
        "critical" | "high" | "medium" | "low" => Ok(()),
        _ => Err("expected one of critical, high, medium, or low".to_string()),
    }
}
