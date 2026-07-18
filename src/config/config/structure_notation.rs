//! Structure notation normalization.
//!
//! This module accepts Assura's concise tree notation and normalizes shorthand
//! path keys, rule fragments, cardinality directives, and capture-based
//! relationships before deserializing into the runtime config model.

use serde_yaml::{Mapping, Value};
use std::collections::{BTreeSet, HashMap};

const STRUCTURE: &str = "structure";
const EXTENSIONS: &str = "extensions";
const RELATIONSHIPS: &str = "relationships";
const USE: &str = "use";
const EXTRA: &str = "extra";
const NEEDS: &str = "needs";
const PROVIDES: &str = "provides";
const SECTIONS: &str = "sections";

/// Normalize concise structure config notation into the internal config shape.
pub(crate) fn normalize_structure_config_value(value: Value) -> Result<Value, String> {
    let Value::Mapping(mut root) = value else {
        return Ok(value);
    };

    let rules = RuleRegistry::from_root(&mut root)?;
    let Some(structure_value) = root.remove(string_value(STRUCTURE)) else {
        return Ok(Value::Mapping(root));
    };

    let mut relationships = Vec::new();
    let structure = normalize_structure(structure_value, &rules, &mut relationships)?;
    root.insert(string_value(STRUCTURE), structure);
    insert_relationships(&mut root, relationships)?;
    Ok(Value::Mapping(root))
}

/// Inspect authored rules for non-blocking configuration quality concerns.
pub(crate) fn structure_config_diagnostics(
    value: &Value,
) -> Result<Vec<super::ConfigQualityDiagnostic>, String> {
    let Some(root) = value.as_mapping() else {
        return Ok(Vec::new());
    };
    let Some(rules) = root.get(string_value("rules")).and_then(Value::as_mapping) else {
        return Ok(Vec::new());
    };

    let mut diagnostics = Vec::new();
    for (name, definition) in rules {
        let Some(name) = name.as_str() else {
            continue;
        };
        let name = normalize_rule_definition_name(name)?;
        if !is_trivial_leaf_rule(definition)? {
            continue;
        }
        let uses = count_rule_references(value, name)?;
        if uses <= 1 {
            diagnostics.push(super::ConfigQualityDiagnostic {
                code: "config_rule_reuse".to_string(),
                severity: "low".to_string(),
                rule: name.to_string(),
                message: if uses == 0 {
                    format!("Rule '${name}' is unused; remove it or apply it where the constraint repeats.")
                } else {
                    format!("Rule '${name}' is used once; keep the primitive inline unless the name carries reusable policy meaning.")
                },
            });
        }
    }
    diagnostics.sort_by(|left, right| left.rule.cmp(&right.rule));
    Ok(diagnostics)
}

fn is_trivial_leaf_rule(value: &Value) -> Result<bool, String> {
    if !value.is_mapping() {
        return Ok(true);
    }
    let mapping = value
        .as_mapping()
        .expect("mapping was checked before access");
    if classify_fragment(value)? != FragmentKind::Node || mapping.len() != 1 {
        return Ok(false);
    }
    Ok(mapping
        .keys()
        .filter_map(Value::as_str)
        .all(is_node_attr_key))
}

fn count_rule_references(value: &Value, target: &str) -> Result<usize, String> {
    match value {
        Value::String(text) => Ok(split_composition(text)?
            .into_iter()
            .filter(|token| is_rule_reference(token))
            .filter_map(|token| parse_rule_reference(token).ok())
            .filter(|name| *name == target)
            .count()),
        Value::Sequence(values) => values.iter().try_fold(0, |count, value| {
            count_rule_references(value, target).map(|nested| count + nested)
        }),
        Value::Mapping(mapping) => mapping.values().try_fold(0, |count, value| {
            count_rule_references(value, target).map(|nested| count + nested)
        }),
        _ => Ok(0),
    }
}

/// Expand authored reusable-rule references without adding metadata to the hot runtime model.
pub(crate) fn structure_rule_provenance(
    value: Value,
) -> Result<Vec<super::AuthoredRuleUse>, String> {
    let Value::Mapping(mut root) = value else {
        return Ok(Vec::new());
    };
    let rules = RuleRegistry::from_root(&mut root)?;
    let Some(Value::Mapping(structure)) = root.get(string_value(STRUCTURE)) else {
        return Ok(Vec::new());
    };
    let mut uses = BTreeSet::new();
    collect_tree_rule_uses(structure, ".", &rules, &mut Vec::new(), &mut uses)?;
    Ok(uses.into_iter().collect())
}

fn collect_tree_rule_uses(
    mapping: &Mapping,
    base: &str,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
    uses: &mut BTreeSet<super::AuthoredRuleUse>,
) -> Result<(), String> {
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            continue;
        };
        if key == USE {
            collect_rule_value_uses(value, base, "directory", rules, stack, uses)?;
            continue;
        }
        if is_node_attr_key(key) || matches!(key, EXTRA | NEEDS | PROVIDES | SECTIONS) {
            continue;
        }

        if matches!(key, "." | "./") {
            collect_rule_value_uses(value, base, "directory", rules, stack, uses)?;
            continue;
        }

        if let Some(selectors) = expand_finite_selector(key)? {
            for selector in selectors {
                let mut expanded = Mapping::new();
                expanded.insert(Value::String(selector), value.clone());
                collect_tree_rule_uses(&expanded, base, rules, stack, uses)?;
            }
            continue;
        }

        let is_directory = key.ends_with('/');
        let selector = if is_directory {
            join_scope_path(base, key.trim_end_matches('/'))
        } else if is_extension_key(key) {
            scoped_file_pattern(base, &format!("*{key}"))
        } else if is_explicit_file_glob(key) {
            scoped_file_pattern(base, key)
        } else {
            scoped_direct_file_pattern(base, key)
        };
        let target_kind = if is_directory { "directory" } else { "file" };
        collect_rule_value_uses(value, &selector, target_kind, rules, stack, uses)?;
        if let Value::Mapping(nested) = value {
            if is_directory {
                collect_tree_rule_uses(nested, &selector, rules, stack, uses)?;
            }
        }
    }
    Ok(())
}

fn collect_rule_value_uses(
    value: &Value,
    selector: &str,
    target_kind: &str,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
    uses: &mut BTreeSet<super::AuthoredRuleUse>,
) -> Result<(), String> {
    let Some(text) = value.as_str() else {
        return Ok(());
    };
    for token in split_composition(text)? {
        if !is_rule_reference(token) {
            continue;
        }
        let name = parse_rule_reference(token)?;
        push_rule_stack(stack, name)?;
        match rules.fragment_kind(token)? {
            FragmentKind::Node => {
                uses.insert(super::AuthoredRuleUse {
                    rule: name.to_string(),
                    selector: normalize_provenance_selector(selector),
                    target_kind: target_kind.to_string(),
                });
            }
            FragmentKind::Tree => {
                stack.push(name.to_string());
                let tree = rules.resolve_tree(token)?;
                collect_tree_rule_uses(&tree, selector, rules, stack, uses)?;
                stack.pop();
            }
        }
    }
    Ok(())
}

fn normalize_provenance_selector(selector: &str) -> String {
    let selector = selector.trim_start_matches("./").trim_end_matches('/');
    if selector.is_empty() || selector == "." {
        ".".to_string()
    } else {
        selector.to_string()
    }
}

fn normalize_structure(
    value: Value,
    rules: &RuleRegistry,
    relationships: &mut Vec<RelationshipSpec>,
) -> Result<Value, String> {
    let Value::Mapping(mapping) = value else {
        return Err("Assura config structure must be a mapping".to_string());
    };

    if !looks_like_legacy_structure(&mapping) {
        let mut stack = Vec::new();
        let expanded = expand_tree_mapping(mapping, rules, &mut stack)?;
        let node =
            normalize_expanded_structure_node(expanded, rules, &mut stack, ".", relationships)?;
        let mut normalized = Mapping::new();
        normalized.insert(string_value("./"), node);
        return Ok(Value::Mapping(normalized));
    }

    let mut normalized = Mapping::new();
    for (key, node_value) in mapping {
        let Some(path) = key.as_str() else {
            return Err("Assura config structure paths must be strings".to_string());
        };
        reject_removed_capture_syntax(path)?;
        let mut stack = Vec::new();
        let Value::Mapping(node_mapping) = node_value else {
            return Err("Assura config structure nodes must be mappings".to_string());
        };
        let mut expanded = expand_tree_mapping(node_mapping, rules, &mut stack)?;
        let scope_exists = take_scope_cardinality(&mut expanded)?;
        validate_top_level_scope_cardinality(path, scope_exists.as_deref())?;
        let node = normalize_expanded_structure_node(
            expanded,
            rules,
            &mut stack,
            &normalize_scope_path(path),
            relationships,
        )?;
        let mut node_mapping = into_mapping(node, "structure node")?;
        if let Some(exists) = scope_exists {
            node_mapping.insert(string_value("required"), Value::Bool(false));
            set_nested_mapping(
                &mut node_mapping,
                "self_directory",
                "exists",
                mapping_entry("*", exists),
            );
        } else if path_has_scope_magic(path) {
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

fn looks_like_legacy_structure(mapping: &Mapping) -> bool {
    if mapping
        .get(string_value("./"))
        .is_some_and(Value::is_mapping)
    {
        return true;
    }

    mapping.iter().all(|(key, value)| {
        key.as_str().is_some_and(|path| !path.starts_with("./")) && value.is_mapping()
    })
}

fn take_scope_cardinality(mapping: &mut Mapping) -> Result<Option<String>, String> {
    let key = string_value("exists");
    let Some(value) = mapping.get(&key) else {
        return Ok(None);
    };
    if value.is_mapping() {
        return Ok(None);
    }
    let value = mapping.remove(&key).expect("exists value was present");
    parse_exists_value(value).map(Some)
}

fn normalize_structure_node(
    value: Value,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
    scope_path: &str,
    relationships: &mut Vec<RelationshipSpec>,
) -> Result<Value, String> {
    let Value::Mapping(mapping) = value else {
        return Err("Assura config structure nodes must be mappings".to_string());
    };
    let expanded = expand_tree_mapping(mapping, rules, stack)?;
    normalize_expanded_structure_node(expanded, rules, stack, scope_path, relationships)
}

fn normalize_expanded_structure_node(
    expanded: Mapping,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
    scope_path: &str,
    relationships: &mut Vec<RelationshipSpec>,
) -> Result<Value, String> {
    let mut output = Mapping::new();

    for (key, value) in expanded {
        let Some(key) = key.as_str() else {
            return Err("Assura config structure node keys must be strings".to_string());
        };
        reject_removed_capture_syntax(key)?;
        if matches!(key, USE | NEEDS | PROVIDES | SECTIONS) {
            continue;
        }
        if key == "required" {
            return Err(
                "Assura config structure nodes no longer use 'required'; omit it for an exact required path or use exists:0-1 for an optional path"
                    .to_string(),
            );
        }
        if key == EXTRA {
            let Some(allow_extra) = value.as_bool() else {
                return Err("Assura config extra must be a boolean".to_string());
            };
            set_nested_bool(&mut output, "files", "allow_extra", allow_extra);
            set_nested_bool(&mut output, "directories", "allow_extra", allow_extra);
            continue;
        }
        if is_directory_node_attr_key(key) {
            insert_directory_node_attr(&mut output, key, value, rules, stack, relationships)?;
            continue;
        }

        normalize_path_key(
            &mut output,
            scope_path,
            key,
            value,
            rules,
            stack,
            relationships,
        )?;
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
    scope_path: &str,
    key: &str,
    value: Value,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
    relationships: &mut Vec<RelationshipSpec>,
) -> Result<(), String> {
    if let Some(expanded) = expand_finite_selector(key)? {
        for selector in expanded {
            normalize_path_key(
                output,
                scope_path,
                &selector,
                value.clone(),
                rules,
                stack,
                relationships,
            )?;
        }
        return Ok(());
    }

    if matches!(key, "." | "./") {
        let (tree, directive) = directory_composition(value, key, rules, stack)?;
        if let Some(tree) = tree {
            let normalized =
                normalize_expanded_structure_node(tree, rules, stack, scope_path, relationships)?;
            merge_mapping(output, into_mapping(normalized, "current directory rule")?);
        }
        apply_self_directory_directive(output, directive)?;
        return Ok(());
    }

    if key == ".dir" {
        let directive = node_directive(value, rules, stack)?;
        apply_self_directory_directive(output, directive)?;
        return Ok(());
    }

    if is_explicit_file_glob(key) {
        let local_pattern = key.strip_prefix("./").unwrap_or(key);
        let scoped_pattern = scoped_file_pattern(scope_path, key);
        let directive = node_directive(value, rules, stack)?;
        apply_file_pattern_directive(output, local_pattern, &scoped_pattern, directive)?;
        return Ok(());
    }

    if is_extension_key(key) {
        let pattern = format!("*{key}");
        let directive = node_directive(value, rules, stack)?;
        apply_file_pattern_directive(output, &pattern, &pattern, directive)?;
        return Ok(());
    }

    let child_path = join_scope_path(scope_path, key);
    let key_has_captures = !capture_names(key).is_empty();
    record_relationship_metadata(&child_path, key_has_captures, &value, relationships)?;

    if key.ends_with('/') {
        let directory = key
            .trim_end_matches('/')
            .strip_prefix("./")
            .unwrap_or_else(|| key.trim_end_matches('/'));
        if is_rule_reference_value(&value)
            || is_tree_value(&value)?
            || value.as_str().is_some_and(|text| text.contains(" | "))
        {
            let child_scope = join_scope_path(scope_path, directory);
            let (expanded, mut directive) = directory_composition(value, key, rules, stack)?;
            let expanded = expanded.unwrap_or_default();
            validate_directory_cardinality(key, directive.exists.as_deref(), !expanded.is_empty())?;
            let parent_exists = directive.exists.take();
            let allows_parent_match = parent_exists.as_deref() != Some("0");
            let mut child = into_mapping(
                normalize_expanded_structure_node(
                    expanded,
                    rules,
                    stack,
                    &child_scope,
                    relationships,
                )?,
                key,
            )?;
            apply_self_directory_directive(&mut child, directive)?;
            let nested_self_exists = child
                .get(string_value("self_directory"))
                .and_then(Value::as_mapping)
                .and_then(|directory| directory.get(string_value("exists")))
                .and_then(Value::as_mapping)
                .and_then(|exists| exists.get(string_value("*")))
                .and_then(Value::as_str)
                .map(str::to_string);
            child.insert(string_value("required"), Value::Bool(false));
            mark_nested_scope_match_only(&mut child);
            let exists = parent_exists
                .or(nested_self_exists)
                .or_else(|| (!path_has_scope_magic(key)).then(|| "1".to_string()));
            if let Some(exists) = exists {
                apply_directory_directive(
                    output,
                    directory,
                    NodeDirective {
                        exists: Some(exists),
                        ..NodeDirective::default()
                    },
                )?;
            }
            if path_has_scope_magic(directory) && allows_parent_match {
                append_nested_sequence(output, "directories", "allowed_patterns", directory);
            }
            set_nested_mapping(output, "children", directory, Value::Mapping(child));
        } else {
            let mut directive = node_directive(value, rules, stack)?;
            if directive.naming.is_some() && path_has_scope_magic(key) {
                let child_scope = join_scope_path(scope_path, directory);
                let mut child = Mapping::new();
                apply_self_directory_directive(&mut child, directive)?;
                child.insert(string_value("required"), Value::Bool(false));
                mark_nested_scope_match_only(&mut child);
                set_nested_mapping(output, "children", directory, Value::Mapping(child));
                let _ = child_scope;
                return Ok(());
            }
            if directive.exists.is_none() && !path_has_scope_magic(key) {
                directive.exists = Some("1".to_string());
            }
            validate_directory_cardinality(key, directive.exists.as_deref(), false)?;
            if key_has_captures {
                apply_captured_directory_directive(output, directory, directive)?;
            } else {
                apply_directory_directive(output, directory, directive)?;
            }
        }
        return Ok(());
    }

    let mut directive = node_directive(value, rules, stack)?;
    if key_has_captures {
        let scoped_pattern = scoped_direct_file_pattern(scope_path, key);
        apply_captured_file_directive(output, key, &scoped_pattern, directive)?;
    } else {
        if directive.exists.is_none() && !path_has_scope_magic(key) {
            directive.exists = Some("1".to_string());
        }
        validate_literal_cardinality(key, directive.exists.as_deref())?;
        let scoped_pattern = scoped_direct_file_pattern(scope_path, key);
        apply_file_directive(output, key, &scoped_pattern, directive)?;
    }
    Ok(())
}

fn mark_nested_scope_match_only(child: &mut Mapping) {
    let has_self_count = child
        .get(string_value("self_directory"))
        .and_then(Value::as_mapping)
        .and_then(|directory| directory.get(string_value("exists")))
        .and_then(Value::as_mapping)
        .is_some_and(|exists| !exists.is_empty());
    if !has_self_count {
        set_nested_mapping(child, "self_directory", "exists", mapping_entry("*", "0-1"));
    }
}

fn directory_composition(
    value: Value,
    key: &str,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
) -> Result<(Option<Mapping>, NodeDirective), String> {
    match value {
        Value::String(text) => {
            let mut tree = Mapping::new();
            let mut directive = NodeDirective::default();
            for token in split_composition(&text)? {
                if is_rule_reference(token) {
                    match rules.fragment_kind(token)? {
                        FragmentKind::Node => {
                            merge_node_directive(
                                &mut directive,
                                node_directive(Value::String(token.to_string()), rules, stack)?,
                            )?;
                        }
                        FragmentKind::Tree => {
                            let name = parse_rule_reference(token)?;
                            push_rule_stack(stack, name)?;
                            stack.push(name.to_string());
                            let expanded = expand_tree_mapping(rules.resolve_tree(token)?, rules, stack)?;
                            stack.pop();
                            merge_mapping(&mut tree, expanded);
                        }
                    }
                } else {
                    merge_node_directive(
                        &mut directive,
                        node_directive(Value::String(token.to_string()), rules, stack)?,
                    )?;
                }
            }
            Ok(((!tree.is_empty()).then_some(tree), directive))
        }
        Value::Mapping(mapping) => {
            let expanded = expand_tree_mapping(mapping, rules, stack)?;
            let mut expanded = expanded;
            let exists = take_scope_cardinality(&mut expanded)?;
            Ok((Some(expanded), NodeDirective { exists, ..NodeDirective::default() }))
        }
        other => Err(format!(
            "Assura config directory tree '{key}' must be a mapping or composed directive, got {other:?}"
        )),
    }
}

fn insert_directory_node_attr(
    output: &mut Mapping,
    key: &str,
    value: Value,
    rules: &RuleRegistry,
    stack: &mut Vec<String>,
    relationships: &mut Vec<RelationshipSpec>,
) -> Result<(), String> {
    if key == "children" {
        let Value::Mapping(children) = value else {
            return Err("Assura config children must be a mapping".to_string());
        };
        let mut normalized = Mapping::new();
        for (child_key, child_value) in children {
            let Some(child_name) = child_key.as_str() else {
                return Err("Assura config child names must be strings".to_string());
            };
            reject_removed_capture_syntax(child_name)?;
            let mut child = into_mapping(
                normalize_structure_node(child_value, rules, stack, child_name, relationships)?,
                key,
            )?;
            if path_has_scope_magic(child_name) {
                child
                    .entry(string_value("required"))
                    .or_insert(Value::Bool(false));
            }
            normalized.insert(Value::String(child_name.to_string()), Value::Mapping(child));
        }
        let target = ensure_mapping(output, key);
        merge_mapping(target, normalized);
    } else {
        merge_directory_node_attr(output, key, value);
    }
    Ok(())
}

fn merge_directory_node_attr(output: &mut Mapping, key: &str, value: Value) {
    let key_value = string_value(key);
    let Some(existing) = output.remove(&key_value) else {
        output.insert(key_value, value);
        return;
    };

    let merged = match (existing, value) {
        (Value::Mapping(mut target), Value::Mapping(source)) => {
            merge_mapping(&mut target, source);
            Value::Mapping(target)
        }
        (_, value) => value,
    };
    output.insert(key_value, merged);
}

include!("structure_notation/directives.rs");
include!("structure_notation/relationships.rs");
include!("structure_notation/helpers.rs");
include!("structure_notation/rule_fragments.rs");
#[cfg(test)]
mod tests;
