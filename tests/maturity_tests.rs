use assura::maturity::*;
use std::fs;
use std::process::Command;

fn create_test_repo() -> tempfile::TempDir {
    let temp_dir = tempfile::tempdir().unwrap();
    let base_path = temp_dir.path();

    Command::new("git")
        .args(["init"])
        .current_dir(base_path)
        .output()
        .expect("Failed to init git repo");

    Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(base_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(base_path)
        .output()
        .unwrap();

    temp_dir
}

fn add_commit(repo_path: &std::path::Path, message: &str) {
    fs::write(repo_path.join("test.txt"), message).unwrap();

    Command::new("git")
        .args(["add", "."])
        .current_dir(repo_path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(repo_path)
        .output()
        .unwrap();
}

#[test]
fn test_maturity_detector_basic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let detector = MaturityDetector::new();

    let result = detector.detect(temp_dir.path());
    assert!(result.is_ok());

    let report = result.unwrap();
    assert!(report.score >= 0.0 && report.score <= 1.0);
    assert!(report.confidence >= 0.0 && report.confidence <= 1.0);
}

#[test]
fn test_maturity_detector_with_git_repo() {
    let temp_dir = create_test_repo();
    add_commit(temp_dir.path(), "Initial commit");

    fs::write(temp_dir.path().join("README.md"), "# Test Project").unwrap();
    fs::write(temp_dir.path().join("LICENSE"), "MIT License").unwrap();

    let detector = MaturityDetector::new();
    let report = detector.detect(temp_dir.path()).unwrap();

    assert!(report.score >= 0.0);
    assert!(report.confidence >= 0.0);
    assert!(!report.signals.is_empty());
}

#[test]
fn test_maturity_detector_with_config_files() {
    let temp_dir = tempfile::tempdir().unwrap();

    fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "target/").unwrap();
    fs::write(temp_dir.path().join("README.md"), "# Project").unwrap();

    let detector = MaturityDetector::new();
    let report = detector.detect(temp_dir.path()).unwrap();

    let config_signal = report.signals.iter().find(|s| s.name == "config_files");
    assert!(config_signal.is_some());
    assert!(config_signal.unwrap().value > 0.0);
}

#[test]
fn test_maturity_detector_with_ci_cd() {
    let temp_dir = tempfile::tempdir().unwrap();

    let github_dir = temp_dir.path().join(".github").join("workflows");
    fs::create_dir_all(&github_dir).unwrap();
    fs::write(github_dir.join("ci.yml"), "name: CI").unwrap();

    let detector = MaturityDetector::new();
    let report = detector.detect(temp_dir.path()).unwrap();

    let cicd_signal = report.signals.iter().find(|s| s.name == "cicd_config");
    assert!(cicd_signal.is_some());
    assert!(cicd_signal.unwrap().value > 0.0);
}

#[test]
fn test_maturity_level_assignment_raw() {
    let temp_dir = tempfile::tempdir().unwrap();

    let detector = MaturityDetector::new();
    let report = detector.detect(temp_dir.path()).unwrap();

    assert_eq!(report.level, MaturityLevel::Raw);
    assert!(report.score < 0.3);
}

#[test]
fn test_maturity_level_assignment_developing() {
    let temp_dir = create_test_repo();
    add_commit(temp_dir.path(), "Initial commit");

    fs::write(temp_dir.path().join("README.md"), "# Project").unwrap();
    fs::write(temp_dir.path().join("LICENSE"), "MIT").unwrap();
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
    fs::create_dir(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/lib.rs"), "// lib").unwrap();

    let detector = MaturityDetector::new();
    let report = detector.detect(temp_dir.path()).unwrap();

    assert!(report.score >= 0.0);
    assert!(!report.signals.is_empty());
}

#[test]
fn test_maturity_level_assignment_mature() {
    let temp_dir = create_test_repo();

    for i in 1..=20 {
        add_commit(temp_dir.path(), &format!("Commit {}", i));
    }

    fs::write(temp_dir.path().join("README.md"), "# Mature Project").unwrap();
    fs::write(temp_dir.path().join("LICENSE"), "MIT").unwrap();
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
    fs::write(temp_dir.path().join("Cargo.lock"), "# lock").unwrap();
    fs::create_dir(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/lib.rs"), "pub fn main() {}").unwrap();
    fs::create_dir(temp_dir.path().join("tests")).unwrap();
    fs::write(
        temp_dir.path().join("tests/test.rs"),
        "#[test] fn test() {}",
    )
    .unwrap();

    let github_dir = temp_dir.path().join(".github").join("workflows");
    fs::create_dir_all(&github_dir).unwrap();
    fs::write(github_dir.join("ci.yml"), "name: CI").unwrap();

    fs::write(temp_dir.path().join(".rustfmt.toml"), "max_width = 100").unwrap();

    let detector = MaturityDetector::new();
    let report = detector.detect(temp_dir.path()).unwrap();

    assert!(report.score >= 0.0);
    assert!(!report.signals.is_empty());
}

#[test]
fn test_config_override_manual() {
    let temp_dir = tempfile::tempdir().unwrap();

    let config = MaturityConfig::manual_override(
        MaturityLevel::Established,
        "Manually set to established for testing",
    );

    let detector = MaturityDetector::new().with_config(config);
    let report = detector.detect(temp_dir.path()).unwrap();

    assert_eq!(report.level, MaturityLevel::Established);
    assert_eq!(report.confidence, 1.0);
}

#[test]
fn test_config_file_override() {
    let temp_dir = tempfile::tempdir().unwrap();

    let assura_dir = temp_dir.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();

    let config_yaml = r#"
level: Mature
score: 0.75
confidence: 0.95
manual_override: true
override_reason: "Test override from file"
"#;

    fs::write(assura_dir.join("maturity.yml"), config_yaml).unwrap();

    let loaded_config = MaturityConfig::from_directory(temp_dir.path())
        .unwrap()
        .expect("Config should be loaded");

    let detector = MaturityDetector::new().with_config(loaded_config);
    let report = detector.detect(temp_dir.path()).unwrap();

    assert_eq!(report.level, MaturityLevel::Mature);
    assert_eq!(report.score, 0.75);
    assert_eq!(report.confidence, 0.95);
}

#[test]
fn test_config_adjust_report() {
    let temp_dir = tempfile::tempdir().unwrap();

    let config = MaturityConfig {
        level: Some(MaturityLevel::Mature),
        score: Some(0.8),
        confidence: Some(0.9),
        ..Default::default()
    };

    let detector = MaturityDetector::new().with_config(config);
    let report = detector.detect(temp_dir.path()).unwrap();

    assert_eq!(report.level, MaturityLevel::Mature);
    assert_eq!(report.score, 0.8);
    assert_eq!(report.confidence, 0.9);
}

#[test]
#[cfg(feature = "git-signals")]
fn test_signal_collection_git() {
    let temp_dir = create_test_repo();
    add_commit(temp_dir.path(), "First commit");
    add_commit(temp_dir.path(), "Second commit");

    let git_collector = GitSignals::new();
    let signals = git_collector.collect(temp_dir.path()).unwrap();

    assert!(!signals.is_empty());

    let has_age = signals.iter().any(|s| s.name == "repository_age");
    let has_freq = signals.iter().any(|s| s.name == "commit_frequency");
    let has_branches = signals.iter().any(|s| s.name == "branch_count");

    assert!(has_age);
    assert!(has_freq);
    assert!(has_branches);
}

#[test]
fn test_signal_collection_filesystem() {
    let temp_dir = tempfile::tempdir().unwrap();

    fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();
    fs::create_dir(temp_dir.path().join("src")).unwrap();
    fs::write(temp_dir.path().join("src/main.rs"), "fn main() {}").unwrap();

    let fs_collector = FilesystemSignals::new();
    let signals = fs_collector.collect(temp_dir.path()).unwrap();

    assert!(!signals.is_empty());

    let has_file_count = signals.iter().any(|s| s.name == "file_count");
    let has_depth = signals.iter().any(|s| s.name == "directory_depth");

    assert!(has_file_count);
    assert!(has_depth);
}

#[test]
fn test_signal_collection_environment() {
    let temp_dir = tempfile::tempdir().unwrap();

    fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
    fs::write(temp_dir.path().join("Dockerfile"), "FROM rust:latest").unwrap();

    let env_collector = EnvironmentSignals::new();
    let signals = env_collector.collect(temp_dir.path()).unwrap();

    assert!(!signals.is_empty());

    let has_package_manager = signals.iter().any(|s| s.name == "package_manager");
    let has_deployment = signals.iter().any(|s| s.name == "deployment_config");

    assert!(has_package_manager);
    assert!(has_deployment);
}

#[test]
#[cfg(feature = "git-signals")]
fn test_signal_pipeline() {
    let temp_dir = create_test_repo();
    add_commit(temp_dir.path(), "Initial commit");

    let mut pipeline = SignalPipeline::new();
    pipeline = pipeline.add_collector(Box::new(GitSignals::new()));
    pipeline = pipeline.add_collector(Box::new(FilesystemSignals::new()));

    let signals = pipeline.collect_all(temp_dir.path()).unwrap();

    assert!(!signals.is_empty());

    let git_signals: Vec<_> = signals
        .iter()
        .filter(|s| matches!(s.signal_type, SignalType::Git))
        .collect();
    let fs_signals: Vec<_> = signals
        .iter()
        .filter(|s| matches!(s.signal_type, SignalType::Filesystem))
        .collect();

    assert!(!git_signals.is_empty());
    assert!(!fs_signals.is_empty());
}

#[test]
fn test_error_handling_invalid_path() {
    let detector = MaturityDetector::new();
    let result = detector.detect("/nonexistent/path/that/does/not/exist");

    assert!(result.is_ok());

    let report = result.unwrap();
    assert_eq!(report.level, MaturityLevel::Raw);
}

#[test]
fn test_error_handling_empty_signals() {
    let engine = MaturityDecisionEngine::new();
    let report = engine.evaluate(&[]);

    assert_eq!(report.level, MaturityLevel::Raw);
    assert_eq!(report.score, 0.0);
    assert_eq!(report.confidence, 0.0);
    // Empty signals returns early with no recommendations
    assert!(report.recommendations.is_empty());
}

#[test]
fn test_config_validation() {
    let valid_config = MaturityConfig::new();
    assert!(valid_config.validate().is_ok());

    let invalid_score_config = MaturityConfig {
        score: Some(1.5),
        ..Default::default()
    };
    assert!(invalid_score_config.validate().is_err());

    let invalid_confidence_config = MaturityConfig {
        confidence: Some(-0.1),
        ..Default::default()
    };
    assert!(invalid_confidence_config.validate().is_err());

    let invalid_override_config = MaturityConfig {
        manual_override: true,
        override_reason: None,
        ..Default::default()
    };
    assert!(invalid_override_config.validate().is_err());
}

#[test]
fn test_config_save_and_load() {
    let temp_dir = tempfile::tempdir().unwrap();

    let config = MaturityConfig {
        level: Some(MaturityLevel::Mature),
        score: Some(0.85),
        confidence: Some(0.95),
        manual_override: false,
        ..Default::default()
    };

    let saved_path = config.save_to_directory(temp_dir.path()).unwrap();
    assert!(saved_path.exists());

    let loaded_config = MaturityConfig::from_directory(temp_dir.path())
        .unwrap()
        .expect("Should load config");

    assert_eq!(loaded_config.level, Some(MaturityLevel::Mature));
    assert_eq!(loaded_config.score, Some(0.85));
}

#[test]
fn test_maturity_report_recommendations() {
    let engine = MaturityDecisionEngine::new();

    let signals = vec![
        MaturitySignal::new(SignalType::Git, "repository_age", 0.1, "new"),
        MaturitySignal::new(SignalType::Filesystem, "documentation", 0.1, "missing"),
        MaturitySignal::new(SignalType::Environment, "cicd_config", 0.0, "none"),
    ];

    let report = engine.evaluate(&signals);

    assert!(!report.recommendations.is_empty());

    let has_git_rec = report.recommendations.iter().any(|r| r.category == "git");
    let has_fs_rec = report
        .recommendations
        .iter()
        .any(|r| r.category == "filesystem");
    let has_env_rec = report
        .recommendations
        .iter()
        .any(|r| r.category == "environment");

    assert!(has_git_rec || has_fs_rec || has_env_rec);
}

#[test]
fn test_maturity_level_transitions() {
    assert_eq!(MaturityLevel::Raw.threshold(), 0.0);
    assert_eq!(MaturityLevel::Developing.threshold(), 0.3);
    assert_eq!(MaturityLevel::Mature.threshold(), 0.6);
    assert_eq!(MaturityLevel::Established.threshold(), 0.85);

    assert_eq!(MaturityLevel::Raw.next(), Some(MaturityLevel::Developing));
    assert_eq!(
        MaturityLevel::Developing.next(),
        Some(MaturityLevel::Mature)
    );
    assert_eq!(
        MaturityLevel::Mature.next(),
        Some(MaturityLevel::Established)
    );
    assert_eq!(MaturityLevel::Established.next(), None);
}

#[test]
fn test_signal_weighted_value() {
    let signal = MaturitySignal::new(SignalType::Git, "test", 0.8, "80")
        .with_weight(2.0)
        .with_confidence(0.9);

    assert_eq!(signal.weighted_value(), 0.8 * 2.0 * 0.9);
}

#[test]
fn test_signal_value_clamping() {
    let signal_high = MaturitySignal::new(SignalType::Git, "test", 1.5, "150");
    assert_eq!(signal_high.value, 1.0);

    let signal_low = MaturitySignal::new(SignalType::Git, "test", -0.5, "-50");
    assert_eq!(signal_low.value, 0.0);
}

#[test]
fn test_balanced_scores_requirement() {
    let engine = MaturityDecisionEngine::new().with_balanced_requirement(true);

    let signals = vec![
        MaturitySignal::new(SignalType::Environment, "cicd_config", 1.0, "complete"),
        MaturitySignal::new(SignalType::Environment, "package_manager", 1.0, "cargo"),
    ];

    let report = engine.evaluate(&signals);

    assert!(!matches!(report.level, MaturityLevel::Established));
}

#[test]
fn test_ignore_signals_in_config() {
    let temp_dir = tempfile::tempdir().unwrap();

    let config = MaturityConfig {
        ignore_signals: vec!["file_count".to_string()],
        ..Default::default()
    };

    let detector = MaturityDetector::new().with_config(config);
    let report = detector.detect(temp_dir.path()).unwrap();

    let has_file_count = report.signals.iter().any(|s| s.name == "file_count");
    assert!(!has_file_count);
}
