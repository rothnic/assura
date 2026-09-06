//! Black-box lifecycle coverage for Assura's managed Git hook entrypoints.

use assura::cli::{GitHooksManager, HookType};

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
fn uninstall_removes_an_orphan_assura_sidecar() {
    let project = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(project.path().join(".git/hooks")).unwrap();
    let assura_hooks_dir = project.path().join(".assura/hooks");
    std::fs::create_dir_all(&assura_hooks_dir).unwrap();
    let sidecar = assura_hooks_dir.join("pre-push");
    std::fs::write(&sidecar, "#!/bin/sh\necho sidecar\n").unwrap();

    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.uninstall(HookType::PrePush).unwrap();

    assert!(!sidecar.exists());
}
