//! Markdown-specific validators for structure-first checks.

mod common_lint;
mod links;
mod outline;
#[cfg(feature = "json-output")]
mod rumdl_adapter;
pub(super) mod suppression;

use super::rules::{display_rel, parse_frontmatter};
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::MarkdownBundle;
use common_lint::validate_markdown_common_lints;
use outline::validate_markdown_outline;
#[cfg(feature = "json-output")]
use rumdl_adapter::validate_rumdl_markdownlint_candidate;
use std::collections::HashSet;
use std::path::Path;
use suppression::MarkdownSuppressions;

impl StructureChecker {
    pub(super) fn validate_markdown(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        let frontmatter = parse_frontmatter(content);
        let mut suppressions = MarkdownSuppressions::parse(content);

        self.validate_markdown_suppression_comments(rel, &suppressions, report);
        self.validate_markdown_frontmatter(rel, markdown, frontmatter, &mut suppressions, report);
        self.validate_markdown_heading_depth(rel, markdown, content, &mut suppressions, report);
        self.validate_markdown_required_sections(rel, markdown, content, &mut suppressions, report);
        validate_markdown_outline(self, rel, markdown, content, &mut suppressions, report);
        self.validate_markdown_trailing_spaces(rel, markdown, content, &mut suppressions, report);
        validate_markdown_common_lints(self, rel, markdown, content, &mut suppressions, report);
        #[cfg(feature = "json-output")]
        validate_rumdl_markdownlint_candidate(
            self,
            rel,
            markdown,
            content,
            &mut suppressions,
            report,
        );
        #[cfg(not(feature = "json-output"))]
        self.validate_markdownlint_candidate_without_json(rel, markdown, report);
        self.validate_markdown_links(rel, markdown, content, &mut suppressions, report);
    }

    #[cfg(not(feature = "json-output"))]
    fn validate_markdownlint_candidate_without_json(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        report: &mut StructureCheckReport,
    ) {
        let Some(candidate) = &markdown.markdownlint_candidate else {
            return;
        };
        if candidate.enabled != Some(true) {
            return;
        }
        self.push_violation(
            report,
            rel.to_path_buf(),
            "markdown_engine",
            format!(
                "Markdown file '{}' enables markdownlint_candidate, but this Assura build cannot parse candidate JSON output",
                display_rel(rel)
            ),
            markdown_severity(markdown, "markdown_engine", "medium"),
        );
    }

    fn validate_markdown_suppression_comments(
        &self,
        rel: &Path,
        suppressions: &MarkdownSuppressions,
        report: &mut StructureCheckReport,
    ) {
        for invalid in suppressions.invalid() {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "markdown_suppression",
                format!(
                    "Markdown file '{}' has invalid assura-ignore suppression on line {}: {}",
                    display_rel(rel),
                    invalid.line_number,
                    invalid.reason
                ),
                "medium",
            );
        }
    }

    fn validate_markdown_frontmatter(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        frontmatter: Option<&str>,
        suppressions: &mut MarkdownSuppressions,
        report: &mut StructureCheckReport,
    ) {
        let rule = "markdown_frontmatter";
        if markdown.require_frontmatter == Some(true)
            && frontmatter.is_none()
            && !suppressions.suppresses(rule, 1)
        {
            self.push_violation(
                report,
                rel.to_path_buf(),
                rule,
                format!(
                    "Markdown file '{}' is missing YAML frontmatter",
                    display_rel(rel)
                ),
                markdown_severity(markdown, rule, "medium"),
            );
        }
    }

    fn validate_markdown_heading_depth(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        suppressions: &mut MarkdownSuppressions,
        report: &mut StructureCheckReport,
    ) {
        let rule = "markdown_heading_depth";
        if let Some(max_depth) = markdown.max_heading_depth {
            for heading in markdown_headings(content) {
                if heading.depth > usize::from(max_depth) {
                    if suppressions.suppresses(rule, heading.line_number) {
                        continue;
                    }
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        rule,
                        format!(
                            "Markdown file '{}' has heading depth {}, exceeding limit {}",
                            display_rel(rel),
                            heading.depth,
                            max_depth
                        ),
                        markdown_severity(markdown, rule, "medium"),
                    );
                    break;
                }
            }
        }
    }

    fn validate_markdown_required_sections(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        suppressions: &mut MarkdownSuppressions,
        report: &mut StructureCheckReport,
    ) {
        let rule = "markdown_required_section";
        if let Some(required_sections) = &markdown.required_sections {
            let finding_line = content.lines().count().saturating_add(1);
            let mut headings = HashSet::new();
            for heading in markdown_headings(content) {
                headings.insert(heading.text);
            }

            for section in required_sections {
                if !headings.contains(section.as_str()) {
                    if suppressions.suppresses(rule, finding_line) {
                        continue;
                    }
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        rule,
                        format!(
                            "Markdown file '{}' is missing required section '{}'",
                            display_rel(rel),
                            section
                        ),
                        markdown_severity(markdown, rule, "medium"),
                    );
                }
            }
        }
    }

    fn validate_markdown_trailing_spaces(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        suppressions: &mut MarkdownSuppressions,
        report: &mut StructureCheckReport,
    ) {
        if markdown.lint_trailing_spaces != Some(true) {
            return;
        }
        let rule = "markdown_trailing_spaces";

        for violation in blank_line_trailing_spaces(content) {
            if suppressions.suppresses(rule, violation.line_number) {
                continue;
            }
            self.push_violation(
                report,
                rel.to_path_buf(),
                rule,
                format!(
                    "Markdown file '{}' has {} trailing whitespace character(s) on blank line {}, column 1",
                    display_rel(rel),
                    violation.trailing_count,
                    violation.line_number
                ),
                markdown_severity(markdown, rule, "low"),
            );
        }
    }
}

pub(super) fn markdown_severity<'a>(
    markdown: &'a MarkdownBundle,
    rule: &str,
    default: &'a str,
) -> &'a str {
    markdown
        .rules
        .as_ref()
        .and_then(|rules| rules.get(rule))
        .and_then(|config| config.severity.as_ref())
        .map(String::as_str)
        .unwrap_or(default)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct MarkdownTrailingSpaces {
    pub(super) line_number: usize,
    pub(super) trailing_count: usize,
}

pub(super) fn blank_line_trailing_spaces(content: &str) -> Vec<MarkdownTrailingSpaces> {
    content
        .lines()
        .enumerate()
        .filter_map(|(line_index, line)| {
            let trailing_count = line.chars().filter(|ch| *ch == ' ' || *ch == '\t').count();
            (trailing_count > 0 && line.chars().all(|ch| ch == ' ' || ch == '\t')).then_some(
                MarkdownTrailingSpaces {
                    line_number: line_index + 1,
                    trailing_count,
                },
            )
        })
        .collect()
}

pub(super) fn fix_blank_line_trailing_spaces(content: &str) -> (String, usize) {
    let mut output = String::with_capacity(content.len());
    let mut fixes = 0;

    for line in content.split_inclusive('\n') {
        let (body, newline) = if let Some(body) = line.strip_suffix("\r\n") {
            (body, "\r\n")
        } else if let Some(body) = line.strip_suffix('\n') {
            (body, "\n")
        } else {
            (line, "")
        };

        if !body.is_empty() && body.chars().all(|ch| ch == ' ' || ch == '\t') {
            output.push_str(newline);
            fixes += 1;
        } else {
            output.push_str(line);
        }
    }

    (output, fixes)
}

pub(super) struct MarkdownHeading<'a> {
    pub(super) depth: usize,
    pub(super) text: &'a str,
    pub(super) line_number: usize,
}

pub(super) fn markdown_headings(content: &str) -> Vec<MarkdownHeading<'_>> {
    let mut headings = Vec::new();
    let mut in_frontmatter = false;
    let mut frontmatter_checked = false;
    let mut in_fence = false;

    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if line_index == 0 && matches!(trimmed.trim(), "---" | "+++") {
            in_frontmatter = true;
            frontmatter_checked = true;
            continue;
        }

        if in_frontmatter {
            if matches!(trimmed.trim(), "---" | "+++") {
                in_frontmatter = false;
            }
            continue;
        }

        if !frontmatter_checked {
            frontmatter_checked = true;
        }

        if is_fence_start(trimmed) {
            in_fence = !in_fence;
            continue;
        }

        if in_fence {
            continue;
        }

        let indent = line.len() - trimmed.len();
        if indent > 3 {
            continue;
        }

        let depth = trimmed.chars().take_while(|ch| *ch == '#').count();
        if !(1..=6).contains(&depth) {
            continue;
        }

        let after_marks = &trimmed[depth..];
        if !after_marks.is_empty()
            && !after_marks
                .chars()
                .next()
                .is_some_and(|ch| ch.is_whitespace())
        {
            continue;
        }

        let text = after_marks.trim().trim_end_matches('#').trim_end().trim();
        if !text.is_empty() {
            headings.push(MarkdownHeading {
                depth,
                text,
                line_number: line_index + 1,
            });
        }
    }

    headings
}

fn is_fence_start(trimmed: &str) -> bool {
    let marker = trimmed
        .chars()
        .next()
        .filter(|marker| *marker == '`' || *marker == '~');
    let Some(marker) = marker else {
        return false;
    };

    trimmed.chars().take_while(|ch| *ch == marker).count() >= 3
}
