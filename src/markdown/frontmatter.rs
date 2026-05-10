//! Frontmatter validation
//!
//! Validates YAML frontmatter in markdown documents against defined schemas.
//! Supports required fields, type checking, and custom validation rules.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{MarkdownError, MarkdownResult, MarkdownValidationError};
use super::parser::MarkdownDocument;
use crate::constraints::ValidationFailure;

/// Schema for frontmatter validation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrontmatterSchema {
    /// Whether frontmatter is required
    #[serde(default)]
    pub required: bool,
    /// Field definitions
    #[serde(default)]
    pub fields: HashMap<String, FieldValidator>,
    /// Additional allowed fields (not in fields map)
    #[serde(default)]
    pub allow_additional_fields: bool,
    /// Required fields list (alternative to marking individual fields)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_fields: Vec<String>,
}

impl FrontmatterSchema {
    /// Create a new frontmatter schema
    pub fn new() -> Self {
        Self::default()
    }

    /// Require frontmatter to exist
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Add a field validator
    pub fn with_field(mut self, name: impl Into<String>, validator: FieldValidator) -> Self {
        self.fields.insert(name.into(), validator);
        self
    }

    /// Set whether to allow additional fields
    pub fn allow_additional_fields(mut self, allow: bool) -> Self {
        self.allow_additional_fields = allow;
        self
    }

    /// Add a required field
    pub fn with_required_field(mut self, name: impl Into<String>) -> Self {
        self.required_fields.push(name.into());
        self
    }

    /// Validate a document's frontmatter
    pub fn validate(
        &self,
        document: &MarkdownDocument,
        path: &std::path::Path,
    ) -> MarkdownResult<Vec<ValidationFailure>> {
        let mut failures = Vec::new();

        // Check if frontmatter is required
        if self.required && !document.has_frontmatter() {
            failures.push(
                MarkdownValidationError::new(
                    "frontmatter_required",
                    path,
                    "Frontmatter is required but not found",
                )
                .with_suggestion(
                    "Add YAML frontmatter between --- delimiters at the start of the file",
                )
                .into_validation_failure(),
            );
            return Ok(failures);
        }

        // If no frontmatter and not required, we're done
        if !document.has_frontmatter() {
            return Ok(failures);
        }

        // Parse frontmatter
        let frontmatter = match document.frontmatter_map() {
            Ok(Some(fm)) => fm,
            Ok(None) => {
                failures.push(
                    MarkdownValidationError::new(
                        "frontmatter_parse",
                        path,
                        "Failed to parse frontmatter as YAML object",
                    )
                    .into_validation_failure(),
                );
                return Ok(failures);
            }
            Err(e) => {
                // YAML parsing error - treat as validation failure
                failures.push(
                    MarkdownValidationError::new(
                        "frontmatter_parse",
                        path,
                        format!("Failed to parse frontmatter YAML: {}", e),
                    )
                    .with_suggestion("Check that the frontmatter is valid YAML syntax")
                    .into_validation_failure(),
                );
                return Ok(failures);
            }
        };

        // Collect all required fields
        let mut all_required: Vec<String> = self.required_fields.clone();
        for (name, validator) in &self.fields {
            if validator.required && !all_required.contains(name) {
                all_required.push(name.clone());
            }
        }

        // Check required fields
        for field_name in &all_required {
            if !frontmatter.contains_key(field_name) {
                failures.push(
                    MarkdownValidationError::new(
                        "missing_field",
                        path,
                        format!("Missing required field: {}", field_name),
                    )
                    .with_suggestion(format!("Add the '{}' field to frontmatter", field_name))
                    .into_validation_failure(),
                );
            }
        }

        // Validate each field
        for (field_name, validator) in &self.fields {
            if let Some(value) = frontmatter.get(field_name) {
                match validator.validate(field_name, value, path) {
                    Ok(()) => {}
                    Err(err) => {
                        failures.push(err.into_validation_failure());
                    }
                }
            }
        }

        // Check for additional fields if not allowed
        if !self.allow_additional_fields {
            for field_name in frontmatter.keys() {
                if !self.fields.contains_key(field_name) && !all_required.contains(field_name) {
                    failures.push(
                        MarkdownValidationError::new(
                            "additional_field",
                            path,
                            format!("Additional field not allowed: {}", field_name),
                        )
                        .with_suggestion(format!(
                            "Remove the '{}' field or add it to the schema",
                            field_name
                        ))
                        .into_validation_failure(),
                    );
                }
            }
        }

        Ok(failures)
    }

    /// Merge with another frontmatter schema (parent)
    pub fn merge_with(mut self, parent: &Self) -> Self {
        // Merge fields (child takes precedence)
        let mut merged_fields = parent.fields.clone();
        merged_fields.extend(self.fields);
        self.fields = merged_fields;

        // Merge required fields
        let mut merged_required = parent.required_fields.clone();
        merged_required.extend(self.required_fields);
        self.required_fields = merged_required;

        // Child settings take precedence for boolean flags
        if !self.required {
            self.required = parent.required;
        }
        if self.allow_additional_fields {
            self.allow_additional_fields = parent.allow_additional_fields;
        }

        self
    }
}

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

    /// Validate a value against this validator
    pub fn validate(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        // First, check the type
        match self.field_type {
            FieldType::String => self.validate_string(field_name, value, path)?,
            FieldType::Integer => self.validate_integer(field_name, value, path)?,
            FieldType::Float => self.validate_float(field_name, value, path)?,
            FieldType::Boolean => self.validate_boolean(field_name, value, path)?,
            FieldType::Array => self.validate_array(field_name, value, path)?,
            FieldType::Object => self.validate_object(field_name, value, path)?,
            FieldType::Date => self.validate_date(field_name, value, path)?,
            FieldType::DateTime => self.validate_datetime(field_name, value, path)?,
            FieldType::Email => self.validate_email(field_name, value, path)?,
            FieldType::Url => self.validate_url(field_name, value, path)?,
        }

        // Check allowed values
        if let Some(ref allowed) = self.allowed_values {
            if !allowed.contains(value) {
                return Err(MarkdownValidationError::new(
                    "field_validation",
                    path,
                    self.message.clone().unwrap_or_else(|| {
                        format!(
                            "Field '{}' has invalid value. Allowed: {:?}",
                            field_name, allowed
                        )
                    }),
                ));
            }
        }

        Ok(())
    }

    fn validate_string(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a string", field_name),
                ));
            }
        };

        // Check pattern
        if let Some(ref pattern) = self.pattern {
            let regex = Regex::new(pattern).map_err(|_| {
                MarkdownValidationError::new(
                    "invalid_pattern",
                    path,
                    format!("Invalid regex pattern for field '{}'", field_name),
                )
            })?;
            if !regex.is_match(s) {
                return Err(MarkdownValidationError::new(
                    "field_pattern",
                    path,
                    self.message.clone().unwrap_or_else(|| {
                        format!("Field '{}' does not match pattern: {}", field_name, pattern)
                    }),
                ));
            }
        }

        // Check length constraints
        if let Some(ref min) = self.min {
            if let Some(min_len) = min.as_u64() {
                if s.len() < min_len as usize {
                    return Err(MarkdownValidationError::new(
                        "field_min_length",
                        path,
                        format!(
                            "Field '{}' must be at least {} characters",
                            field_name, min_len
                        ),
                    ));
                }
            }
        }

        if let Some(ref max) = self.max {
            if let Some(max_len) = max.as_u64() {
                if s.len() > max_len as usize {
                    return Err(MarkdownValidationError::new(
                        "field_max_length",
                        path,
                        format!(
                            "Field '{}' must be at most {} characters",
                            field_name, max_len
                        ),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_integer(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let n = match value {
            serde_yaml::Value::Number(n) => n.as_i64().ok_or_else(|| {
                MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be an integer", field_name),
                )
            })?,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be an integer", field_name),
                ));
            }
        };

        // Check min/max
        if let Some(ref min) = self.min {
            if let Some(min_val) = min.as_i64() {
                if n < min_val {
                    return Err(MarkdownValidationError::new(
                        "field_min_value",
                        path,
                        format!("Field '{}' must be >= {}", field_name, min_val),
                    ));
                }
            }
        }

        if let Some(ref max) = self.max {
            if let Some(max_val) = max.as_i64() {
                if n > max_val {
                    return Err(MarkdownValidationError::new(
                        "field_max_value",
                        path,
                        format!("Field '{}' must be <= {}", field_name, max_val),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_float(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let n = match value {
            serde_yaml::Value::Number(n) => n.as_f64().ok_or_else(|| {
                MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a number", field_name),
                )
            })?,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a number", field_name),
                ));
            }
        };

        // Check min/max
        if let Some(ref min) = self.min {
            if let Some(min_val) = min.as_f64() {
                if n < min_val {
                    return Err(MarkdownValidationError::new(
                        "field_min_value",
                        path,
                        format!("Field '{}' must be >= {}", field_name, min_val),
                    ));
                }
            }
        }

        if let Some(ref max) = self.max {
            if let Some(max_val) = max.as_f64() {
                if n > max_val {
                    return Err(MarkdownValidationError::new(
                        "field_max_value",
                        path,
                        format!("Field '{}' must be <= {}", field_name, max_val),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_boolean(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        match value {
            serde_yaml::Value::Bool(_) => Ok(()),
            _ => Err(MarkdownValidationError::new(
                "field_type",
                path,
                format!("Field '{}' must be a boolean", field_name),
            )),
        }
    }

    fn validate_array(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let arr = match value {
            serde_yaml::Value::Sequence(arr) => arr,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be an array", field_name),
                ));
            }
        };

        // Check length constraints
        if let Some(ref min) = self.min {
            if let Some(min_len) = min.as_u64() {
                if arr.len() < min_len as usize {
                    return Err(MarkdownValidationError::new(
                        "field_min_length",
                        path,
                        format!(
                            "Field '{}' must have at least {} items",
                            field_name, min_len
                        ),
                    ));
                }
            }
        }

        if let Some(ref max) = self.max {
            if let Some(max_len) = max.as_u64() {
                if arr.len() > max_len as usize {
                    return Err(MarkdownValidationError::new(
                        "field_max_length",
                        path,
                        format!("Field '{}' must have at most {} items", field_name, max_len),
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_object(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        match value {
            serde_yaml::Value::Mapping(_) => Ok(()),
            _ => Err(MarkdownValidationError::new(
                "field_type",
                path,
                format!("Field '{}' must be an object", field_name),
            )),
        }
    }

    fn validate_date(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a date string (YYYY-MM-DD)", field_name),
                ));
            }
        };

        // Validate date format YYYY-MM-DD
        let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
        if !date_regex.is_match(s) {
            return Err(MarkdownValidationError::new(
                "field_format",
                path,
                format!("Field '{}' must be in format YYYY-MM-DD", field_name),
            ));
        }

        Ok(())
    }

    fn validate_datetime(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!(
                        "Field '{}' must be a datetime string (ISO 8601)",
                        field_name
                    ),
                ));
            }
        };

        // Validate ISO 8601 datetime format
        let datetime_regex =
            Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})?$")
                .unwrap();
        if !datetime_regex.is_match(s) {
            return Err(MarkdownValidationError::new(
                "field_format",
                path,
                format!("Field '{}' must be in ISO 8601 datetime format", field_name),
            ));
        }

        Ok(())
    }

    fn validate_email(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be an email string", field_name),
                ));
            }
        };

        // Simple email validation regex
        let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        if !email_regex.is_match(s) {
            return Err(MarkdownValidationError::new(
                "field_format",
                path,
                format!("Field '{}' must be a valid email address", field_name),
            ));
        }

        Ok(())
    }

    fn validate_url(
        &self,
        field_name: &str,
        value: &serde_yaml::Value,
        path: &std::path::Path,
    ) -> Result<(), MarkdownValidationError> {
        let s = match value {
            serde_yaml::Value::String(s) => s,
            _ => {
                return Err(MarkdownValidationError::new(
                    "field_type",
                    path,
                    format!("Field '{}' must be a URL string", field_name),
                ));
            }
        };

        // Simple URL validation
        let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$").unwrap();
        if !url_regex.is_match(s) {
            return Err(MarkdownValidationError::new(
                "field_format",
                path,
                format!("Field '{}' must be a valid URL (http/https)", field_name),
            ));
        }

        Ok(())
    }
}

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

/// A constraint for validating frontmatter
#[derive(Debug)]
pub struct FrontmatterConstraint {
    name: String,
    schema: FrontmatterSchema,
}

impl FrontmatterConstraint {
    /// Create a new frontmatter constraint
    pub fn new(name: impl Into<String>, schema: FrontmatterSchema) -> Self {
        Self {
            name: name.into(),
            schema,
        }
    }

    /// Get the schema
    pub fn schema(&self) -> &FrontmatterSchema {
        &self.schema
    }
}

impl crate::constraints::Constraint for FrontmatterConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "Validates markdown frontmatter against a schema"
    }

    fn validate(
        &self,
        path: &std::path::Path,
        context: &crate::constraints::ConstraintContext,
    ) -> crate::constraints::ConstraintResult<crate::constraints::ConstraintOutput> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::constraints::ConstraintError::io(path, format!("Failed to read file: {}", e))
        })?;

        let parser = super::parser::MarkdownParser::new();
        let document = parser.parse(&content).map_err(|e| {
            crate::constraints::ConstraintError::validation(
                &self.name,
                path,
                format!("Failed to parse markdown: {}", e),
            )
        })?;

        let failures = self.schema.validate(&document, path).map_err(|e| {
            crate::constraints::ConstraintError::validation(
                &self.name,
                path,
                format!("Validation failed: {}", e),
            )
        })?;

        let passed = failures.is_empty();
        let failures_collection = crate::constraints::ValidationFailures::from(failures);

        Ok(
            crate::constraints::ConstraintOutput::new(&self.name, path, passed)
                .with_failures(failures_collection),
        )
    }

    fn applies_to(&self, path: &std::path::Path) -> bool {
        path.extension()
            .map(|ext| ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown"))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frontmatter_schema_builder() {
        let schema = FrontmatterSchema::new()
            .required()
            .with_field("title", FieldValidator::new(FieldType::String).required())
            .with_field("date", FieldValidator::new(FieldType::Date).required())
            .allow_additional_fields(true);

        assert!(schema.required);
        assert_eq!(schema.fields.len(), 2);
        assert!(schema.allow_additional_fields);
    }

    #[test]
    fn test_field_validator_builder() {
        let validator = FieldValidator::new(FieldType::String)
            .required()
            .with_pattern(r"^\d{4}$")
            .with_min(4u64)
            .with_max(4u64)
            .with_allowed_values(vec!["2023", "2024"]);

        assert!(validator.required);
        assert!(validator.pattern.is_some());
        assert!(validator.min.is_some());
        assert!(validator.max.is_some());
        assert!(validator.allowed_values.is_some());
    }

    #[test]
    fn test_validate_string_type() {
        let validator = FieldValidator::new(FieldType::String);
        let path = std::path::PathBuf::from("/test.md");

        let valid = serde_yaml::Value::String("test".to_string());
        assert!(validator.validate("field", &valid, &path).is_ok());

        let invalid = serde_yaml::Value::Number(42.into());
        assert!(validator.validate("field", &invalid, &path).is_err());
    }

    #[test]
    fn test_validate_string_pattern() {
        let validator = FieldValidator::new(FieldType::String).with_pattern(r"^\d{4}-\d{2}-\d{2}$");
        let path = std::path::PathBuf::from("/test.md");

        let valid = serde_yaml::Value::String("2024-01-15".to_string());
        assert!(validator.validate("field", &valid, &path).is_ok());

        let invalid = serde_yaml::Value::String("invalid".to_string());
        assert!(validator.validate("field", &invalid, &path).is_err());
    }

    #[test]
    fn test_validate_integer() {
        let validator = FieldValidator::new(FieldType::Integer)
            .with_min(0i64)
            .with_max(100i64);
        let path = std::path::PathBuf::from("/test.md");

        let valid = serde_yaml::Value::Number(50.into());
        assert!(validator.validate("field", &valid, &path).is_ok());

        let too_small = serde_yaml::Value::Number((-1i64).into());
        assert!(validator.validate("field", &too_small, &path).is_err());

        let too_large = serde_yaml::Value::Number(101i64.into());
        assert!(validator.validate("field", &too_large, &path).is_err());
    }

    #[test]
    fn test_validate_date() {
        let validator = FieldValidator::new(FieldType::Date);
        let path = std::path::PathBuf::from("/test.md");

        let valid = serde_yaml::Value::String("2024-01-15".to_string());
        assert!(validator.validate("field", &valid, &path).is_ok());

        let invalid = serde_yaml::Value::String("not-a-date".to_string());
        assert!(validator.validate("field", &invalid, &path).is_err());
    }

    #[test]
    fn test_validate_email() {
        let validator = FieldValidator::new(FieldType::Email);
        let path = std::path::PathBuf::from("/test.md");

        let valid = serde_yaml::Value::String("test@example.com".to_string());
        assert!(validator.validate("field", &valid, &path).is_ok());

        let invalid = serde_yaml::Value::String("not-an-email".to_string());
        assert!(validator.validate("field", &invalid, &path).is_err());
    }

    #[test]
    fn test_validate_allowed_values() {
        let validator = FieldValidator::new(FieldType::String).with_allowed_values(vec![
            "draft",
            "published",
            "archived",
        ]);
        let path = std::path::PathBuf::from("/test.md");

        let valid = serde_yaml::Value::String("published".to_string());
        assert!(validator.validate("field", &valid, &path).is_ok());

        let invalid = serde_yaml::Value::String("invalid".to_string());
        assert!(validator.validate("field", &invalid, &path).is_err());
    }

    #[test]
    fn test_frontmatter_schema_validation() {
        let schema = FrontmatterSchema::new()
            .required()
            .with_field("title", FieldValidator::new(FieldType::String).required())
            .with_field("author", FieldValidator::new(FieldType::String));

        // Document with frontmatter
        let doc_with_frontmatter = super::MarkdownDocument {
            content: "---\ntitle: Test\nauthor: John\n---\n\n# Test".to_string(),
            frontmatter: Some("title: Test\nauthor: John".to_string()),
            body: "# Test".to_string(),
            headings: vec![],
            links: vec![],
            code_blocks: vec![],
            text_content: "Test".to_string(),
            line_count: 5,
            word_count: 1,
        };

        let path = std::path::PathBuf::from("/test.md");
        let failures = schema.validate(&doc_with_frontmatter, &path).unwrap();
        assert!(failures.is_empty());

        // Document without frontmatter
        let doc_without_frontmatter = super::MarkdownDocument {
            content: "# Test".to_string(),
            frontmatter: None,
            body: "# Test".to_string(),
            headings: vec![],
            links: vec![],
            code_blocks: vec![],
            text_content: "Test".to_string(),
            line_count: 1,
            word_count: 1,
        };

        let failures = schema.validate(&doc_without_frontmatter, &path).unwrap();
        assert_eq!(failures.len(), 1);
    }

    #[test]
    fn test_field_type_display() {
        assert_eq!(FieldType::String.display_name(), "string");
        assert_eq!(FieldType::Integer.display_name(), "integer");
        assert_eq!(FieldType::Date.display_name(), "date");
    }
}
