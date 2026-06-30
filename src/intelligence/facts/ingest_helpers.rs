//! Private helpers for projecting runtime data into project facts.

use crate::content_repository::{AdapterKind, CollectionSpec};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

pub(super) fn schema_fields(
    collection: &CollectionSpec,
    schema: Option<&Value>,
) -> Vec<(String, String, bool)> {
    let class_name = collection
        .schema_class
        .as_deref()
        .unwrap_or(collection.object_type.as_str());
    let required = schema
        .and_then(|schema| schema.pointer(&format!("/$defs/{class_name}/required")))
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default();
    let mut fields = schema
        .and_then(|schema| schema.pointer(&format!("/$defs/{class_name}/properties")))
        .and_then(Value::as_object)
        .map(|properties| {
            properties
                .iter()
                .map(|(name, value)| {
                    (
                        name.clone(),
                        schema_property_kind(value),
                        required.contains(name),
                    )
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    if !fields
        .iter()
        .any(|(name, _, _)| name == &collection.id_field)
    {
        fields.push((collection.id_field.clone(), "string".to_string(), true));
    }
    fields.sort_by(|left, right| left.0.cmp(&right.0));
    fields
}

pub(super) fn adapter_name(adapter: AdapterKind) -> &'static str {
    match adapter {
        AdapterKind::MarkdownFrontmatter => "markdown_frontmatter",
        AdapterKind::JsonRecord => "json_record",
        AdapterKind::YamlRecord => "yaml_record",
        AdapterKind::JsonlRecord => "jsonl_record",
    }
}

pub(super) fn searchable_object_text(data: &Map<String, Value>) -> String {
    let mut parts = Vec::new();
    for (key, value) in data {
        if let Some(text) = value.as_str() {
            parts.push(format!("{key}: {text}"));
        }
    }
    parts.join("\n")
}

pub(super) fn line_from_message(message: &str) -> Option<usize> {
    number_after_marker(message, "line ")
}

pub(super) fn column_from_message(message: &str) -> Option<usize> {
    number_after_marker(message, "column ")
}

fn schema_property_kind(value: &Value) -> String {
    match value.get("type").and_then(Value::as_str) {
        Some("array") => value
            .get("items")
            .and_then(|items| items.get("type"))
            .and_then(Value::as_str)
            .map(|item| format!("array<{item}>"))
            .unwrap_or_else(|| "array".to_string()),
        Some(kind) => kind.to_string(),
        None if value.get("enum").is_some() => "enum".to_string(),
        None => "unknown".to_string(),
    }
}

fn number_after_marker(message: &str, marker: &str) -> Option<usize> {
    let start = message.find(marker)? + marker.len();
    let digits = message[start..]
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    digits.parse().ok()
}
