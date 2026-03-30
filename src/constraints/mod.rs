//! Constraint system for validation rules
//!
//! This module provides the core constraint validation infrastructure:
//! - Constraint trait for implementing validation rules
//! - Trigger system for when constraints run
//! - Severity mapping based on project maturity
//! - Built-in constraints (file_size, naming)

pub mod children_limit;
pub mod error;
pub mod file_size;
pub mod ls_lint;
pub mod naming;
pub mod severity;
pub mod r#trait;
pub mod trigger;

pub use children_limit::ChildrenLimitConstraint;
pub use error::{ConstraintError, ConstraintResult, ValidationFailure, ValidationFailures};
pub use file_size::{FileSizeConstraint, FileSizeLimit, FileSizeRule};
pub use ls_lint::{
    ComplexExtension, DirectoryConstraint, DirectoryRule, DirectoryValidationConfig,
    ExtensionPattern, MultiPartExtensionRule, MultipleRuleSyntax, PathRule, PathRuleConfig,
};
pub use naming::{
    CaseConvention, ExtensionRule, NamingConstraint, NamingPattern, NamingRule,
};
pub use severity::{Severity, SeverityConfig, SeverityMapping};
pub use r#trait::{Constraint, ConstraintContext, ConstraintOutput};
pub use trigger::{ConstraintTrigger, FileChangeTrigger, ManualTrigger, MaturityTrigger, TriggerRegistry};

use std::path::Path;
use serde::{Serialize, Deserialize};

/// Configuration for the constraint system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConstraintConfig {
    /// Severity configuration for maturity-based mapping
    pub severity: SeverityConfig,
    /// Whether to enable file watching triggers
    pub enable_file_watching: bool,
    /// Whether to enable manual triggers
    pub enable_manual_triggers: bool,
}

impl ConstraintConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_severity_config(mut self, config: SeverityConfig) -> Self {
        self.severity = config;
        self
    }

    pub fn enable_file_watching(mut self) -> Self {
        self.enable_file_watching = true;
        self
    }

    pub fn enable_manual_triggers(mut self) -> Self {
        self.enable_manual_triggers = true;
        self
    }
}

/// The main constraint engine that orchestrates validation
#[derive(Debug)]
pub struct ConstraintEngine {
    constraints: Vec<Box<dyn Constraint>>,
    trigger_registry: TriggerRegistry,
    config: ConstraintConfig,
}

impl ConstraintEngine {
    pub fn new(config: ConstraintConfig) -> Self {
        Self {
            constraints: Vec::new(),
            trigger_registry: TriggerRegistry::new(),
            config,
        }
    }

    /// Register a constraint with the engine
    pub fn register_constraint(&mut self, constraint: Box<dyn Constraint>) {
        self.constraints.push(constraint);
    }

    /// Register a trigger
    pub fn register_trigger(&mut self, trigger: Box<dyn ConstraintTrigger>) {
        self.trigger_registry.register(trigger);
    }

    /// Validate a path against all registered constraints
    pub fn validate<P: AsRef<Path>>(
        &self,
        path: P,
        context: &ConstraintContext,
    ) -> Vec<ConstraintResult<ConstraintOutput>> {
        let path = path.as_ref();
        let mut results = Vec::new();

        for constraint in &self.constraints {
            // Check if trigger conditions are met
            if self.trigger_registry.should_trigger(constraint.name(), path, context) {
                let result = constraint.validate(path, context);
                results.push(result);
            }
        }

        results
    }

    /// Validate a specific constraint by name
    pub fn validate_constraint<P: AsRef<Path>>(
        &self,
        name: &str,
        path: P,
        context: &ConstraintContext,
    ) -> Option<ConstraintResult<ConstraintOutput>> {
        let path = path.as_ref();

        for constraint in &self.constraints {
            if constraint.name() == name {
                return Some(constraint.validate(path, context));
            }
        }

        None
    }

    /// Get all registered constraint names
    pub fn constraint_names(&self) -> Vec<&str> {
        self.constraints.iter().map(|c| c.name()).collect()
    }

    /// Get the severity mapping configuration
    pub fn severity_config(&self) -> &SeverityConfig {
        &self.config.severity
    }
}

impl Default for ConstraintEngine {
    fn default() -> Self {
        Self::new(ConstraintConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_constraint_config_default() {
        let config = ConstraintConfig::default();
        assert!(!config.enable_file_watching);
        assert!(!config.enable_manual_triggers);
    }

    #[test]
    fn test_constraint_config_builder() {
        let config = ConstraintConfig::new()
            .enable_file_watching()
            .enable_manual_triggers();
        
        assert!(config.enable_file_watching);
        assert!(config.enable_manual_triggers);
    }

    #[test]
    fn test_constraint_engine_empty() {
        let engine = ConstraintEngine::default();
        let context = ConstraintContext::new();
        let results = engine.validate(PathBuf::from("/tmp"), &context);
        assert!(results.is_empty());
    }
}