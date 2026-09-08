//! Unit coverage for Git hook lifecycle resolution and status behavior.
use super::*;
use std::process::Command;

fn git(project: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {} failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_effective_hooks_path(project: &Path) -> PathBuf {
    let output = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["rev-parse", "--path-format=absolute", "--git-path", "hooks"])
        .output()
        .expect("resolve effective hooks path through git");
    assert!(
        output.status.success(),
        "git rev-parse failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    PathBuf::from(
        String::from_utf8(output.stdout)
            .expect("Git path is UTF-8 in this fixture")
            .trim_end(),
    )
}

#[test]
fn test_hook_type_from_str() {
    assert_eq!(
        "pre-commit".parse::<HookType>().unwrap(),
        HookType::PreCommit
    );
    assert!("invalid".parse::<HookType>().is_err());
}

#[test]
fn test_hook_status_display() {
    let status = HookStatus {
        hook_type: HookType::PreCommit,
        is_installed: true,
        is_managed: true,
        is_current: true,
        git_runnable: true,
        assura_runnable: true,
        git_path: PathBuf::from(".git/hooks/pre-commit"),
        assura_path: PathBuf::from(".assura/hooks/pre-commit"),
    };

    assert!(status.display().contains("installed"));
}

#[test]
fn git_hooks_dir_resolves_regular_git_directory() {
    let project = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(project.path().join(".git/hooks")).unwrap();

    let hooks_dir = resolve_git_hooks_dir(project.path()).unwrap();

    assert_eq!(hooks_dir, project.path().join(".git/hooks"));
}

#[test]
fn git_hooks_dir_uses_relative_core_hooks_path_from_a_real_repository() {
    let project = tempfile::TempDir::new().unwrap();
    git(project.path(), &["init", "--quiet"]);
    git(
        project.path(),
        &["config", "core.hooksPath", ".assura-git-hooks"],
    );

    let hooks_dir = resolve_git_hooks_dir(project.path()).unwrap();

    assert_eq!(hooks_dir, git_effective_hooks_path(project.path()));
}

#[test]
fn git_hooks_dir_uses_absolute_core_hooks_path_from_a_real_repository() {
    let project = tempfile::TempDir::new().unwrap();
    let configured_hooks = tempfile::TempDir::new().unwrap();
    git(project.path(), &["init", "--quiet"]);
    git(
        project.path(),
        &[
            "config",
            "core.hooksPath",
            configured_hooks.path().to_str().unwrap(),
        ],
    );

    let hooks_dir = resolve_git_hooks_dir(project.path()).unwrap();

    assert_eq!(hooks_dir, git_effective_hooks_path(project.path()));
}

#[test]
fn linked_worktree_lifecycle_uses_its_effective_hooks_path() {
    let project = tempfile::TempDir::new().unwrap();
    let linked_worktree = project.path().join("linked-worktree");
    let linked_worktree_string = linked_worktree.to_str().unwrap();
    git(project.path(), &["init", "--quiet"]);
    git(
        project.path(),
        &["config", "user.email", "assura@example.test"],
    );
    git(project.path(), &["config", "user.name", "Assura test"]);
    git(
        project.path(),
        &["commit", "--quiet", "--allow-empty", "-m", "initial"],
    );
    git(
        project.path(),
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            linked_worktree_string,
        ],
    );
    git(
        project.path(),
        &["config", "extensions.worktreeConfig", "true"],
    );
    git(
        &linked_worktree,
        &[
            "config",
            "--worktree",
            "core.hooksPath",
            ".linked-assura-hooks",
        ],
    );

    let manager = GitHooksManager::new(&linked_worktree).unwrap();
    let expected_hooks_dir = git_effective_hooks_path(&linked_worktree);
    manager.install(HookType::PrePush, false).unwrap();

    let status = manager.status(HookType::PrePush);
    assert_eq!(status.git_path, expected_hooks_dir.join("pre-push"));
    assert!(status.git_path.is_file());
    assert!(!project.path().join(".git/hooks/pre-push").exists());

    manager.uninstall(HookType::PrePush).unwrap();
    assert!(!expected_hooks_dir.join("pre-push").exists());
    assert!(!project.path().join(".git/hooks/pre-push").exists());
}

#[test]
fn git_hooks_dir_resolves_worktree_git_file_to_common_hooks() {
    let project = tempfile::TempDir::new().unwrap();
    let git_dir = project.path().join("main.git/worktrees/agent");
    std::fs::create_dir_all(&git_dir).unwrap();
    std::fs::create_dir_all(project.path().join("main.git/hooks")).unwrap();
    std::fs::write(git_dir.join("commondir"), "../..\n# ignored metadata\n").unwrap();
    std::fs::write(
        project.path().join(".git"),
        format!("gitdir: {}\n# ignored metadata\n", git_dir.display()),
    )
    .unwrap();

    let hooks_dir = resolve_git_hooks_dir(project.path()).unwrap();

    assert_eq!(
        hooks_dir,
        std::fs::canonicalize(project.path().join("main.git"))
            .unwrap()
            .join("hooks")
    );
}
