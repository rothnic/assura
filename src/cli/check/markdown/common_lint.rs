//! Common Rust-native Markdown lint checks.

use super::{markdown_headings, markdown_severity, suppression::MarkdownSuppressions};
use crate::cli::check::rules::display_rel;
use crate::cli::check::{StructureCheckReport, StructureChecker};
use crate::config::config::MarkdownBundle;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn validate_markdown_common_lints(
    checker: &StructureChecker,
    rel: &Path,
    markdown: &MarkdownBundle,
    content: &str,
    suppressions: &mut MarkdownSuppressions,
    report: &mut StructureCheckReport,
) {
    if markdown.lint_common != Some(true) {
        return;
    }
    validate_heading_increment(checker, rel, markdown, content, suppressions, report);
    validate_heading_marker_spacing(checker, rel, markdown, content, suppressions, report);
    validate_duplicate_headings(checker, rel, markdown, content, suppressions, report);
    validate_multiple_blank_lines(checker, rel, markdown, content, suppressions, report);
}

fn validate_heading_increment(
    checker: &StructureChecker,
    rel: &Path,
    markdown: &MarkdownBundle,
    content: &str,
    suppressions: &mut MarkdownSuppressions,
    report: &mut StructureCheckReport,
) {
    let rule = "markdown_heading_increment";
    for pair in markdown_headings(content).windows(2) {
        let previous = &pair[0];
        let current = &pair[1];
        if current.depth > previous.depth + 1 && !suppressions.suppresses(rule, current.line_number)
        {
            checker.push_violation(
                report,
                rel.to_path_buf(),
                rule,
                format!(
                    "Markdown file '{}' skips from H{} '{}' to H{} '{}' on line {}",
                    display_rel(rel),
                    previous.depth,
                    previous.text,
                    current.depth,
                    current.text,
                    current.line_number
                ),
                markdown_severity(markdown, rule, "medium"),
            );
        }
    }
}

fn validate_heading_marker_spacing(
    checker: &StructureChecker,
    rel: &Path,
    markdown: &MarkdownBundle,
    content: &str,
    suppressions: &mut MarkdownSuppressions,
    report: &mut StructureCheckReport,
) {
    let rule = "markdown_heading_marker_spacing";
    for body_line in markdown_body_lines(content) {
        let MarkdownBodyLine::Content(line_index, line) = body_line else {
            continue;
        };
        let trimmed = line.trim_start();
        if line.len() - trimmed.len() > 3 {
            continue;
        }
        let depth = trimmed.chars().take_while(|ch| *ch == '#').count();
        if !(1..=6).contains(&depth) {
            continue;
        }
        let after = &trimmed[depth..];
        let invalid = after.is_empty()
            || !after.chars().next().is_some_and(char::is_whitespace)
            || after.chars().take_while(|ch| ch.is_whitespace()).count() != 1;
        let line_number = line_index + 1;
        if invalid && !suppressions.suppresses(rule, line_number) {
            checker.push_violation(
                report,
                rel.to_path_buf(),
                rule,
                format!(
                    "Markdown file '{}' has malformed heading marker spacing on line {}",
                    display_rel(rel),
                    line_number
                ),
                markdown_severity(markdown, rule, "medium"),
            );
        }
    }
}

fn validate_duplicate_headings(
    checker: &StructureChecker,
    rel: &Path,
    markdown: &MarkdownBundle,
    content: &str,
    suppressions: &mut MarkdownSuppressions,
    report: &mut StructureCheckReport,
) {
    let rule = "markdown_duplicate_heading";
    let mut first_seen = HashMap::new();
    for heading in markdown_headings(content) {
        if let Some(first_line) = first_seen.get(heading.text).copied() {
            if suppressions.suppresses(rule, heading.line_number) {
                continue;
            }
            checker.push_violation(
                report,
                rel.to_path_buf(),
                rule,
                format!(
                    "Markdown file '{}' repeats heading '{}' on line {}; first seen on line {}",
                    display_rel(rel),
                    heading.text,
                    heading.line_number,
                    first_line
                ),
                markdown_severity(markdown, rule, "medium"),
            );
        } else {
            first_seen.insert(heading.text, heading.line_number);
        }
    }
}

fn validate_multiple_blank_lines(
    checker: &StructureChecker,
    rel: &Path,
    markdown: &MarkdownBundle,
    content: &str,
    suppressions: &mut MarkdownSuppressions,
    report: &mut StructureCheckReport,
) {
    let rule = "markdown_multiple_blank_lines";
    let mut previous_blank = false;
    for body_line in markdown_body_lines(content) {
        let MarkdownBodyLine::Content(line_index, line) = body_line else {
            previous_blank = false;
            continue;
        };
        let blank = line.trim().is_empty();
        let line_number = line_index + 1;
        if blank && previous_blank && !suppressions.suppresses(rule, line_number) {
            checker.push_violation(
                report,
                rel.to_path_buf(),
                rule,
                format!(
                    "Markdown file '{}' has multiple consecutive blank lines ending on line {}",
                    display_rel(rel),
                    line_number
                ),
                markdown_severity(markdown, rule, "medium"),
            );
        }
        previous_blank = blank;
    }
}

enum MarkdownBodyLine<'a> {
    Content(usize, &'a str),
    Boundary,
}

fn markdown_body_lines(content: &str) -> Vec<MarkdownBodyLine<'_>> {
    let mut in_frontmatter = false;
    let mut frontmatter_checked = false;
    let mut in_fence = false;
    let mut body_lines = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        if line_index == 0 && matches!(line.trim(), "---" | "+++") {
            in_frontmatter = true;
            frontmatter_checked = true;
            body_lines.push(MarkdownBodyLine::Boundary);
            continue;
        }
        if in_frontmatter {
            if matches!(line.trim(), "---" | "+++") {
                in_frontmatter = false;
            }
            body_lines.push(MarkdownBodyLine::Boundary);
            continue;
        }
        if !frontmatter_checked {
            frontmatter_checked = true;
        }
        let trimmed = line.trim_start();
        if is_fence_start(trimmed) {
            in_fence = !in_fence;
            body_lines.push(MarkdownBodyLine::Boundary);
            continue;
        }
        if in_fence {
            body_lines.push(MarkdownBodyLine::Boundary);
        } else {
            body_lines.push(MarkdownBodyLine::Content(line_index, line));
        }
    }
    body_lines
}

fn is_fence_start(trimmed: &str) -> bool {
    let Some(marker) = trimmed
        .chars()
        .next()
        .filter(|marker| *marker == '`' || *marker == '~')
    else {
        return false;
    };
    trimmed.chars().take_while(|ch| *ch == marker).count() >= 3
}
