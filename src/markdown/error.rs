//! Error types for markdown validation

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for markdown operations
#[derive(Error, Debug, Clone)]
pub enum MarkdownError {
    /// I/O error during parsing
    #[error("I/O error: {message}")]
    Io { path: PathBuf, message: String },

    /// Parsing error
    #[error("Parse error: {message}")]
    Parse {
        path: PathBuf,
        line: Option<usize>,
        message: String,
    },

    /// Frontmatter error
    #[error("Frontmatter error: {message}")]
    Frontmatter { path: PathBuf, message: String },

    /// YAML parsing error
    #[error("YAML error: {message}")]
    Yaml { path: PathBuf, message: String },

    /// Schema error
    #[error("Schema error: {message}")]
    Schema { schema: String, message: String },

    /// Template error
    #[error("Template error: {message}")]
    Template { template: String, message: String },

    /// Validation error
    #[error("Validation error: {message}")]
    Validation { path: PathBuf, message: String },

    /// Invalid field type
    #[error("Invalid field type for '{field}': expected {expected}, got {actual}")]
    FieldType {
        field: String,
        expected: String,
        actual: String,
    },

    /// Missing required field
    #[error("Missing required field: {field}")]
    MissingField { field: String },

    /// Heading structure error
    #[error("Heading error: {message}")]
    Heading { message: String },

    /// Configuration error
    #[error("Configuration error: {message}")]
    Configuration { message: String },
}

impl MarkdownError {
    /// Create an I/O error
    pub fn io<P: Into<PathBuf>>(path: P, message: impl Into<String>) -> Self {
        Self::Io {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a parse error
    pub fn parse<P: Into<PathBuf>>(
        path: P,
        line: Option<usize>,
        message: impl Into<String>,
    ) -> Self {
        Self::Parse {
            path: path.into(),
            line,
            message: message.into(),
        }
    }

    /// Create a frontmatter error
    pub fn frontmatter<P: Into<PathBuf>>(path: P, message: impl Into<String>) -> Self {
        Self::Frontmatter {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a YAML error
    pub fn yaml<P: Into<PathBuf>>(path: P, message: impl Into<String>) -> Self {
        Self::Yaml {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a schema error
    pub fn schema(schema: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Schema {
            schema: schema.into(),
            message: message.into(),
        }
    }

    /// Create a template error
    pub fn template(template: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Template {
            template: template.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation<P: Into<PathBuf>>(path: P, message: impl Into<String>) -> Self {
        Self::Validation {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a field type error
    pub fn field_type(
        field: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::FieldType {
            field: field.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create a missing field error
    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    /// Create a heading error
    pub fn heading(message: impl Into<String>) -> Self {
        Self::Heading {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Get the path associated with this error, if any
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Io { path, .. }
            | Self::Parse { path, .. }
            | Self::Frontmatter { path, .. }
            | Self::Yaml { path, .. }
            | Self::Validation { path, .. } => Some(path),
            _ => None,
        }
    }

    /// Get the line number associated with this error, if any
    pub fn line(&self) -> Option<usize> {
        match self {
            Self::Parse { line, .. } => *line,
            _ => None,
        }
    }
}

/// Result type for markdown operations
pub type MarkdownResult<T> = Result<T, MarkdownError>;

/// Validation error for markdown content
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownValidationError {
    /// The validation rule that failed
    pub rule: String,
    /// The path that failed
    pub path: PathBuf,
    /// The line number where the error occurred
    pub line: Option<usize>,
    /// Human-readable error message
    pub message: String,
    /// Optional suggestion for fixing
    pub suggestion: Option<String>,
}

impl MarkdownValidationError {
    pub fn new<P: Into<PathBuf>>(
        rule: impl Into<String>,
        path: P,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule: rule.into(),
            path: path.into(),
            line: None,
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Convert to a Constraint ValidationFailure
    pub fn into_validation_failure(self) -> crate::constraints::ValidationFailure {
        crate::constraints::ValidationFailure::new(self.rule, self.path, self.message)
            .with_suggestion(self.suggestion.unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_error_io() {
        let err = MarkdownError::io("/test.md", "permission denied");
        assert!(matches!(err, MarkdownError::Io { .. }));
        assert_eq!(err.path(), Some(&PathBuf::from("/test.md")));
    }

    #[test]
    fn test_markdown_error_parse_with_line() {
        let err = MarkdownError::parse("/test.md", Some(10), "unexpected token");
        assert!(matches!(err, MarkdownError::Parse { .. }));
        assert_eq!(err.line(), Some(10));
    }

    #[test]
    fn test_markdown_validation_error() {
        let error = MarkdownValidationError::new("heading_rule", "/test.md", "H1 missing")
            .with_line(1)
            .with_suggestion("Add an H1 heading");

        assert_eq!(error.rule, "heading_rule");
        assert_eq!(error.path, PathBuf::from("/test.md"));
        assert_eq!(error.line, Some(1));
        assert_eq!(error.message, "H1 missing");
        assert_eq!(error.suggestion, Some("Add an H1 heading".to_string()));
    }

    #[test]
    fn test_missing_field_error() {
        let err = MarkdownError::missing_field("title");
        assert!(matches!(err, MarkdownError::MissingField { .. }));
    }

    #[test]
    fn test_field_type_error() {
        let err = MarkdownError::field_type("date", "string", "number");
        assert!(matches!(err, MarkdownError::FieldType { .. }));
    }
}
