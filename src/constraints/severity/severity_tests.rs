//! Unit tests for the parent module.
use super::*;

#[test]
fn test_severity_ordering() {
    assert!(Severity::Critical > Severity::High);
    assert!(Severity::High > Severity::Medium);
    assert!(Severity::Medium > Severity::Low);
}

#[test]
fn test_severity_escalation() {
    assert_eq!(Severity::Low.escalate(), Severity::Medium);
    assert_eq!(Severity::Medium.escalate(), Severity::High);
    assert_eq!(Severity::High.escalate(), Severity::Critical);
    assert_eq!(Severity::Critical.escalate(), Severity::Critical);
}

#[test]
fn test_severity_de_escalation() {
    assert_eq!(Severity::Critical.de_escalate(), Severity::High);
    assert_eq!(Severity::High.de_escalate(), Severity::Medium);
    assert_eq!(Severity::Medium.de_escalate(), Severity::Low);
    assert_eq!(Severity::Low.de_escalate(), Severity::Low);
}

#[test]
fn test_severity_adjustment() {
    let adj = SeverityAdjustment::Escalate(2);
    assert_eq!(adj.apply(Severity::Low), Severity::High);

    let adj = SeverityAdjustment::DeEscalate(1);
    assert_eq!(adj.apply(Severity::High), Severity::Medium);

    let adj = SeverityAdjustment::Force(Severity::Critical);
    assert_eq!(adj.apply(Severity::Low), Severity::Critical);

    let adj = SeverityAdjustment::CapAt(Severity::Medium);
    assert_eq!(adj.apply(Severity::Critical), Severity::Medium);
    assert_eq!(adj.apply(Severity::Low), Severity::Low);
}

#[test]
fn test_maturity_severity_mapping() {
    let mapping = MaturitySeverityMapping::default();

    // Raw should escalate
    assert_eq!(
        mapping.adjust_severity(Severity::Low, MaturityLevel::Raw),
        Severity::Medium
    );

    // Mature should be as-is
    assert_eq!(
        mapping.adjust_severity(Severity::High, MaturityLevel::Mature),
        Severity::High
    );
}

#[test]
fn test_severity_config() {
    let config = SeverityConfig::new()
        .with_base_severity("file_size", Severity::High)
        .with_min_severity(Severity::Medium)
        .fail_on(Severity::Critical);

    assert_eq!(
        config.get_effective_severity("file_size", MaturityLevel::Mature, Severity::Medium),
        Severity::High
    );

    assert!(config.should_report(Severity::Medium));
    assert!(!config.should_report(Severity::Low));

    assert!(config.should_fail(Severity::Critical));
    assert!(!config.should_fail(Severity::High));
}

#[test]
fn test_severity_override() {
    let mut mapping = SeverityMapping::new();

    let override_spec =
        SeverityOverride::new("test_constraint", Severity::Low).with_reason("Temporary exception");

    mapping.add_override(override_spec);

    let effective =
        mapping.get_severity("test_constraint", Severity::Critical, MaturityLevel::Mature);
    assert_eq!(effective, Severity::Low);

    // Non-overridden constraint
    let effective = mapping.get_severity("other_constraint", Severity::High, MaturityLevel::Mature);
    assert_eq!(effective, Severity::High);
}

#[test]
fn test_severity_override_expiration() {
    let override_spec = SeverityOverride::new("test", Severity::Low).expires_at(0); // Already expired

    assert!(!override_spec.is_valid());
}

#[test]
fn test_severity_from_priority() {
    assert_eq!(
        Severity::from_priority(Priority::Critical),
        Severity::Critical
    );
    assert_eq!(Severity::from_priority(Priority::High), Severity::High);
    assert_eq!(Severity::from_priority(Priority::Medium), Severity::Medium);
    assert_eq!(Severity::from_priority(Priority::Low), Severity::Low);
}
