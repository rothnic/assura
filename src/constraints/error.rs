//! Error types for constraint validation

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for constraint operations
#[derive(Error, Debug, Clone)]
pub enum ConstraintError {
    /// I/O error during validation
    #[error("I/O error: {message}")]
    Io { path: PathBuf, message: String },

    /// Constraint validation failed
    #[error("Constraint '{constraint}' failed: {message}")]
    Validation {
        constraint: String,
        path: PathBuf,
        message: String,
    },

    /// Invalid configuration for constraint
    #[error("Invalid configuration for '{constraint}': {message}")]
    Configuration { constraint: String, message: String },

    /// Pattern matching error
    #[error("Pattern error in '{constraint}': {message}")]
    Pattern { constraint: String, message: String },

    /// Trigger condition error
    #[error("Trigger error: {message}")]
    Trigger { message: String },

    /// Severity mapping error
    #[error("Severity mapping error: {message}")]
    Severity { message: String },

    /// Unknown constraint
    #[error("Unknown constraint: {name}")]
    UnknownConstraint { name: String },

    /// Constraint execution error
    #[error("Constraint execution failed: {message}")]
    Execution { constraint: String, message: String },
}

impl ConstraintError {
    /// Create an I/O error
    pub fn io<P: Into<PathBuf>>(path: P, message: impl Into<String>) -> Self {
        Self::Io {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a validation error
    pub fn validation<P: Into<PathBuf>>(
        constraint: impl Into<String>,
        path: P,
        message: impl Into<String>,
    ) -> Self {
        Self::Validation {
            constraint: constraint.into(),
            path: path.into(),
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration(constraint: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Configuration {
            constraint: constraint.into(),
            message: message.into(),
        }
    }

    /// Create a pattern error
    pub fn pattern(constraint: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Pattern {
            constraint: constraint.into(),
            message: message.into(),
        }
    }

    /// Create a trigger error
    pub fn trigger(message: impl Into<String>) -> Self {
        Self::Trigger {
            message: message.into(),
        }
    }

    /// Create a severity error
    pub fn severity(message: impl Into<String>) -> Self {
        Self::Severity {
            message: message.into(),
        }
    }

    /// Create an unknown constraint error
    pub fn unknown_constraint(name: impl Into<String>) -> Self {
        Self::UnknownConstraint { name: name.into() }
    }

    /// Create an execution error
    pub fn execution(constraint: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Execution {
            constraint: constraint.into(),
            message: message.into(),
        }
    }

    /// Get the path associated with this error, if any
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            Self::Io { path, .. } | Self::Validation { path, .. } => Some(path),
            _ => None,
        }
    }

    /// Get the constraint name associated with this error, if any
    pub fn constraint(&self) -> Option<&str> {
        match self {
            Self::Validation { constraint, .. }
            | Self::Configuration { constraint, .. }
            | Self::Pattern { constraint, .. }
            | Self::Execution { constraint, .. } => Some(constraint),
            _ => None,
        }
    }
}

/// Result type for constraint operations
pub type ConstraintResult<T> = Result<T, ConstraintError>;

/// Validation failure details
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValidationFailure {
    /// The constraint that failed
    pub constraint: String,
    /// The path that failed validation
    pub path: PathBuf,
    /// Human-readable failure message
    pub message: String,
    /// Optional suggestion for fixing the issue
    pub suggestion: Option<String>,
}

impl ValidationFailure {
    pub fn new<P: Into<PathBuf>>(
        constraint: impl Into<String>,
        path: P,
        message: impl Into<String>,
    ) -> Self {
        Self {
            constraint: constraint.into(),
            path: path.into(),
            message: message.into(),
            suggestion: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

/// Collection of validation failures
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationFailures {
    failures: Vec<ValidationFailure>,
}

impl ValidationFailures {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_failure(mut self, failure: ValidationFailure) -> Self {
        self.failures.push(failure);
        self
    }

    pub fn add(&mut self, failure: ValidationFailure) {
        self.failures.push(failure);
    }

    pub fn is_empty(&self) -> bool {
        self.failures.is_empty()
    }

    pub fn len(&self) -> usize {
        self.failures.len()
    }

    pub fn failures(&self) -> &[ValidationFailure] {
        &self.failures
    }

    pub fn into_failures(self) -> Vec<ValidationFailure> {
        self.failures
    }
}

impl From<Vec<ValidationFailure>> for ValidationFailures {
    fn from(failures: Vec<ValidationFailure>) -> Self {
        Self { failures }
    }
}

impl IntoIterator for ValidationFailures {
    type Item = ValidationFailure;
    type IntoIter = std::vec::IntoIter<ValidationFailure>;

    fn into_iter(self) -> Self::IntoIter {
        self.failures.into_iter()
    }
}

impl From<crate::markdown::MarkdownError> for ConstraintError {
    fn from(err: crate::markdown::MarkdownError) -> Self {
        ConstraintError::execution("markdown", err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_error_io() {
        let err = ConstraintError::io("/test/path", "permission denied");
        assert!(matches!(err, ConstraintError::Io { .. }));
        assert_eq!(err.path(), Some(&PathBuf::from("/test/path")));
    }

    #[test]
    fn test_constraint_error_validation() {
        let err = ConstraintError::validation("file_size", "/test/file.txt", "too large");
        assert!(matches!(err, ConstraintError::Validation { .. }));
        assert_eq!(err.constraint(), Some("file_size"));
    }

    #[test]
    fn test_validation_failure() {
        let failure = ValidationFailure::new("naming", "/test/File.txt", "wrong case")
            .with_suggestion("rename to file.txt");

        assert_eq!(failure.constraint, "naming");
        assert_eq!(failure.path, PathBuf::from("/test/File.txt"));
        assert_eq!(failure.message, "wrong case");
        assert_eq!(failure.suggestion, Some("rename to file.txt".to_string()));
    }

    #[test]
    fn test_validation_failures_collection() {
        let mut failures = ValidationFailures::new();
        failures.add(ValidationFailure::new("test", "/a", "error 1"));
        failures.add(ValidationFailure::new("test", "/b", "error 2"));

        assert_eq!(failures.len(), 2);
        assert!(!failures.is_empty());
    }
}
