/// Record relationship metadata from one concise structure path.
fn record_relationship_metadata(
    path: &str,
    value: &Value,
    relationships: &mut Vec<RelationshipSpec>,
) -> Result<(), String> {
    let captures = capture_names(path);
    let Value::Mapping(mapping) = value else {
        if !captures.is_empty() {
            if let Value::String(text) = value {
                if parse_exists_shorthand(text)?.as_deref() == Some("1") {
                    relationships.push(RelationshipSpec::required_counterpart(path, captures));
                    return Ok(());
                }
            }
            relationships.push(RelationshipSpec::implicit_producer(path, captures));
        }
        return Ok(());
    };

    let exists = mapping
        .get(string_value("exists"))
        .cloned()
        .map(parse_exists_value)
        .transpose()?;
    let needs = mapping
        .get(string_value(NEEDS))
        .map(relation_names)
        .transpose()?
        .unwrap_or_default();
    let provides = mapping
        .get(string_value(PROVIDES))
        .map(relation_names)
        .transpose()?
        .unwrap_or_default();

    if !captures.is_empty() {
        if exists.is_none() || !needs.is_empty() {
            relationships.push(RelationshipSpec::implicit_producer(path, captures.clone()));
        }
        for need in needs {
            relationships.push(RelationshipSpec::need(path, captures.clone(), need));
        }
        for provided in provides {
            relationships.push(RelationshipSpec::provider(
                path,
                captures.clone(),
                provided,
                None,
            ));
        }
        if exists.as_deref() == Some("1") && mapping.get(string_value(PROVIDES)).is_none() {
            relationships.push(RelationshipSpec::required_counterpart(
                path,
                captures.clone(),
            ));
        }
    }

    if let Some(sections) = mapping.get(string_value(SECTIONS)) {
        let Value::Mapping(sections) = sections else {
            return Err("Assura config sections must be a mapping".to_string());
        };
        for (section_key, section_value) in sections {
            let Some(section) = section_key.as_str() else {
                return Err("Assura config section names must be strings".to_string());
            };
            let mut section_captures = captures.clone();
            section_captures.extend(capture_names(section));
            let Value::Mapping(section_mapping) = section_value else {
                continue;
            };
            let provides = section_mapping
                .get(string_value(PROVIDES))
                .map(relation_names)
                .transpose()?
                .unwrap_or_default();
            for provided in provides {
                relationships.push(RelationshipSpec::provider(
                    path,
                    section_captures.clone(),
                    provided,
                    Some(section.to_string()),
                ));
            }
        }
    }

    Ok(())
}

fn insert_relationships(root: &mut Mapping, specs: Vec<RelationshipSpec>) -> Result<(), String> {
    let constraints = compile_relationship_specs(specs)?;
    if constraints.is_empty() {
        return Ok(());
    }

    let extensions = root
        .entry(string_value(EXTENSIONS))
        .or_insert_with(|| Value::Mapping(Mapping::new()));
    let Value::Mapping(extension_map) = extensions else {
        return Err("Assura config extensions must be a mapping".to_string());
    };
    let relationships = extension_map
        .entry(string_value(RELATIONSHIPS))
        .or_insert_with(|| Value::Sequence(Vec::new()));
    let Value::Sequence(sequence) = relationships else {
        return Err("Assura config extensions.relationships must be a sequence".to_string());
    };
    sequence.extend(constraints);
    Ok(())
}

fn compile_relationship_specs(specs: Vec<RelationshipSpec>) -> Result<Vec<Value>, String> {
    let producers = specs
        .iter()
        .filter(|spec| matches!(spec.kind, RelationshipKind::ImplicitProducer))
        .collect::<Vec<_>>();
    let counterparts = specs
        .iter()
        .filter(|spec| matches!(spec.kind, RelationshipKind::RequiredCounterpart))
        .collect::<Vec<_>>();
    let needs = specs
        .iter()
        .filter_map(|spec| {
            if let RelationshipKind::Need(name) = &spec.kind {
                Some((spec, name.as_str()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let providers = specs
        .iter()
        .filter_map(|spec| {
            if let RelationshipKind::Provider(name) = &spec.kind {
                Some((spec, name.as_str()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut output = Vec::new();
    let mut index = 0usize;
    for counterpart in counterparts {
        for producer in producers.iter().copied() {
            if producer.path == counterpart.path || producer.captures != counterpart.captures {
                continue;
            }
            index += 1;
            output.push(relationship_value(
                &format!("captured-counterpart-{index}"),
                &producer.path,
                &format!("counterpart-{index}"),
                vec![provider_value(&counterpart.path, None)],
            ));
        }
    }

    for (need_source, need) in needs {
        let alternatives = providers
            .iter()
            .filter(|(provider, provided)| {
                *provided == need && provider.captures == need_source.captures
            })
            .map(|(provider, _)| provider_value(&provider.path, provider.section.as_deref()))
            .collect::<Vec<_>>();
        if alternatives.is_empty() {
            return Err(format!(
                "Assura config relationship need '{need}' at '{}' has no matching provider",
                need_source.path
            ));
        }
        index += 1;
        output.push(relationship_value(
            &format!("captured-{need}-{index}"),
            &need_source.path,
            need,
            alternatives,
        ));
    }

    Ok(output)
}

fn relationship_value(id: &str, source: &str, need: &str, providers: Vec<Value>) -> Value {
    let mut mapping = Mapping::new();
    mapping.insert(string_value("id"), Value::String(id.to_string()));
    mapping.insert(string_value("source"), Value::String(source.to_string()));
    mapping.insert(string_value("need"), Value::String(need.to_string()));
    mapping.insert(string_value("providers"), Value::Sequence(providers));
    Value::Mapping(mapping)
}

fn provider_value(path: &str, section: Option<&str>) -> Value {
    let mut mapping = Mapping::new();
    mapping.insert(string_value("path"), Value::String(path.to_string()));
    if let Some(section) = section {
        mapping.insert(string_value("section"), Value::String(section.to_string()));
    }
    Value::Mapping(mapping)
}

#[derive(Debug, Clone)]
struct RelationshipSpec {
    path: String,
    captures: BTreeSet<String>,
    kind: RelationshipKind,
    section: Option<String>,
}

#[derive(Debug, Clone)]
enum RelationshipKind {
    ImplicitProducer,
    RequiredCounterpart,
    Need(String),
    Provider(String),
}

impl RelationshipSpec {
    fn implicit_producer(path: &str, captures: BTreeSet<String>) -> Self {
        Self {
            path: path.to_string(),
            captures,
            kind: RelationshipKind::ImplicitProducer,
            section: None,
        }
    }

    fn required_counterpart(path: &str, captures: BTreeSet<String>) -> Self {
        Self {
            path: path.to_string(),
            captures,
            kind: RelationshipKind::RequiredCounterpart,
            section: None,
        }
    }

    fn need(path: &str, captures: BTreeSet<String>, name: String) -> Self {
        Self {
            path: path.to_string(),
            captures,
            kind: RelationshipKind::Need(name),
            section: None,
        }
    }

    fn provider(
        path: &str,
        captures: BTreeSet<String>,
        name: String,
        section: Option<String>,
    ) -> Self {
        Self {
            path: path.to_string(),
            captures,
            kind: RelationshipKind::Provider(name),
            section,
        }
    }
}

fn relation_names(value: &Value) -> Result<Vec<String>, String> {
    match value {
        Value::String(name) => Ok(vec![name.to_string()]),
        Value::Sequence(names) => names
            .iter()
            .map(|entry| {
                entry
                    .as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "Assura config relationship names must be strings".to_string())
            })
            .collect(),
        Value::Mapping(mapping) => mapping
            .keys()
            .map(|key| {
                key.as_str()
                    .map(str::to_string)
                    .ok_or_else(|| "Assura config relationship names must be strings".to_string())
            })
            .collect(),
        _ => Err("Assura config relationship names must be a string, list, or map".to_string()),
    }
}
