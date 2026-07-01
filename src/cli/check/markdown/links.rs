//! Markdown-authored local link validation.

use super::{markdown_headings, markdown_severity, suppression::MarkdownSuppressions};
use crate::cli::check::rules::display_rel;
use crate::cli::check::{StructureCheckReport, StructureChecker};
use crate::config::config::MarkdownBundle;
use crate::markdown_links::{
    is_markdown_file, markdown_bare_references, markdown_links, parse_line_anchor,
    parse_markdown_link_target,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;

impl StructureChecker {
    pub(super) fn validate_markdown_links(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        suppressions: &mut MarkdownSuppressions,
        report: &mut StructureCheckReport,
    ) {
        if markdown.check_links != Some(true) {
            return;
        }

        for link in markdown_links(content) {
            let Some(target) = parse_markdown_link_target(rel, &link.target) else {
                let rule = "markdown_link_format";
                if suppressions.suppresses(rule, link.line_number) {
                    continue;
                }
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    rule,
                    format!(
                        "Markdown file '{}' has non-relative internal link '{}' on line {}; use a relative Markdown link",
                        display_rel(rel),
                        link.target,
                        link.line_number
                    ),
                    markdown_severity(markdown, rule, "medium"),
                );
                continue;
            };
            let target_path = self.project_root.join(&target.path);
            if !target_path.is_file() {
                let rule = "markdown_link_target";
                if suppressions.suppresses(rule, link.line_number) {
                    continue;
                }
                self.push_violation(
                    report,
                    rel.to_path_buf(),
                    rule,
                    format!(
                        "Markdown file '{}' links to missing local target '{}' on line {}",
                        display_rel(rel),
                        display_rel(&target.path),
                        link.line_number
                    ),
                    markdown_severity(markdown, rule, "medium"),
                );
                continue;
            }
            let Ok(target_content) = std::fs::read_to_string(&target_path) else {
                continue;
            };
            if let Some(anchor) = target.anchor.as_deref() {
                if let Some((start, end)) = parse_line_anchor(anchor) {
                    let line_count = target_content.lines().count();
                    if start == 0 || start > line_count || end < start || end > line_count {
                        let rule = "markdown_link_line_anchor";
                        if suppressions.suppresses(rule, link.line_number) {
                            continue;
                        }
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            rule,
                            format!(
                                "Markdown file '{}' links to invalid line anchor '#{}' in '{}' on line {}; target has {} line(s)",
                                display_rel(rel),
                                anchor,
                                display_rel(&target.path),
                                link.line_number,
                                line_count
                            ),
                            markdown_severity(markdown, rule, "medium"),
                        );
                    }
                } else if is_markdown_file(&target.path) {
                    let slugs = github_heading_slugs(&target_content);
                    if !slugs.contains(anchor) {
                        let rule = "markdown_link_heading_anchor";
                        if suppressions.suppresses(rule, link.line_number) {
                            continue;
                        }
                        self.push_violation(
                            report,
                            rel.to_path_buf(),
                            rule,
                            format!(
                                "Markdown file '{}' links to missing heading anchor '#{}' in '{}' on line {}",
                                display_rel(rel),
                                anchor,
                                display_rel(&target.path),
                                link.line_number
                            ),
                            markdown_severity(markdown, rule, "medium"),
                        );
                    }
                }
            }
        }

        self.validate_markdown_bare_references(rel, markdown, content, suppressions, report);
    }

    fn validate_markdown_bare_references(
        &self,
        rel: &Path,
        markdown: &MarkdownBundle,
        content: &str,
        suppressions: &mut MarkdownSuppressions,
        report: &mut StructureCheckReport,
    ) {
        let rule = "markdown_link_format";
        for reference in markdown_bare_references(rel, &self.project_root, content) {
            if suppressions.suppresses(rule, reference.line_number) {
                continue;
            }
            self.push_violation(
                report,
                rel.to_path_buf(),
                rule,
                format!(
                    "Markdown file '{}' has unrendered local reference '{}' to '{}' on line {}, column {}; use a relative Markdown link",
                    display_rel(rel),
                    reference.text,
                    display_rel(&reference.target_path),
                    reference.line_number,
                    reference.column_number
                ),
                markdown_severity(markdown, rule, "medium"),
            );
        }
    }
}

fn github_heading_slugs(content: &str) -> HashSet<String> {
    let mut counts = HashMap::<String, usize>::new();
    let mut slugs = HashSet::new();
    for heading in markdown_headings(content) {
        let base = github_heading_slug(heading.text);
        let count = counts.entry(base.clone()).or_insert(0);
        let slug = if *count == 0 {
            base
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        slugs.insert(slug);
    }
    slugs
}

fn github_heading_slug(text: &str) -> String {
    let text = rendered_heading_text(text);
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in text.to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            slug.push(ch);
            previous_dash = false;
        } else if (ch.is_whitespace() || ch == '-') && !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn rendered_heading_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'`' => {
                index += 1;
                while index < bytes.len() && bytes[index] != b'`' {
                    output.push(bytes[index] as char);
                    index += 1;
                }
                if index < bytes.len() {
                    index += 1;
                }
            }
            b'!' if index + 1 < bytes.len() && bytes[index + 1] == b'[' => {
                if let Some((label, next_index)) = markdown_inline_label(text, index + 1) {
                    output.push_str(&rendered_heading_text(label));
                    index = next_index;
                } else {
                    index += 1;
                }
            }
            b'[' => {
                if let Some((label, next_index)) = markdown_inline_label(text, index) {
                    output.push_str(&rendered_heading_text(label));
                    index = next_index;
                } else {
                    output.push(bytes[index] as char);
                    index += 1;
                }
            }
            b'*' | b'_' | b'~' => {
                index += 1;
            }
            _ => {
                output.push(bytes[index] as char);
                index += 1;
            }
        }
    }
    output
}

fn markdown_inline_label(text: &str, open_bracket: usize) -> Option<(&str, usize)> {
    let close_bracket = text[open_bracket + 1..].find("](")? + open_bracket + 1;
    let target_start = close_bracket + 2;
    let close_paren = text[target_start..].find(')')? + target_start;
    Some((&text[open_bracket + 1..close_bracket], close_paren + 1))
}
