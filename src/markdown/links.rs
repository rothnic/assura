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
    /// Byte offset where the Markdown link starts on its source line.
    pub(crate) start_byte: usize,
    /// Byte offset just after the Markdown link closes on its source line.
    pub(crate) end_byte: usize,
}

/// Repository-relative path reference that should be rendered as a Markdown link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MarkdownBareReference {
    /// Raw reference text from the Markdown source.
    pub(crate) text: String,
    /// One-based source line.
    pub(crate) line_number: usize,
    /// One-based character column.
    pub(crate) column_number: usize,
    /// Repository-relative target path.
    pub(crate) target_path: PathBuf,
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
                    start_byte: open_bracket,
                    end_byte: end + 1,
                });
            }
            search_from = end + 1;
        }
    }

    links
}

/// Extract existing local path references that are not rendered as Markdown links.
pub(crate) fn markdown_bare_references(
    source_rel: &Path,
    project_root: &Path,
    content: &str,
) -> Vec<MarkdownBareReference> {
    let mut references = Vec::new();
    let mut in_fence = false;
    let mut in_frontmatter = false;
    let mut frontmatter_checked = false;

    for (line_index, line) in content.lines().enumerate() {
        if line_index == 0 && is_frontmatter_delimiter(line) {
            in_frontmatter = true;
            frontmatter_checked = true;
            continue;
        }
        if in_frontmatter {
            if is_frontmatter_delimiter(line) {
                in_frontmatter = false;
            }
            continue;
        }
        if !frontmatter_checked {
            frontmatter_checked = true;
        }

        let trimmed = line.trim_start();
        if is_fence_start(trimmed) {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        let link_spans = inline_link_spans(line);
        references.extend(markdown_bare_references_in_line(
            source_rel,
            project_root,
            line,
            line_index + 1,
            &link_spans,
        ));
    }

    references
}

fn inline_link_spans(line: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut search_from = 0;
    while let Some(relative_start) = line[search_from..].find("](") {
        let start = search_from + relative_start;
        let Some(open_bracket) = line[..start].rfind('[') else {
            search_from = start + 2;
            continue;
        };
        if is_inside_inline_code_span(line, open_bracket) {
            search_from = start + 2;
            continue;
        }
        let span_start = if open_bracket > 0 && line.as_bytes()[open_bracket - 1] == b'!' {
            open_bracket - 1
        } else {
            open_bracket
        };
        let target_start = start + 2;
        let Some(relative_end) = line[target_start..].find(')') else {
            break;
        };
        let end = target_start + relative_end + 1;
        spans.push((span_start, end));
        search_from = end;
    }
    spans
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

fn markdown_bare_references_in_line(
    source_rel: &Path,
    project_root: &Path,
    line: &str,
    line_number: usize,
    masked_spans: &[(usize, usize)],
) -> Vec<MarkdownBareReference> {
    candidate_spans(line)
        .into_iter()
        .filter(|(start, end, _)| {
            !masked_spans
                .iter()
                .any(|(masked_start, masked_end)| start < masked_end && end > masked_start)
        })
        .filter_map(|(start, _, raw)| {
            let candidate = normalize_reference_token(raw)?;
            let target_text = reference_target_path(&candidate)?;
            let target = parse_markdown_link_target(source_rel, target_text)?;
            project_root
                .join(&target.path)
                .is_file()
                .then(|| MarkdownBareReference {
                    text: candidate.to_string(),
                    line_number,
                    column_number: line[..start].chars().count() + 1,
                    target_path: target.path,
                })
        })
        .collect()
}

fn candidate_spans(line: &str) -> Vec<(usize, usize, &str)> {
    let mut spans = Vec::new();
    let mut start = None;
    for (index, ch) in line.char_indices() {
        if is_reference_token_char(ch) {
            start.get_or_insert(index);
        } else if let Some(token_start) = start.take() {
            spans.push((token_start, index, &line[token_start..index]));
        }
    }
    if let Some(token_start) = start {
        spans.push((token_start, line.len(), &line[token_start..]));
    }
    spans
}

fn is_reference_token_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
        || matches!(
            ch,
            '/' | '\\' | '.' | '_' | '-' | ':' | '#' | '~' | '`' | '\'' | '"'
        )
}

fn normalize_reference_token(raw: &str) -> Option<&str> {
    let wrapper = |ch: char| {
        matches!(
            ch,
            '`' | '\''
                | '"'
                | '<'
                | '>'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | ','
                | ';'
                | '!'
                | '?'
        )
    };
    let trimmed = raw.trim_matches(wrapper);
    let trimmed = trimmed.trim_end_matches('.');
    let trimmed = trimmed.trim_matches(wrapper);
    if trimmed.is_empty()
        || trimmed.starts_with('#')
        || trimmed.starts_with('/')
        || trimmed.contains("://")
        || trimmed.starts_with("mailto:")
        || !trimmed.contains('/')
    {
        return None;
    }
    Some(trimmed)
}

fn reference_target_path(candidate: &str) -> Option<&str> {
    let candidate = candidate
        .split_once('#')
        .map_or(candidate, |(path, _)| path);
    if let Some((path, _line)) = split_extension_adjacent_line(candidate) {
        return Some(path);
    }
    if let Some((path, line_range)) = candidate.rsplit_once(':') {
        if is_line_range(line_range) && has_extension(path) {
            return Some(path);
        }
    }
    has_extension(candidate).then_some(candidate)
}

fn is_line_range(value: &str) -> bool {
    let (start, end) = value.split_once('-').unwrap_or((value, value));
    !start.is_empty()
        && start.chars().all(|ch| ch.is_ascii_digit())
        && !end.is_empty()
        && end.chars().all(|ch| ch.is_ascii_digit())
}

fn split_extension_adjacent_line(candidate: &str) -> Option<(&str, &str)> {
    let (before_colon, after_colon) = candidate.rsplit_once(':')?;
    if after_colon.is_empty() || !after_colon.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    let digit_start = before_colon
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_ascii_digit())
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(0);
    if digit_start == before_colon.len() {
        return None;
    }
    let path = &before_colon[..digit_start];
    let line = &before_colon[digit_start..];
    (!line.is_empty() && has_extension(path)).then_some((path, line))
}

fn has_extension(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.rsplit_once('.')
                .is_some_and(|(_, extension)| !extension.is_empty())
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

fn is_frontmatter_delimiter(line: &str) -> bool {
    matches!(line.trim(), "---" | "+++")
}

#[cfg(test)]
mod tests {
    use super::markdown_bare_references;
    use std::fs;
    use std::path::Path;

    #[test]
    fn bare_references_include_colon_and_adjacent_line_forms() {
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("src")).unwrap();
        fs::write(project.path().join("src/lib.rs"), "fn one() {}\n").unwrap();

        let references = markdown_bare_references(
            Path::new("docs/note.md"),
            project.path(),
            "See ../src/lib.rs:1-2 and `../src/lib.rs12:34`.\n",
        );

        assert_eq!(
            references
                .iter()
                .map(|reference| reference.text.as_str())
                .collect::<Vec<_>>(),
            vec!["../src/lib.rs:1-2", "../src/lib.rs12:34"]
        );
    }
}
