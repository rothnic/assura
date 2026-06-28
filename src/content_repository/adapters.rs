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
    let mut objects = parse_objects(collection, rel_path, content)?;
    if objects.len() != 1 {
        return Err(Box::new(ContentFinding::new(
            "parse_error",
            Some(rel_path.to_path_buf()),
            format!(
                "Adapter for '{}' produced {} records where one was expected",
                rel_path.display(),
                objects.len()
            ),
        )));
    }
    Ok(objects.remove(0))
}

pub(super) fn parse_objects(
    collection: &CollectionSpec,
    rel_path: &Path,
    content: &str,
) -> ParseResult<Vec<RepoObject>> {
    let parsed = match collection.adapter {
        AdapterKind::MarkdownFrontmatter => vec![parse_markdown_frontmatter(content, rel_path)?],
        AdapterKind::JsonRecord => vec![parse_json_record(content, rel_path)?],
        AdapterKind::YamlRecord => vec![parse_yaml_record(content, rel_path)?],
        AdapterKind::JsonlRecord => parse_jsonl_records(content, rel_path)?,
    };

    parsed
        .into_iter()
        .map(|parsed| repo_object_from_data(collection, rel_path, parsed))
        .collect()
}

pub(super) fn serialize_record(
    collection: &CollectionSpec,
    rel_path: &Path,
    data: &Map<String, Value>,
    body: Option<&str>,
) -> Result<String, Box<ContentFinding>> {
    match collection.adapter {
        AdapterKind::MarkdownFrontmatter => serialize_markdown_frontmatter(rel_path, data, body),
        AdapterKind::JsonRecord => serialize_json_record(rel_path, data),
        AdapterKind::YamlRecord => serialize_yaml_record(rel_path, data),
        AdapterKind::JsonlRecord => serialize_jsonl_record(rel_path, data),
    }
}

pub(super) fn serialize_jsonl_records(
    collection: &CollectionSpec,
    rel_path: &Path,
    records: Vec<Map<String, Value>>,
) -> Result<String, Box<ContentFinding>> {
    let mut records = records;
    records.sort_by(|left, right| {
        record_id(collection, rel_path, left).cmp(&record_id(collection, rel_path, right))
    });
    let mut content = String::new();
    for record in records {
        content.push_str(&serialize_jsonl_record(rel_path, &record)?);
    }
    Ok(content)
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
    let data = yaml_mapping_to_json_map(yaml, rel_path, "Frontmatter")?;
    Ok(ParsedObjectData {
        data,
        body: Some(body.to_string()),
        headings: markdown_headings(body),
    })
}

fn parse_yaml_record(content: &str, rel_path: &Path) -> ParseResult<ParsedObjectData> {
    let yaml = serde_yaml::from_str::<serde_yaml::Value>(content).map_err(|error| {
        Box::new(ContentFinding::new(
            "parse_error",
            Some(rel_path.to_path_buf()),
            format!("Invalid YAML in '{}': {error}", rel_path.display()),
        ))
    })?;
    let data = yaml_mapping_to_json_map(yaml, rel_path, "YAML record")?;
    Ok(ParsedObjectData {
        data,
        body: None,
        headings: Vec::new(),
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

fn parse_jsonl_records(content: &str, rel_path: &Path) -> ParseResult<Vec<ParsedObjectData>> {
    let mut records = Vec::new();
    for (line_index, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let value = serde_json::from_str::<Value>(line).map_err(|error| {
            Box::new(ContentFinding::new(
                "parse_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "Invalid JSONL in '{}' at line {}: {error}",
                    rel_path.display(),
                    line_index + 1
                ),
            ))
        })?;
        let Value::Object(data) = value else {
            return Err(Box::new(ContentFinding::new(
                "parse_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "JSONL record '{}' line {} must be an object",
                    rel_path.display(),
                    line_index + 1
                ),
            )));
        };
        records.push(ParsedObjectData {
            data,
            body: None,
            headings: Vec::new(),
        });
    }
    Ok(records)
}

fn serialize_markdown_frontmatter(
    rel_path: &Path,
    data: &Map<String, Value>,
    body: Option<&str>,
) -> Result<String, Box<ContentFinding>> {
    let mut frontmatter = serde_yaml::to_string(data).map_err(|error| {
        Box::new(ContentFinding::new(
            "serialize_error",
            Some(rel_path.to_path_buf()),
            format!(
                "Failed to serialize Markdown frontmatter for '{}': {error}",
                rel_path.display()
            ),
        ))
    })?;
    frontmatter = frontmatter
        .strip_prefix("---\n")
        .unwrap_or(frontmatter.as_str())
        .to_string();
    if !frontmatter.ends_with('\n') {
        frontmatter.push('\n');
    }
    Ok(format!("---\n{frontmatter}---\n{}", body.unwrap_or("")))
}

fn serialize_json_record(
    rel_path: &Path,
    data: &Map<String, Value>,
) -> Result<String, Box<ContentFinding>> {
    serde_json::to_string_pretty(&Value::Object(data.clone()))
        .map(|mut content| {
            content.push('\n');
            content
        })
        .map_err(|error| {
            Box::new(ContentFinding::new(
                "serialize_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "Failed to serialize JSON record for '{}': {error}",
                    rel_path.display()
                ),
            ))
        })
}

fn serialize_yaml_record(
    rel_path: &Path,
    data: &Map<String, Value>,
) -> Result<String, Box<ContentFinding>> {
    serde_yaml::to_string(data)
        .map(|mut content| {
            content = content
                .strip_prefix("---\n")
                .unwrap_or(content.as_str())
                .to_string();
            if !content.ends_with('\n') {
                content.push('\n');
            }
            content
        })
        .map_err(|error| {
            Box::new(ContentFinding::new(
                "serialize_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "Failed to serialize YAML record for '{}': {error}",
                    rel_path.display()
                ),
            ))
        })
}

fn serialize_jsonl_record(
    rel_path: &Path,
    data: &Map<String, Value>,
) -> Result<String, Box<ContentFinding>> {
    serde_json::to_string(&Value::Object(data.clone()))
        .map(|mut content| {
            content.push('\n');
            content
        })
        .map_err(|error| {
            Box::new(ContentFinding::new(
                "serialize_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "Failed to serialize JSONL record for '{}': {error}",
                    rel_path.display()
                ),
            ))
        })
}

fn yaml_mapping_to_json_map(
    value: serde_yaml::Value,
    rel_path: &Path,
    label: &str,
) -> ParseResult<Map<String, Value>> {
    let serde_yaml::Value::Mapping(mapping) = value else {
        return Err(Box::new(ContentFinding::new(
            "parse_error",
            Some(rel_path.to_path_buf()),
            format!("{label} in '{}' must be a YAML object", rel_path.display()),
        )));
    };

    let mut data = Map::new();
    for (key, value) in mapping {
        let serde_yaml::Value::String(key) = key else {
            return Err(Box::new(ContentFinding::new(
                "parse_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "{label} in '{}' contains a non-string key",
                    rel_path.display()
                ),
            )));
        };
        let value = serde_json::to_value(value).map_err(|error| {
            Box::new(ContentFinding::new(
                "parse_error",
                Some(rel_path.to_path_buf()),
                format!(
                    "{label} field '{}' in '{}' cannot be represented as JSON: {error}",
                    key,
                    rel_path.display()
                ),
            ))
        })?;
        data.insert(key, value);
    }
    Ok(data)
}

fn repo_object_from_data(
    collection: &CollectionSpec,
    rel_path: &Path,
    parsed: ParsedObjectData,
) -> ParseResult<RepoObject> {
    let id = parsed
        .data
        .get(&collection.id_field)
        .and_then(Value::as_str)
        .ok_or_else(|| missing_id_finding(collection, rel_path))?
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

fn record_id(collection: &CollectionSpec, rel_path: &Path, data: &Map<String, Value>) -> String {
    match data.get(&collection.id_field).and_then(Value::as_str) {
        Some(id) => id.to_string(),
        None => rel_path.to_string_lossy().to_string(),
    }
}

fn missing_id_finding(collection: &CollectionSpec, rel_path: &Path) -> Box<ContentFinding> {
    Box::new(ContentFinding::new(
        "missing_object_id",
        Some(rel_path.to_path_buf()),
        format!(
            "Object in '{}' is missing string id field '{}'",
            rel_path.display(),
            collection.id_field
        ),
    ))
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
