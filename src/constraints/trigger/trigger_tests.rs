//! Unit tests for the parent module.
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
    let trigger = MaturityTrigger::new("test").at_least(MaturityLevel::Developing);

    assert!(trigger.should_fire(MaturityLevel::Developing));
    assert!(trigger.should_fire(MaturityLevel::Mature));
    assert!(!trigger.should_fire(MaturityLevel::Raw));

    let trigger2 =
        MaturityTrigger::new("test2").between(MaturityLevel::Developing, MaturityLevel::Mature);

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

    fn should_trigger(
        &self,
        _constraint_name: &str,
        _path: &Path,
        _context: &ConstraintContext,
    ) -> bool {
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

    fn should_trigger(
        &self,
        _constraint_name: &str,
        _path: &Path,
        _context: &ConstraintContext,
    ) -> bool {
        false
    }

    fn trigger_type(&self) -> TriggerType {
        TriggerType::Manual
    }
}
