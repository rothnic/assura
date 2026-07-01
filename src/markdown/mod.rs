//! Markdown schema validation module
//!
//! This module provides comprehensive markdown validation capabilities:
//! - Frontmatter schema validation (YAML)
//! - Heading structure hierarchy checks
//! - Template enforcement for documentation
//! - Content pattern validation

pub mod error;
pub mod frontmatter;
pub mod headings;
pub mod links {
    pub(crate) use crate::markdown_links::*;
}
pub mod parser;
pub mod schema;
pub mod template;

pub use error::{MarkdownError, MarkdownResult, MarkdownValidationError};
pub use frontmatter::{FieldType, FieldValidator, FrontmatterConstraint, FrontmatterSchema};
pub use headings::{HeadingConstraint, HeadingStructure, HeadingValidator};
pub use parser::{Heading, HeadingHierarchy, HeadingLevel, MarkdownDocument, MarkdownParser};
pub use schema::{MarkdownSchema, MarkdownValidationRule, SchemaDefinition, ValidationConfig};
pub use template::{SectionDefinition, SectionValidator, TemplateConstraint, TemplateDefinition};

use crate::constraints::{
    Constraint, ConstraintContext, ConstraintOutput, ConstraintResult, ValidationFailures,
};
use std::path::Path;

/// Markdown constraint that validates documents against schemas
#[derive(Debug)]
pub struct MarkdownConstraint {
    schemas: Vec<MarkdownSchema>,
    default_schema: Option<String>,
}

impl MarkdownConstraint {
    /// Create a new markdown constraint
    pub fn new() -> Self {
        Self {
            schemas: Vec::new(),
            default_schema: None,
        }
    }

    /// Register a schema with this constraint
    pub fn register_schema(mut self, schema: MarkdownSchema) -> Self {
        self.schemas.push(schema);
        self
    }

    /// Set the default schema name
    pub fn with_default_schema(mut self, name: impl Into<String>) -> Self {
        self.default_schema = Some(name.into());
        self
    }

    /// Get a schema by name
    pub fn get_schema(&self, name: &str) -> Option<&MarkdownSchema> {
        self.schemas.iter().find(|s| s.name == name)
    }

    /// Validate a markdown document
    fn validate_markdown(
        &self,
        path: &Path,
        content: &str,
        schema: &MarkdownSchema,
    ) -> ConstraintResult<ConstraintOutput> {
        let start = std::time::Instant::now();
        let parser = MarkdownParser::new();
        let document = parser.parse(content)?;

        let mut failures = ValidationFailures::new();

        // Validate frontmatter if schema requires it
        if let Some(ref frontmatter_schema) = schema.frontmatter {
            let frontmatter_failures = frontmatter_schema.validate(&document, path)?;
            for failure in frontmatter_failures {
                failures.add(failure);
            }
        }

        // Validate heading structure if schema requires it
        if let Some(ref heading_validator) = schema.headings {
            let heading_failures = heading_validator.validate(&document, path)?;
            for failure in heading_failures {
                failures.add(failure);
            }
        }

        // Validate template if schema requires it
        if let Some(ref template) = schema.template {
            let template_failures = template.validate(&document, path)?;
            for failure in template_failures {
                failures.add(failure);
            }
        }

        let duration = start.elapsed().as_millis() as u64;
        let passed = failures.is_empty();

        Ok(ConstraintOutput::new("markdown_schema", path, passed)
            .with_duration(duration)
            .with_failures(failures))
    }
}

impl Default for MarkdownConstraint {
    fn default() -> Self {
        Self::new()
    }
}

impl Constraint for MarkdownConstraint {
    fn name(&self) -> &str {
        "markdown_schema"
    }

    fn description(&self) -> &str {
        "Validates markdown documents against defined schemas"
    }

    fn validate(
        &self,
        path: &Path,
        context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            crate::constraints::ConstraintError::io(
                path,
                format!("Failed to read markdown file: {}", e),
            )
        })?;

        // Determine which schema to use
        let schema_name = context
            .metadata
            .get("markdown_schema")
            .cloned()
            .or_else(|| self.default_schema.clone())
            .unwrap_or_else(|| "default".to_string());

        let schema = self.get_schema(&schema_name).ok_or_else(|| {
            crate::constraints::ConstraintError::configuration(
                "markdown_schema",
                format!("Schema '{}' not found", schema_name),
            )
        })?;

        self.validate_markdown(path, &content, schema)
    }

    fn applies_to(&self, path: &Path) -> bool {
        path.extension()
            .map(|ext| {
                ext.eq_ignore_ascii_case("md")
                    || ext.eq_ignore_ascii_case("markdown")
                    || ext.eq_ignore_ascii_case("mdown")
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_constraint_applies_to() {
        let constraint = MarkdownConstraint::new();

        assert!(constraint.applies_to(Path::new("/test/file.md")));
        assert!(constraint.applies_to(Path::new("/test/file.markdown")));
        assert!(constraint.applies_to(Path::new("/test/file.mdown")));
        assert!(!constraint.applies_to(Path::new("/test/file.txt")));
        assert!(!constraint.applies_to(Path::new("/test/file")));
    }

    #[test]
    fn test_markdown_constraint_schema_lookup() {
        let constraint =
            MarkdownConstraint::new().register_schema(MarkdownSchema::new("test_schema"));

        assert!(constraint.get_schema("test_schema").is_some());
        assert!(constraint.get_schema("nonexistent").is_none());
    }
}
