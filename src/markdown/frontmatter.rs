//! Frontmatter validation
//!
//! Validates YAML frontmatter in markdown documents against defined schemas.
//! Supports required fields, type checking, and custom validation rules.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{MarkdownResult, MarkdownValidationError};
use super::parser::MarkdownDocument;
use crate::constraints::ValidationFailure;

mod frontmatter_field;
pub use frontmatter_field::{FieldType, FieldValidator};

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
        _context: &crate::constraints::ConstraintContext,
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
mod frontmatter_tests;
