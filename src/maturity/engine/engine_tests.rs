//! Unit tests for the parent module.
use super::super::signal::{MaturitySignal, SignalType};
use super::*;

#[test]
fn test_maturity_level_thresholds() {
    assert_eq!(MaturityLevel::Raw.threshold(), 0.0);
    assert_eq!(MaturityLevel::Developing.threshold(), 0.3);
    assert_eq!(MaturityLevel::Mature.threshold(), 0.6);
    assert_eq!(MaturityLevel::Established.threshold(), 0.85);
}

#[test]
fn test_level_display() {
    assert_eq!(MaturityLevel::Raw.to_string(), "raw");
    assert_eq!(MaturityLevel::Mature.to_string(), "mature");
}

#[test]
fn test_empty_signals() {
    let engine = MaturityDecisionEngine::new();
    let report = engine.evaluate(&[]);

    assert_eq!(report.level, MaturityLevel::Raw);
    assert_eq!(report.score, 0.0);
    assert_eq!(report.confidence, 0.0);
}

#[test]
fn test_high_score_signals() {
    let engine = MaturityDecisionEngine::new();

    let signals = vec![
        MaturitySignal::new(SignalType::Git, "repository_age", 1.0, "5 years"),
        MaturitySignal::new(SignalType::Git, "commit_frequency", 0.9, "daily"),
        MaturitySignal::new(SignalType::Filesystem, "config_files", 1.0, "complete"),
        MaturitySignal::new(SignalType::Filesystem, "test_coverage", 0.9, "90%"),
        MaturitySignal::new(
            SignalType::Environment,
            "cicd_config",
            1.0,
            "github actions",
        ),
        MaturitySignal::new(SignalType::Environment, "package_manager", 1.0, "cargo"),
    ];

    let report = engine.evaluate(&signals);
    assert!(report.score > 0.7);
    assert!(matches!(
        report.level,
        MaturityLevel::Mature | MaturityLevel::Established
    ));
}

#[test]
fn test_balanced_scores() {
    let engine = MaturityDecisionEngine::new().with_balanced_requirement(true);

    // Unbalanced: high environment, low others
    let signals = vec![
        MaturitySignal::new(
            SignalType::Environment,
            "cicd_config",
            1.0,
            "github actions",
        ),
        MaturitySignal::new(SignalType::Environment, "package_manager", 1.0, "cargo"),
    ];

    let report = engine.evaluate(&signals);
    // Should cap level due to imbalance
    assert!(!matches!(report.level, MaturityLevel::Established));
}

#[test]
fn test_category_scores_calculation() {
    let engine = MaturityDecisionEngine::new();

    let signals = vec![
        MaturitySignal::new(SignalType::Git, "test1", 0.5, "value"),
        MaturitySignal::new(SignalType::Git, "test2", 1.0, "value"),
        MaturitySignal::new(SignalType::Filesystem, "test3", 0.75, "value"),
    ];

    let scores = engine.calculate_category_scores(&signals);
    assert_eq!(scores.git, 0.75); // Average of 0.5 and 1.0
    assert_eq!(scores.filesystem, 0.75);
    assert_eq!(scores.environment, 0.0);
}

#[test]
fn test_recommendations() {
    let engine = MaturityDecisionEngine::new();

    let signals = vec![
        MaturitySignal::new(SignalType::Git, "repository_age", 0.2, "new"),
        MaturitySignal::new(SignalType::Filesystem, "documentation", 0.1, "missing"),
    ];

    let report = engine.evaluate(&signals);
    assert!(!report.recommendations.is_empty());
}
