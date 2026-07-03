//! Markdown parsing helpers for agent guidance contract checks.

use serde_yaml::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(super) fn markdown_heading_texts(content: &str) -> HashSet<&str> {
    markdown_heading_sequence(content).into_iter().collect()
}

pub(super) fn markdown_heading_sequence(content: &str) -> Vec<&str> {
    content.lines().filter_map(markdown_heading_text).collect()
}

fn markdown_heading_text(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    let depth = trimmed.chars().take_while(|ch| *ch == '#').count();
    if !(1..=6).contains(&depth) {
        return None;
    }
    let after_marks = &trimmed[depth..];
    if !after_marks.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    let text = after_marks.trim().trim_end_matches('#').trim_end().trim();
    (!text.is_empty()).then_some(text)
}

pub(super) fn heading_anchor(heading: &str) -> String {
    let mut anchor = String::new();
    let mut last_dash = false;
    for ch in heading.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            anchor.push(ch);
            last_dash = false;
        } else if !last_dash && !anchor.is_empty() {
            anchor.push('-');
            last_dash = true;
        }
    }
    anchor.trim_end_matches('-').to_string()
}

pub(super) fn markdown_section<'a>(content: &'a str, section: &str) -> Option<&'a str> {
    let mut start = None;
    let mut start_depth = 0usize;
    for (offset, line) in line_offsets(content) {
        let Some(heading) = markdown_heading_text(line) else {
            continue;
        };
        let depth = line
            .trim_start()
            .chars()
            .take_while(|ch| *ch == '#')
            .count();
        if start.is_none() && heading == section {
            start = Some(offset + line.len());
            start_depth = depth;
            continue;
        }
        if let Some(start_offset) = start {
            if depth <= start_depth {
                return content.get(start_offset..offset);
            }
        }
    }
    start.and_then(|start_offset| content.get(start_offset..))
}

fn line_offsets(content: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0usize;
    content.split_inclusive('\n').map(move |line| {
        let start = offset;
        offset += line.len();
        (start, line.trim_end_matches(['\r', '\n']))
    })
}

pub(super) fn markdown_links(content: &str) -> Vec<String> {
    let mut links = Vec::new();
    for line in content.lines() {
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            let after_start = &rest[start + 2..];
            let Some(end) = after_start.find(')') else {
                break;
            };
            links.push(after_start[..end].trim().to_string());
            rest = &after_start[end + 1..];
        }
    }
    links
}

pub(super) fn inline_code_spans(content: &str) -> Vec<String> {
    let mut spans = Vec::new();
    for line in content.lines() {
        let mut rest = line;
        while let Some(start) = rest.find('`') {
            let after_start = &rest[start + 1..];
            let Some(end) = after_start.find('`') else {
                break;
            };
            spans.push(after_start[..end].trim().to_string());
            rest = &after_start[end + 1..];
        }
    }
    spans
}

pub(super) fn path_to_slash(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

pub(super) fn frontmatter_mapping(content: &str) -> Option<HashMap<String, Value>> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    let frontmatter = rest.split_once("\n---")?.0;
    serde_yaml::from_str::<HashMap<String, Value>>(frontmatter).ok()
}
