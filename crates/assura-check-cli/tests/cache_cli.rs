use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::Duration;

fn write_project(root: &Path, file_name: &str) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join(".assura/config.yml"),
        r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": kebab-case
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(root.join("src").join(file_name), "").unwrap();
}

fn first_json_file(root: &Path) -> Option<std::path::PathBuf> {
    for entry in fs::read_dir(root).ok()? {
        let path = entry.ok()?.path();
        if path.is_dir() {
            if let Some(found) = first_json_file(&path) {
                return Some(found);
            }
        } else if path.extension().is_some_and(|value| value == "json")
            && path
                .file_name()
                .is_some_and(|name| name != ".assura-cache-root.json")
        {
            return Some(path);
        }
    }
    None
}

#[test]
fn cache_dir_reuses_report_but_invalidates_on_directory_change() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("cached-project");
    let cache = temp.path().join("cache");
    write_project(&project, "valid-file.ts");
    let first = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(first.status.success());
    std::thread::sleep(Duration::from_millis(20));
    fs::write(project.join("bad_name.ts"), "").unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert_eq!(second.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&second.stdout).contains("bad_name.ts"));
}

#[test]
fn cache_dir_persists_and_invalidates_direct_file_results() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("file-cache-project");
    let cache = temp.path().join("cache");
    write_project(&project, "valid-file.ts");
    let file = project.join("src/valid-file.ts");
    let first = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&file)
        .output()
        .unwrap();
    assert!(first.status.success());
    let cache_entry = first_json_file(&cache).expect("file cache record");
    let before = fs::read(&cache_entry).unwrap();
    fs::write(&file, "export const changed = true;\n").unwrap();
    let second = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&file)
        .output()
        .unwrap();
    assert!(second.status.success());
    assert_ne!(before, fs::read(cache_entry).unwrap());
}

#[test]
fn cache_dir_does_not_claim_a_preexisting_nonempty_directory() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let cache = temp.path().join("unrelated");
    write_project(&project, "valid-file.ts");
    fs::create_dir_all(&cache).unwrap();
    fs::write(cache.join("keep.txt"), "keep\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(cache.join("keep.txt").is_file());
    assert!(!cache.join(".assura-cache-root.json").exists());
    assert!(first_json_file(&cache).is_none());
}

#[test]
fn default_cache_preserves_scoped_relationship_discovery_after_config_changes() {
    for poisoned in [false, true] {
        let temp = tempfile::tempdir().unwrap();
        let project = temp.path().join("project");
        fs::create_dir_all(project.join(".assura")).unwrap();
        fs::create_dir_all(project.join("src/components")).unwrap();
        let config_path = project.join(".assura/config.yml");
        let policy = "structure:\n  ./:\n    extra: true\n  src/components/:\n    \"{component}.tsx\": {}\n    \"{component}.test.tsx\": exists:1\n";
        fs::write(&config_path, policy).unwrap();
        let producer = project.join("src/components/Button.tsx");
        fs::write(&producer, "").unwrap();
        let nested_cache = project.join("src/components/.assura/cache/check");
        if poisoned {
            fs::create_dir_all(&nested_cache).unwrap();
            fs::write(nested_cache.join("keep.json"), "legacy-cache\n").unwrap();
        }
        let check = || {
            Command::new(env!("CARGO_BIN_EXE_assura-check"))
                .args(["--cache", "--format", "json"])
                .arg(&producer)
                .output()
                .unwrap()
        };
        for required in [true, false, true] {
            fs::write(
                &config_path,
                if required {
                    policy.to_string()
                } else {
                    policy.replace("exists:1", "{}")
                },
            )
            .unwrap();
            let output = check();
            assert_eq!(
                output.status.code(),
                Some(i32::from(required)),
                "poisoned={poisoned}, required={required}, stderr={}",
                String::from_utf8_lossy(&output.stderr)
            );
            let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
            for (field, expected) in [
                ("project_root", &project),
                ("config_path", &config_path),
                ("checked_path", &producer),
            ] {
                assert_eq!(
                    report[field],
                    expected
                        .canonicalize()
                        .unwrap()
                        .to_string_lossy()
                        .replace('\\', "/")
                );
            }
            if required {
                assert_eq!(report["violations"].as_array().unwrap().len(), 1);
                assert_eq!(
                    report["violations"][0]["rule"],
                    "relationship:captured-counterpart-1"
                );
            } else {
                assert!(report["violations"].as_array().unwrap().is_empty());
            }
        }
        assert!(first_json_file(&project.join(".assura/cache/check")).is_some());
        if poisoned {
            assert_eq!(
                fs::read_to_string(nested_cache.join("keep.json")).unwrap(),
                "legacy-cache\n"
            );
        } else {
            assert!(!project.join("src/components/.assura").exists());
        }
    }
}
