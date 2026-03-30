//! Trigger system for constraint execution
//!
//! Triggers determine when constraints should run. The system supports:
//! - File change triggers (on file creation/modification/deletion)
//! - Maturity-based triggers (run based on project maturity level)
//! - Manual triggers (explicit invocation)
//! - Trigger registration and management

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::error::{ConstraintError, ConstraintResult};
use super::r#trait::ConstraintContext;
use crate::maturity::MaturityLevel;

/// A trigger that determines when a constraint should run
pub trait ConstraintTrigger: Send + Sync + std::fmt::Debug {
    /// Get the trigger name
    fn name(&self) -> &str;

    /// Check if this trigger should fire for the given path and context
    fn should_trigger(&self, constraint_name: &str, path: &Path, context: &ConstraintContext) -> bool;

    /// Get the trigger type
    fn trigger_type(&self) -> TriggerType;
}

/// Types of triggers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TriggerType {
    /// Triggered by file system changes
    FileChange,
    /// Triggered by maturity level changes
    Maturity,
    /// Triggered manually
    Manual,
    /// Triggered on a schedule
    Scheduled,
    /// Composite trigger
    Composite,
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TriggerType::FileChange => write!(f, "file_change"),
            TriggerType::Maturity => write!(f, "maturity"),
            TriggerType::Manual => write!(f, "manual"),
            TriggerType::Scheduled => write!(f, "scheduled"),
            TriggerType::Composite => write!(f, "composite"),
        }
    }
}

/// Trigger for file system changes
#[derive(Debug, Clone)]
pub struct FileChangeTrigger {
    name: String,
    /// File patterns to match (glob patterns)
    patterns: Vec<String>,
    /// Event types to trigger on
    events: Vec<FileEvent>,
    /// Whether to match directories
    include_dirs: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileEvent {
    Create,
    Modify,
    Delete,
    Rename,
}

impl FileChangeTrigger {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            patterns: Vec::new(),
            events: vec![FileEvent::Create, FileEvent::Modify],
            include_dirs: false,
        }
    }

    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.patterns.push(pattern.into());
        self
    }

    pub fn with_patterns(mut self, patterns: Vec<String>) -> Self {
        self.patterns = patterns;
        self
    }

    pub fn on_event(mut self, event: FileEvent) -> Self {
        if !self.events.contains(&event) {
            self.events.push(event);
        }
        self
    }

    pub fn on_events(mut self, events: Vec<FileEvent>) -> Self {
        self.events = events;
        self
    }

    pub fn include_directories(mut self) -> Self {
        self.include_dirs = true;
        self
    }

    /// Check if a path matches any of the patterns
    pub fn matches(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check if it's a directory
        if !self.include_dirs {
            if path.is_dir() {
                return false;
            }
        }

        // Check patterns
        if self.patterns.is_empty() {
            return true;
        }

        for pattern in &self.patterns {
            if matches_pattern(&path_str, pattern) {
                return true;
            }
        }

        false
    }

    /// Check if the trigger should fire for an event
    pub fn should_fire(&self, _event: &FileEvent) -> bool {
        // For now, we just check if the event type is in our list
        // In a real implementation, we'd also match the path
        true
    }
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    // Simple glob matching - in production, use the `glob` crate
    if pattern == "*" || pattern == "**/*" {
        return true;
    }

    if pattern.starts_with("*.") {
        let ext = &pattern[1..]; // Remove the *
        return path.ends_with(ext);
    }

    if pattern.contains('*') || pattern.contains('?') {
        // Use glob crate for complex patterns
        match glob::Pattern::new(pattern) {
            Ok(p) => return p.matches(path),
            Err(_) => return false,
        }
    }

    // Exact match or contains
    path.contains(pattern)
}

impl ConstraintTrigger for FileChangeTrigger {
    fn name(&self) -> &str {
        &self.name
    }

    fn should_trigger(&self, _constraint_name: &str, path: &Path, _context: &ConstraintContext) -> bool {
        self.matches(path)
    }

    fn trigger_type(&self) -> TriggerType {
        TriggerType::FileChange
    }
}

/// Trigger based on project maturity level
#[derive(Debug, Clone)]
pub struct MaturityTrigger {
    name: String,
    /// Minimum maturity level to trigger
    min_level: MaturityLevel,
    /// Maximum maturity level to trigger (inclusive)
    max_level: Option<MaturityLevel>,
    /// Whether to trigger only when level changes
    on_change_only: bool,
    /// Previously seen maturity level
    last_level: Option<MaturityLevel>,
}

impl MaturityTrigger {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            min_level: MaturityLevel::Raw,
            max_level: None,
            on_change_only: false,
            last_level: None,
        }
    }

    pub fn at_least(mut self, level: MaturityLevel) -> Self {
        self.min_level = level;
        self
    }

    pub fn at_most(mut self, level: MaturityLevel) -> Self {
        self.max_level = Some(level);
        self
    }

    pub fn between(mut self, min: MaturityLevel, max: MaturityLevel) -> Self {
        self.min_level = min;
        self.max_level = Some(max);
        self
    }

    pub fn only_on_change(mut self) -> Self {
        self.on_change_only = true;
        self
    }

    /// Check if the trigger should fire for the given maturity level
    pub fn should_fire(&self, level: MaturityLevel) -> bool {
        // Check minimum level
        if level < self.min_level {
            return false;
        }

        // Check maximum level
        if let Some(max) = self.max_level {
            if level > max {
                return false;
            }
        }

        // Check if we only trigger on change
        if self.on_change_only {
            if let Some(last) = self.last_level {
                if last == level {
                    return false;
                }
            }
        }

        true
    }

    /// Update the last seen level
    pub fn update_level(&mut self, level: MaturityLevel) {
        self.last_level = Some(level);
    }
}

impl ConstraintTrigger for MaturityTrigger {
    fn name(&self) -> &str {
        &self.name
    }

    fn should_trigger(&self, _constraint_name: &str, _path: &Path, context: &ConstraintContext) -> bool {
        self.should_fire(context.maturity_level())
    }

    fn trigger_type(&self) -> TriggerType {
        TriggerType::Maturity
    }
}

/// Manual trigger that only fires on explicit invocation
#[derive(Debug, Clone)]
pub struct ManualTrigger {
    name: String,
    /// Constraints that this trigger applies to
    constraints: Vec<String>,
}

impl ManualTrigger {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            constraints: Vec::new(),
        }
    }

    pub fn for_constraint(mut self, constraint: impl Into<String>) -> Self {
        self.constraints.push(constraint.into());
        self
    }

    pub fn for_constraints(mut self, constraints: Vec<String>) -> Self {
        self.constraints = constraints;
        self
    }

    /// Check if this trigger applies to a constraint
    pub fn applies_to(&self, constraint_name: &str) -> bool {
        self.constraints.is_empty() || self.constraints.contains(&constraint_name.to_string())
    }
}

impl ConstraintTrigger for ManualTrigger {
    fn name(&self) -> &str {
        &self.name
    }

    fn should_trigger(&self, constraint_name: &str, _path: &Path, context: &ConstraintContext) -> bool {
        // Only trigger if explicitly marked as manual invocation
        if !context.is_manual {
            return false;
        }

        self.applies_to(constraint_name)
    }

    fn trigger_type(&self) -> TriggerType {
        TriggerType::Manual
    }
}

/// A trigger that combines multiple triggers with AND/OR logic
#[derive(Debug)]
pub struct CompositeTrigger {
    name: String,
    triggers: Vec<Box<dyn ConstraintTrigger>>,
    strategy: CompositeStrategy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositeStrategy {
    /// All triggers must fire
    All,
    /// At least one trigger must fire
    Any,
}

impl CompositeTrigger {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            triggers: Vec::new(),
            strategy: CompositeStrategy::All,
        }
    }

    pub fn with_strategy(mut self, strategy: CompositeStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    pub fn add_trigger(mut self, trigger: Box<dyn ConstraintTrigger>) -> Self {
        self.triggers.push(trigger);
        self
    }
}

impl ConstraintTrigger for CompositeTrigger {
    fn name(&self) -> &str {
        &self.name
    }

    fn should_trigger(&self, constraint_name: &str, path: &Path, context: &ConstraintContext) -> bool {
        match self.strategy {
            CompositeStrategy::All => {
                self.triggers.iter().all(|t| t.should_trigger(constraint_name, path, context))
            }
            CompositeStrategy::Any => {
                self.triggers.iter().any(|t| t.should_trigger(constraint_name, path, context))
            }
        }
    }

    fn trigger_type(&self) -> TriggerType {
        TriggerType::Composite
    }
}

/// Registry for managing triggers
#[derive(Debug, Default)]
pub struct TriggerRegistry {
    triggers: Vec<Box<dyn ConstraintTrigger>>,
    /// Mapping from constraint names to trigger names
    constraint_triggers: HashMap<String, Vec<String>>,
}

impl TriggerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a trigger
    pub fn register(&mut self, trigger: Box<dyn ConstraintTrigger>) {
        self.triggers.push(trigger);
    }

    /// Associate a constraint with a trigger
    pub fn associate(&mut self, constraint_name: impl Into<String>, trigger_name: impl Into<String>) {
        let constraint = constraint_name.into();
        let trigger = trigger_name.into();

        self.constraint_triggers
            .entry(constraint)
            .or_default()
            .push(trigger);
    }

    /// Check if any trigger should fire for a constraint
    pub fn should_trigger(
        &self,
        constraint_name: &str,
        path: &Path,
        context: &ConstraintContext,
    ) -> bool {
        // If no triggers are registered for this constraint, always trigger
        let trigger_names = match self.constraint_triggers.get(constraint_name) {
            Some(names) => names,
            None => return true,
        };

        // Check if any of the associated triggers fire
        for trigger in &self.triggers {
            if trigger_names.contains(&trigger.name().to_string()) {
                if trigger.should_trigger(constraint_name, path, context) {
                    return true;
                }
            }
        }

        false
    }

    /// Get all triggers of a specific type
    pub fn triggers_by_type(&self, trigger_type: TriggerType) -> Vec<&dyn ConstraintTrigger> {
        self.triggers
            .iter()
            .filter(|t| t.trigger_type() == trigger_type)
            .map(|t| t.as_ref())
            .collect()
    }

    /// Get trigger by name
    pub fn get_trigger(&self, name: &str) -> Option<&dyn ConstraintTrigger> {
        self.triggers
            .iter()
            .find(|t| t.name() == name)
            .map(|t| t.as_ref())
    }

    /// Remove a trigger by name
    pub fn remove_trigger(&mut self, name: &str) -> bool {
        let initial_len = self.triggers.len();
        self.triggers.retain(|t| t.name() != name);
        self.triggers.len() < initial_len
    }

    /// List all registered trigger names
    pub fn trigger_names(&self) -> Vec<&str> {
        self.triggers.iter().map(|t| t.name()).collect()
    }

    /// Clear all triggers
    pub fn clear(&mut self) {
        self.triggers.clear();
        self.constraint_triggers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_change_trigger_pattern_matching() {
        let trigger = FileChangeTrigger::new("test")
            .with_pattern("*.rs")
            .with_pattern("*.toml");

        assert!(trigger.matches(Path::new("/test/file.rs")));
        assert!(trigger.matches(Path::new("/test/Cargo.toml")));
        assert!(!trigger.matches(Path::new("/test/file.txt")));
    }

    #[test]
    fn test_maturity_trigger() {
        let trigger = MaturityTrigger::new("test")
            .at_least(MaturityLevel::Developing);

        assert!(trigger.should_fire(MaturityLevel::Developing));
        assert!(trigger.should_fire(MaturityLevel::Mature));
        assert!(!trigger.should_fire(MaturityLevel::Raw));

        let trigger2 = MaturityTrigger::new("test2")
            .between(MaturityLevel::Developing, MaturityLevel::Mature);

        assert!(trigger2.should_fire(MaturityLevel::Developing));
        assert!(trigger2.should_fire(MaturityLevel::Mature));
        assert!(!trigger2.should_fire(MaturityLevel::Raw));
        assert!(!trigger2.should_fire(MaturityLevel::Established));
    }

    #[test]
    fn test_manual_trigger() {
        let trigger = ManualTrigger::new("test").for_constraint("my_constraint");

        let manual_context = ConstraintContext::new().manual();
        let auto_context = ConstraintContext::new();

        assert!(trigger.should_trigger("my_constraint", Path::new("/test"), &manual_context));
        assert!(!trigger.should_trigger("my_constraint", Path::new("/test"), &auto_context));
        assert!(!trigger.should_trigger("other_constraint", Path::new("/test"), &manual_context));
    }

    #[test]
    fn test_composite_trigger_all() {
        let trigger = CompositeTrigger::new("test")
            .with_strategy(CompositeStrategy::All)
            .add_trigger(Box::new(AlwaysPassTrigger))
            .add_trigger(Box::new(AlwaysPassTrigger));

        let context = ConstraintContext::new();
        assert!(trigger.should_trigger("test", Path::new("/test"), &context));
    }

    #[test]
    fn test_composite_trigger_any() {
        let trigger = CompositeTrigger::new("test")
            .with_strategy(CompositeStrategy::Any)
            .add_trigger(Box::new(AlwaysFailTrigger))
            .add_trigger(Box::new(AlwaysPassTrigger));

        let context = ConstraintContext::new();
        assert!(trigger.should_trigger("test", Path::new("/test"), &context));
    }

    #[test]
    fn test_trigger_registry() {
        let mut registry = TriggerRegistry::new();

        let trigger = Box::new(FileChangeTrigger::new("file_trigger").with_pattern("*.rs"));
        registry.register(trigger);
        registry.associate("rust_constraint", "file_trigger");

        let context = ConstraintContext::new();
        assert!(registry.should_trigger("rust_constraint", Path::new("/test/main.rs"), &context));
        assert!(!registry.should_trigger("rust_constraint", Path::new("/test/main.txt"), &context));
    }

    // Helper triggers for testing
    #[derive(Debug)]
    struct AlwaysPassTrigger;

    impl ConstraintTrigger for AlwaysPassTrigger {
        fn name(&self) -> &str {
            "always_pass"
        }

        fn should_trigger(&self, _constraint_name: &str, _path: &Path, _context: &ConstraintContext) -> bool {
            true
        }

        fn trigger_type(&self) -> TriggerType {
            TriggerType::Manual
        }
    }

    #[derive(Debug)]
    struct AlwaysFailTrigger;

    impl ConstraintTrigger for AlwaysFailTrigger {
        fn name(&self) -> &str {
            "always_fail"
        }

        fn should_trigger(&self, _constraint_name: &str, _path: &Path, _context: &ConstraintContext) -> bool {
            false
        }

        fn trigger_type(&self) -> TriggerType {
            TriggerType::Manual
        }
    }
}
