//! Black-box lifecycle coverage for Assura's managed Git hook entrypoints.

use assura::cli::{GitHooksManager, HookType};
use std::path::Path;
use std::process::{Command, Output};

fn git(project: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .arg("-C")
        .arg(project)
        .args(args)
        .output()
        .expect("run git")
}

fn assert_git(project: &Path, args: &[&str]) {
    let output = git(project, args);
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assura_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_assura"))
}

fn command_output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn install_preserves_an_existing_custom_hook_and_reports_it() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    let custom_hook = hooks_dir.join("pre-push");
    let custom_content = "#!/bin/sh\necho custom-pre-push\n";
    std::fs::write(&custom_hook, custom_content).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    let outcome = manager.install_all(false).unwrap();

    assert!(outcome.installed.contains(&HookType::PreCommit));
    assert!(outcome.installed.contains(&HookType::PostCheckout));
    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert_eq!(
        std::fs::read_to_string(custom_hook).unwrap(),
        custom_content
    );
}

#[test]
fn force_install_never_overwrites_an_existing_custom_hook() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    let custom_hook = hooks_dir.join("pre-push");
    let custom_content = "#!/bin/sh\necho custom-pre-push\n";
    std::fs::write(&custom_hook, custom_content).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    let outcome = manager.install_all(true).unwrap();

    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert_eq!(
        std::fs::read_to_string(custom_hook).unwrap(),
        custom_content
    );
}

#[test]
fn repeat_install_reports_an_existing_managed_hook_without_calling_it_custom() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install_all(false).unwrap();
    let outcome = manager.install_all(false).unwrap();

    assert!(outcome.preserved.is_empty());
    assert_eq!(outcome.unchanged, HookType::all());
}

#[test]
fn force_install_refreshes_an_existing_managed_hook() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install_all(false).unwrap();
    let outcome = manager.install_all(true).unwrap();
    let status = manager.status(HookType::PrePush);

    assert!(outcome.installed.is_empty());
    assert!(outcome.unchanged.is_empty());
    assert!(outcome.preserved.is_empty());
    assert_eq!(outcome.refreshed, HookType::all());
    assert!(status.is_ready());
}

#[test]
fn force_install_preserves_a_custom_hook_that_mentions_the_legacy_marker() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    let custom_hook = hooks_dir.join("pre-push");
    let custom_content = "#!/bin/sh\n# Git hook managed by Assura\necho custom-pre-push\n";
    std::fs::write(&custom_hook, custom_content).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    let outcome = manager.install_all(true).unwrap();

    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert_eq!(
        std::fs::read_to_string(custom_hook).unwrap(),
        custom_content
    );
}

#[test]
fn uninstall_preserves_a_custom_hook_that_mentions_the_legacy_marker() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    let custom_hook = hooks_dir.join("pre-push");
    let custom_content = "#!/bin/sh\n# Git hook managed by Assura\necho custom-pre-push\n";
    std::fs::write(&custom_hook, custom_content).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.uninstall(HookType::PrePush).unwrap();

    assert_eq!(
        std::fs::read_to_string(custom_hook).unwrap(),
        custom_content
    );
}

#[test]
fn uninstall_all_does_not_report_a_preserved_custom_hook_as_removed() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    let custom_hook = hooks_dir.join("pre-push");
    let custom_content = "#!/bin/sh\necho custom-pre-push\n";
    std::fs::write(&custom_hook, custom_content).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    let outcome = manager.uninstall_all().unwrap();

    assert!(outcome.removed.is_empty());
    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert_eq!(
        std::fs::read_to_string(custom_hook).unwrap(),
        custom_content
    );
}

#[test]
fn uninstall_all_removes_exact_managed_entrypoints_and_sidecars() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install_all(false).unwrap();
    let outcome = manager.uninstall_all().unwrap();

    assert_eq!(outcome.removed, HookType::all());
    assert!(outcome.preserved.is_empty());
    for hook_type in HookType::all() {
        assert!(!hooks_dir.join(hook_type.as_str()).exists());
        assert!(!project
            .path()
            .join(".assura/hooks")
            .join(hook_type.as_str())
            .exists());
    }
}

#[test]
fn uninstall_preserves_the_sidecar_used_by_a_custom_hook() {
    let project = tempfile::TempDir::new().unwrap();
    let hooks_dir = project.path().join(".git/hooks");
    let assura_hooks_dir = project.path().join(".assura/hooks");
    std::fs::create_dir_all(&hooks_dir).unwrap();
    std::fs::create_dir_all(&assura_hooks_dir).unwrap();
    let custom_hook = hooks_dir.join("pre-push");
    let sidecar = assura_hooks_dir.join("pre-push");
    std::fs::write(&custom_hook, "#!/bin/sh\nexec ../.assura/hooks/pre-push\n").unwrap();
    std::fs::write(&sidecar, "#!/bin/sh\necho sidecar\n").unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.uninstall(HookType::PrePush).unwrap();

    assert!(custom_hook.is_file());
    assert!(sidecar.is_file());
}

#[test]
fn uninstall_preserves_an_unowned_orphan_assura_sidecar() {
    let project = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(project.path().join(".git/hooks")).unwrap();
    let assura_hooks_dir = project.path().join(".assura/hooks");
    std::fs::create_dir_all(&assura_hooks_dir).unwrap();
    let sidecar = assura_hooks_dir.join("pre-push");
    std::fs::write(&sidecar, "#!/bin/sh\necho sidecar\n").unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.uninstall(HookType::PrePush).unwrap();

    assert_eq!(
        std::fs::read_to_string(sidecar).unwrap(),
        "#!/bin/sh\necho sidecar\n"
    );
}

#[test]
fn effective_hooks_path_preserves_custom_hook_without_touching_default_hooks() {
    let project = tempfile::TempDir::new().unwrap();
    assert_git(project.path(), &["init", "--quiet"]);
    assert_git(
        project.path(),
        &["config", "core.hooksPath", ".effective-hooks"],
    );
    let effective_hooks = project.path().join(".effective-hooks");
    let custom_hook = effective_hooks.join("pre-push");
    let custom_content = b"#!/bin/sh\necho custom-effective-hook\n";
    std::fs::create_dir_all(&effective_hooks).unwrap();
    std::fs::write(&custom_hook, custom_content).unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    let installed = manager.install_all(true).unwrap();
    let removed = manager.uninstall_all().unwrap();

    assert_eq!(installed.preserved, vec![HookType::PrePush]);
    assert_eq!(removed.preserved, vec![HookType::PrePush]);
    assert_eq!(std::fs::read(&custom_hook).unwrap(), custom_content);
    assert!(!project.path().join(".git/hooks/pre-push").exists());
    assert!(!project.path().join(".assura/hooks/pre-push").exists());
}

#[test]
fn configured_effective_pre_push_runs_for_advisory_and_blocking_pushes() {
    let project = tempfile::TempDir::new().unwrap();
    let remote = tempfile::TempDir::new().unwrap();
    let effective_hooks = project.path().join(".effective-hooks");
    assert_git(project.path(), &["init", "--quiet"]);
    assert_git(
        project.path(),
        &["config", "user.email", "assura@example.test"],
    );
    assert_git(project.path(), &["config", "user.name", "Assura test"]);
    assert_git(
        project.path(),
        &["config", "core.hooksPath", ".effective-hooks"],
    );
    std::fs::create_dir_all(project.path().join(".assura")).unwrap();
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    std::fs::write(
        project.path().join(".assura/config.yml"),
        "structure:\n  ./:\n    extra: true\n    children:\n      src/:\n        files:\n          naming: kebab-case\n          extensions: [\\\"rs\\\"]\n",
    )
    .unwrap();
    std::fs::write(project.path().join("src/BadName.rs"), "fn main() {}\n").unwrap();
    assert_git(project.path(), &["add", "."]);
    assert_git(project.path(), &["commit", "--quiet", "-m", "initial"]);
    assert_git(
        project.path(),
        &["checkout", "--quiet", "-b", "proof-hook-lifecycle"],
    );
    assert_git(remote.path(), &["init", "--bare", "--quiet"]);
    assert_git(
        project.path(),
        &["remote", "add", "origin", remote.path().to_str().unwrap()],
    );

    let installed = Command::new(assura_bin())
        .args(["hooks", "install"])
        .arg(project.path())
        .output()
        .expect("install hooks");
    assert!(
        installed.status.success(),
        "hook install failed: {}",
        command_output_text(&installed)
    );
    assert!(effective_hooks.join("pre-push").is_file());
    assert!(!project.path().join(".git/hooks/pre-push").exists());

    let binary_dir = assura_bin().parent().unwrap().to_path_buf();
    let inherited_path = std::env::var("PATH").expect("UTF-8 test PATH");
    let path_separator = if cfg!(windows) { ";" } else { ":" };
    let path = format!(
        "{}{}{}",
        binary_dir.display(),
        path_separator,
        inherited_path
    );
    let advisory = Command::new("git")
        .arg("-C")
        .arg(project.path())
        .args(["push", "--set-upstream", "origin", "proof-hook-lifecycle"])
        .env("PATH", &path)
        .output()
        .expect("run advisory push");
    assert!(
        advisory.status.success(),
        "advisory push failed: {}",
        command_output_text(&advisory)
    );
    assert!(
        command_output_text(&advisory).contains("WARNING: Assura validation found issues"),
        "advisory push did not run Assura hook: {}",
        command_output_text(&advisory)
    );

    assert_git(
        project.path(),
        &["commit", "--quiet", "--allow-empty", "-m", "second"],
    );
    let blocking = Command::new("git")
        .arg("-C")
        .arg(project.path())
        .args(["push", "origin", "proof-hook-lifecycle"])
        .env("PATH", path)
        .env("ASSURA_BLOCKING_PUSH", "1")
        .output()
        .expect("run blocking push");
    assert!(!blocking.status.success());
    assert!(
        command_output_text(&blocking).contains("ASSURA_BLOCKING_PUSH=1 is set"),
        "blocking push did not run Assura hook: {}",
        command_output_text(&blocking)
    );
}
