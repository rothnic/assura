use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;
use std::process::{Child, Command, Stdio};

fn write_project(root: &Path, file_name: &str) {
    fs::create_dir_all(root.join(".assura")).unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    write_project_config(root, "kebab-case");
    fs::write(root.join("src").join(file_name), "").unwrap();
}

fn write_project_config(root: &Path, naming: &str) {
    fs::write(
        root.join(".assura/config.yml"),
        format!(
            r#"
structure:
  ./:
    files:
      naming_patterns:
        "*.ts": {naming}
    directories:
      naming: kebab-case
    children:
      .assura/:
        inherit: false
        files:
          naming: kebab-case
exclude:
  - ".assura/**"
"#
        ),
    )
    .unwrap();
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

fn daemon_listen_arg(temp: &tempfile::TempDir, name: &str) -> String {
    #[cfg(unix)]
    {
        format!("unix:{}", temp.path().join(name).display())
    }
    #[cfg(not(unix))]
    {
        let _ = temp;
        let _ = name;
        "127.0.0.1:0".to_string()
    }
}

fn read_daemon_addr_or_skip(server: &mut Child) -> Option<String> {
    let stdout = server.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut addr = String::new();
    reader.read_line(&mut addr).unwrap();
    let addr = addr.trim().to_string();
    if !addr.is_empty() {
        return Some(addr);
    }

    let mut stderr = String::new();
    if let Some(stderr_pipe) = server.stderr.take() {
        let mut reader = BufReader::new(stderr_pipe);
        let _ = reader.read_to_string(&mut stderr);
    }
    if stderr.contains("Operation not permitted") {
        let _ = server.wait();
        return None;
    }
    panic!("assura-checkd exited before publishing an address:\n{stderr}");
}

fn refresh_daemon_status(addr: &str) {
    let refresh = Command::new(env!("CARGO_BIN_EXE_assura-check-client"))
        .arg(addr)
        .output()
        .unwrap();
    assert!(
        refresh.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&refresh.stdout),
        String::from_utf8_lossy(&refresh.stderr)
    );
}

#[test]
fn quiet_single_project_suppresses_success_output() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("quiet-project");
    write_project(&project, "valid-file.ts");

    let output = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
}

#[test]
fn quiet_single_project_falls_back_to_diagnostics_on_failure() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("quiet-project");
    write_project(&project, "BadName.ts");

    let output = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BadName.ts"));
}

#[test]
fn batch_cli_validates_multiple_project_roots() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first-project");
    let second = temp.path().join("second-project");
    write_project(&first, "first-file.ts");
    write_project(&second, "second-file.ts");

    let output = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--format")
        .arg("json")
        .arg(&first)
        .arg(&second)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["success"], true);
    assert_eq!(report["reports"].as_array().unwrap().len(), 2);
}

#[test]
fn batch_cli_fails_when_any_project_fails() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first-project");
    let second = temp.path().join("second-project");
    write_project(&first, "first-file.ts");
    write_project(&second, "BadName.ts");

    let output = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--quiet")
        .arg(&first)
        .arg(&second)
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("BadName.ts"));
}

#[test]
fn cache_dir_reuses_clean_immutable_results_across_git_worktrees() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("project");
    let sibling = temp.path().join("sibling");
    let cache = temp.path().join("cache");
    write_project(&project, "valid-file.ts");
    for args in [
        vec!["init"],
        vec!["config", "user.email", "assura@example.test"],
        vec!["config", "user.name", "Assura Test"],
        vec!["add", "."],
        vec!["commit", "-m", "baseline"],
    ] {
        let output = Command::new("git")
            .arg("-C")
            .arg(&project)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let worktree = Command::new("git")
        .arg("-C")
        .arg(&project)
        .args(["worktree", "add", "--detach"])
        .arg(&sibling)
        .arg("HEAD")
        .output()
        .unwrap();
    assert!(worktree.status.success());

    let first = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(project.join("src"))
        .output()
        .unwrap();
    assert!(first.status.success());
    assert!(cache.join("shared").is_dir());
    fs::remove_dir_all(cache.join("worktrees")).unwrap();

    let second = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(sibling.join("src"))
        .output()
        .unwrap();
    assert!(second.status.success());
    assert!(
        !cache.join("worktrees").exists(),
        "a clean sibling worktree should return the shared immutable report"
    );
}

#[test]
fn cache_dir_does_not_publish_shared_results_when_ignored_files_exist() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("ignored-project");
    let cache = temp.path().join("cache");
    write_project(&project, "valid-file.ts");
    fs::write(project.join(".gitignore"), "ignored/\n").unwrap();
    for args in [
        vec!["init"],
        vec!["config", "user.email", "assura@example.test"],
        vec!["config", "user.name", "Assura Test"],
        vec!["add", "."],
        vec!["commit", "-m", "baseline"],
    ] {
        let output = Command::new("git")
            .arg("-C")
            .arg(&project)
            .args(args)
            .output()
            .unwrap();
        assert!(output.status.success());
    }
    fs::create_dir_all(project.join("ignored")).unwrap();
    fs::write(project.join("ignored/generated.ts"), "generated\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(project.join("src"))
        .output()
        .unwrap();

    assert!(output.status.success());
    assert!(cache.join("worktrees").is_dir());
    assert!(!cache.join("shared").exists());
}

#[test]
fn cache_dir_validates_changed_config_before_reuse() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("cached-config-project");
    let cache = temp.path().join("cache");
    write_project(&project, "valid-file.ts");

    let first = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );

    write_project_config(&project, "not_a_real_case");
    let second = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();

    assert_eq!(second.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&second.stderr);
    assert!(
        stderr.contains("not_a_real_case"),
        "stderr:\n{stderr}\nstdout:\n{}",
        String::from_utf8_lossy(&second.stdout)
    );
}

#[test]
fn cache_dir_recomputes_after_valid_config_rule_change() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("cached-valid-config-project");
    let cache = temp.path().join("cache");
    write_project(&project, "valid-file.ts");

    let first = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );

    write_project_config(&project, "snake_case");
    let second = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();

    assert_eq!(second.status.code(), Some(1));
    let stdout = String::from_utf8_lossy(&second.stdout);
    assert!(
        stdout.contains("valid-file.ts"),
        "stdout:\n{stdout}\nstderr:\n{}",
        String::from_utf8_lossy(&second.stderr)
    );
}

#[test]
fn cache_dir_ignores_corrupt_cache_entry() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("corrupt-cache-project");
    let cache = temp.path().join("cache");
    write_project(&project, "valid-file.ts");

    let first = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        first.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr)
    );

    let cache_entry = first_json_file(&cache).expect("expected cache entry");
    fs::write(cache_entry, b"not json").unwrap();

    let second = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--cache-dir")
        .arg(&cache)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();

    assert!(
        second.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&second.stdout),
        String::from_utf8_lossy(&second.stderr)
    );
}

#[test]
fn check_fast_exists_counts_prune_excluded_children() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("fast-exists-exclude-project");
    fs::create_dir_all(project.join(".assura")).unwrap();
    fs::write(
        project.join(".assura/config.yml"),
        r#"
structure:
  ./:
    files:
      exists:
        "*.ts": "1"
exclude:
  - "ignored.ts"
"#,
    )
    .unwrap();
    fs::write(project.join("valid-file.ts"), "").unwrap();
    fs::write(project.join("ignored.ts"), "").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_assura-check"))
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn status_cli_reads_clean_daemon_status_file() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("status-project");
    let status_file = temp.path().join("assura-check.status");
    write_project(&project, "valid-file.ts");

    let mut server = Command::new(env!("CARGO_BIN_EXE_assura-checkd"))
        .arg("--listen")
        .arg(daemon_listen_arg(&temp, "assura-checkd.sock"))
        .arg("--root")
        .arg(&project)
        .arg("--status-file")
        .arg(&status_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let Some(addr) = read_daemon_addr_or_skip(&mut server) else {
        return;
    };
    refresh_daemon_status(&addr);

    let status = Command::new(env!("CARGO_BIN_EXE_assura-check-status"))
        .arg(&status_file)
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );

    let status = Command::new(env!("CARGO_BIN_EXE_assura-check-status"))
        .current_dir(temp.path())
        .output()
        .unwrap();
    assert!(
        status.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&status.stdout),
        String::from_utf8_lossy(&status.stderr)
    );

    let _ = server.kill();
    let _ = server.wait();
}
