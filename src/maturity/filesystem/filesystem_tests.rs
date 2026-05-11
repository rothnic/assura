//! Unit tests for the parent module.
use super::*;
use std::fs;

#[test]
fn test_filesystem_empty_dir() {
    let temp_dir = tempfile::tempdir().unwrap();
    let collector = FilesystemSignals::new();

    let signals = collector.collect(temp_dir.path()).unwrap();
    assert!(!signals.is_empty());

    let file_count = signals.iter().find(|s| s.name == "file_count").unwrap();
    assert_eq!(file_count.value, 0.0);
}

#[test]
fn test_filesystem_with_files() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("file1.txt"), "test").unwrap();
    fs::write(temp_dir.path().join("file2.txt"), "test").unwrap();

    let collector = FilesystemSignals::new();
    let signals = collector.collect(temp_dir.path()).unwrap();

    let file_count = signals.iter().find(|s| s.name == "file_count").unwrap();
    assert!(file_count.value > 0.0);
}

#[test]
fn test_config_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("Cargo.toml"), "[package]").unwrap();
    fs::write(temp_dir.path().join(".gitignore"), "target/").unwrap();

    let collector = FilesystemSignals::new();
    let signals = collector.collect(temp_dir.path()).unwrap();

    let config_signal = signals.iter().find(|s| s.name == "config_files").unwrap();
    assert!(config_signal.value > 0.0);
    assert!(config_signal.raw_value.contains("Cargo.toml"));
}

#[test]
fn test_documentation_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::write(temp_dir.path().join("README.md"), "# Project").unwrap();
    fs::write(temp_dir.path().join("LICENSE"), "MIT").unwrap();

    let collector = FilesystemSignals::new();
    let signals = collector.collect(temp_dir.path()).unwrap();

    let doc_signal = signals.iter().find(|s| s.name == "documentation").unwrap();
    assert!(doc_signal.value > 0.0);
    assert!(doc_signal.raw_value.contains("README"));
}

#[test]
fn test_test_coverage_detection() {
    let temp_dir = tempfile::tempdir().unwrap();
    fs::create_dir(temp_dir.path().join("tests")).unwrap();
    fs::write(temp_dir.path().join("tests/test.rs"), "# test").unwrap();

    let collector = FilesystemSignals::new();
    let signals = collector.collect(temp_dir.path()).unwrap();

    let test_signal = signals.iter().find(|s| s.name == "test_coverage").unwrap();
    assert!(test_signal.value > 0.0);
}
