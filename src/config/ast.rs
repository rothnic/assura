//! Abstract Syntax Tree for Assura Configuration
//!
//! Follows Constitution principles:
//! - Structure-first representation
//! - YAML/JSON compatible
//! - No custom string parsing

use crate::config::preprocessor::YamlPreprocessor;
use serde::{Deserialize, Serialize, Serializer};
use std::collections::HashMap;

/// Top-level configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    /// Reusable rule definitions
    #[serde(default)]
    pub rules: HashMap<String, Rule>,

    /// Context definitions (when/where)
    #[serde(default)]
    pub contexts: HashMap<String, Context>,

    /// Message templates (optional)
    #[serde(default)]
    pub messages: HashMap<String, MessageTemplate>,

    /// Policy tree (required)
    #[serde(default)]
    pub policy: PolicyNode,
}

/// A reusable rule definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rule {
    /// File pattern with structural constraints
    /// Key: file pattern (e.g., "${name}.tsx")
    /// Value: array of constraint items
    #[serde(flatten)]
    pub patterns: HashMap<String, Vec<ConstraintItem>>,
}

/// An item in a constraint array
/// Can be: constraint, violation level, message, or context override
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ConstraintItem {
    /// Constraints array with key: constraints: [PascalCase, lines:..400]
    Constraints { constraints: Vec<Constraint> },

    /// Violation array with key: violation: [warn, ci:block]
    Violation { violation: Vec<ViolationEntry> },

    /// Constraint definition (bare constraint without key)
    Constraint(Constraint),

    /// Message definition: message: {warn: "..."}
    Message {
        #[serde(rename = "message")]
        message: Message,
    },

    /// Context override at file level
    ContextOverride {
        #[serde(rename = "context")]
        name: String,
        violation: Vec<ViolationEntry>,
    },
}

/// A single constraint
#[derive(Debug, Clone, PartialEq)]
pub enum Constraint {
    /// Naming convention
    Naming(NamingConvention),

    /// Lines constraint: lines: ..400
    Lines { lines: Range },

    /// Size constraint: size: ..1MB
    Size { size: String },

    /// Exists constraint: exists: 1..10
    Exists { exists: Range },

    /// Constraints array: constraints: [PascalCase, lines:..400]
    ConstraintsArray(Vec<Constraint>),
}

impl<'de> Deserialize<'de> for Constraint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, SeqAccess, Visitor};
        use std::fmt;

        struct ConstraintVisitor;

        impl<'de> Visitor<'de> for ConstraintVisitor {
            type Value = Constraint;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a constraint string like 'PascalCase', 'lines:..400', or a map like {lines: '..400'}")
            }

            // Handle string values like "PascalCase", "lines:..400"
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Try to parse as naming convention
                if let Ok(naming) = serde_yaml::from_str::<NamingConvention>(value) {
                    return Ok(Constraint::Naming(naming));
                }

                // Try to parse as "key:value" format
                if let Some(pos) = value.find(':') {
                    let key = &value[..pos];
                    let val = &value[pos + 1..];

                    match key {
                        "lines" => {
                            let range = if val.parse::<u64>().is_ok() {
                                Range::Exact(val.parse().unwrap())
                            } else {
                                Range::RangeString(val.to_string())
                            };
                            return Ok(Constraint::Lines { lines: range });
                        }
                        "size" => {
                            return Ok(Constraint::Size {
                                size: val.to_string(),
                            });
                        }
                        "exists" => {
                            let range = if val.parse::<u64>().is_ok() {
                                Range::Exact(val.parse().unwrap())
                            } else {
                                Range::RangeString(val.to_string())
                            };
                            return Ok(Constraint::Exists { exists: range });
                        }
                        _ => {}
                    }
                }

                Err(de::Error::custom(format!("Unknown constraint: {}", value)))
            }

            // Handle map values like {lines: "..400"}
            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                use serde_yaml::Value;

                let mut lines_val: Option<Value> = None;
                let mut size_val: Option<Value> = None;
                let mut exists_val: Option<Value> = None;

                while let Some((key, value)) = map.next_entry::<String, Value>()? {
                    match key.as_str() {
                        "lines" => lines_val = Some(value),
                        "size" => size_val = Some(value),
                        "exists" => exists_val = Some(value),
                        _ => {}
                    }
                }

                if let Some(val) = lines_val {
                    let range: Range = serde::Deserialize::deserialize(val)
                        .map_err(|e| de::Error::custom(format!("Invalid lines value: {}", e)))?;
                    return Ok(Constraint::Lines { lines: range });
                }

                if let Some(val) = size_val {
                    let size: String = serde::Deserialize::deserialize(val)
                        .map_err(|e| de::Error::custom(format!("Invalid size value: {}", e)))?;
                    return Ok(Constraint::Size { size });
                }

                if let Some(val) = exists_val {
                    let range: Range = serde::Deserialize::deserialize(val)
                        .map_err(|e| de::Error::custom(format!("Invalid exists value: {}", e)))?;
                    return Ok(Constraint::Exists { exists: range });
                }

                Err(de::Error::custom(
                    "Constraint map must have lines, size, or exists key",
                ))
            }

            // Handle sequence values (array of constraints)
            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let constraints: Vec<Constraint> =
                    Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))?;
                Ok(Constraint::ConstraintsArray(constraints))
            }
        }

        deserializer.deserialize_any(ConstraintVisitor)
    }
}

impl Serialize for Constraint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Constraint::Naming(naming) => naming.serialize(serializer),
            Constraint::Lines { lines } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("lines", lines)?;
                map.end()
            }
            Constraint::Size { size } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("size", size)?;
                map.end()
            }
            Constraint::Exists { exists } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("exists", exists)?;
                map.end()
            }
            Constraint::ConstraintsArray(constraints) => constraints.serialize(serializer),
        }
    }
}

/// Naming conventions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NamingConvention {
    #[serde(rename = "PascalCase")]
    PascalCase,
    #[serde(rename = "camelCase")]
    CamelCase,
    #[serde(rename = "snake_case")]
    SnakeCase,
    #[serde(rename = "kebab-case")]
    KebabCase,
    #[serde(rename = "SCREAMING_SNAKE_CASE")]
    ScreamingSnakeCase,
    #[serde(rename = "lowercase")]
    Lowercase,
    #[serde(rename = "UPPERCASE")]
    Uppercase,
}

/// Range for constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Range {
    /// Exact number: exists: 1
    Exact(u64),

    /// Range string: "1..10", "..400", "100.."
    RangeString(String),
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Range::Exact(n) => write!(f, "{}", n),
            Range::RangeString(s) => write!(f, "{}", s),
        }
    }
}

/// A violation entry in the array
/// Can be: "warn", "block", "notify", or "ci:block" (context:value)
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationEntry {
    /// Default level: "warn", "block", "notify"
    Level(String),

    /// Context-specific: "ci:block", "feature:warn"
    ContextSpecific { context: String, level: String },
}

impl<'de> Deserialize<'de> for ViolationEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Check if it contains a colon (context:level format)
        if let Some(pos) = s.find(':') {
            let (context, level) = s.split_at(pos);
            let level = &level[1..]; // Remove the colon
            Ok(ViolationEntry::ContextSpecific {
                context: context.to_string(),
                level: level.to_string(),
            })
        } else {
            Ok(ViolationEntry::Level(s))
        }
    }
}

impl Serialize for ViolationEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ViolationEntry::Level(level) => serializer.serialize_str(level),
            ViolationEntry::ContextSpecific { context, level } => {
                serializer.serialize_str(&format!("{}:{}", context, level))
            }
        }
    }
}

/// Message definition attached to violations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    /// Context-specific messages
    #[serde(flatten)]
    pub contexts: HashMap<String, String>,

    /// Fix suggestion
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,

    /// Documentation link
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,

    /// Override instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#override: Option<String>,
}

/// Context definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Context {
    /// Hook type: tool, pre-commit, pre-push, ci, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook: Option<String>,

    /// Branch pattern: "feature/*", "main"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Version range: "2.x..", "..1.x"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Environment variables
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<HashMap<String, String>>,
}

/// Message template (reusable)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageTemplate {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs: Option<String>,
}

/// Policy tree node
/// Mirrors directory structure
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PolicyNode {
    /// Directory or file entries
    /// Key: path component ("src/", "${name}.tsx")
    /// Value: node contents
    #[serde(flatten)]
    pub entries: HashMap<String, PolicyEntry>,
}

/// Entry in policy tree
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PolicyEntry {
    /// Subdirectory
    Directory(PolicyNode),

    /// File with constraints
    File(Vec<FileItem>),

    /// Strict mode directive
    Strict { strict: bool },

    /// Violation default at directory level
    ViolationDefault { violation: Vec<ViolationEntry> },

    /// Context definition at directory level
    ContextDef {
        context: String,
        violation: Vec<ViolationEntry>,
    },
}

/// Apply value - can be single rule or array of rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ApplyValue {
    Single(String),
    Multiple(Vec<String>),
}

/// Item in a file's constraint array
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FileItem {
    /// Apply rules: apply: rule-name or apply: [rule1, rule2]
    Apply { apply: ApplyValue },

    /// Constraints: constraints: [PascalCase, lines:..400]
    Constraints { constraints: Vec<Constraint> },

    /// Violation levels
    Violation { violation: Vec<ViolationEntry> },

    /// Exists requirement: exists: 1
    Exists { exists: Range },

    /// Message attachment
    Message(Message),
}

impl Config {
    /// Parse configuration from YAML string
    /// Preprocesses to add quotes where needed before parsing
    pub fn from_yaml(yaml: &str) -> Result<Self, serde_yaml::Error> {
        let processed = YamlPreprocessor::process(yaml);
        serde_yaml::from_str(&processed)
    }

    /// Serialize to YAML
    pub fn to_yaml(&self) -> Result<String, serde_yaml::Error> {
        serde_yaml::to_string(self)
    }

    /// Serialize to JSON (for tooling)
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let yaml = r#"
rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase, lines:..400]
      - violation: [warn, ci:block]

policy:
  src/components/:
    ${name}.tsx:
      - apply: react
"#;

        let config = Config::from_yaml(yaml).expect("Should parse");
        assert!(config.rules.contains_key("react"));
    }

    #[test]
    fn test_parse_violation_array() {
        let yaml = r#"[warn, ci:block, feature:warn]"#;

        // This would be part of a larger structure
        let entries: Vec<ViolationEntry> = serde_yaml::from_str(yaml).expect("Should parse");
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_json_roundtrip() {
        let yaml = r#"
rules:
  react:
    ${name}.tsx:
      - constraints: [PascalCase]
      - violation: [warn]

policy:
  src/:
    ${name}.tsx:
      - apply: react
"#;

        let config = Config::from_yaml(yaml).expect("Should parse YAML");
        let json = config.to_json().expect("Should serialize to JSON");
        let config2: Config = serde_json::from_str(&json).expect("Should parse JSON");

        assert_eq!(config.rules.len(), config2.rules.len());
    }
}
