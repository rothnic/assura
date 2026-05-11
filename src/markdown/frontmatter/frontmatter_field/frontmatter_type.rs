//! Frontmatter field type enum.

use serde::{Deserialize, Serialize};

/// Field types supported in frontmatter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    Date,
    DateTime,
    Email,
    Url,
}

impl FieldType {
    /// Get a human-readable name for this type
    pub fn display_name(&self) -> &'static str {
        match self {
            FieldType::String => "string",
            FieldType::Integer => "integer",
            FieldType::Float => "float",
            FieldType::Boolean => "boolean",
            FieldType::Array => "array",
            FieldType::Object => "object",
            FieldType::Date => "date",
            FieldType::DateTime => "datetime",
            FieldType::Email => "email",
            FieldType::Url => "url",
        }
    }
}
