//! Ownership-boundary coverage for Assura's managed Git hook pairs.

use assura::cli::{GitHooksManager, HookType};
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

static OWNERSHIP_FIXTURE_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

fn ownership_fixture_mutex() -> &'static Mutex<()> {
    OWNERSHIP_FIXTURE_MUTEX.get_or_init(|| Mutex::new(()))
}

fn ownership_fixture_guard() -> MutexGuard<'static, ()> {
    ownership_fixture_mutex()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

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

fn write_exact_legacy_pre_push_pair(project: &Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let (wrapper, sidecar) = hook_paths(project, HookType::PrePush);
    let canonical_sidecar = include_str!("../.assura/hooks/pre-push").replace("\r\n", "\n");
    std::fs::write(&sidecar, canonical_sidecar).unwrap();
    std::fs::write(&wrapper, legacy_wrapper_content(&sidecar)).unwrap();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    (wrapper, sidecar)
}

fn legacy_wrapper_content(sidecar: &Path) -> String {
    format!(
        "#!/bin/sh\n# Git hook managed by Assura\n# This file was auto-generated. Do not modify manually.\n\nASSURA_HOOK=\"{}\"\n\nif [ -f \"$ASSURA_HOOK\" ]; then\n    exec \"$ASSURA_HOOK\" \"$@\"\nelse\n    echo \"Warning: Assura hook not found at $ASSURA_HOOK\" >&2\n    exit 0\nfi\n",
        sidecar.display()
    )
}

#[test]
fn direct_force_install_preserves_a_custom_wrapper() {
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
fn bulk_lifecycle_preserves_and_reports_non_utf8_hook_content() {
    let _fixture_guard = ownership_fixture_guard();
    let project = project_with_git_hooks();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    let wrapper_bytes = [0xff, 0x00, b'w'];
    let sidecar_bytes = [0xfe, 0x00, b's'];
    std::fs::write(&wrapper, wrapper_bytes).unwrap();
    std::fs::write(&sidecar, sidecar_bytes).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    let install = manager.install_all(true).unwrap();
    let uninstall = manager.uninstall_all().unwrap();

    assert_eq!(install.preserved, vec![HookType::PrePush]);
    assert_eq!(uninstall.preserved, vec![HookType::PrePush]);
    assert_eq!(std::fs::read(wrapper).unwrap(), wrapper_bytes);
    assert_eq!(std::fs::read(sidecar).unwrap(), sidecar_bytes);
}

#[test]
fn install_cli_reports_preserved_artifacts_without_calling_all_of_them_custom_hooks() {
    let _fixture_guard = ownership_fixture_guard();
    let project = project_with_git_hooks();
    let (_, sidecar) = hook_paths(project.path(), HookType::PrePush);
    std::fs::write(sidecar, "arbitrary orphan content\n").unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_assura-full"))
        .args(["hooks", "install"])
        .arg(project.path())
        .arg("--force")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Preserved existing hook artifacts:"),
        "stdout was:\n{stdout}"
    );
    assert!(!stdout.contains("Preserved existing custom hooks:"));
}

#[test]
fn bulk_uninstall_preserves_a_managed_wrapper_with_a_modified_sidecar() {
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
    let project = project_with_git_hooks();
    let (wrapper, sidecar) = write_exact_legacy_pre_push_pair(project.path());
    let manager = GitHooksManager::new(project.path()).unwrap();
    let status = manager.status(HookType::PrePush);

    assert!(status.is_managed);
    assert!(!status.is_current);
    assert!(!status.is_ready());
    manager.uninstall(HookType::PrePush).unwrap();
    assert!(!wrapper.exists());
    assert!(!sidecar.exists());
}

#[test]
fn exact_legacy_wrapper_is_owned_with_a_known_current_sidecar() {
    let _fixture_guard = ownership_fixture_guard();
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (wrapper, sidecar) = hook_paths(project.path(), HookType::PrePush);
    assert!(manager.status(HookType::PrePush).is_current);

    std::fs::write(&wrapper, legacy_wrapper_content(&sidecar)).unwrap();
    let status = manager.status(HookType::PrePush);

    assert!(status.is_managed);
    assert!(!status.is_current);
}

#[test]
fn embedded_sidecar_is_owned_with_a_known_current_wrapper() {
    let _fixture_guard = ownership_fixture_guard();
    let project = project_with_git_hooks();
    let manager = GitHooksManager::new(project.path()).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (_, sidecar) = hook_paths(project.path(), HookType::PrePush);
    assert!(manager.status(HookType::PrePush).is_current);

    std::fs::write(&sidecar, include_str!("../.assura/hooks/pre-push")).unwrap();
    assert!(manager.status(HookType::PrePush).is_current);
}

#[test]
fn default_install_upgrades_an_exact_legacy_pair_and_reruns_idempotently() {
    let _fixture_guard = ownership_fixture_guard();
    let project = project_with_git_hooks();
    let (wrapper, sidecar) = write_exact_legacy_pre_push_pair(project.path());
    let legacy_wrapper = std::fs::read(&wrapper).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();
    let before = manager.status(HookType::PrePush);

    let first = manager.install_all(false).unwrap();
    let upgraded_wrapper = std::fs::read(&wrapper).unwrap();
    let after = manager.status(HookType::PrePush);
    let second = manager.install_all(false).unwrap();
    let forced = manager.install_all(true).unwrap();
    manager.uninstall(HookType::PrePush).unwrap();

    assert!(before.is_managed);
    assert!(!before.is_current);
    assert!(!before.is_ready());
    assert!(before.display().contains("upgrade required"));
    assert_eq!(first.refreshed, vec![HookType::PrePush]);
    assert_ne!(upgraded_wrapper, legacy_wrapper);
    assert!(after.is_ready());
    assert!(after.is_current);
    assert_eq!(second.unchanged, HookType::all());
    assert_eq!(forced.refreshed, HookType::all());
    assert!(!wrapper.exists());
    assert!(!sidecar.exists());
}

#[test]
fn default_install_preserves_a_legacy_wrapper_with_a_drifted_sidecar() {
    let _fixture_guard = ownership_fixture_guard();
    let project = project_with_git_hooks();
    let (wrapper, sidecar) = write_exact_legacy_pre_push_pair(project.path());
    let legacy_wrapper = std::fs::read(&wrapper).unwrap();
    let drifted_sidecar = b"#!/bin/sh\necho user-changed-sidecar\n";
    std::fs::write(&sidecar, drifted_sidecar).unwrap();
    let manager = GitHooksManager::new(project.path()).unwrap();

    let outcome = manager.install_all(false).unwrap();

    assert_eq!(outcome.preserved, vec![HookType::PrePush]);
    assert_eq!(std::fs::read(wrapper).unwrap(), legacy_wrapper);
    assert_eq!(std::fs::read(sidecar).unwrap(), drifted_sidecar);
}

#[cfg(unix)]
#[test]
fn default_install_upgrades_legacy_shell_expansion_before_real_invocation() {
    let _fixture_guard = ownership_fixture_guard();
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::TempDir::new().unwrap();
    let project = root
        .path()
        .join("project $(touch dollar-sentinel) `touch backtick-sentinel` \"quoted\"");
    std::fs::create_dir_all(project.join(".git/hooks")).unwrap();
    std::fs::create_dir_all(project.join(".assura/hooks")).unwrap();
    let (wrapper, sidecar) = write_exact_legacy_pre_push_pair(&project);
    let manager = GitHooksManager::new(&project).unwrap();
    let before = manager.status(HookType::PrePush);

    let install_result = manager.install(HookType::PrePush, false);
    let after = manager.status(HookType::PrePush);
    let invoked = root.path().join("legacy-upgrade-invoked");
    std::fs::write(&sidecar, "#!/bin/sh\nprintf invoked > \"$1\"\n").unwrap();
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o755)).unwrap();
    let invocation = Command::new(&wrapper)
        .arg(&invoked)
        .current_dir(root.path())
        .status()
        .unwrap();

    assert!(!root.path().join("dollar-sentinel").exists());
    assert!(!root.path().join("backtick-sentinel").exists());
    assert!(before.is_managed);
    assert!(!before.is_current);
    assert!(!before.is_ready());
    assert!(install_result.is_ok());
    assert!(after.is_ready());
    assert!(after.is_current);
    assert!(invocation.success());
    assert_eq!(std::fs::read_to_string(invoked).unwrap(), "invoked");
}

#[test]
fn direct_uninstall_removes_an_exact_managed_orphan_sidecar() {
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
    let _fixture_guard = ownership_fixture_guard();
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
fn generated_wrapper_invokes_a_literal_metacharacter_path_without_expansion() {
    let _fixture_guard = ownership_fixture_guard();
    use std::os::unix::fs::PermissionsExt;

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

#[cfg(target_os = "linux")]
fn non_utf8_project(root: &Path) -> std::path::PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let project_name = OsString::from_vec(b"project non-utf8 \xff ' quoted".to_vec());
    let project = root.join(project_name);
    std::fs::create_dir_all(project.join(".git/hooks")).unwrap();
    std::fs::create_dir_all(project.join(".assura/hooks")).unwrap();
    project
}

#[cfg(target_os = "linux")]
fn status_once_with_exec_context<F>(
    executable: &Path,
    run: F,
) -> Result<std::process::ExitStatus, String>
where
    F: FnOnce() -> std::io::Result<std::process::ExitStatus>,
{
    run().map_err(|error| {
        format!(
            "direct exec failed: error={error}; raw_os_error={:?}; {}",
            error.raw_os_error(),
            exec_failure_context(executable)
        )
    })
}

#[cfg(target_os = "linux")]
fn exec_failure_context(executable: &Path) -> String {
    use std::os::unix::fs::MetadataExt;

    let escaped = escape_path(executable);
    let Ok(metadata) = std::fs::metadata(executable) else {
        return format!("executable={escaped}; metadata=unavailable; holder_snapshot=incomplete");
    };
    let (holders, scanned_processes, scanned_fds, read_failures, truncated) =
        matching_inode_holders(metadata.dev(), metadata.ino());
    let holders = if holders.is_empty() {
        "none observed".to_string()
    } else {
        holders.join(",")
    };
    let mount = matching_mount(executable).unwrap_or_else(|| "unavailable".to_string());
    format!(
        "executable={escaped}; dev={}; inode={}; mode={:#o}; holders=[{holders}]; \
         holder_snapshot=incomplete(racy_proc_scan); scanned_fds={scanned_fds}; \
         scanned_processes={scanned_processes}; read_failures={read_failures}; \
         truncated={truncated}; mount={mount}",
        metadata.dev(),
        metadata.ino(),
        metadata.mode()
    )
}

#[cfg(target_os = "linux")]
fn matching_inode_holders(dev: u64, inode: u64) -> (Vec<String>, usize, usize, usize, bool) {
    use std::ffi::OsString;
    use std::os::unix::fs::MetadataExt;

    const MAX_PROCESSES: usize = 4_096;
    const MAX_SCANNED_FDS: usize = 16_384;
    const MAX_HOLDERS: usize = 32;
    let mut holders = Vec::new();
    let mut scanned_processes = 0;
    let mut scanned_fds = 0;
    let mut read_failures = 0;
    let Ok(process_entries) = std::fs::read_dir("/proc") else {
        return (holders, scanned_processes, scanned_fds, 1, false);
    };
    let current_pid = OsString::from(std::process::id().to_string());
    let entries = process_entries.map(|entry| entry.map(|entry| entry.file_name()));
    let (processes, process_read_failures, mut truncated) =
        bounded_process_ids(current_pid, entries, MAX_PROCESSES);
    read_failures += process_read_failures;

    'processes: for pid in processes {
        scanned_processes += 1;
        let process = Path::new("/proc").join(&pid);
        let fd_directory = process.join("fd");
        let descriptors = match std::fs::read_dir(&fd_directory) {
            Ok(descriptors) => descriptors,
            Err(_) => {
                read_failures += 1;
                continue;
            }
        };
        for descriptor in descriptors.flatten() {
            if scanned_fds == MAX_SCANNED_FDS {
                truncated = true;
                break 'processes;
            }
            scanned_fds += 1;
            let descriptor_metadata = match std::fs::metadata(descriptor.path()) {
                Ok(metadata) => metadata,
                Err(_) => {
                    read_failures += 1;
                    continue;
                }
            };
            if descriptor_metadata.dev() != dev || descriptor_metadata.ino() != inode {
                continue;
            }
            let fd = descriptor.file_name();
            let fd_info = process.join("fdinfo").join(&fd);
            let (flags, access) = match std::fs::read_to_string(fd_info)
                .ok()
                .and_then(|contents| descriptor_access(&contents).map(str::to_owned))
            {
                Some(flags) => {
                    let access = access_from_flags(&flags).unwrap_or("unknown");
                    (flags, access)
                }
                None => {
                    read_failures += 1;
                    ("unavailable".to_string(), "unknown")
                }
            };
            holders.push(format!(
                "pid={} fd={} flags={flags} access={access}",
                pid.to_string_lossy(),
                fd.to_string_lossy()
            ));
            if holders.len() == MAX_HOLDERS {
                truncated = true;
                break 'processes;
            }
        }
    }

    (
        holders,
        scanned_processes,
        scanned_fds,
        read_failures,
        truncated,
    )
}

#[cfg(target_os = "linux")]
fn bounded_process_ids<I>(
    current_pid: std::ffi::OsString,
    entries: I,
    limit: usize,
) -> (Vec<std::ffi::OsString>, usize, bool)
where
    I: IntoIterator<Item = std::io::Result<std::ffi::OsString>>,
{
    if limit == 0 {
        return (Vec::new(), 0, true);
    }
    let mut processes = vec![current_pid.clone()];
    let mut read_failures = 0;
    let mut truncated = false;
    for entry in entries {
        let pid = match entry {
            Ok(pid) => pid,
            Err(_) => {
                read_failures += 1;
                continue;
            }
        };
        if !pid.as_encoded_bytes().iter().all(u8::is_ascii_digit) || pid == current_pid {
            continue;
        }
        if processes.len() == limit {
            truncated = true;
            break;
        }
        processes.push(pid);
    }
    (processes, read_failures, truncated)
}

#[cfg(target_os = "linux")]
fn descriptor_access(fd_info: &str) -> Option<&str> {
    fd_info
        .lines()
        .find_map(|line| line.strip_prefix("flags:\t"))
}

#[cfg(target_os = "linux")]
fn access_from_flags(flags: &str) -> Option<&'static str> {
    match u32::from_str_radix(flags, 8).ok()? & 0b11 {
        0 => Some("read"),
        1 => Some("write"),
        2 => Some("read-write"),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn matching_mount(path: &Path) -> Option<String> {
    let mount_info = std::fs::read_to_string("/proc/self/mountinfo").ok()?;
    mount_info
        .lines()
        .filter_map(|line| {
            let fields = line.split_whitespace().collect::<Vec<_>>();
            let separator = fields.iter().position(|field| *field == "-")?;
            let mount_point = decode_mount_field(fields.get(4)?);
            path.starts_with(&mount_point).then(|| {
                (
                    mount_point.as_os_str().as_encoded_bytes().len(),
                    format!(
                        "id={} device={} point={} fs={}",
                        fields[0],
                        fields[2],
                        escape_path(&mount_point),
                        fields.get(separator + 1).unwrap_or(&"unknown")
                    ),
                )
            })
        })
        .max_by_key(|(length, _)| *length)
        .map(|(_, mount)| mount)
}

#[cfg(target_os = "linux")]
fn decode_mount_field(field: &str) -> std::path::PathBuf {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let bytes = field.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' && index + 3 < bytes.len() {
            let octal = &field[index + 1..index + 4];
            if let Ok(byte) = u8::from_str_radix(octal, 8) {
                decoded.push(byte);
                index += 4;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    std::path::PathBuf::from(OsString::from_vec(decoded))
}

#[cfg(target_os = "linux")]
fn escape_path(path: &Path) -> String {
    use std::fmt::Write;
    use std::os::unix::ffi::OsStrExt;

    let mut escaped = String::new();
    for byte in path.as_os_str().as_bytes() {
        match byte {
            b' '..=b'~' if *byte != b'\\' => escaped.push(char::from(*byte)),
            b'\\' => escaped.push_str("\\\\"),
            _ => write!(escaped, "\\x{byte:02x}").unwrap(),
        }
    }
    escaped
}

#[cfg(target_os = "linux")]
#[test]
fn proc_holder_scan_prioritizes_current_pid_before_global_cap() {
    let _fixture_guard = ownership_fixture_guard();
    use std::ffi::OsString;

    let entries = ["7", "42", "8", "9"]
        .into_iter()
        .map(|pid| Ok::<_, std::io::Error>(OsString::from(pid)));

    let (processes, read_failures, truncated) =
        bounded_process_ids(OsString::from("42"), entries, 3);

    assert_eq!(processes, ["42", "7", "8"].map(OsString::from).to_vec());
    assert_eq!(read_failures, 0);
    assert!(truncated);
}

#[cfg(target_os = "linux")]
#[test]
fn ownership_fixture_guard_excludes_writer_launch_overlap() {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{mpsc, TryLockError};

    let root = tempfile::TempDir::new().unwrap();
    let temporary = root.path().join(".guarded-wrapper.assura-tmp");
    let wrapper = root.path().join("guarded-wrapper");
    let invoked = root.path().join("guarded-invoked");
    let (writer_ready_tx, writer_ready_rx) = mpsc::channel();
    let (release_writer_tx, release_writer_rx) = mpsc::channel();
    let (launch_blocked_tx, launch_blocked_rx) = mpsc::channel();

    let writer_wrapper = wrapper.clone();
    let writer = std::thread::spawn(move || {
        let _fixture_guard = ownership_fixture_guard();
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .unwrap();
        file.write_all(b"#!/bin/sh\nprintf invoked > \"$1\"\n")
            .unwrap();
        file.set_permissions(std::fs::Permissions::from_mode(0o755))
            .unwrap();
        writer_ready_tx.send(()).unwrap();
        release_writer_rx.recv().unwrap();
        drop(file);
        std::fs::rename(temporary, writer_wrapper).unwrap();
    });

    writer_ready_rx.recv().unwrap();
    let launch_wrapper = wrapper.clone();
    let launch_invoked = invoked.clone();
    let launcher = std::thread::spawn(move || {
        assert!(matches!(
            ownership_fixture_mutex().try_lock(),
            Err(TryLockError::WouldBlock)
        ));
        launch_blocked_tx.send(()).unwrap();
        let _fixture_guard = ownership_fixture_guard();
        Command::new(launch_wrapper)
            .arg(launch_invoked)
            .status()
            .unwrap()
    });

    launch_blocked_rx.recv().unwrap();
    assert!(!invoked.exists());
    release_writer_tx.send(()).unwrap();
    writer.join().unwrap();
    let status = launcher.join().unwrap();
    assert!(status.success());
    assert_eq!(std::fs::read_to_string(invoked).unwrap(), "invoked");
}

#[cfg(target_os = "linux")]
#[test]
fn exec_failure_context_reports_writer_without_retrying() {
    let _fixture_guard = ownership_fixture_guard();
    use std::cell::Cell;
    use std::ffi::OsString;
    use std::fs::OpenOptions;
    use std::os::unix::ffi::OsStringExt;
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::TempDir::new().unwrap();
    let executable = root
        .path()
        .join(OsString::from_vec(b"diagnostic-\xff-hook".to_vec()));
    let invoked = root.path().join("diagnostic-invoked");
    std::fs::write(&executable, "#!/bin/sh\nprintf invoked > \"$1\"\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755)).unwrap();
    let writer = OpenOptions::new().write(true).open(&executable).unwrap();
    let attempts = Cell::new(0);

    let error = status_once_with_exec_context(&executable, || {
        attempts.set(attempts.get() + 1);
        Command::new(&executable).arg(&invoked).status()
    })
    .unwrap_err();

    assert_eq!(attempts.get(), 1);
    assert!(error.contains("raw_os_error=Some(26)"), "{error}");
    assert!(error.contains("diagnostic-\\xff-hook"), "{error}");
    assert!(
        error.contains(&format!("pid={}", std::process::id())),
        "{error}"
    );
    assert!(error.contains("access=write"), "{error}");
    assert!(!invoked.exists());

    drop(writer);
    let status = Command::new(&executable).arg(&invoked).status().unwrap();
    assert!(status.success());
    assert_eq!(std::fs::read_to_string(invoked).unwrap(), "invoked");
}

#[cfg(target_os = "linux")]
#[test]
fn generated_wrapper_preserves_non_utf8_project_path_bytes_end_to_end() {
    let _fixture_guard = ownership_fixture_guard();
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::TempDir::new().unwrap();
    let project = non_utf8_project(root.path());
    let manager = GitHooksManager::new(&project).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (wrapper, sidecar) = hook_paths(&project, HookType::PrePush);

    assert!(manager.status(HookType::PrePush).is_ready());

    let invoked = root.path().join("non-utf8-invoked");
    std::fs::write(&sidecar, "#!/bin/sh\nprintf invoked > \"$1\"\n").unwrap();
    std::fs::set_permissions(&sidecar, std::fs::Permissions::from_mode(0o755)).unwrap();
    let status =
        status_once_with_exec_context(&wrapper, || Command::new(&wrapper).arg(&invoked).status())
            .unwrap();

    assert!(status.success());
    assert_eq!(std::fs::read_to_string(invoked).unwrap(), "invoked");
}

#[cfg(target_os = "linux")]
#[test]
fn lossy_legacy_wrapper_is_not_managed_for_a_non_utf8_project_path() {
    let _fixture_guard = ownership_fixture_guard();
    let root = tempfile::TempDir::new().unwrap();
    let project = non_utf8_project(root.path());
    let manager = GitHooksManager::new(&project).unwrap();
    manager.install(HookType::PrePush, false).unwrap();
    let (wrapper, sidecar) = hook_paths(&project, HookType::PrePush);
    let legacy_lossy_wrapper = format!(
        "#!/bin/sh\n# Git hook managed by Assura\n# This file was auto-generated. Do not modify manually.\n\nASSURA_HOOK=\"{}\"\n\nif [ -f \"$ASSURA_HOOK\" ]; then\n    exec \"$ASSURA_HOOK\" \"$@\"\nelse\n    echo \"Warning: Assura hook not found at $ASSURA_HOOK\" >&2\n    exit 0\nfi\n",
        sidecar.to_string_lossy()
    );

    std::fs::write(wrapper, legacy_lossy_wrapper).unwrap();

    assert!(!manager.status(HookType::PrePush).is_managed);
}
