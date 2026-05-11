//! Frontmatter field type and value validation.

use serde::{Deserialize, Serialize};

mod frontmatter_field_validation;
mod frontmatter_type;
pub use frontmatter_type::FieldType;

/// Validator for a single frontmatter field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidator {
    /// Field type
    #[serde(rename = "type")]
    pub field_type: FieldType,
    /// Whether the field is required
    #[serde(default)]
    pub required: bool,
    /// Regex pattern for string fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Minimum value/length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<serde_yaml::Value>,
    /// Maximum value/length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<serde_yaml::Value>,
    /// Allowed values (enum)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_values: Option<Vec<serde_yaml::Value>>,
    /// Custom validation message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl FieldValidator {
    /// Create a new field validator
    pub fn new(field_type: FieldType) -> Self {
        Self {
            field_type,
            required: false,
            pattern: None,
            min: None,
            max: None,
            allowed_values: None,
            message: None,
        }
    }

    /// Mark the field as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set a regex pattern for string validation
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set minimum value/length
    pub fn with_min(mut self, min: impl Into<serde_yaml::Value>) -> Self {
        self.min = Some(min.into());
        self
    }

    /// Set maximum value/length
    pub fn with_max(mut self, max: impl Into<serde_yaml::Value>) -> Self {
        self.max = Some(max.into());
        self
    }

    /// Set allowed values
    pub fn with_allowed_values(mut self, values: Vec<impl Into<serde_yaml::Value>>) -> Self {
        self.allowed_values = Some(values.into_iter().map(Into::into).collect());
        self
    }

    /// Set custom error message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }
}
