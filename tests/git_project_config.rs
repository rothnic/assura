#![cfg(feature = "full-cli")]

use assura::cli::check::PreparedStructureCheck;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let temp = TempDir::new().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let repo = root.join("repo");
        fs::create_dir_all(repo.join("docs")).unwrap();
        fs::write(repo.join("README.md"), "a\nb\n").unwrap();
        fs::write(repo.join("docs/guide.md"), "").unwrap();
        fs::write(root.join("gitconfig"), "").unwrap();
        let fixture = Self {
            _temp: temp,
            root,
            repo,
        };
        fixture.git(&["init", "-q"]);
        fixture
    }

    fn command(&self, binary: impl AsRef<Path>) -> Command {
        let mut command = Command::new(binary.as_ref());
        for (key, _) in std::env::vars_os() {
            if key.to_string_lossy().starts_with("GIT_") {
                command.env_remove(key);
            }
        }
        command
            .current_dir(&self.repo)
            .env("GIT_CONFIG_GLOBAL", self.root.join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1");
        command
    }

    fn git(&self, args: &[&str]) {
        assert_exit(self.command("git").args(args).output().unwrap(), 0);
    }

    fn assura(&self, args: &[&str]) -> Output {
        self.command(env!("CARGO_BIN_EXE_assura"))
            .args(args)
            .output()
            .unwrap()
    }
}

fn assert_exit(output: Output, code: i32) -> Output {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[cfg(unix)]
#[test]
fn missing_git_executable_cannot_bypass_a_private_project_policy() {
    let fixture = Fixture::new();
    let checked_in = fixture.repo.join(".assura/config.yml");
    fs::create_dir(checked_in.parent().unwrap()).unwrap();
    fs::write(&checked_in, "structure:\n  README.md: exists:1\n").unwrap();
    let private = fixture.root.join("private.yml");
    fs::write(&private, "structure:\n  MISSING.md: exists:1\n").unwrap();
    fixture.git(&[
        "config",
        "--local",
        "assura.projectConfig",
        private.to_str().unwrap(),
    ]);
    assert_exit(fixture.assura(&["check", "."]), 1);

    let no_git = fixture.root.join("empty-path");
    fs::create_dir(&no_git).unwrap();
    let invoke = |args: &[&str]| {
        fixture
            .command(env!("CARGO_BIN_EXE_assura"))
            .env("PATH", &no_git)
            .args(args)
            .output()
            .unwrap()
    };
    let error = assert_exit(invoke(&["check", "."]), 2);
    assert!(String::from_utf8_lossy(&error.stderr).contains("run git config"));
    assert_exit(
        invoke(&["check", "--config", checked_in.to_str().unwrap(), "."]),
        0,
    );
}

#[test]
fn git_local_project_policy_replaces_checkout_policy_and_explains_its_origin() {
    let fixture = Fixture::new();
    let project = fixture.root.join("layout.yml");
    let other = fixture.root.join("other.yml");
    let owner = fixture.root.join("owner.yml");
    fs::write(&project, "structure:\n  README.md: exists:1\n").unwrap();
    fs::write(&other, "structure:\n  MISSING.md: exists:1\n").unwrap();
    fs::write(&owner, "max_lines: 1\n").unwrap();
    fixture.git(&[
        "config",
        "--global",
        "assura.projectConfig",
        other.to_str().unwrap(),
    ]);
    fixture.git(&[
        "config",
        "--local",
        "assura.projectConfig",
        project.to_str().unwrap(),
    ]);
    fixture.git(&["config", "assura.ownerPolicy", owner.to_str().unwrap()]);

    // README exceeds the owner budget, but the ordinary check uses project policy.
    assert_exit(fixture.assura(&["check", "."]), 0);
    let plain = assert_exit(fixture.assura(&["explain", "README.md"]), 0);
    assert!(String::from_utf8_lossy(&plain.stdout).contains("project_config="));
    for format in ["json", "agent"] {
        let output = assert_exit(
            fixture.assura(&["explain", "docs/guide.md", "-f", format]),
            0,
        );
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["relative_path"], "docs/guide.md");
        assert_eq!(
            report["config_path"],
            project.to_str().unwrap().replace('\\', "/")
        );
        assert!(report["project_config_git_origin"]
            .as_str()
            .unwrap()
            .starts_with("file:"));
    }

    fs::create_dir(fixture.repo.join(".assura")).unwrap();
    fs::copy(&other, fixture.repo.join(".assura/config.yml")).unwrap();
    assert_exit(fixture.assura(&["check", "."]), 0);
    assert_exit(
        fixture.assura(&["check", "--config", other.to_str().unwrap(), "."]),
        1,
    );

    fixture.git(&["config", "--local", "assura.projectConfig", "relative.yml"]);
    let invalid = assert_exit(fixture.assura(&["check", "."]), 2);
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("relative.yml"));
}

#[test]
fn git_selector_switch_refreshes_prepared_check_and_disk_cache() {
    let fixture = Fixture::new();
    fs::remove_file(fixture.repo.join("docs/guide.md")).unwrap();
    let permissive = fixture.root.join("first.yml");
    let strict = fixture.root.join("second.yml");
    for (path, naming) in [
        (&permissive, "SCREAMING_SNAKE_CASE"),
        (&strict, "snake_case"),
    ] {
        fs::write(
            path,
            format!(
                "structure:\n  ./:\n    files:\n      naming_patterns:\n        '*.md': {naming}\n"
            ),
        )
        .unwrap();
    }
    fixture.git(&[
        "config",
        "--local",
        "assura.projectConfig",
        permissive.to_str().unwrap(),
    ]);

    let mut prepared =
        PreparedStructureCheck::load_for_path(Some(fixture.repo.clone()), None, false).unwrap();
    assert!(prepared.check_path(fixture.repo.clone()).unwrap().success);
    let cache_dir = fixture.root.join("cache");
    let cached_check =
        || fixture.assura(&["check", ".", "--cache-dir", cache_dir.to_str().unwrap()]);
    assert_exit(cached_check(), 0);
    assert!(cache_dir.join("worktrees").is_dir());
    assert_exit(cached_check(), 0);

    fixture.git(&[
        "config",
        "--local",
        "assura.projectConfig",
        strict.to_str().unwrap(),
    ]);
    assert_exit(cached_check(), 1);
    assert_eq!(prepared.config_path(), permissive);
    assert!(prepared.reload_if_config_changed().unwrap());
    assert_eq!(prepared.config_path(), strict);
    assert!(!prepared.check_path(fixture.repo.clone()).unwrap().success);
    assert!(!prepared.reload_if_config_changed().unwrap());

    // Identical bytes at another selected path still replace the source.
    fs::copy(&strict, &permissive).unwrap();
    fixture.git(&[
        "config",
        "assura.projectConfig",
        permissive.to_str().unwrap(),
    ]);
    assert!(prepared.reload_if_config_changed().unwrap());
    assert_eq!(prepared.config_path(), permissive);

    // Move the winner from the local config to an included file.
    let include = fixture.root.join("selection.gitconfig");
    fs::write(
        &include,
        format!("[assura]\n\tprojectConfig = {}\n", permissive.display()),
    )
    .unwrap();
    fixture.git(&["config", "--unset", "assura.projectConfig"]);
    fixture.git(&["config", "include.path", include.to_str().unwrap()]);
    assert!(prepared.config_content_changed().unwrap());
    assert!(prepared.reload_if_config_changed().unwrap());
    assert!(!prepared.reload_if_config_changed().unwrap());

    let mut explicit = PreparedStructureCheck::load_for_path(
        Some(fixture.repo.clone()),
        Some(strict.clone()),
        false,
    )
    .unwrap();
    assert!(!explicit.reload_if_config_changed().unwrap());
    fs::remove_file(&permissive).unwrap();
    assert!(prepared.reload_if_config_changed().is_err());
    assert!(!explicit.reload_if_config_changed().unwrap());
    fs::write(&permissive, "structure:\n  README.md: exists:1\n").unwrap();
    assert!(prepared.reload_if_config_changed().unwrap());
    assert!(prepared.check_path(fixture.repo.clone()).unwrap().success);
}

#[test]
fn daemon_marks_a_changed_git_selection_stale_and_refreshes_its_policy() {
    use assura::daemon::{DaemonCoreError, LocalDaemonCore};

    let fixture = Fixture::new();
    let first = fixture.root.join("first.yml");
    let second = fixture.root.join("second.yml");
    fs::write(&first, "structure:\n  README.md: exists:1\n").unwrap();
    fs::write(&second, "structure:\n  MISSING.md: exists:1\n").unwrap();
    fixture.git(&["config", "assura.projectConfig", first.to_str().unwrap()]);
    let mut daemon = LocalDaemonCore::load(fixture.repo.clone(), None).unwrap();
    assert_eq!(daemon.health().config_path, first);

    fixture.git(&["config", "assura.projectConfig", second.to_str().unwrap()]);
    let path = fixture.repo.join("README.md");
    assert!(matches!(
        daemon.check_changed_path(path.clone()),
        Err(DaemonCoreError::Stale(_))
    ));
    assert_eq!(daemon.refresh().unwrap().config_path, second);
    assert!(!daemon.check_changed_path(path).unwrap().success);
}
