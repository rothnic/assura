/// Normalized directive attached to a concise file or directory key.
#[derive(Debug, Default)]
struct NodeDirective {
    exists: Option<String>,
    naming: Option<String>,
    attributes: Mapping,
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
            supported if is_node_attr_key(supported) => {
                directive
                    .attributes
                    .insert(Value::String(supported.to_string()), value);
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
    attribute_pattern: &str,
    directive: NodeDirective,
) -> Result<(), String> {
    if directive.naming.is_some() {
        return Err(format!(
            "Assura config exact file key '{filename}' only supports exists"
        ));
    }
    apply_file_pattern_attributes(output, attribute_pattern, directive.attributes)?;
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

fn apply_file_pattern_directive(
    output: &mut Mapping,
    local_pattern: &str,
    attribute_pattern: &str,
    directive: NodeDirective,
) -> Result<(), String> {
    if directive.exists.is_some() && local_pattern.contains('/') {
        return Err(format!(
            "Assura config file glob '{local_pattern}' cannot use exists across directories; place a direct-child exists rule in the matching structure scope"
        ));
    }
    apply_file_pattern_attributes(output, attribute_pattern, directive.attributes)?;
    let allows_by_count = directive.exists.is_some() && directive.naming.is_none();
    if let Some(naming) = directive.naming {
        set_nested_mapping(
            output,
            "files",
            "naming_patterns",
            mapping_entry(attribute_pattern, naming),
        );
    }
    if let Some(exists) = directive.exists {
        set_nested_mapping(
            output,
            "files",
            "exists",
            mapping_entry(local_pattern, exists),
        );
    }
    if allows_by_count {
        append_nested_sequence(output, "files", "allowed_patterns", local_pattern);
    }
    Ok(())
}

fn apply_captured_file_directive(
    output: &mut Mapping,
    local_pattern: &str,
    attribute_pattern: &str,
    directive: NodeDirective,
) -> Result<(), String> {
    apply_file_pattern_attributes(output, attribute_pattern, directive.attributes)?;
    if let Some(naming) = directive.naming {
        set_nested_mapping(
            output,
            "files",
            "naming_patterns",
            mapping_entry(attribute_pattern, naming),
        );
    }
    if directive.exists.as_deref() != Some("0") {
        append_nested_sequence(output, "files", "allowed_patterns", local_pattern);
    }
    Ok(())
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
    apply_directory_attributes(output, directive.attributes)?;
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
    apply_directory_attributes(output, directive.attributes)?;
    if let Some(exists) = directive.exists.as_ref() {
        set_nested_mapping(
            output,
            "directories",
            "exists",
            mapping_entry(directory, exists),
        );
    }
    if directive.exists.as_deref() != Some("0") {
        append_nested_sequence(output, "directories", "allowed_patterns", directory);
    }
    Ok(())
}

fn apply_self_directory_directive(
    output: &mut Mapping,
    directive: NodeDirective,
) -> Result<(), String> {
    apply_directory_attributes(output, directive.attributes)?;
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
    Ok(())
}

fn is_tree_value(value: &Value) -> Result<bool, String> {
    let Value::Mapping(mapping) = value else {
        return Ok(false);
    };

    for key in mapping.keys() {
        let Some(key) = key.as_str() else {
            return Err("Assura config fragments must use string keys".to_string());
        };
        if key == USE
            || key == EXTRA
            || (is_directory_node_attr_key(key) && key != "exists")
            || !is_node_attr_key(key)
        {
            return Ok(true);
        }
    }

    Ok(false)
}

fn is_rule_reference_value(value: &Value) -> bool {
    value.as_str().is_some_and(is_rule_reference)
}

fn apply_file_attributes(output: &mut Mapping, attributes: Mapping) -> Result<(), String> {
    for (key, value) in attributes {
        let Some(key) = key.as_str() else {
            return Err("Assura config file attributes must use string keys".to_string());
        };
        match key {
            "markdown" => merge_top_level_attr(output, "markdown", value)?,
            "outline" | "validate" => merge_nested_attr(output, "markdown", key, value),
            "files" => merge_top_level_attr(output, "files", value)?,
            "directories" | "self_directory" | "children" | "inherit" => {
                return Err(format!(
                    "Assura config file key cannot use directory attribute '{key}'"
                ));
            }
            key if is_file_bundle_attr_key(key) => merge_nested_attr(output, "files", key, value),
            key => {
                return Err(format!(
                    "Assura config file attribute '{key}' is not supported in this notation"
                ));
            }
        }
    }
    Ok(())
}

fn apply_file_pattern_attributes(
    output: &mut Mapping,
    pattern: &str,
    mut attributes: Mapping,
) -> Result<(), String> {
    for (attribute, target) in [
        ("max_lines", "max_lines_patterns"),
        ("max_size", "max_size_patterns"),
    ] {
        if let Some(value) = attributes.remove(string_value(attribute)) {
            let mut entry = Mapping::new();
            entry.insert(string_value(pattern), value);
            set_nested_mapping(output, "files", target, Value::Mapping(entry));
        }
    }
    apply_file_attributes(output, attributes)
}

fn apply_directory_attributes(output: &mut Mapping, attributes: Mapping) -> Result<(), String> {
    for (key, value) in attributes {
        let Some(key) = key.as_str() else {
            return Err("Assura config directory attributes must use string keys".to_string());
        };
        match key {
            "directories" => merge_top_level_attr(output, "directories", value)?,
            key if is_directory_bundle_attr_key(key) => {
                merge_nested_attr(output, "directories", key, value);
            }
            "files" | "markdown" | "self_directory" | "children" | "inherit" => {
                return Err(format!(
                    "Assura config directory shorthand cannot use scope attribute '{key}'; expand the directory as a nested structure node"
                ));
            }
            key => {
                return Err(format!(
                    "Assura config directory attribute '{key}' is not supported in this notation"
                ));
            }
        }
    }
    Ok(())
}

fn merge_top_level_attr(output: &mut Mapping, key: &str, value: Value) -> Result<(), String> {
    let Value::Mapping(value) = value else {
        return Err(format!("Assura config {key} attribute must be a mapping"));
    };
    let target = ensure_mapping(output, key);
    merge_mapping(target, value);
    Ok(())
}

fn merge_nested_attr(output: &mut Mapping, parent: &str, key: &str, value: Value) {
    match value {
        Value::Mapping(mapping) => set_nested_mapping(output, parent, key, Value::Mapping(mapping)),
        value => set_nested_value(output, parent, key, value),
    }
}
