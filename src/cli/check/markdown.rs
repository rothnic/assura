//! Markdown-specific validators for structure-first checks.

mod links;
mod outline;

use super::rules::{display_rel, parse_frontmatter};
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::MarkdownBundle;
use outline::validate_markdown_outline;
use std::collections::HashSet;
use std::path::Path;

impl StructureChecker {
    pub(super) fn validate_markdown(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        let frontmatter = parse_frontmatter(content);

        self.validate_markdown_frontmatter(rel, markdown, frontmatter, report);
        self.validate_markdown_heading_depth(rel, markdown, content, report);
        self.validate_markdown_required_sections(rel, markdown, content, report);
        validate_markdown_outline(self, rel, markdown, content, report);
        self.validate_markdown_trailing_spaces(rel, markdown, content, report);
        self.validate_markdown_links(rel, markdown, content, report);
    }

    fn validate_markdown_frontmatter(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        frontmatter: Option<&str>,
        report: &mut StructureCheckReport,
    ) {
        if markdown.require_frontmatter == Some(true) && frontmatter.is_none() {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "markdown_frontmatter",
                format!(
                    "Markdown file '{}' is missing YAML frontmatter",
                    display_rel(rel)
                ),
                "medium",
            );
        }
    }

    fn validate_markdown_heading_depth(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        if let Some(max_depth) = markdown.max_heading_depth {
            for heading in markdown_headings(content) {
                if heading.depth > usize::from(max_depth) {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "markdown_heading_depth",
                        format!(
                            "Markdown file '{}' has heading depth {}, exceeding limit {}",
                            display_rel(rel),
                            heading.depth,
                            max_depth
                        ),
                        "medium",
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
        report: &mut StructureCheckReport,
    ) {
        if let Some(required_sections) = &markdown.required_sections {
            let mut headings = HashSet::new();
            for heading in markdown_headings(content) {
                headings.insert(heading.text);
            }

            for section in required_sections {
                if !headings.contains(section.as_str()) {
                    self.push_violation(
                        report,
                        rel.to_path_buf(),
                        "markdown_required_section",
                        format!(
                            "Markdown file '{}' is missing required section '{}'",
                            display_rel(rel),
                            section
                        ),
                        "medium",
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
        report: &mut StructureCheckReport,
    ) {
        if markdown.lint_trailing_spaces != Some(true) {
            return;
        }

        for violation in blank_line_trailing_spaces(content) {
            self.push_violation(
                report,
                rel.to_path_buf(),
                "markdown_trailing_spaces",
                format!(
                    "Markdown file '{}' has {} trailing whitespace character(s) on blank line {}, column 1",
                    display_rel(rel),
                    violation.trailing_count,
                    violation.line_number
                ),
                "low",
            );
        }
    }
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
    let mut in_fence = false;

    for (line_index, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
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
