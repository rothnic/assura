//! Markdown inline-link extraction shared by validators and fact ingestion.

use std::path::{Component, Path, PathBuf};

/// Markdown inline link authored in one source document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownLink {
    /// Raw target text from the Markdown link destination.
    pub(crate) target: String,
    /// One-based source line.
    pub(crate) line_number: usize,
    /// One-based source column at the opening bracket.
    pub(crate) column_number: usize,
}

/// Repository-relative local link target after resolving from a source path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedMarkdownTarget {
    /// Repository-relative target path.
    pub(crate) path: PathBuf,
    /// Optional target anchor without the leading `#`.
    pub(crate) anchor: Option<String>,
}

/// Extract local Markdown inline links from `content`.
pub(crate) fn markdown_links(content: &str) -> Vec<MarkdownLink> {
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
                    column_number: line[..open_bracket].chars().count() + 1,
                });
            }
            search_from = end + 1;
        }
    }

    links
}

/// Resolve a raw Markdown link target relative to its source document.
pub(crate) fn parse_markdown_link_target(
    source_rel: &Path,
    raw: &str,
) -> Option<ParsedMarkdownTarget> {
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

/// Parse a GitHub-style line anchor such as `L4` or `L4-L8`.
pub(crate) fn parse_line_anchor(anchor: &str) -> Option<(usize, usize)> {
    let raw = anchor.strip_prefix('L')?;
    let (start, end) = raw.split_once('-').unwrap_or((raw, raw));
    let start = start.parse::<usize>().ok()?;
    let end = end.strip_prefix('L').unwrap_or(end).parse::<usize>().ok()?;
    Some((start, end))
}

/// Whether a path points to a Markdown file by extension.
pub(crate) fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
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
