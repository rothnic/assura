//! Compact effective-rule summaries for path explanation output.

use super::patterns::best_file_pattern_match;
use super::rules::EffectiveRules;
use crate::config::config::MarkdownRuleConfig;
use glob::Pattern;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// File-pattern attributes grouped under one configured matcher.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PathExplainFilePatternRule {
    /// Normalized file pattern.
    pub pattern: String,
    /// Naming convention for this pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub naming: Option<String>,
    /// Maximum lines for this pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<usize>,
    /// Maximum file size for this pattern.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_size: Option<String>,
    /// Effective severity for this matcher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
    /// Project-owned repair guidance for this matcher.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Whether this winning matcher is checked for the requested file.
    pub status: &'static str,
}

#[cfg(feature = "full-cli")]
impl PathExplainFilePatternRule {
    pub(crate) fn render_compact(&self) -> String {
        let mut attributes = Vec::new();
        if let Some(naming) = &self.naming {
            attributes.push(format!("naming={naming}"));
        }
        if let Some(max_lines) = self.max_lines {
            attributes.push(format!("max_lines={max_lines}"));
        }
        if let Some(max_size) = &self.max_size {
            attributes.push(format!("max_size={max_size}"));
        }
        if let Some(severity) = &self.severity {
            attributes.push(format!("severity={severity}"));
        }
        if let Some(message) = &self.message {
            attributes.push(format!("message={message}"));
        }
        attributes.push(format!("status={}", self.status));
        format!("{}[{}]", self.pattern, attributes.join(","))
    }
}

/// Compact summary of effective rules.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PathExplainRules {
    /// Directory-wide file naming default.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_naming: Option<String>,
    /// Directory-wide file line limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_max_lines: Option<usize>,
    /// Directory-wide file size limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_max_size: Option<String>,
    /// Pattern-scoped file directives in the effective scope.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_patterns: Vec<PathExplainFilePatternRule>,
    /// Direct file count constraints.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_exists: Vec<String>,
    /// Allowed direct file names.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_allowed_names: Vec<String>,
    /// Allowed direct file patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_allowed_patterns: Vec<String>,
    /// Forbidden direct file patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_forbidden_patterns: Vec<String>,
    /// Whether unlisted direct files are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_allow_extra: Option<bool>,
    /// Effective file-rule severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_severity: Option<String>,
    /// Direct directory naming rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_naming: Option<String>,
    /// Direct directory count constraints.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directory_exists: Vec<String>,
    /// Allowed direct directory names.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directory_allowed_names: Vec<String>,
    /// Allowed direct directory patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directory_allowed_patterns: Vec<String>,
    /// Forbidden direct directory patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directory_forbidden_patterns: Vec<String>,
    /// Whether unlisted direct directories are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_allow_extra: Option<bool>,
    /// Effective direct-directory severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_severity: Option<String>,
    /// Effective configured-directory severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_directory_severity: Option<String>,
    /// Whether Markdown frontmatter is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_require_frontmatter: Option<bool>,
    /// Required Markdown sections.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub markdown_required_sections: Vec<String>,
    /// Whether trailing-space lint is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_lint_trailing_spaces: Option<bool>,
    /// Effective Markdown rule severities.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub markdown_rule_severities: Vec<String>,
    /// Maximum allowed direct children in this directory.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_children_max: Option<usize>,
    /// Severity for the direct-child limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_children_severity: Option<String>,
    /// Repair guidance for the direct-child limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_children_message: Option<String>,
}

pub(super) fn summarize_effective_rules(rules: &EffectiveRules) -> PathExplainRules {
    let mut summary = PathExplainRules::default();
    if let Some(files) = rules.files.as_ref() {
        summary.file_naming = files.naming.clone();
        summary.file_max_lines = files.max_lines;
        summary.file_max_size = files.max_size.clone();
        summary.file_patterns = all_file_patterns(files);
        summary.file_exists = files
            .exists
            .as_ref()
            .map(sorted_count_entries)
            .unwrap_or_default();
        summary.file_allowed_names = files.allowed_names.clone().unwrap_or_default();
        summary.file_allowed_patterns = files.allowed_patterns.clone().unwrap_or_default();
        summary.file_forbidden_patterns = files.forbidden_patterns.clone().unwrap_or_default();
        summary.file_allow_extra = files.allow_extra;
        summary.file_severity = Some(files.severity.clone().unwrap_or_else(|| "medium".into()));
    }
    if let Some(directories) = rules.directories.as_ref() {
        summary.directory_naming = directories.naming.clone();
        summary.directory_exists = directories
            .exists
            .as_ref()
            .map(sorted_count_entries)
            .unwrap_or_default();
        summary.directory_allowed_names = directories.allowed_names.clone().unwrap_or_default();
        summary.directory_allowed_patterns =
            directories.allowed_patterns.clone().unwrap_or_default();
        summary.directory_forbidden_patterns =
            directories.forbidden_patterns.clone().unwrap_or_default();
        summary.directory_allow_extra = directories.allow_extra;
        summary.directory_severity = Some(
            directories
                .severity
                .clone()
                .unwrap_or_else(|| "medium".into()),
        );
    }
    if let Some(directory) = rules.self_directory.as_ref() {
        summary.self_directory_severity = Some(
            directory
                .severity
                .clone()
                .unwrap_or_else(|| "medium".into()),
        );
    }
    if let Some(markdown) = rules.markdown.as_ref() {
        summary.markdown_require_frontmatter = markdown.require_frontmatter;
        summary.markdown_required_sections = markdown.required_sections.clone().unwrap_or_default();
        summary.markdown_lint_trailing_spaces = markdown.lint_trailing_spaces;
        summary.markdown_rule_severities = markdown
            .rules
            .as_ref()
            .map(markdown_rule_severities)
            .unwrap_or_default();
    }
    if let Some(limit) = rules.limit_children.as_ref() {
        summary.limit_children_max = limit.max;
        summary.limit_children_severity = Some(
            limit
                .severity
                .as_ref()
                .map(|severity| format!("{severity:?}").to_lowercase())
                .unwrap_or_else(|| "medium".to_string()),
        );
        summary.limit_children_message = limit.message.clone();
    }
    summary
}

pub(super) fn matched_file_pattern_rules(
    rules: &EffectiveRules,
    filename: &str,
    rel: &std::path::Path,
    compiled: &HashMap<String, Pattern>,
) -> Vec<PathExplainFilePatternRule> {
    let Some(files) = rules.files.as_ref() else {
        return Vec::new();
    };
    let mut matched = BTreeMap::<String, PathExplainFilePatternRule>::new();
    if let Some((pattern, naming)) = files
        .naming_patterns
        .as_ref()
        .and_then(|patterns| best_file_pattern_match(patterns, filename, rel, compiled))
    {
        matched.entry(pattern.into()).or_default().pattern = pattern.into();
        matched.get_mut(pattern).unwrap().naming = Some(naming.clone());
    }
    if let Some((pattern, max_lines)) = files
        .max_lines_patterns
        .as_ref()
        .and_then(|patterns| best_file_pattern_match(patterns, filename, rel, compiled))
    {
        matched.entry(pattern.into()).or_default().pattern = pattern.into();
        matched.get_mut(pattern).unwrap().max_lines = Some(*max_lines);
    }
    if let Some((pattern, max_size)) = files
        .max_size_patterns
        .as_ref()
        .and_then(|patterns| best_file_pattern_match(patterns, filename, rel, compiled))
    {
        matched.entry(pattern.into()).or_default().pattern = pattern.into();
        matched.get_mut(pattern).unwrap().max_size = Some(max_size.clone());
    }
    if let Some((pattern, severity)) = files
        .severity_patterns
        .as_ref()
        .and_then(|patterns| best_file_pattern_match(patterns, filename, rel, compiled))
    {
        matched.entry(pattern.into()).or_default().pattern = pattern.into();
        matched.get_mut(pattern).unwrap().severity = Some(severity.clone());
    }
    if let Some((pattern, message)) = files
        .message_patterns
        .as_ref()
        .and_then(|patterns| best_file_pattern_match(patterns, filename, rel, compiled))
    {
        matched.entry(pattern.into()).or_default().pattern = pattern.into();
        matched.get_mut(pattern).unwrap().message = Some(message.clone());
    }
    for rule in matched.values_mut() {
        rule.status = "checked";
    }
    matched.into_values().collect()
}

fn all_file_patterns(files: &crate::config::config::FileBundle) -> Vec<PathExplainFilePatternRule> {
    let mut patterns = BTreeSet::new();
    patterns.extend(
        files
            .naming_patterns
            .iter()
            .flat_map(|map| map.keys().cloned()),
    );
    patterns.extend(
        files
            .max_lines_patterns
            .iter()
            .flat_map(|map| map.keys().cloned()),
    );
    patterns.extend(
        files
            .max_size_patterns
            .iter()
            .flat_map(|map| map.keys().cloned()),
    );
    patterns
        .into_iter()
        .map(|pattern| PathExplainFilePatternRule {
            naming: files
                .naming_patterns
                .as_ref()
                .and_then(|map| map.get(&pattern))
                .cloned(),
            max_lines: files
                .max_lines_patterns
                .as_ref()
                .and_then(|map| map.get(&pattern))
                .copied(),
            max_size: files
                .max_size_patterns
                .as_ref()
                .and_then(|map| map.get(&pattern))
                .cloned(),
            severity: files
                .severity_patterns
                .as_ref()
                .and_then(|map| map.get(&pattern))
                .cloned(),
            message: files
                .message_patterns
                .as_ref()
                .and_then(|map| map.get(&pattern))
                .cloned(),
            status: "configured",
            pattern,
        })
        .collect()
}

fn markdown_rule_severities(rules: &HashMap<String, MarkdownRuleConfig>) -> Vec<String> {
    let mut entries = rules
        .iter()
        .filter_map(|(rule, config)| {
            config
                .severity
                .as_ref()
                .map(|severity| format!("{rule}:{severity}"))
        })
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn sorted_count_entries(map: &HashMap<String, String>) -> Vec<String> {
    let mut entries = map
        .iter()
        .map(|(pattern, count)| format!("{pattern}:{count}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}
