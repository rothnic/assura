/// Reusable rule fragments declared at the top level of an Assura config.
#[derive(Default)]
struct RuleRegistry {
    rules: HashMap<String, Value>,
}

const BUILTIN_AGENT_SKILL_FILE: &str = r#"
exists: 1
max_lines: 600
max_size: "24KB"
"#;

const BUILTIN_AGENT_SKILL_RESOURCE_DIR: &str = r#"
inherit: false
.dir: kebab-case
files:
  naming: kebab-case
  max_lines: 600
  max_size: "24KB"
directories:
  naming: kebab-case
extra: true
"#;

const BUILTIN_AGENT_SKILL_DIR: &str = r#"
inherit: false
.dir: kebab-case
SKILL.md: "@agent-skill-file"
agents/:
  required: false
  use: "@agent-skill-resource-dir"
references/:
  required: false
  use: "@agent-skill-resource-dir"
scripts/:
  required: false
  use: "@agent-skill-resource-dir"
assets/:
  required: false
  use: "@agent-skill-resource-dir"
extra: false
"#;

const BUILTIN_AGENTS_DIR: &str = r#"
inherit: false
README.md: exists:0-1
SKILL.md: exists:0-1
TEMPLATE.md: exists:0-1
prd.json: exists:0-1
skills/:
  inherit: false
  README.md: exists:0-1
  TEMPLATE.md: exists:0-1
  files:
    allow_extra: false
  directories:
    allow_extra: true
  built-in/:
    required: false
    "{skill}/":
      use: "@agent-skill-dir"
    files:
      allow_extra: false
    directories:
      allow_extra: true
  custom/:
    required: false
    "{skill}/":
      use: "@agent-skill-dir"
    files:
      allow_extra: false
    directories:
      allow_extra: true
  "{skill}/":
    use: "@agent-skill-dir"
extra: false
"#;

impl RuleRegistry {
    fn from_root(root: &mut Mapping) -> Result<Self, String> {
        let mut rules = builtin_rules()?;
        let Some(value) = root.remove(string_value("rules")) else {
            return Ok(Self { rules });
        };
        let Value::Mapping(mapping) = value else {
            return Err("Assura config rules must be a mapping".to_string());
        };

        for (key, value) in mapping {
            let Some(name) = key.as_str() else {
                return Err("Assura config rule names must be strings".to_string());
            };
            let name = normalize_rule_name(name)?;
            rules.insert(name.to_string(), value);
        }
        Ok(Self { rules })
    }

    fn resolve_node(&self, reference: &str) -> Result<Value, String> {
        let name = parse_rule_reference(reference)?;
        let value = self.resolve(name)?;
        match classify_fragment(&value)? {
            FragmentKind::Node => Ok(value),
            FragmentKind::Tree => Err(format!(
                "Assura config rule '@{name}' is a tree fragment but a node fragment is required"
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
                        "Assura config rule '@{name}' must be a mapping when used through use"
                    ));
                };
                Ok(mapping)
            }
            FragmentKind::Node => Err(format!(
                "Assura config rule '@{name}' is a node fragment but a tree fragment is required"
            )),
        }
    }

    fn resolve(&self, name: &str) -> Result<Value, String> {
        self.rules
            .get(name)
            .cloned()
            .ok_or_else(|| format!("unknown Assura config rule '@{name}'"))
    }
}

fn builtin_rules() -> Result<HashMap<String, Value>, String> {
    let mut rules = HashMap::new();
    for (name, source) in [
        ("agent-skill-file", BUILTIN_AGENT_SKILL_FILE),
        ("agent-skill-resource-dir", BUILTIN_AGENT_SKILL_RESOURCE_DIR),
        ("agent-skill-dir", BUILTIN_AGENT_SKILL_DIR),
        ("agents-dir", BUILTIN_AGENTS_DIR),
    ] {
        rules.insert(
            name.to_string(),
            serde_yaml::from_str(source).map_err(|error| {
                format!("invalid built-in Assura config rule '@{name}': {error}")
            })?,
        );
    }
    Ok(rules)
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
    normalize_rule_name(reference).and_then(|name| {
        if reference.starts_with('@') {
            Ok(name)
        } else {
            Err(format!(
                "Assura config rule reference must start with '@': {reference}"
            ))
        }
    })
}

fn normalize_rule_name(name: &str) -> Result<&str, String> {
    let name = name.strip_prefix('@').unwrap_or(name);
    if name.is_empty() {
        return Err("Assura config rule names must not be empty".to_string());
    }
    Ok(name)
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
    value.starts_with('@')
}
