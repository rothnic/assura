/// Normalized directive attached to a concise file or directory key.
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
            "Assura config node directive numbers are not supported; use exists:N or { exists: N }"
                .to_string(),
        ),
        Value::Mapping(mapping) => node_directive_from_mapping(mapping),
        other => Err(format!(
            "Assura config node directives must be strings or mappings, got {other:?}"
        )),
    }
}

fn node_directive_from_mapping(mapping: Mapping) -> Result<NodeDirective, String> {
    let mut directive = NodeDirective::default();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err("Assura config node attributes must use string keys".to_string());
        };
        match key {
            "exists" => directive.exists = Some(parse_exists_value(value)?),
            "naming" => {
                let Some(naming) = value.as_str() else {
                    return Err("Assura config naming must be a string".to_string());
                };
                directive.naming = Some(naming.to_string());
            }
            USE | NEEDS | PROVIDES | SECTIONS => {}
            unsupported if is_node_attr_key(unsupported) => {
                return Err(format!(
                    "Assura config node attribute '{unsupported}' is not supported in this notation"
                ));
            }
            unsupported => {
                return Err(format!(
                    "Assura config node attribute '{unsupported}' is not supported in this notation"
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
            "Assura config exact file key '{filename}' only supports exists"
        ));
    }
    if let Some(exists) = directive.exists {
        set_nested_mapping(output, "files", "exists", mapping_entry(filename, exists));
        if filename_count_is_allowed(output, "files", filename) {
            if path_has_scope_magic(filename) {
                append_nested_sequence(output, "files", "allowed_patterns", filename);
            } else {
                append_nested_sequence(output, "files", "allowed_names", filename);
            }
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

fn apply_captured_file_directive(output: &mut Mapping, pattern: &str, directive: NodeDirective) {
    if let Some(naming) = directive.naming {
        set_nested_mapping(
            output,
            "files",
            "naming_patterns",
            mapping_entry(pattern, naming),
        );
    }
    if directive.exists.as_deref() != Some("0") {
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
            "Assura config exact directory key '{directory}/' only supports exists"
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

fn apply_captured_directory_directive(
    output: &mut Mapping,
    directory: &str,
    directive: NodeDirective,
) -> Result<(), String> {
    if directive.naming.is_some() {
        return Err(format!(
            "Assura config captured directory key '{directory}/' only supports exists"
        ));
    }
    if directive.exists.as_deref() != Some("0") {
        append_nested_sequence(output, "directories", "allowed_patterns", directory);
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
            return Err("Assura config fragments must use string keys".to_string());
        };
        if key == USE || key == EXTRA || !is_node_attr_key(key) {
            return Ok(true);
        }
    }

    Ok(false)
}
