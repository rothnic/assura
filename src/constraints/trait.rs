//! Constraint trait definition
//!
//! The Constraint trait is the core abstraction for all validation rules.
//! Constraints can be combined, configured, and executed in parallel.

use std::fmt::Debug;
use std::path::{Path, PathBuf};

use super::error::{ConstraintResult, ValidationFailures};
use super::severity::Severity;
use crate::maturity::MaturityLevel;

/// Context for constraint execution
#[derive(Debug, Clone)]
pub struct ConstraintContext {
    /// The project root path
    pub project_root: Option<PathBuf>,
    /// The current maturity level
    pub maturity_level: MaturityLevel,
    /// Whether this is a manual invocation
    pub is_manual: bool,
    /// Whether to fail fast on first validation failure
    pub fail_fast: bool,
    /// Whether to enable recursive validation for directories
    pub recursive_validation: bool,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl ConstraintContext {
    pub fn new() -> Self {
        Self {
            project_root: None,
            maturity_level: MaturityLevel::Raw,
            is_manual: false,
            fail_fast: false,
            recursive_validation: true,
            metadata: std::collections::HashMap::new(),
        }
    }

    pub fn with_project_root<P: Into<PathBuf>>(mut self, root: P) -> Self {
        self.project_root = Some(root.into());
        self
    }

    pub fn with_maturity_level(mut self, level: MaturityLevel) -> Self {
        self.maturity_level = level;
        self
    }

    pub fn manual(mut self) -> Self {
        self.is_manual = true;
        self
    }

    pub fn with_fail_fast(mut self) -> Self {
        self.fail_fast = true;
        self
    }

    pub fn with_recursive_validation(mut self, enabled: bool) -> Self {
        self.recursive_validation = enabled;
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get the effective project root
    pub fn project_root(&self) -> Option<&Path> {
        self.project_root.as_deref()
    }

    /// Get the current maturity level
    pub fn maturity_level(&self) -> MaturityLevel {
        self.maturity_level
    }
}

impl Default for ConstraintContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Output from constraint validation
#[derive(Debug, Clone)]
pub struct ConstraintOutput {
    /// Constraint name
    pub constraint_name: String,
    /// Path that was validated
    pub path: PathBuf,
    /// Whether validation passed
    pub passed: bool,
    /// Severity level
    pub severity: Severity,
    /// Duration of validation
    pub duration_ms: u64,
    /// Validation failures (empty if passed)
    pub failures: ValidationFailures,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
}

impl ConstraintOutput {
    pub fn new<P: Into<PathBuf>>(
        constraint_name: impl Into<String>,
        path: P,
        passed: bool,
    ) -> Self {
        Self {
            constraint_name: constraint_name.into(),
            path: path.into(),
            passed,
            severity: Severity::Medium,
            duration_ms: 0,
            failures: ValidationFailures::new(),
            metadata: None,
        }
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    pub fn with_failures(mut self, failures: ValidationFailures) -> Self {
        self.failures = failures;
        self
    }

    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn success<P: Into<PathBuf>>(
        constraint_name: impl Into<String>,
        path: P,
    ) -> Self {
        Self::new(constraint_name, path, true)
    }

    pub fn failure<P: Into<PathBuf>>(
        constraint_name: impl Into<String>,
        path: P,
        failures: ValidationFailures,
    ) -> Self {
        Self::new(constraint_name, path, false).with_failures(failures)
    }
}

/// The core constraint trait
///
/// All validation rules implement this trait. Constraints should be:
/// - Stateless (all state in config)
/// - Send + Sync (for parallel execution)
/// - Deterministic (same input always produces same output)
pub trait Constraint: Send + Sync + Debug {
    /// Get the unique name of this constraint
    fn name(&self) -> &str;

    /// Get a human-readable description
    fn description(&self) -> &str {
        "No description provided"
    }

    /// Validate a path
    ///
    /// # Arguments
    /// * `path` - The path to validate
    /// * `context` - Execution context with maturity level and other info
    ///
    /// # Returns
    /// Result containing validation output
    fn validate(
        &self,
        path: &Path,
        context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput>;

    /// Check if this constraint applies to a path
    ///
    /// Default implementation always returns true
    fn applies_to(&self, _path: &Path) -> bool {
        true
    }

    /// Get the default severity for this constraint
    fn default_severity(&self) -> Severity {
        Severity::Medium
    }

    /// Get severity adjusted for maturity level
    fn severity_for_maturity(&self, level: MaturityLevel) -> Severity {
        use MaturityLevel::*;
        use Severity::*;

        match (self.default_severity(), level) {
            // Critical stays critical at all levels
            (Critical, _) => Critical,
            // High stays high at mature+ levels, becomes critical at raw
            (High, Raw) => Critical,
            (High, _) => High,
            // Medium becomes high at established, stays medium elsewhere
            (Medium, Established) => High,
            (Medium, _) => Medium,
            // Low becomes medium at established
            (Low, Established) => Medium,
            (Low, Mature) => Low,
            (Low, _) => Low,
        }
    }
}

/// A composite constraint that runs multiple constraints
#[derive(Debug)]
pub struct CompositeConstraint {
    name: String,
    description: String,
    constraints: Vec<Box<dyn Constraint>>,
    strategy: CompositeStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeStrategy {
    /// All constraints must pass
    All,
    /// At least one constraint must pass
    Any,
    /// Exactly one constraint must pass
    ExactlyOne,
}

impl CompositeConstraint {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            constraints: Vec::new(),
            strategy: CompositeStrategy::All,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_strategy(mut self, strategy: CompositeStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn add_constraint(mut self, constraint: Box<dyn Constraint>) -> Self {
        self.constraints.push(constraint);
        self
    }
}

impl Constraint for CompositeConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn validate(
        &self,
        path: &Path,
        context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        let start = std::time::Instant::now();
        let mut all_failures = ValidationFailures::new();
        let mut passed_count = 0;

        for constraint in &self.constraints {
            match constraint.validate(path, context) {
                Ok(output) => {
                    if output.passed {
                        passed_count += 1;
                    } else {
                        for failure in output.failures.into_iter() {
                            all_failures.add(failure);
                        }
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        let passed = match self.strategy {
            CompositeStrategy::All => passed_count == self.constraints.len(),
            CompositeStrategy::Any => passed_count > 0,
            CompositeStrategy::ExactlyOne => passed_count == 1,
        };

        let duration = start.elapsed().as_millis() as u64;

        Ok(ConstraintOutput::new(&self.name, path, passed)
            .with_duration(duration)
            .with_failures(all_failures))
    }
}

/// A constraint that always passes (useful for testing)
#[derive(Debug)]
pub struct AlwaysPassConstraint {
    name: String,
}

impl AlwaysPassConstraint {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Constraint for AlwaysPassConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn validate(
        &self,
        path: &Path,
        _context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        Ok(ConstraintOutput::success(&self.name, path))
    }
}

/// A constraint that always fails (useful for testing)
#[derive(Debug)]
pub struct AlwaysFailConstraint {
    name: String,
    message: String,
}

impl AlwaysFailConstraint {
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            message: message.into(),
        }
    }
}

impl Constraint for AlwaysFailConstraint {
    fn name(&self) -> &str {
        &self.name
    }

    fn validate(
        &self,
        path: &Path,
        _context: &ConstraintContext,
    ) -> ConstraintResult<ConstraintOutput> {
        let failure = super::error::ValidationFailure::new(
            &self.name,
            path,
            &self.message,
        );
        Ok(ConstraintOutput::failure(
            &self.name,
            path,
            ValidationFailures::new().with_failure(failure),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constraints::error::ValidationFailure;

    #[test]
    fn test_constraint_context_builder() {
        let context = ConstraintContext::new()
            .with_project_root("/project")
            .with_maturity_level(MaturityLevel::Mature)
            .manual()
            .with_fail_fast()
            .with_metadata("key", "value");

        assert_eq!(context.project_root, Some(PathBuf::from("/project")));
        assert_eq!(context.maturity_level, MaturityLevel::Mature);
        assert!(context.is_manual);
        assert!(context.fail_fast);
        assert_eq!(context.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_constraint_output_builder() {
        let output = ConstraintOutput::new("test", "/path", true)
            .with_severity(Severity::High)
            .with_duration(100)
            .with_metadata(serde_json::json!({"key": "value"}));

        assert_eq!(output.constraint_name, "test");
        assert_eq!(output.path, PathBuf::from("/path"));
        assert!(output.passed);
        assert_eq!(output.severity, Severity::High);
        assert_eq!(output.duration_ms, 100);
        assert!(output.metadata.is_some());
    }

    #[test]
    fn test_composite_constraint_all() {
        let composite = CompositeConstraint::new("test")
            .with_strategy(CompositeStrategy::All)
            .add_constraint(Box::new(AlwaysPassConstraint::new("pass1")))
            .add_constraint(Box::new(AlwaysPassConstraint::new("pass2")));

        let context = ConstraintContext::new();
        let result = composite.validate(Path::new("/test"), &context).unwrap();

        assert!(result.passed);
    }

    #[test]
    fn test_composite_constraint_any() {
        let composite = CompositeConstraint::new("test")
            .with_strategy(CompositeStrategy::Any)
            .add_constraint(Box::new(AlwaysFailConstraint::new("fail", "fail")))
            .add_constraint(Box::new(AlwaysPassConstraint::new("pass")));

        let context = ConstraintContext::new();
        let result = composite.validate(Path::new("/test"), &context).unwrap();

        assert!(result.passed);
    }

    #[test]
    fn test_always_pass_constraint() {
        let constraint = AlwaysPassConstraint::new("always_pass");
        let context = ConstraintContext::new();
        let result = constraint.validate(Path::new("/test"), &context).unwrap();

        assert!(result.passed);
        assert!(result.failures.is_empty());
    }

    #[test]
    fn test_always_fail_constraint() {
        let constraint = AlwaysFailConstraint::new("always_fail", "test failure");
        let context = ConstraintContext::new();
        let result = constraint.validate(Path::new("/test"), &context).unwrap();

        assert!(!result.passed);
        assert_eq!(result.failures.len(), 1);
    }
}
