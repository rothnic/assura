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
