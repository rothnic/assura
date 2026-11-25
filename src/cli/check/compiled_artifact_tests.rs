//! Compiled artifact freshness tests.

use super::{
    compiled_artifact::path_to_portable, run_structure_check_with_artifact,
    run_structure_check_with_fast_artifact, CompiledStructureConfigArtifact,
};
use crate::config::config::{Config, DirectoryBundle, DirectoryNode, FileBundle};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

fn test_config() -> Config {
    Config::new().with_node(
        "src/",
        DirectoryNode::new().with_files(FileBundle::new().with_naming("snake_case")),
    )
}

#[test]
fn compiled_directory_counts_survive_fast_only_and_fail_fast_checks() {
    let project = tempfile::tempdir().unwrap();
    let root = project.path().to_path_buf();
    let config = Config::new().with_node(
        "required/",
        DirectoryNode {
            required: false,
            self_directory: Some(DirectoryBundle {
                exists: Some(HashMap::from([("*".to_owned(), "1".to_owned())])),
                ..DirectoryBundle::default()
            }),
            ..DirectoryNode::default()
        },
    );
    let config_path = root.join("check-config.bin");
    let fast = run_structure_check_with_fast_artifact(
        root.clone(),
        config_path.clone(),
        root.clone(),
        CompiledStructureConfigArtifact::new(config.clone()),
    )
    .unwrap();
    let fail_fast = run_structure_check_with_artifact(
        root.clone(),
        config_path,
        root,
        CompiledStructureConfigArtifact::new(config),
        true,
    )
    .unwrap();

    for report in [fast, fail_fast] {
        assert!(!report.success, "{report:#?}");
        assert_eq!(report.violations.len(), 1, "{report:#?}");
        assert_eq!(report.violations[0].rule, "exists_count");
        assert!(report.violations[0]
            .message
            .contains("exists 0 times, expected 1"));
    }
}

#[test]
fn source_fingerprint_matches_unchanged_config() {
    let temp = tempfile::tempdir().unwrap();
    let assura_dir = temp.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    let config_path = assura_dir.join("config.yml");
    let source = b"structure:\n  src/:\n    files:\n      naming: snake_case\n";
    fs::write(&config_path, source).unwrap();

    let artifact =
        CompiledStructureConfigArtifact::new_with_source(test_config(), &config_path, source)
            .unwrap();

    assert!(artifact.matches_source_config(&config_path).unwrap());
}

#[cfg(unix)]
#[test]
fn source_fingerprint_detects_same_size_rewrite_on_unix() {
    let temp = tempfile::tempdir().unwrap();
    let assura_dir = temp.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    let config_path = assura_dir.join("config.yml");
    let source = b"structure:\n  src/:\n    files:\n      naming: snake_case\n";
    fs::write(&config_path, source).unwrap();
    let artifact =
        CompiledStructureConfigArtifact::new_with_source(test_config(), &config_path, source)
            .unwrap();

    let changed = b"structure:\n  src/:\n    files:\n      naming: kebab_case\n";
    assert_eq!(source.len(), changed.len());
    fs::write(&config_path, changed).unwrap();

    assert!(!artifact.matches_source_config(&config_path).unwrap());
}

#[test]
fn source_fingerprint_falls_back_to_hash_and_rejects_changed_config() {
    let temp = tempfile::tempdir().unwrap();
    let assura_dir = temp.path().join(".assura");
    fs::create_dir_all(&assura_dir).unwrap();
    let config_path = assura_dir.join("config.yml");
    let source = b"structure:\n  src/:\n    files:\n      naming: snake_case\n";
    fs::write(&config_path, source).unwrap();
    let artifact =
        CompiledStructureConfigArtifact::new_with_source(test_config(), &config_path, source)
            .unwrap();

    fs::write(
        &config_path,
        b"structure:\n  src/:\n    files:\n      naming: kebab-case\nexclude:\n  - target/**\n",
    )
    .unwrap();

    assert!(!artifact.matches_source_config(&config_path).unwrap());
}

#[test]
fn path_to_portable_preserves_unix_absolute_paths() {
    assert_eq!(
        path_to_portable(PathBuf::from("/tmp/assura/config.yaml")),
        "/tmp/assura/config.yaml"
    );
}

#[test]
fn path_to_portable_normalizes_windows_separators() {
    assert_eq!(
        path_to_portable(PathBuf::from(r"C:\Users\nick\assura\config.yaml")),
        "C:/Users/nick/assura/config.yaml"
    );
}
