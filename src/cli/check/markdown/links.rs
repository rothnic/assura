//! Markdown-authored local link validation.

use super::{
    is_fence_start, markdown_headings, markdown_severity, suppression::MarkdownSuppressions,
};
use crate::cli::check::rules::display_rel;
use crate::cli::check::{StructureCheckReport, StructureChecker};
use crate::config::config::MarkdownBundle;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

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
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownLink {
    target: String,
    line_number: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedMarkdownTarget {
    path: PathBuf,
    anchor: Option<String>,
}

fn markdown_links(content: &str) -> Vec<MarkdownLink> {
    let mut links = Vec::new();
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
        let mut search_from = 0;
        while let Some(relative_start) = line[search_from..].find("](") {
            let start = search_from + relative_start;
            let Some(open_bracket) = line[..start].rfind('[') else {
                search_from = start + 2;
                continue;
            };
            if open_bracket > 0 && line.as_bytes()[open_bracket - 1] == b'!' {
                search_from = start + 2;
                continue;
            }
            if is_inside_inline_code_span(line, open_bracket) {
                search_from = start + 2;
                continue;
            }
            let target_start = start + 2;
            let Some(relative_end) = line[target_start..].find(')') else {
                break;
            };
            let end = target_start + relative_end;
            let raw = line[target_start..end]
                .split_whitespace()
                .next()
                .unwrap_or_default();
            if should_check_link_target(raw) {
                links.push(MarkdownLink {
                    target: raw.to_string(),
                    line_number: line_index + 1,
                });
            }
            search_from = end + 1;
        }
    }

    links
}

fn is_inside_inline_code_span(line: &str, byte_index: usize) -> bool {
    let bytes = line.as_bytes();
    let mut in_code = false;
    let mut index = 0;
    while index < byte_index {
        if bytes[index] == b'`' {
            while index < byte_index && bytes[index] == b'`' {
                index += 1;
            }
            in_code = !in_code;
        } else {
            index += 1;
        }
    }
    in_code
}

fn should_check_link_target(raw: &str) -> bool {
    !raw.is_empty() && !raw.starts_with('#') && !raw.contains("://") && !raw.starts_with("mailto:")
}

fn parse_markdown_link_target(source_rel: &Path, raw: &str) -> Option<ParsedMarkdownTarget> {
    if raw.starts_with('/') {
        return None;
    }
    let (path_text, anchor) = raw.split_once('#').unwrap_or((raw, ""));
    if path_text.is_empty() {
        return None;
    }
    let mut path = source_rel
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .to_path_buf();
    path.push(path_text);
    Some(ParsedMarkdownTarget {
        path: normalize_relative_path(&path)?,
        anchor: (!anchor.is_empty()).then(|| anchor.to_string()),
    })
}

fn normalize_relative_path(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(normalized)
}

fn parse_line_anchor(anchor: &str) -> Option<(usize, usize)> {
    let raw = anchor.strip_prefix('L')?;
    let (start, end) = raw.split_once('-').unwrap_or((raw, raw));
    let start = start.parse::<usize>().ok()?;
    let end = end.strip_prefix('L').unwrap_or(end).parse::<usize>().ok()?;
    Some((start, end))
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
        } else if ch.is_whitespace() || ch == '-' {
            if !previous_dash && !slug.is_empty() {
                slug.push('-');
                previous_dash = true;
            }
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

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
}
