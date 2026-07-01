use super::repository_references::RepositoryReferenceEdge;
use super::set::{normalize_path, resource_id};
use super::types::{EdgeId, FactId, FactOrigin, ProjectEdge, ProjectFact, Resource};
use super::FactIngestor;
use std::path::{Component, Path, PathBuf};

impl FactIngestor {
    /// Ingest conservative source/comment/string repository references.
    pub fn ingest_source_references(
        &mut self,
        project_root: &Path,
        source_rel: &Path,
        content: &str,
    ) {
        let references = source_references(source_rel, content);
        if references.is_empty() {
            return;
        }
        let source_id = self.upsert_repository_reference_resource(source_rel);
        for reference in references {
            let target_exists = project_root.join(&reference.target_path).is_file();
            let target_id = target_exists
                .then(|| self.upsert_repository_reference_resource(&reference.target_path));
            self.facts
                .upsert_edge(ProjectEdge::RepositoryReference(RepositoryReferenceEdge {
                    id: EdgeId::from_parts(
                        "repository_reference",
                        &format!(
                            "{}:{}:{}:{}",
                            normalize_path(source_rel),
                            reference.line_number,
                            reference.column_number,
                            reference.raw
                        ),
                    ),
                    generation: self.generation.clone(),
                    origin: FactOrigin::Derived,
                    source_id: source_id.clone(),
                    target_id,
                    source_path: source_rel.to_path_buf(),
                    source_line: Some(reference.line_number),
                    source_column: Some(reference.column_number),
                    target_path: reference.target_path.clone(),
                    target_anchor: reference.target_anchor.clone(),
                    target_line_start: reference.target_line_start,
                    target_line_end: reference.target_line_end,
                    target_exists,
                    reference_kind: reference.kind.to_string(),
                    rule: reference_rule(&reference, target_exists),
                    confidence: reference.confidence.to_string(),
                }));
        }
    }

    fn upsert_repository_reference_resource(&mut self, path: &Path) -> FactId {
        let id = resource_id(path);
        self.facts.upsert_fact(ProjectFact::Resource(Resource {
            id: id.clone(),
            generation: self.generation.clone(),
            origin: FactOrigin::Source,
            path: path.to_path_buf(),
            extension: path
                .extension()
                .and_then(|extension| extension.to_str())
                .map(ToOwned::to_owned),
        }));
        id
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceReference {
    raw: String,
    line_number: usize,
    column_number: usize,
    target_path: PathBuf,
    target_anchor: Option<String>,
    target_line_start: Option<usize>,
    target_line_end: Option<usize>,
    kind: &'static str,
    confidence: &'static str,
}

fn source_references(source_rel: &Path, content: &str) -> Vec<SourceReference> {
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
            let Some(target) = parse_source_reference_target(source_rel, candidate) else {
                continue;
            };
            let kind = context.unwrap_or(SourceReferenceContext::StringLiteral);
            references.push(SourceReference {
                raw: candidate.to_string(),
                line_number,
                column_number: line[..start].chars().count() + 1,
                target_path: target.path,
                target_anchor: target.anchor,
                target_line_start: target.line_start,
                target_line_end: target.line_end,
                kind: kind.reference_kind(),
                confidence: kind.confidence(),
            });
        }
    }

    references
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

fn split_reference_target(
    candidate: &str,
) -> Option<(&str, Option<String>, Option<(usize, usize)>)> {
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

fn reference_rule(reference: &SourceReference, target_exists: bool) -> String {
    if !target_exists {
        "repository_reference_target".to_string()
    } else if reference.target_line_start.is_some() {
        "repository_reference_line_anchor".to_string()
    } else if reference.target_anchor.is_some() {
        "repository_reference_anchor".to_string()
    } else {
        "repository_reference_target".to_string()
    }
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
