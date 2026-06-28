//! File format adapters for repo-native content runtime validation.

use super::model::{AdapterKind, CollectionSpec, ContentFinding, MarkdownHeading, RepoObject};
use serde_json::{Map, Value};
use std::path::Path;

type ParseResult<T> = Result<T, Box<ContentFinding>>;

pub(super) fn parse_object(
    collection: &CollectionSpec,
    rel_path: &Path,
    content: &str,
) -> ParseResult<RepoObject> {
    let parsed = match collection.adapter {
        AdapterKind::MarkdownFrontmatter => parse_markdown_frontmatter(content, rel_path)?,
        AdapterKind::JsonRecord => parse_json_record(content, rel_path)?,
    };

    let id = parsed
        .data
        .get(&collection.id_field)
        .and_then(Value::as_str)
        .ok_or_else(|| {
            Box::new(ContentFinding::new(
                "missing_object_id",
                Some(rel_path.to_path_buf()),
                format!(
                    "Object in '{}' is missing string id field '{}'",
                    rel_path.display(),
                    collection.id_field
                ),
            ))
        })?
        .to_string();

    Ok(RepoObject {
        collection: collection.name.clone(),
        object_type: collection.object_type.clone(),
        id,
        rel_path: rel_path.to_path_buf(),
        data: parsed.data,
        body: parsed.body,
        headings: parsed.headings,
    })
}

struct ParsedObjectData {
    data: Map<String, Value>,
    body: Option<String>,
    headings: Vec<MarkdownHeading>,
}

fn parse_markdown_frontmatter(content: &str, rel_path: &Path) -> ParseResult<ParsedObjectData> {
    let Some((frontmatter, body)) = split_markdown_frontmatter(content) else {
        return Err(Box::new(ContentFinding::new(
            "frontmatter_missing",
            Some(rel_path.to_path_buf()),
            format!(
                "Markdown object '{}' is missing YAML frontmatter",
                rel_path.display()
            ),
        )));
    };
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(frontmatter).map_err(|error| {
        Box::new(ContentFinding::new(
            "parse_error",
            Some(rel_path.to_path_buf()),
            format!(
                "Invalid YAML frontmatter in '{}': {error}",
                rel_path.display()
            ),
        ))
    })?;
    let data = yaml_mapping_to_json_map(yaml, rel_path)?;
    Ok(ParsedObjectData {
        data,
        body: Some(body.to_string()),
        headings: markdown_headings(body),
    })
}

fn parse_json_record(content: &str, rel_path: &Path) -> ParseResult<ParsedObjectData> {
    let value = serde_json::from_str::<Value>(content).map_err(|error| {
        Box::new(ContentFinding::new(
            "parse_error",
            Some(rel_path.to_path_buf()),
            format!("Invalid JSON in '{}': {error}", rel_path.display()),
        ))
    })?;
    let Value::Object(data) = value else {
        return Err(Box::new(ContentFinding::new(
            "parse_error",
            Some(rel_path.to_path_buf()),
            format!("JSON record '{}' must be an object", rel_path.display()),
        )));
    };
    Ok(ParsedObjectData {
        data,
        body: None,
        headings: Vec::new(),
    })
}

fn yaml_mapping_to_json_map(
    value: serde_yaml::Value,
    rel_path: &Path,
) -> ParseResult<Map<String, Value>> {
    let serde_yaml::Value::Mapping(mapping) = value else {
        return Err(Box::new(ContentFinding::new(
            "parse_error",
            Some(rel_path.to_path_buf()),
            format!(
                "Frontmatter in '{}' must be a YAML object",
                rel_path.display()
            ),
        )));
    };

    let mut data = Map::new();
    for (key, value) in mapping {
        let serde_yaml::Value::String(key) = key else {
            return Err(Box::new(ContentFinding::new(
                "parse_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "Frontmatter in '{}' contains a non-string key",
                    rel_path.display()
                ),
            )));
        };
        let value = serde_json::to_value(value).map_err(|error| {
            Box::new(ContentFinding::new(
                "parse_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "Frontmatter field '{}' in '{}' cannot be represented as JSON: {error}",
                    key,
                    rel_path.display()
                ),
            ))
        })?;
        data.insert(key, value);
    }
    Ok(data)
}

fn split_markdown_frontmatter(content: &str) -> Option<(&str, &str)> {
    let rest = content.strip_prefix("---")?;
    let rest = rest.strip_prefix('\n').unwrap_or(rest);
    let (frontmatter, body) = rest.split_once("\n---")?;
    let body = body
        .strip_prefix('\n')
        .or_else(|| body.strip_prefix("\r\n"))
        .unwrap_or(body);
    Some((frontmatter, body))
}

fn markdown_headings(body: &str) -> Vec<MarkdownHeading> {
    let mut headings = Vec::new();
    let mut in_fence = false;

    for (line_index, line) in body.lines().enumerate() {
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
        let level = trimmed.chars().take_while(|ch| *ch == '#').count();
        if !(1..=6).contains(&level) {
            continue;
        }
        let after_marks = &trimmed[level..];
        if !after_marks
            .chars()
            .next()
            .is_some_and(|ch| ch.is_whitespace())
        {
            continue;
        }
        let text = after_marks.trim().trim_end_matches('#').trim().to_string();
        if !text.is_empty() {
            headings.push(MarkdownHeading {
                level,
                text,
                line_number: line_index + 1,
            });
        }
    }
    headings
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
