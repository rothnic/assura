//! Focused correctness tests for cache freshness fingerprints.

use super::*;

#[test]
fn config_hash_is_stable_across_git_line_ending_conversion() {
    assert_eq!(
        stable_config_hash("structure: {}\n"),
        stable_config_hash("structure: {}\r\n")
    );
}

#[test]
fn cached_report_requires_hash_even_when_config_fingerprint_matches() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join(".assura")).unwrap();
    fs::write(temp.path().join(".assura/config.yml"), "structure: {}\n").unwrap();
    fs::write(temp.path().join("valid-file.ts"), "").unwrap();
    let config_path = temp.path().join(".assura/config.yml");
    let checked_path = temp.path();
    let cached = CachedCheckReport {
        schema_version: CACHE_SCHEMA_VERSION,
        assura_version: env!("CARGO_PKG_VERSION").to_string(),
        config_hash: 123,
        config_fingerprint: SourceConfigFingerprint::from_path(&config_path).ok(),
        project_root: temp.path().to_path_buf(),
        config_path: config_path.clone(),
        checked_path: checked_path.to_path_buf(),
        exclude_patterns: Vec::new(),
        dir_snapshot: collect_directory_snapshot(checked_path, checked_path, &[]).unwrap(),
        file_snapshot: None,
        report: StructureCheckReport {
            success: true,
            project_root: temp.path().to_path_buf(),
            config_path: config_path.clone(),
            checked_path: checked_path.to_path_buf(),
            files_checked: 1,
            dirs_checked: 1,
            violations: Vec::new(),
        },
    };
    assert!(
        fresh_cached_report(Some(&cached), None, temp.path(), &config_path, checked_path).is_none()
    );
}

#[test]
fn cached_report_rejects_stale_config_without_hash() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join(".assura")).unwrap();
    fs::write(temp.path().join(".assura/config.yml"), "structure: {}\n").unwrap();
    let config_path = temp.path().join(".assura/config.yml");
    let cached = CachedCheckReport {
        schema_version: CACHE_SCHEMA_VERSION,
        assura_version: env!("CARGO_PKG_VERSION").to_string(),
        config_hash: stable_hash(b"structure: {}\n"),
        config_fingerprint: SourceConfigFingerprint::from_path(&config_path).ok(),
        project_root: temp.path().to_path_buf(),
        config_path: config_path.clone(),
        checked_path: temp.path().to_path_buf(),
        exclude_patterns: Vec::new(),
        dir_snapshot: collect_directory_snapshot(temp.path(), temp.path(), &[]).unwrap(),
        file_snapshot: None,
        report: StructureCheckReport {
            success: true,
            project_root: temp.path().to_path_buf(),
            config_path: config_path.clone(),
            checked_path: temp.path().to_path_buf(),
            files_checked: 0,
            dirs_checked: 1,
            violations: Vec::new(),
        },
    };
    fs::write(&config_path, "structure:\n  src/: {}\n").unwrap();
    assert!(
        fresh_cached_report(Some(&cached), None, temp.path(), &config_path, temp.path()).is_none()
    );
}

#[test]
fn child_fingerprint_changes_when_child_name_changes() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("valid-file.ts"), "").unwrap();
    let (_, before_hash, _) =
        snapshot::collect_child_fingerprint(temp.path(), Path::new(""), &[]).unwrap();
    fs::rename(
        temp.path().join("valid-file.ts"),
        temp.path().join("bad_name.ts"),
    )
    .unwrap();
    let (_, after_hash, _) =
        snapshot::collect_child_fingerprint(temp.path(), Path::new(""), &[]).unwrap();
    assert_ne!(before_hash, after_hash);
}

#[test]
fn child_fingerprint_changes_when_child_type_changes() {
    let temp = tempfile::tempdir().unwrap();
    let child = temp.path().join("child");
    fs::write(&child, "").unwrap();
    let (_, before_hash, _) =
        snapshot::collect_child_fingerprint(temp.path(), Path::new(""), &[]).unwrap();
    fs::remove_file(&child).unwrap();
    fs::create_dir(&child).unwrap();
    let (_, after_hash, child_dirs) =
        snapshot::collect_child_fingerprint(temp.path(), Path::new(""), &[]).unwrap();
    assert_ne!(before_hash, after_hash);
    assert_eq!(child_dirs, vec![(PathBuf::from("child"), child)]);
}

#[test]
fn directory_snapshot_prunes_excluded_children() {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("dist")).unwrap();
    fs::create_dir(temp.path().join("src")).unwrap();
    fs::write(temp.path().join("dist/one.ts"), "").unwrap();
    fs::write(temp.path().join("src/one.ts"), "").unwrap();
    let exclude = vec![CompiledExclusion::new("dist/**")];
    let before = collect_directory_snapshot(temp.path(), temp.path(), &exclude).unwrap();
    fs::write(temp.path().join("dist/two.ts"), "").unwrap();
    let after = collect_directory_snapshot(temp.path(), temp.path(), &exclude).unwrap();
    assert_eq!(before, after);
    assert!(before.iter().all(|fingerprint| fingerprint.rel != "dist"));
}

#[test]
fn file_fingerprint_changes_when_file_bytes_change() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("valid-file.ts");
    fs::write(&path, "first\n").unwrap();
    let before = collect_file_snapshot(&path).unwrap();
    fs::write(&path, "second\n").unwrap();
    let after = collect_file_snapshot(&path).unwrap();
    assert_ne!(before, after);
}

#[test]
fn corrupt_cache_record_falls_back_to_fresh_validation() {
    let project = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    fs::create_dir(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        "structure:\n  ./:\n    files:\n      naming: kebab-case\n",
    )
    .unwrap();
    fs::write(project.path().join("good-name.rs"), "").unwrap();

    let first = run_structure_check_cached(
        Some(project.path().to_path_buf()),
        None,
        false,
        cache.path().to_path_buf(),
    )
    .unwrap();
    assert!(
        first.success,
        "initial check should populate a passing cache"
    );

    let record = walkdir::WalkDir::new(cache.path().join("worktrees"))
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| {
            entry.file_type().is_file() && entry.path().extension().is_some_and(|ext| ext == "json")
        })
        .expect("worktree cache record")
        .into_path();
    fs::write(&record, "not valid cache json").unwrap();

    let recovered = run_structure_check_cached(
        Some(project.path().to_path_buf()),
        None,
        false,
        cache.path().to_path_buf(),
    )
    .unwrap();
    assert!(
        recovered.success,
        "corrupt cache must fall back to a fresh passing validation"
    );
    let rewritten = fs::read(&record).expect("rewritten cache record");
    assert!(
        serde_json::from_slice::<CachedCheckReport>(&rewritten).is_ok(),
        "fresh validation must replace the corrupt cache record"
    );

    fs::write(project.path().join("BadName.rs"), "").unwrap();

    let refreshed = run_structure_check_cached(
        Some(project.path().to_path_buf()),
        None,
        false,
        cache.path().to_path_buf(),
    )
    .unwrap();
    assert!(
        !refreshed.success,
        "corrupt cache must not suppress a fresh naming violation"
    );
    assert!(
        refreshed
            .violations
            .iter()
            .any(|violation| violation.rule == "file_naming"),
        "fresh report must expose the seeded naming violation"
    );
}
