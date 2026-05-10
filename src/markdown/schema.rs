//! Markdown schema definitions
//!
//! Schemas define the structure and validation rules for markdown documents.
//! Each schema can specify:
//! - Frontmatter requirements (YAML fields and types)
//! - Heading structure (hierarchy and required headings)
//! - Template structure (required sections and their order)
//! - Content patterns (regex-based validation)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{MarkdownError, MarkdownResult};
use super::frontmatter::FrontmatterSchema;
use super::headings::HeadingValidator;
use super::template::TemplateDefinition;

/// A complete markdown schema definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkdownSchema {
    /// Schema name (identifier)
    pub name: String,
    /// Human-readable description
    pub description: Option<String>,
    /// Optional frontmatter schema
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frontmatter: Option<FrontmatterSchema>,
    /// Optional heading validation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headings: Option<HeadingValidator>,
    /// Optional template definition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<TemplateDefinition>,
    /// Custom validation rules
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rules: Vec<MarkdownValidationRule>,
    /// Schema inheritance (name of parent schema)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends: Option<String>,
}

impl MarkdownSchema {
    /// Create a new schema with the given name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            frontmatter: None,
            headings: None,
            template: None,
            rules: Vec::new(),
            extends: None,
        }
    }

    /// Set the schema description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add frontmatter schema
    pub fn with_frontmatter(mut self, frontmatter: FrontmatterSchema) -> Self {
        self.frontmatter = Some(frontmatter);
        self
    }

    /// Add heading validation
    pub fn with_headings(mut self, headings: HeadingValidator) -> Self {
        self.headings = Some(headings);
        self
    }

    /// Add template definition
    pub fn with_template(mut self, template: TemplateDefinition) -> Self {
        self.template = Some(template);
        self
    }

    /// Add a validation rule
    pub fn with_rule(mut self, rule: MarkdownValidationRule) -> Self {
        self.rules.push(rule);
        self
    }

    /// Set parent schema for inheritance
    pub fn extends(mut self, parent: impl Into<String>) -> Self {
        self.extends = Some(parent.into());
        self
    }

    /// Load a schema from a YAML string
    pub fn from_yaml(yaml: &str) -> MarkdownResult<Self> {
        serde_yaml::from_str(yaml).map_err(|e| {
            MarkdownError::schema("unknown", format!("Failed to parse schema YAML: {}", e))
        })
    }

    /// Serialize schema to YAML
    pub fn to_yaml(&self) -> MarkdownResult<String> {
        serde_yaml::to_string(self).map_err(|e| {
            MarkdownError::schema(&self.name, format!("Failed to serialize schema: {}", e))
        })
    }

    /// Merge this schema with another (parent schema)
    /// Child schema values take precedence
    pub fn merge_with(mut self, parent: &Self) -> Self {
        // Merge frontmatter
        if let Some(parent_frontmatter) = &parent.frontmatter {
            let child_frontmatter = self.frontmatter.take();
            self.frontmatter = Some(match child_frontmatter {
                Some(child) => child.merge_with(parent_frontmatter),
                None => parent_frontmatter.clone(),
            });
        }

        // Merge headings
        if let Some(parent_headings) = &parent.headings {
            let child_headings = self.headings.take();
            self.headings = Some(match child_headings {
                Some(child) => child.merge_with(parent_headings),
                None => parent_headings.clone(),
            });
        }

        // Merge template
        if let Some(parent_template) = &parent.template {
            let child_template = self.template.take();
            self.template = Some(match child_template {
                Some(child) => child.merge_with(parent_template),
                None => parent_template.clone(),
            });
        }

        // Merge rules (child rules come first)
        let mut merged_rules = self.rules.clone();
        merged_rules.extend(parent.rules.clone());
        self.rules = merged_rules;

        self
    }
}

/// A single validation rule for markdown content
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MarkdownValidationRule {
    /// Validate a regex pattern in the content
    Pattern {
        /// Regex pattern to match
        pattern: String,
        /// Whether the pattern must exist (true) or must not exist (false)
        required: bool,
        /// Error message on validation failure
        message: String,
        /// Optional line range to restrict search
        #[serde(skip_serializing_if = "Option::is_none")]
        line_range: Option<(usize, usize)>,
    },
    /// Validate that certain content exists
    RequiredContent {
        /// Content that must exist
        content: String,
        /// Error message on validation failure
        message: String,
    },
    /// Validate that certain content does not exist
    ForbiddenContent {
        /// Content that must not exist
        content: String,
        /// Error message on validation failure
        message: String,
    },
    /// Validate word count
    WordCount {
        /// Minimum word count (inclusive)
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<usize>,
        /// Maximum word count (inclusive)
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
        /// Error message on validation failure
        message: String,
    },
    /// Validate line count
    LineCount {
        /// Minimum line count (inclusive)
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<usize>,
        /// Maximum line count (inclusive)
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<usize>,
        /// Error message on validation failure
        message: String,
    },
}

/// Validation configuration for markdown schemas
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether to enforce strict mode (fail on warnings)
    #[serde(default)]
    pub strict: bool,
    /// Whether to validate frontmatter
    #[serde(default = "default_true")]
    pub validate_frontmatter: bool,
    /// Whether to validate heading structure
    #[serde(default = "default_true")]
    pub validate_headings: bool,
    /// Whether to validate templates
    #[serde(default = "default_true")]
    pub validate_templates: bool,
    /// Custom error messages
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub error_messages: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

impl ValidationConfig {
    /// Create a new validation config with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict mode
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Disable frontmatter validation
    pub fn skip_frontmatter(mut self) -> Self {
        self.validate_frontmatter = false;
        self
    }

    /// Disable heading validation
    pub fn skip_headings(mut self) -> Self {
        self.validate_headings = false;
        self
    }

    /// Disable template validation
    pub fn skip_templates(mut self) -> Self {
        self.validate_templates = false;
        self
    }

    /// Add a custom error message
    pub fn with_error_message(
        mut self,
        key: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        self.error_messages.insert(key.into(), message.into());
        self
    }

    /// Get an error message by key, or return default
    pub fn get_error_message(&self, key: &str, default: &str) -> String {
        self.error_messages
            .get(key)
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }
}

/// Schema definition format for configuration files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaDefinition {
    /// Schema version
    pub version: String,
    /// List of schemas
    pub schemas: Vec<MarkdownSchema>,
    /// Default schema to use
    pub default_schema: Option<String>,
    /// Global validation configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<ValidationConfig>,
}

impl SchemaDefinition {
    /// Create a new schema definition
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            schemas: Vec::new(),
            default_schema: None,
            config: None,
        }
    }

    /// Add a schema
    pub fn add_schema(mut self, schema: MarkdownSchema) -> Self {
        self.schemas.push(schema);
        self
    }

    /// Set the default schema
    pub fn with_default_schema(mut self, name: impl Into<String>) -> Self {
        self.default_schema = Some(name.into());
        self
    }

    /// Set global configuration
    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Load from YAML
    pub fn from_yaml(yaml: &str) -> MarkdownResult<Self> {
        serde_yaml::from_str(yaml).map_err(|e| {
            MarkdownError::schema(
                "definition",
                format!("Failed to parse schema definition: {}", e),
            )
        })
    }

    /// Serialize to YAML
    pub fn to_yaml(&self) -> MarkdownResult<String> {
        serde_yaml::to_string(self).map_err(|e| {
            MarkdownError::schema(
                "definition",
                format!("Failed to serialize schema definition: {}", e),
            )
        })
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Option<&MarkdownSchema> {
        self.schemas.iter().find(|s| s.name == name)
    }

    /// Resolve schema with inheritance
    pub fn resolve_schema(&self, name: &str) -> MarkdownResult<Option<MarkdownSchema>> {
        let schema = self.get_schema(name);
        if schema.is_none() {
            return Ok(None);
        }

        let mut resolved = schema.unwrap().clone();

        // Resolve inheritance chain
        while let Some(parent_name) = &resolved.extends {
            let parent = self.get_schema(parent_name).ok_or_else(|| {
                MarkdownError::schema(
                    &resolved.name,
                    format!("Parent schema '{}' not found", parent_name),
                )
            })?;

            resolved = resolved.merge_with(parent);
            // Remove extends to prevent infinite loop
            resolved.extends = None;
        }

        Ok(Some(resolved))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_schema_builder() {
        let schema = MarkdownSchema::new("test")
            .with_description("Test schema")
            .extends("base");

        assert_eq!(schema.name, "test");
        assert_eq!(schema.description, Some("Test schema".to_string()));
        assert_eq!(schema.extends, Some("base".to_string()));
    }

    #[test]
    fn test_schema_yaml_serialization() {
        let schema = MarkdownSchema::new("test").with_description("A test schema");

        let yaml = schema.to_yaml().unwrap();
        assert!(yaml.contains("name: test"));
        assert!(yaml.contains("description: A test schema"));
    }

    #[test]
    fn test_schema_yaml_deserialization() {
        let yaml = r#"
name: test
description: A test schema
rules: []
"#;

        let schema = MarkdownSchema::from_yaml(yaml).unwrap();
        assert_eq!(schema.name, "test");
        assert_eq!(schema.description, Some("A test schema".to_string()));
    }

    #[test]
    fn test_validation_config_builder() {
        let config = ValidationConfig::new()
            .strict()
            .skip_frontmatter()
            .with_error_message("test", "custom message");

        assert!(config.strict);
        assert!(!config.validate_frontmatter);
        assert_eq!(
            config.get_error_message("test", "default"),
            "custom message"
        );
        assert_eq!(config.get_error_message("missing", "default"), "default");
    }

    #[test]
    fn test_schema_definition() {
        let def = SchemaDefinition::new("1.0")
            .add_schema(MarkdownSchema::new("base"))
            .add_schema(MarkdownSchema::new("child").extends("base"))
            .with_default_schema("base");

        assert_eq!(def.version, "1.0");
        assert_eq!(def.schemas.len(), 2);
        assert_eq!(def.default_schema, Some("base".to_string()));
    }

    #[test]
    fn test_resolve_schema_inheritance() {
        let yaml = r#"
version: "1.0"
schemas:
  - name: base
    description: Base schema
    rules:
      - type: required_content
        content: "base content"
        message: "Missing base content"
  - name: child
    description: Child schema
    extends: base
    rules:
      - type: required_content
        content: "child content"
        message: "Missing child content"
"#;

        let def = SchemaDefinition::from_yaml(yaml).unwrap();
        let child = def.resolve_schema("child").unwrap().unwrap();

        assert_eq!(child.description, Some("Child schema".to_string()));
        // Should have rules from both parent and child
        assert_eq!(child.rules.len(), 2);
    }
}
