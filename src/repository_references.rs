//! Lightweight repository-reference tokenization shared by check and graph paths.

use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceReference {
    pub(crate) raw: String,
    pub(crate) line_number: usize,
    pub(crate) column_number: usize,
    pub(crate) target_path: PathBuf,
    pub(crate) target_anchor: Option<String>,
    pub(crate) target_line_start: Option<usize>,
    pub(crate) target_line_end: Option<usize>,
    pub(crate) kind: &'static str,
    pub(crate) confidence: &'static str,
}

pub(crate) fn source_references(source_rel: &Path, content: &str) -> Vec<SourceReference> {
    let mut references = Vec::new();
    let mut block_comment = false;
    let mut docstring = false;

    for (line_index, line) in content.lines().enumerate() {
        let line_number = line_index + 1;
        let context = line_context(line, &mut block_comment, &mut docstring);
        for (start, _, raw) in candidate_spans(line) {
            let quoted = is_wrapped_reference_token(raw);
            if context.is_none() && !quoted {
                continue;
            }
            let Some(candidate) = normalize_reference_token(raw) else {
                continue;
            };
            let kind = context.unwrap_or(SourceReferenceContext::StringLiteral);
            let Some(reference) = reference_from_raw_token(
                source_rel,
                candidate,
                line_number,
                line[..start].chars().count() + 1,
                kind.reference_kind(),
                kind.confidence(),
            ) else {
                continue;
            };
            references.push(reference);
        }
    }

    references
}

pub(crate) fn frontmatter_references(
    source_rel: &Path,
    content: &str,
    fields: &[String],
) -> Vec<SourceReference> {
    if fields.is_empty() {
        return Vec::new();
    }
    let Some((frontmatter, frontmatter_start_line)) = markdown_frontmatter(content) else {
        return Vec::new();
    };
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(frontmatter) else {
        return Vec::new();
    };

    fields
        .iter()
        .filter_map(|field| {
            let value = yaml_path_value(&value, field)?;
            let (line_number, column_number) = frontmatter_field_location(frontmatter, field)
                .unwrap_or((frontmatter_start_line, 1));
            Some(
                yaml_reference_values(value)
                    .into_iter()
                    .filter_map(move |raw| {
                        reference_from_raw_token(
                            source_rel,
                            raw,
                            line_number,
                            column_number,
                            "frontmatter_reference",
                            "exact",
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .flatten()
        .collect()
}

fn reference_from_raw_token(
    source_rel: &Path,
    raw: &str,
    line_number: usize,
    column_number: usize,
    kind: &'static str,
    confidence: &'static str,
) -> Option<SourceReference> {
    let candidate = normalize_reference_token(raw)?;
    let target = parse_source_reference_target(source_rel, candidate)?;
    Some(SourceReference {
        raw: candidate.to_string(),
        line_number,
        column_number,
        target_path: target.path,
        target_anchor: target.anchor,
        target_line_start: target.line_start,
        target_line_end: target.line_end,
        kind,
        confidence,
    })
}

fn markdown_frontmatter(content: &str) -> Option<(&str, usize)> {
    if let Some(rest) = content.strip_prefix("---\r\n") {
        let (frontmatter, _) = rest.split_once("\r\n---")?;
        return Some((frontmatter, 2));
    }
    let rest = content.strip_prefix("---\n")?;
    let (frontmatter, _) = rest.split_once("\n---")?;
    Some((frontmatter, 2))
}

fn yaml_path_value<'a>(value: &'a serde_yaml::Value, field: &str) -> Option<&'a serde_yaml::Value> {
    let mut current = value;
    for segment in field.split('.') {
        let serde_yaml::Value::Mapping(mapping) = current else {
            return None;
        };
        current = mapping.get(serde_yaml::Value::String(segment.to_string()))?;
    }
    Some(current)
}

fn yaml_reference_values(value: &serde_yaml::Value) -> Vec<&str> {
    match value {
        serde_yaml::Value::String(value) => vec![value.as_str()],
        serde_yaml::Value::Sequence(values) => values
            .iter()
            .filter_map(|value| match value {
                serde_yaml::Value::String(value) => Some(value.as_str()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn frontmatter_field_location(frontmatter: &str, field: &str) -> Option<(usize, usize)> {
    let field_name = field.rsplit('.').next().unwrap_or(field);
    for (index, line) in frontmatter.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with(field_name) && trimmed[field_name.len()..].starts_with(':') {
            let column = line.len() - trimmed.len() + 1;
            return Some((index + 2, column));
        }
    }
    None
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceReferenceContext {
    Comment,
    DocComment,
    Docstring,
    StringLiteral,
}

impl SourceReferenceContext {
    fn reference_kind(self) -> &'static str {
        match self {
            Self::Comment => "comment_reference",
            Self::DocComment => "doc_comment_reference",
            Self::Docstring => "docstring_reference",
            Self::StringLiteral => "string_literal_reference",
        }
    }

    fn confidence(self) -> &'static str {
        match self {
            Self::Comment | Self::DocComment | Self::Docstring => "medium",
            Self::StringLiteral => "low",
        }
    }
}

fn line_context(
    line: &str,
    block_comment: &mut bool,
    docstring: &mut bool,
) -> Option<SourceReferenceContext> {
    let trimmed = line.trim_start();
    let started_in_block = *block_comment;
    let started_in_docstring = *docstring;

    if trimmed.contains("/*") {
        *block_comment = !trimmed.contains("*/");
    } else if *block_comment && trimmed.contains("*/") {
        *block_comment = false;
    }

    let triple_double = trimmed.matches("\"\"\"").count();
    let triple_single = trimmed.matches("'''").count();
    if (triple_double + triple_single) % 2 == 1 {
        *docstring = !*docstring;
    }

    if started_in_docstring || trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
        return Some(SourceReferenceContext::Docstring);
    }
    if started_in_block || trimmed.starts_with("/*") {
        return if trimmed.starts_with("/**") {
            Some(SourceReferenceContext::DocComment)
        } else {
            Some(SourceReferenceContext::Comment)
        };
    }
    if trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return Some(SourceReferenceContext::DocComment);
    }
    if trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || trimmed.starts_with("--")
        || trimmed.starts_with('*')
    {
        return Some(SourceReferenceContext::Comment);
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedSourceReferenceTarget {
    path: PathBuf,
    anchor: Option<String>,
    line_start: Option<usize>,
    line_end: Option<usize>,
}

fn parse_source_reference_target(
    source_rel: &Path,
    candidate: &str,
) -> Option<ParsedSourceReferenceTarget> {
    let (path_text, anchor, line_range) = split_reference_target(candidate)?;
    let path = if path_text.starts_with("./") || path_text.starts_with("../") {
        source_rel
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(path_text)
    } else {
        PathBuf::from(path_text)
    };
    let path = normalize_relative_path(&path)?;
    let (line_start, line_end) = line_range
        .or_else(|| anchor.as_deref().and_then(parse_line_anchor))
        .map(|(start, end)| (Some(start), Some(end)))
        .unwrap_or((None, None));

    Some(ParsedSourceReferenceTarget {
        path,
        anchor,
        line_start,
        line_end,
    })
}

type SplitReferenceTarget<'a> = (&'a str, Option<String>, Option<(usize, usize)>);

fn split_reference_target(candidate: &str) -> Option<SplitReferenceTarget<'_>> {
    let (candidate, anchor) = candidate
        .split_once('#')
        .map_or((candidate, None), |(path, anchor)| {
            (path, (!anchor.is_empty()).then(|| anchor.to_string()))
        });
    if candidate.is_empty() {
        return None;
    }
    if let Some((path, line_range)) = candidate.rsplit_once(':') {
        if is_line_range(line_range) && has_extension(path) {
            return Some((path, anchor, parse_line_range(line_range)));
        }
    }
    has_extension(candidate).then_some((candidate, anchor, None))
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

fn is_wrapped_reference_token(raw: &str) -> bool {
    matches!(raw.chars().next(), Some('`' | '\'' | '"'))
        || matches!(raw.chars().last(), Some('`' | '\'' | '"'))
}

fn is_line_range(value: &str) -> bool {
    let (start, end) = value.split_once('-').unwrap_or((value, value));
    !start.is_empty()
        && start.chars().all(|ch| ch.is_ascii_digit())
        && !end.is_empty()
        && end.chars().all(|ch| ch.is_ascii_digit())
}

fn parse_line_range(value: &str) -> Option<(usize, usize)> {
    let (start, end) = value.split_once('-').unwrap_or((value, value));
    Some((start.parse().ok()?, end.parse().ok()?))
}

fn parse_line_anchor(anchor: &str) -> Option<(usize, usize)> {
    let raw = anchor.strip_prefix('L')?;
    let (start, end) = raw.split_once('-').unwrap_or((raw, raw));
    let start = start.parse::<usize>().ok()?;
    let end = end.strip_prefix('L').unwrap_or(end).parse::<usize>().ok()?;
    Some((start, end))
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
