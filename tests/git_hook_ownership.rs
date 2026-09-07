//! Ownership-boundary coverage for Assura's managed Git hook pairs.

use assura::cli::{GitHooksManager, HookType};
use std::path::Path;

fn project_with_git_hooks() -> tempfile::TempDir {
    let project = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(project.path().join(".git/hooks")).unwrap();
    std::fs::create_dir_all(project.path().join(".assura/hooks")).unwrap();
    project
}

fn hook_paths(project: &Path, hook_type: HookType) -> (std::path::PathBuf, std::path::PathBuf) {
    let hook_name = hook_type.as_str();
    (
        project.join(".git/hooks").join(hook_name),
        project.join(".assura/hooks").join(hook_name),
    )
}

#[test]
fn direct_force_install_preserves_a_custom_wrapper() {
    let project = project_with_git_hooks();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    let custom = "#!/bin/sh\necho custom\n";
    std::fs::write(&wrapper, custom).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    assert!(manager.install(HookType::PrePush, true).is_err());
    assert_eq!(std::fs::read_to_string(wrapper).unwrap(), custom);
    assert!(!sidecar.exists());
}

#[test]
fn direct_force_install_preserves_a_managed_wrapper_with_a_modified_sidecar() {
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    let wrapper_before = std::fs::read_to_string(&wrapper).unwrap();
    let custom = "#!/bin/sh\necho modified-sidecar\n";
    std::fs::write(&sidecar, custom).unwrap();

    assert!(manager.install(HookType::PrePush, true).is_err());
    assert_eq!(std::fs::read_to_string(wrapper).unwrap(), wrapper_before);
    assert_eq!(std::fs::read_to_string(sidecar).unwrap(), custom);
}

#[test]
fn bulk_force_install_preserves_a_managed_wrapper_with_a_modified_sidecar() {
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install_all(false).unwrap();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    let wrapper_before = std::fs::read_to_string(&wrapper).unwrap();
    let custom = "#!/bin/sh\necho modified-sidecar\n";
    std::fs::write(&sidecar, custom).unwrap();

    let outcome = manager.install_all(true).unwrap();

    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert!(!outcome.refreshed.contains(&HookType::PrePush));
    assert_eq!(std::fs::read_to_string(wrapper).unwrap(), wrapper_before);
    assert_eq!(std::fs::read_to_string(sidecar).unwrap(), custom);
}

#[test]
fn sidecar_drift_is_not_reported_as_managed_or_ready() {
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (_, sidecar) = hook_paths(project.path(), HookType::PrePush);
    std::fs::write(sidecar, "#!/bin/sh\necho modified-sidecar\n").unwrap();

    let status = manager.status(HookType::PrePush);

    assert!(!status.is_managed);
    assert!(!status.is_ready());
}

#[test]
fn direct_install_preserves_an_unowned_orphan_sidecar() {
    let project = project_with_git_hooks();
    let (_, sidecar) = hook_paths(project.path(), HookType::PrePush);
    std::fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
    let custom = "#!/bin/sh\necho arbitrary-orphan\n";
    std::fs::write(&sidecar, custom).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    assert!(manager.install(HookType::PrePush, true).is_err());
    assert_eq!(std::fs::read_to_string(sidecar).unwrap(), custom);
}

#[test]
fn bulk_install_preserves_and_reports_an_unowned_orphan_sidecar() {
    let project = project_with_git_hooks();
    let (_, sidecar) = hook_paths(project.path(), HookType::PrePush);
    std::fs::create_dir_all(sidecar.parent().unwrap()).unwrap();
    let custom = "#!/bin/sh\necho arbitrary-orphan\n";
    std::fs::write(&sidecar, custom).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    let outcome = manager.install_all(true).unwrap();

    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert_eq!(std::fs::read_to_string(sidecar).unwrap(), custom);
}

#[test]
fn bulk_uninstall_preserves_a_managed_wrapper_with_a_modified_sidecar() {
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install_all(false).unwrap();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    let wrapper_before = std::fs::read_to_string(&wrapper).unwrap();
    let custom = "#!/bin/sh\necho modified-sidecar\n";
    std::fs::write(&sidecar, custom).unwrap();

    let outcome = manager.uninstall_all().unwrap();

    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert!(!outcome.removed.contains(&HookType::PrePush));
    assert_eq!(std::fs::read_to_string(wrapper).unwrap(), wrapper_before);
    assert_eq!(std::fs::read_to_string(sidecar).unwrap(), custom);
}

#[test]
fn direct_uninstall_preserves_a_managed_wrapper_with_a_modified_sidecar() {
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    let wrapper_before = std::fs::read_to_string(&wrapper).unwrap();
    let custom = "#!/bin/sh\necho modified-sidecar\n";
    std::fs::write(&sidecar, custom).unwrap();

    manager.uninstall(HookType::PrePush).unwrap();

    assert_eq!(std::fs::read_to_string(wrapper).unwrap(), wrapper_before);
    assert_eq!(std::fs::read_to_string(sidecar).unwrap(), custom);
}

#[test]
fn install_repairs_a_missing_sidecar_only_for_an_exact_managed_wrapper() {
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install_all(false).unwrap();
    let (_, sidecar) = hook_paths(project.path(), HookType::PrePush);
    std::fs::remove_file(&sidecar).unwrap();

    let outcome = manager.install_all(false).unwrap();

    assert_eq!(outcome.refreshed, vec![HookType::PrePush]);
    assert!(manager.status(HookType::PrePush).is_ready());
}

#[test]
fn install_repairs_a_missing_wrapper_only_for_an_exact_managed_sidecar() {
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install_all(false).unwrap();
    let (wrapper, _) = hook_paths(project.path(), HookType::PrePush);
    std::fs::remove_file(&wrapper).unwrap();

    let outcome = manager.install_all(false).unwrap();

    assert_eq!(outcome.refreshed, vec![HookType::PrePush]);
    assert!(manager.status(HookType::PrePush).is_ready());
}

#[test]
fn exact_legacy_wrapper_remains_managed_for_removal() {
    let project = project_with_git_hooks();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    std::fs::write(&sidecar, include_str!("../.assura/hooks/pre-push")).unwrap();
    let legacy_wrapper = format!(
        "#!/bin/sh\n# Git hook managed by Assura\n# This file was auto-generated. Do not modify manually.\n\nASSURA_HOOK=\"{}\"\n\nif [ -f \"$ASSURA_HOOK\" ]; then\n    exec \"$ASSURA_HOOK\" \"$@\"\nelse\n    echo \"Warning: Assura hook not found at $ASSURA_HOOK\" >&2\n    exit 0\nfi\n",
        sidecar.display()
    );
    std::fs::write(&wrapper, legacy_wrapper).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    assert!(manager.status(HookType::PrePush).is_managed);
    manager.uninstall(HookType::PrePush).unwrap();
    assert!(!wrapper.exists());
    assert!(!sidecar.exists());
}

#[test]
fn direct_uninstall_removes_an_exact_managed_orphan_sidecar() {
    let project = project_with_git_hooks();
    let (_, sidecar) = hook_paths(project.path(), HookType::PrePush);
    std::fs::write(&sidecar, include_str!("../.assura/hooks/pre-push")).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    manager.uninstall(HookType::PrePush).unwrap();

    assert!(!sidecar.exists());
}

#[cfg(unix)]
#[test]
fn hook_file_symlinks_are_preserved_without_touching_their_targets() {
    use std::os::unix::fs::symlink;

    let project = project_with_git_hooks();
    let external = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(external.path(), "external-user-content\n").unwrap();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    symlink(external.path(), &wrapper).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    let outcome = manager.install_all(true).unwrap();

    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert!(std::fs::symlink_metadata(wrapper)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(
        std::fs::read_to_string(external.path()).unwrap(),
        "external-user-content\n"
    );
    assert!(!sidecar.exists());
}

#[cfg(unix)]
#[test]
fn dangling_hook_symlinks_are_not_treated_as_absent() {
    use std::os::unix::fs::symlink;

    let project = project_with_git_hooks();
    let external_dir = tempfile::TempDir::new().unwrap();
    let external_target = external_dir.path().join("missing-target");
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    symlink(&external_target, &wrapper).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    let outcome = manager.install_all(true).unwrap();

    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert!(std::fs::symlink_metadata(wrapper)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(!external_target.exists());
    assert!(!sidecar.exists());
}

#[cfg(unix)]
#[test]
fn sidecar_symlinks_preserve_the_managed_wrapper_and_external_target() {
    use std::os::unix::fs::symlink;

    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install_all(false).unwrap();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    let wrapper_before = std::fs::read_to_string(&wrapper).unwrap();
    std::fs::remove_file(&sidecar).unwrap();
    let external = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(external.path(), "external-user-content\n").unwrap();
    symlink(external.path(), &sidecar).unwrap();

    let install = manager.install_all(true).unwrap();
    let uninstall = manager.uninstall_all().unwrap();

    assert_eq!(install.preserved, vec![HookType::PrePush]);
    assert_eq!(uninstall.preserved, vec![HookType::PrePush]);
    assert_eq!(std::fs::read_to_string(wrapper).unwrap(), wrapper_before);
    assert!(std::fs::symlink_metadata(sidecar)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(
        std::fs::read_to_string(external.path()).unwrap(),
        "external-user-content\n"
    );
}

#[cfg(unix)]
#[test]
fn symlinked_assura_hook_directory_is_preserved_without_external_writes() {
    use std::os::unix::fs::symlink;

    let project = project_with_git_hooks();
    let external = tempfile::TempDir::new().unwrap();
    std::fs::remove_dir(project.path().join(".assura/hooks")).unwrap();
    symlink(external.path(), project.path().join(".assura/hooks")).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    let outcome = manager.install_all(true).unwrap();

    assert_eq!(outcome.preserved, HookType::all());
    assert_eq!(std::fs::read_dir(external.path()).unwrap().count(), 0);
    assert_eq!(
        std::fs::read_dir(project.path().join(".git/hooks"))
            .unwrap()
            .count(),
        0
    );
}

#[cfg(unix)]
#[test]
fn failed_wrapper_write_rolls_back_the_changed_live_sidecar() {
    use std::os::unix::fs::PermissionsExt;

    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o555)).unwrap();
    std::fs::set_permissions(
        project.path().join(".git/hooks"),
        std::fs::Permissions::from_mode(0o555),
    )
    .unwrap();

    let result = manager.install(HookType::PrePush, true);

    std::fs::set_permissions(
        project.path().join(".git/hooks"),
        std::fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.is_err());
    assert_eq!(
        std::fs::metadata(sidecar).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[cfg(unix)]
#[test]
fn generated_wrapper_invokes_a_literal_metacharacter_path_without_expansion() {
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    let root = tempfile::TempDir::new().unwrap();
    let project = root
        .path()
        .join("project $(touch injected-marker) ; ' quoted");
    std::fs::create_dir_all(project.join(".git/hooks")).unwrap();
    std::fs::create_dir_all(project.join(".assura/hooks")).unwrap();
    let manager = GitHooksManager::new(&project).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (wrapper, sidecar) = hook_paths(&project, HookType::PrePush);
    let invoked = root.path().join("invoked");
    std::fs::write(&sidecar, "#!/bin/sh\nprintf invoked > \"$1\"\n").unwrap();
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o755)).unwrap();

    let status = Command::new(&wrapper)
        .arg(&invoked)
        .current_dir(root.path())
        .status()
        .unwrap();

    assert!(status.success());
    assert_eq!(std::fs::read_to_string(invoked).unwrap(), "invoked");
    assert!(!root.path().join("injected-marker").exists());
}
