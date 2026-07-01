//! Compiled artifact freshness tests.

use super::{compiled_artifact::path_to_portable, CompiledStructureConfigArtifact};
use crate::config::config::{Config, DirectoryNode, FileBundle};
use std::fs;
use std::path::PathBuf;

fn test_config() -> Config {
    Config::new().with_node(
        "src/",
        DirectoryNode::new().with_files(FileBundle::new().with_naming("snake_case")),
    )
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
