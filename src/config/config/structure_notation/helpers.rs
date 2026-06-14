/// Parse one or more reusable rule references from a `use:` directive.
fn use_references(value: &Value) -> Result<Vec<String>, String> {
    match value {
        Value::String(reference) => Ok(vec![reference.to_string()]),
        Value::Sequence(sequence) => sequence
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "Assura config use entries must be strings".to_string())
            })
            .collect(),
        _ => Err("Assura config use must be a string or list of strings".to_string()),
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
            "Assura config exists value must be a string or number, got {other:?}"
        )),
    }
}

fn parse_exists_shorthand(value: &str) -> Result<Option<String>, String> {
    if value == "exists" {
        return Err(
            "Assura config exists shorthand must include cardinality; use exists:1".to_string(),
        );
    }
    let Some(raw) = value.strip_prefix("exists:") else {
        return Ok(None);
    };
    parse_exists_count(raw).map(Some)
}

fn parse_exists_count(raw: &str) -> Result<String, String> {
    if raw.is_empty() {
        return Err("Assura config exists value must not be empty".to_string());
    }
    let parts = raw.split('-').collect::<Vec<_>>();
    if parts.len() > 2 {
        return Err(format!(
            "Assura config exists value '{raw}' must be N or N-M"
        ));
    }
    let mut bounds = Vec::new();
    for part in parts {
        if part.is_empty() {
            return Err(format!(
                "Assura config exists value '{raw}' has an empty range bound"
            ));
        }
        let bound = part.parse::<u16>().map_err(|error| {
            format!("Assura config exists value '{raw}' has an invalid bound: {error}")
        })?;
        bounds.push(bound);
    }
    if bounds.len() == 2 && bounds[0] > bounds[1] {
        return Err(format!(
            "Assura config exists value '{raw}' has a lower bound greater than its upper bound"
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

fn merge_mapping(target: &mut Mapping, source: Mapping) {
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

fn set_nested_bool(output: &mut Mapping, parent: &str, key: &str, value: bool) {
    set_nested_value(output, parent, key, Value::Bool(value));
}

fn set_nested_value(output: &mut Mapping, parent: &str, key: &str, value: Value) {
    let parent = ensure_mapping(output, parent);
    parent.insert(string_value(key), value);
}

fn set_nested_mapping(output: &mut Mapping, parent: &str, key: &str, value: Value) {
    let Value::Mapping(value) = value else {
        return;
    };
    let parent = ensure_mapping(output, parent);
    let target = ensure_mapping(parent, key);
    merge_mapping(target, value);
}

fn append_nested_sequence(output: &mut Mapping, parent: &str, key: &str, item: &str) {
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

fn mapping_entry(key: &str, value: impl Into<String>) -> Value {
    let mut mapping = Mapping::new();
    mapping.insert(Value::String(key.to_string()), Value::String(value.into()));
    Value::Mapping(mapping)
}

fn into_mapping(value: Value, context: &str) -> Result<Mapping, String> {
    let Value::Mapping(mapping) = value else {
        return Err(format!("Assura config {context} must be a mapping"));
    };
    Ok(mapping)
}

fn string_value(value: &str) -> Value {
    Value::String(value.to_string())
}

fn is_extension_key(key: &str) -> bool {
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

fn is_node_attr_key(key: &str) -> bool {
    matches!(
        key,
        "exists"
            | "naming"
            | NEEDS
            | PROVIDES
            | SECTIONS
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

fn is_directory_node_attr_key(key: &str) -> bool {
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

fn path_has_scope_magic(path: &str) -> bool {
    path.contains('*') || path.contains('?') || path.contains('[') || path.contains('{')
}

fn normalize_scope_path(path: &str) -> String {
    path.trim_end_matches('/')
        .strip_prefix("./")
        .unwrap_or_else(|| path.trim_end_matches('/'))
        .to_string()
}

fn join_scope_path(scope: &str, child: &str) -> String {
    let child = child.trim_end_matches('/');
    if scope.is_empty() || scope == "." || scope == "./" {
        child.to_string()
    } else {
        format!("{}/{}", scope.trim_end_matches('/'), child)
    }
}

fn capture_names(pattern: &str) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut rest = pattern;
    while let Some(start) = rest.find('{') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('}') else {
            break;
        };
        let name = &after_start[..end];
        if !name.is_empty()
            && name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
        {
            names.insert(name.to_string());
        }
        rest = &after_start[end + 1..];
    }
    names
}

fn reject_removed_capture_syntax(value: &str) -> Result<(), String> {
    if value.contains("${") || value.contains("{{") || value.contains("}}") {
        return Err(
            "Assura config captures use single braces like {name}; ${name} and {{name}} are not supported"
                .to_string(),
        );
    }
    Ok(())
}
