//! Compact structure notation normalization.
//!
//! The runtime validator still consumes the verbose structure-first `Config`
//! model. This module accepts the concise notation from the product spec and
//! lowers it into that existing shape before deserialization.

use serde_yaml::{Mapping, Value};

use super::compact_helpers::{
    append_nested_sequence, into_mapping, is_extension_key, is_node_attr_key,
    is_verbose_directory_node_key, mapping_entry, merge_mapping, path_has_scope_magic,
    set_nested_bool, set_nested_mapping, set_nested_value, string_value,
};
use super::compact_rules::{
    is_rule_reference, parse_rule_reference, push_rule_stack, RuleRegistry,
};

const STRUCTURE: &str = "structure";
const USE: &str = "use";
const EXTRA: &str = "extra";

/// Normalize compact Assura config notation into the verbose config shape.
pub(crate) fn normalize_compact_config_value(value: Value) -> Result<Value, String> {
    let Value::Mapping(mut root) = value else {
        return Ok(value);
    };

    let rules = RuleRegistry::from_root(&mut root)?;
    let Some(structure_value) = root.remove(string_value(STRUCTURE)) else {
        return Ok(Value::Mapping(root));
    };

    let structure = normalize_structure(structure_value, &rules)?;
    root.insert(string_value(STRUCTURE), structure);
    Ok(Value::Mapping(root))
}

fn normalize_structure(value: Value, rules: &RuleRegistry) -> Result<Value, String> {
    let Value::Mapping(mapping) = value else {
        return Err("compact config structure must be a mapping".to_string());
    };

    let mut normalized = Mapping::new();
    for (key, node_value) in mapping {
        let Some(path) = key.as_str() else {
            return Err("compact config structure paths must be strings".to_string());
        };
        let mut stack = Vec::new();
        let node = normalize_structure_node(node_value, rules, &mut stack)?;
        let mut node_mapping = into_mapping(node, "structure node")?;
        if path_has_scope_magic(path) {
            node_mapping
                .entry(string_value("required"))
                .or_insert(Value::Bool(false));
        }
        normalized.insert(
            Value::String(path.to_string()),
            Value::Mapping(node_mapping),
        );
    }
    Ok(Value::Mapping(normalized))
}

fn normalize_structure_node(
    value: Value,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
) -> Result<Value, String> {
    let Value::Mapping(mapping) = value else {
        return Err("compact config structure nodes must be mappings".to_string());
    };
    let expanded = expand_tree_mapping(mapping, rules, stack)?;
    let mut output = Mapping::new();

    for (key, value) in expanded {
        let Some(key) = key.as_str() else {
            return Err("compact config structure node keys must be strings".to_string());
        };
        if key == USE {
            continue;
        }
        if key == EXTRA {
            let Some(allow_extra) = value.as_bool() else {
                return Err("compact config extra must be a boolean".to_string());
            };
            set_nested_bool(&mut output, "files", "allow_extra", allow_extra);
            set_nested_bool(&mut output, "directories", "allow_extra", allow_extra);
            continue;
        }
        if is_verbose_directory_node_key(key) {
            insert_verbose_node_key(&mut output, key, value, rules, stack)?;
            continue;
        }

        normalize_path_key(&mut output, key, value, rules, stack)?;
    }

    Ok(Value::Mapping(output))
}

fn expand_tree_mapping(
    mapping: Mapping,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
) -> Result<Mapping, String> {
    let mut expanded = Mapping::new();

    if let Some(use_value) = mapping.get(string_value(USE)) {
        for reference in use_references(use_value)? {
            let name = parse_rule_reference(&reference)?;
            push_rule_stack(stack, name)?;
            stack.push(name.to_string());
            let tree = rules.resolve_tree(&reference)?;
            let tree = expand_tree_mapping(tree, rules, stack)?;
            stack.pop();
            merge_mapping(&mut expanded, tree);
        }
    }

    let mut local = Mapping::new();
    for (key, value) in mapping {
        if key.as_str() == Some(USE) {
            continue;
        }
        local.insert(key, value);
    }
    merge_mapping(&mut expanded, local);
    Ok(expanded)
}

fn normalize_path_key(
    output: &mut Mapping,
    key: &str,
    value: Value,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
) -> Result<(), String> {
    if key == ".dir" {
        let directive = node_directive(value, rules, stack)?;
        apply_self_directory_directive(output, directive);
        return Ok(());
    }

    if is_extension_key(key) {
        let pattern = format!("*{key}");
        let directive = node_directive(value, rules, stack)?;
        apply_file_pattern_directive(output, &pattern, directive);
        return Ok(());
    }

    if key.ends_with('/') {
        let directory = key.trim_end_matches('/');
        if is_tree_value(&value)? {
            let mut child = into_mapping(normalize_structure_node(value, rules, stack)?, key)?;
            if path_has_scope_magic(key) {
                child
                    .entry(string_value("required"))
                    .or_insert(Value::Bool(false));
            }
            set_nested_mapping(output, "children", directory, Value::Mapping(child));
        } else {
            let directive = node_directive(value, rules, stack)?;
            apply_directory_directive(output, directory, directive)?;
        }
        return Ok(());
    }

    let directive = node_directive(value, rules, stack)?;
    apply_file_directive(output, key, directive)?;
    Ok(())
}

fn insert_verbose_node_key(
    output: &mut Mapping,
    key: &str,
    value: Value,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
) -> Result<(), String> {
    if key == "children" {
        let Value::Mapping(children) = value else {
            return Err("compact config children must be a mapping".to_string());
        };
        let mut normalized = Mapping::new();
        for (child_key, child_value) in children {
            let Some(child_name) = child_key.as_str() else {
                return Err("compact config child names must be strings".to_string());
            };
            let mut child =
                into_mapping(normalize_structure_node(child_value, rules, stack)?, key)?;
            if path_has_scope_magic(child_name) {
                child
                    .entry(string_value("required"))
                    .or_insert(Value::Bool(false));
            }
            normalized.insert(Value::String(child_name.to_string()), Value::Mapping(child));
        }
        output.insert(string_value(key), Value::Mapping(normalized));
    } else {
        output.insert(string_value(key), value);
    }
    Ok(())
}

#[derive(Debug, Default)]
struct NodeDirective {
    exists: Option<String>,
    naming: Option<String>,
}

fn node_directive(
    value: Value,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
) -> Result<NodeDirective, String> {
    match value {
        Value::String(text) if is_rule_reference(&text) => {
            let name = parse_rule_reference(&text)?;
            push_rule_stack(stack, name)?;
            stack.push(name.to_string());
            let resolved = rules.resolve_node(&text)?;
            let directive = node_directive(resolved, rules, stack);
            stack.pop();
            directive
        }
        Value::String(text) => {
            if let Some(exists) = parse_exists_shorthand(&text)? {
                Ok(NodeDirective {
                    exists: Some(exists),
                    ..NodeDirective::default()
                })
            } else {
                Ok(NodeDirective {
                    naming: Some(text),
                    ..NodeDirective::default()
                })
            }
        }
        Value::Number(_) => Err(
            "compact config node directive numbers are not supported; use exists:N or { exists: N }"
                .to_string(),
        ),
        Value::Mapping(mapping) => node_directive_from_mapping(mapping),
        other => Err(format!(
            "compact config node directives must be strings or mappings, got {other:?}"
        )),
    }
}

fn node_directive_from_mapping(mapping: Mapping) -> Result<NodeDirective, String> {
    let mut directive = NodeDirective::default();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err("compact config node attributes must use string keys".to_string());
        };
        match key {
            "exists" => directive.exists = Some(parse_exists_value(value)?),
            "naming" => {
                let Some(naming) = value.as_str() else {
                    return Err("compact config naming must be a string".to_string());
                };
                directive.naming = Some(naming.to_string());
            }
            "severity" => {
                return Err(
                    "compact config attribute 'severity' is not supported in this MVP".to_string(),
                );
            }
            "markdown" | "outline" | "relations" | "validate" => {
                return Err(format!(
                    "compact config attribute '{key}' is not supported in this MVP"
                ));
            }
            unsupported => {
                return Err(format!(
                    "compact config node attribute '{unsupported}' is not supported in this MVP"
                ));
            }
        }
    }
    Ok(directive)
}

fn apply_file_directive(
    output: &mut Mapping,
    filename: &str,
    directive: NodeDirective,
) -> Result<(), String> {
    if directive.naming.is_some() {
        return Err(format!(
            "compact config exact file key '{filename}' only supports exists in this MVP"
        ));
    }
    if let Some(exists) = directive.exists {
        set_nested_mapping(output, "files", "exists", mapping_entry(filename, exists));
        if filename_count_is_allowed(output, "files", filename) {
            append_nested_sequence(output, "files", "allowed_names", filename);
        }
    }
    Ok(())
}

fn apply_file_pattern_directive(output: &mut Mapping, pattern: &str, directive: NodeDirective) {
    let allows_by_count = directive.exists.is_some() && directive.naming.is_none();
    if let Some(naming) = directive.naming {
        set_nested_mapping(
            output,
            "files",
            "naming_patterns",
            mapping_entry(pattern, naming),
        );
    }
    if let Some(exists) = directive.exists {
        set_nested_mapping(output, "files", "exists", mapping_entry(pattern, exists));
    }
    if allows_by_count {
        append_nested_sequence(output, "files", "allowed_patterns", pattern);
    }
}

fn apply_directory_directive(
    output: &mut Mapping,
    directory: &str,
    directive: NodeDirective,
) -> Result<(), String> {
    if directive.naming.is_some() {
        return Err(format!(
            "compact config exact directory key '{directory}/' only supports exists in this MVP"
        ));
    }
    if let Some(exists) = directive.exists {
        set_nested_mapping(
            output,
            "directories",
            "exists",
            mapping_entry(directory, exists),
        );
        if directory_count_is_allowed(output, directory) {
            if path_has_scope_magic(directory) {
                append_nested_sequence(output, "directories", "allowed_patterns", directory);
            } else {
                append_nested_sequence(output, "directories", "allowed_names", directory);
            }
        }
    }
    Ok(())
}

fn apply_self_directory_directive(output: &mut Mapping, directive: NodeDirective) {
    if let Some(exists) = directive.exists {
        set_nested_mapping(
            output,
            "self_directory",
            "exists",
            mapping_entry("*", exists),
        );
    }
    if let Some(naming) = directive.naming {
        set_nested_value(output, "self_directory", "naming", Value::String(naming));
    }
}

fn is_tree_value(value: &Value) -> Result<bool, String> {
    let Value::Mapping(mapping) = value else {
        return Ok(false);
    };

    for key in mapping.keys() {
        let Some(key) = key.as_str() else {
            return Err("compact config fragments must use string keys".to_string());
        };
        if key == USE || key == EXTRA || !is_node_attr_key(key) {
            return Ok(true);
        }
    }

    Ok(false)
}

fn use_references(value: &Value) -> Result<Vec<String>, String> {
    match value {
        Value::String(reference) => Ok(vec![reference.to_string()]),
        Value::Sequence(sequence) => sequence
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "compact config use entries must be strings".to_string())
            })
            .collect(),
        _ => Err("compact config use must be a string or list of strings".to_string()),
    }
}

fn parse_exists_value(value: Value) -> Result<String, String> {
    match value {
        Value::Number(number) => parse_exists_count(&number.to_string()),
        Value::String(text) => {
            if let Some(exists) = parse_exists_shorthand(&text)? {
                Ok(exists)
            } else {
                parse_exists_count(&text)
            }
        }
        other => Err(format!(
            "compact config exists value must be a string or number, got {other:?}"
        )),
    }
}

fn parse_exists_shorthand(value: &str) -> Result<Option<String>, String> {
    if value == "exists" {
        return Err(
            "compact config exists shorthand must include cardinality; use exists:1".to_string(),
        );
    }
    let Some(raw) = value.strip_prefix("exists:") else {
        return Ok(None);
    };
    parse_exists_count(raw).map(Some)
}

fn parse_exists_count(raw: &str) -> Result<String, String> {
    if raw.is_empty() {
        return Err("compact config exists value must not be empty".to_string());
    }
    let parts: Vec<&str> = raw.split('-').collect();
    if parts.len() > 2 {
        return Err(format!(
            "compact config exists value '{raw}' must be N or N-M"
        ));
    }
    let mut bounds = Vec::new();
    for part in parts {
        if part.is_empty() {
            return Err(format!(
                "compact config exists value '{raw}' has an empty range bound"
            ));
        }
        let bound = part.parse::<u16>().map_err(|error| {
            format!("compact config exists value '{raw}' has an invalid bound: {error}")
        })?;
        bounds.push(bound);
    }
    if bounds.len() == 2 && bounds[0] > bounds[1] {
        return Err(format!(
            "compact config exists value '{raw}' has a lower bound greater than its upper bound"
        ));
    }
    Ok(raw.to_string())
}

fn filename_count_is_allowed(output: &Mapping, parent: &str, filename: &str) -> bool {
    count_for(output, parent, filename)
        .map(|count| count != "0")
        .unwrap_or(true)
}

fn directory_count_is_allowed(output: &Mapping, directory: &str) -> bool {
    count_for(output, "directories", directory)
        .map(|count| count != "0")
        .unwrap_or(true)
}

fn count_for<'a>(output: &'a Mapping, parent: &str, child: &str) -> Option<&'a str> {
    output
        .get(string_value(parent))?
        .as_mapping()?
        .get(string_value("exists"))?
        .as_mapping()?
        .get(string_value(child))?
        .as_str()
}
