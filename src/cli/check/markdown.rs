//! Markdown-specific validators for structure-first checks.

use super::rules::{display_rel, parse_frontmatter};
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::MarkdownBundle;
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

        if let Some(required_fields) = &markdown.required_fields {
            match frontmatter {
                Some(frontmatter) => match serde_yaml::from_str::<serde_yaml::Value>(frontmatter) {
                    Ok(value) => {
                        for field in required_fields {
                            if value.get(field).is_none() {
                                push_missing_frontmatter_field(self, report, rel, field);
                            }
                        }
                    }
                    Err(error) => {
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            "markdown_frontmatter_parse",
                            format!(
                                "Markdown file '{}' has invalid frontmatter: {}",
                                display_rel(rel),
                                error
                            ),
                            "medium",
                        );
                    }
                },
                None => {
                    for field in required_fields {
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            "markdown_frontmatter_field",
                            format!(
                                "Markdown file '{}' cannot satisfy required field '{}' without frontmatter",
                                display_rel(rel),
                                field
                            ),
                            "medium",
                        );
                    }
                }
            }
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
}

struct MarkdownHeading<'a> {
    depth: usize,
    text: &'a str,
}

fn markdown_headings(content: &str) -> Vec<MarkdownHeading<'_>> {
    let mut headings = Vec::new();
    let mut in_fence = false;

    for line in content.lines() {
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
            headings.push(MarkdownHeading { depth, text });
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

fn push_missing_frontmatter_field(
    checker: &StructureChecker,
    report: &mut StructureCheckReport,
    rel: &Path,
    field: &str,
) {
    checker.push_violation(
        report,
        rel.to_path_buf(),
        "markdown_frontmatter_field",
        format!(
            "Markdown file '{}' is missing frontmatter field '{}'",
            display_rel(rel),
            field
        ),
        "medium",
    );
}
