/// Reusable rule fragments declared at the top level of an Assura config.
#[derive(Default)]
struct RuleRegistry {
    rules: HashMap<String, Value>,
}

impl RuleRegistry {
    fn from_root(root: &mut Mapping) -> Result<Self, String> {
        let mut rules = HashMap::new();
        let Some(value) = root.remove(string_value("rules")) else {
            let registry = Self { rules };
            registry.validate_references()?;
            return Ok(registry);
        };
        let Value::Mapping(mapping) = value else {
            return Err("Assura config rules must be a mapping".to_string());
        };

        for (key, value) in mapping {
            let Some(name) = key.as_str() else {
                return Err("Assura config rule names must be strings".to_string());
            };
            let name = normalize_rule_definition_name(name)?;
            validate_authored_rule_shape(name, &value)?;
            rules.insert(name.to_string(), value);
        }
        let registry = Self { rules };
        registry.validate_references()?;
        Ok(registry)
    }

    fn resolve_node(&self, reference: &str) -> Result<Value, String> {
        let name = parse_rule_reference(reference)?;
        let value = self.resolve(name)?;
        match classify_fragment(&value)? {
            FragmentKind::Node => Ok(value),
            FragmentKind::Tree => Err(format!(
                "Assura config rule '${name}' is a tree fragment but a node fragment is required"
            )),
        }
    }

    fn resolve_tree(&self, reference: &str) -> Result<Mapping, String> {
        let name = parse_rule_reference(reference)?;
        let value = self.resolve(name)?;
        match classify_fragment(&value)? {
            FragmentKind::Tree => {
                let Value::Mapping(mapping) = value else {
                    return Err(format!(
                        "Assura config rule '${name}' must be a mapping when used through use"
                    ));
                };
                Ok(mapping)
            }
            FragmentKind::Node => Err(format!(
                "Assura config rule '${name}' is a node fragment but a tree fragment is required"
            )),
        }
    }

    fn resolve(&self, name: &str) -> Result<Value, String> {
        self.rules
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown Assura config rule '${name}'"))
    }

    fn fragment_kind(&self, reference: &str) -> Result<FragmentKind, String> {
        let name = parse_rule_reference(reference)?;
        classify_fragment(&self.resolve(name)?)
    }

    fn validate_references(&self) -> Result<(), String> {
        let mut names = self.rules.keys().cloned().collect::<Vec<_>>();
        names.sort();
        let mut complete = BTreeSet::new();
        for name in names {
            self.validate_rule_references(&name, &mut Vec::new(), &mut complete)?;
        }
        Ok(())
    }

    fn validate_rule_references(
        &self,
        name: &str,
        stack: &mut Vec<String>,
        complete: &mut BTreeSet<String>,
    ) -> Result<(), String> {
        if complete.contains(name) {
            return Ok(());
        }
        push_rule_stack(stack, name)?;
        let value = self
            .rules
            .get(name)
            .ok_or_else(|| format!("unknown Assura config rule '${name}'"))?;
        stack.push(name.to_string());
        let mut references = BTreeSet::new();
        collect_rule_references(value, &mut references)?;
        for reference in references {
            self.validate_rule_references(&reference, stack, complete)?;
        }
        stack.pop();
        complete.insert(name.to_string());
        Ok(())
    }
}

fn validate_authored_rule_shape(name: &str, value: &Value) -> Result<(), String> {
    let Value::Mapping(mapping) = value else {
        return Ok(());
    };
    let has_node = mapping
        .keys()
        .filter_map(Value::as_str)
        .any(is_node_attr_key);
    let has_tree = mapping
        .keys()
        .filter_map(Value::as_str)
        .any(|key| !is_node_attr_key(key));
    if has_node && has_tree {
        return Err(format!(
            "Assura config rule '${name}' mixes node directives with child selectors; keep the node constraint and child tree as separate rules"
        ));
    }
    Ok(())
}

fn collect_rule_references(value: &Value, references: &mut BTreeSet<String>) -> Result<(), String> {
    let Value::Mapping(mapping) = value else {
        return Ok(());
    };

    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            continue;
        };
        if key == USE {
            for reference in use_references(value)? {
                references.insert(parse_rule_reference(&reference)?.to_string());
            }
            continue;
        }

        if is_node_attr_key(key) {
            if key == "children" {
                collect_rule_references(value, references)?;
            }
            continue;
        }

        if let Value::String(reference) = value {
            if is_rule_reference(reference) {
                references.insert(parse_rule_reference(reference)?.to_string());
            }
        } else {
            collect_rule_references(value, references)?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FragmentKind {
    Node,
    Tree,
}

fn classify_fragment(value: &Value) -> Result<FragmentKind, String> {
    let Value::Mapping(mapping) = value else {
        return Ok(FragmentKind::Node);
    };

    let mut has_node_attrs = false;
    let mut has_tree_attrs = false;
    for key in mapping.keys() {
        let Some(key) = key.as_str() else {
            return Err("Assura config fragments must use string keys".to_string());
        };
        if is_node_attr_key(key) {
            has_node_attrs = true;
        } else {
            has_tree_attrs = true;
        }
    }

    match (has_node_attrs, has_tree_attrs) {
        (true, true) => Ok(FragmentKind::Tree),
        (true, false) => Ok(FragmentKind::Node),
        _ => Ok(FragmentKind::Tree),
    }
}

fn parse_rule_reference(reference: &str) -> Result<&str, String> {
    if reference.starts_with('@') {
        return Err(format!(
            "Assura config rule references no longer use '@'; replace '{reference}' with '${}'",
            reference.trim_start_matches('@')
        ));
    }
    normalize_rule_name(reference).and_then(|name| {
        if reference.starts_with('$') {
            Ok(name)
        } else {
            Err(format!(
                "Assura config rule reference must start with '$': {reference}"
            ))
        }
    })
}

fn normalize_rule_name(name: &str) -> Result<&str, String> {
    let name = name.strip_prefix('$').unwrap_or(name);
    if name.is_empty() {
        return Err("Assura config rule names must not be empty".to_string());
    }
    Ok(name)
}

fn normalize_rule_definition_name(name: &str) -> Result<&str, String> {
    if let Some(name) = name.strip_prefix('@') {
        return Err(format!(
            "Assura config rule definitions no longer use '@'; replace '@{name}' with '{name}'"
        ));
    }
    if let Some(name) = name.strip_prefix('$') {
        return Err(format!(
            "Assura config rule definitions are plain names; replace '${name}' with '{name}'"
        ));
    }
    normalize_rule_name(name)
}

fn push_rule_stack(stack: &[String], name: &str) -> Result<(), String> {
    if stack.iter().any(|entry| entry == name) {
        let mut cycle = stack.to_vec();
        cycle.push(name.to_string());
        return Err(format!(
            "Assura config rule cycle detected: {}",
            cycle.join(" -> ")
        ));
    }
    Ok(())
}

fn is_rule_reference(value: &str) -> bool {
    value.starts_with('$') || value.starts_with('@')
}
