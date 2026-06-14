//! Helper functions for Assura config normalization.

use serde_yaml::{Mapping, Value};

pub(super) fn merge_mapping(target: &mut Mapping, source: Mapping) {
    for (key, value) in source {
        match (target.get_mut(&key), value) {
            (Some(Value::Mapping(target_map)), Value::Mapping(source_map)) => {
                merge_mapping(target_map, source_map);
            }
            (_, value) => {
                target.insert(key, value);
            }
        }
    }
}

pub(super) fn set_nested_bool(output: &mut Mapping, parent: &str, key: &str, value: bool) {
    set_nested_value(output, parent, key, Value::Bool(value));
}

pub(super) fn set_nested_value(output: &mut Mapping, parent: &str, key: &str, value: Value) {
    let parent = ensure_mapping(output, parent);
    parent.insert(string_value(key), value);
}

pub(super) fn set_nested_mapping(output: &mut Mapping, parent: &str, key: &str, value: Value) {
    let Value::Mapping(value) = value else {
        return;
    };
    let parent = ensure_mapping(output, parent);
    let target = ensure_mapping(parent, key);
    merge_mapping(target, value);
}

pub(super) fn append_nested_sequence(output: &mut Mapping, parent: &str, key: &str, item: &str) {
    let parent = ensure_mapping(output, parent);
    let value = parent
        .entry(string_value(key))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Value::Sequence(sequence) = value else {
        return;
    };
    if !sequence.iter().any(|entry| entry.as_str() == Some(item)) {
        sequence.push(Value::String(item.to_string()));
    }
}

fn ensure_mapping<'a>(mapping: &'a mut Mapping, key: &str) -> &'a mut Mapping {
    let value = mapping
        .entry(string_value(key))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    if !matches!(value, Value::Mapping(_)) {
        *value = Value::Mapping(Mapping::new());
    }
    let Value::Mapping(mapping) = value else {
        unreachable!("value was just normalized to a mapping");
    };
    mapping
}

pub(super) fn mapping_entry(key: &str, value: impl Into<String>) -> Value {
    let mut mapping = Mapping::new();
    mapping.insert(Value::String(key.to_string()), Value::String(value.into()));
    Value::Mapping(mapping)
}

pub(super) fn into_mapping(value: Value, context: &str) -> Result<Mapping, String> {
    let Value::Mapping(mapping) = value else {
        return Err(format!("Assura config {context} must be a mapping"));
    };
    Ok(mapping)
}

pub(super) fn string_value(value: &str) -> Value {
    Value::String(value.to_string())
}

pub(super) fn is_extension_key(key: &str) -> bool {
    let Some(token) = key.strip_prefix('.') else {
        return false;
    };
    if token.is_empty() || key.ends_with('/') || key == ".dir" {
        return false;
    }
    if is_common_dotfile_name(key) {
        return false;
    }
    if token.contains('.') {
        return true;
    }
    token.chars().all(|char| char.is_ascii_alphanumeric())
}

fn is_common_dotfile_name(key: &str) -> bool {
    matches!(
        key,
        ".env"
            | ".gitignore"
            | ".gitattributes"
            | ".dockerignore"
            | ".npmrc"
            | ".yarnrc"
            | ".editorconfig"
            | ".prettierrc"
            | ".eslintrc"
            | ".markdownlintignore"
    )
}

pub(super) fn is_node_attr_key(key: &str) -> bool {
    matches!(
        key,
        "exists"
            | "naming"
            | "max_lines"
            | "max_size"
            | "require_docs"
            | "extensions"
            | "severity"
            | "required"
            | "allowed_names"
            | "allowed_patterns"
            | "forbidden_patterns"
            | "allow_extra"
            | "markdown"
            | "outline"
            | "relations"
            | "validate"
            | "files"
            | "directories"
            | "self_directory"
            | "children"
            | "inherit"
    )
}

pub(super) fn is_directory_node_attr_key(key: &str) -> bool {
    matches!(
        key,
        "files"
            | "directories"
            | "self_directory"
            | "markdown"
            | "exists"
            | "children"
            | "inherit"
            | "required"
    )
}

pub(super) fn path_has_scope_magic(path: &str) -> bool {
    path.contains('*') || path.contains('?') || path.contains('[') || path.contains('{')
}
