//! Hermetic Git fixtures for policy source discovery and precedence.

use super::*;
use tempfile::TempDir;

struct Fixture {
    tmp: TempDir,
    global: PathBuf,
    repo: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let global = tmp.path().join("gitconfig");
        let repo = tmp.path().canonicalize().unwrap().join("group/repo");
        fs::create_dir_all(&repo).unwrap();
        fs::write(&global, "").unwrap();
        let fixture = Self { tmp, global, repo };
        fixture.git(&fixture.repo, &["init", "-q"]);
        fixture
    }

    fn command(&self) -> Command {
        let mut command = Command::new("git");
        for (key, _) in env::vars_os().filter(|(key, _)| key.to_string_lossy().starts_with("GIT_"))
        {
            command.env_remove(key);
        }
        command
            .env("GIT_CONFIG_GLOBAL", &self.global)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CEILING_DIRECTORIES", self.tmp.path().parent().unwrap());
        command
    }

    fn git(&self, dir: &Path, args: &[&str]) {
        let output = self.command().current_dir(dir).args(args).output().unwrap();
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn source(&self, relative: &str) -> PathBuf {
        let path = self.tmp.path().canonicalize().unwrap().join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "{}").unwrap();
        path
    }

    fn select(
        &self,
        target: &Path,
        baseline: Option<&Path>,
        project: Option<&Path>,
    ) -> Result<PolicySources, SelectionError> {
        let home = self.tmp.path().canonicalize().unwrap();
        select(
            target,
            baseline,
            project,
            Some(&home.join("default.yml")),
            Some(&home),
            false,
            || self.command(),
        )
    }
}

#[test]
fn linked_worktree_uses_native_group_repo_and_worktree_precedence() {
    let fixture = Fixture::new();
    let group_policy = fixture.source("group/owner.yml");
    let repo_policy = fixture.source("repo-owner.yml");
    let worktree_policy = fixture.source("worktree-owner.yml");
    let external_project = fixture.source("private.yml");
    let default = fixture.source("default.yml");
    let explicit = fixture.source("explicit.yml");
    fs::write(
        &fixture.global,
        "[includeIf \"gitdir:./group/\"]\n path = group.gitconfig\n",
    )
    .unwrap();
    fs::write(
        fixture.global.with_file_name("group.gitconfig"),
        "[assura]\n ownerPolicy = relative-must-lose.yml\n ownerPolicy = ~/group/owner.yml\n",
    )
    .unwrap();
    fixture.git(
        &fixture.repo,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "--allow-empty",
            "-qm",
            "init",
        ],
    );
    let linked = fixture.tmp.path().join("out-of-group");
    fixture.git(
        &fixture.repo,
        &[
            "worktree",
            "add",
            "-q",
            "--detach",
            linked.to_str().unwrap(),
            "HEAD",
        ],
    );
    let linked = linked.canonicalize().unwrap();

    let grouped = fixture.select(&linked, None, None).unwrap();
    let checkout = grouped.checkout.unwrap();
    assert_eq!(checkout.root, linked);
    assert_eq!(
        checkout.common_dir,
        fixture.repo.join(".git").canonicalize().unwrap()
    );
    assert_ne!(checkout.git_dir, checkout.common_dir);
    let group = grouped.scoped_owner.unwrap();
    assert_eq!(group.path, group_policy);
    assert!(
        matches!(group.origin, SourceOrigin::Git(origin) if origin.ends_with("group.gitconfig"))
    );
    assert_eq!(grouped.user_baseline.unwrap().path, default);
    assert_eq!(grouped.project_root, linked);
    assert!(grouped.project.is_none());

    fixture.git(
        &fixture.repo,
        &["config", "assura.ownerPolicy", "~/repo-owner.yml"],
    );
    fixture.git(
        &fixture.repo,
        &["config", "assura.projectConfig", "~/private.yml"],
    );
    let overridden = fixture.select(&linked, Some(&explicit), None).unwrap();
    assert_eq!(
        overridden.user_baseline.unwrap(),
        Selection {
            path: explicit,
            origin: SourceOrigin::Explicit
        }
    );
    let repo_owner = overridden.scoped_owner.unwrap();
    assert_eq!(repo_owner.path, repo_policy);
    assert!(matches!(repo_owner.origin, SourceOrigin::Git(origin) if origin.ends_with("/config")));
    let project = overridden.project.unwrap();
    assert_eq!(project.path, external_project);
    assert!(matches!(project.origin, SourceOrigin::Git(origin) if origin.ends_with("/config")));

    fixture.git(
        &fixture.repo,
        &["config", "extensions.worktreeConfig", "true"],
    );
    fixture.git(
        &linked,
        &[
            "config",
            "--worktree",
            "assura.ownerPolicy",
            "~/worktree-owner.yml",
        ],
    );
    let worktree_owner = fixture
        .select(&linked, None, None)
        .unwrap()
        .scoped_owner
        .unwrap();
    assert_eq!(worktree_owner.path, worktree_policy);
    assert!(
        matches!(worktree_owner.origin, SourceOrigin::Git(origin) if origin.ends_with("/config.worktree"))
    );
    assert_eq!(
        fixture
            .select(&fixture.repo, None, None)
            .unwrap()
            .scoped_owner
            .unwrap()
            .path,
        repo_policy
    );

    let unrelated = fixture.tmp.path().join("unrelated");
    fs::create_dir(&unrelated).unwrap();
    fixture.git(&unrelated, &["init", "-q"]);
    let unrelated = fixture.select(&unrelated, None, None).unwrap();
    assert!(unrelated.scoped_owner.is_none());
    assert!(unrelated.project.is_none());
    assert_eq!(unrelated.user_baseline.unwrap().path, default);
}

#[test]
fn file_target_selects_nearest_project_and_stops_at_checkout() {
    let fixture = Fixture::new();
    fixture.source("group/.assura/config.yml");
    let file = fixture.source("group/repo/subdir/file");
    let no_project = fixture.select(&file, None, None).unwrap();
    assert!(no_project.project.is_none());
    assert!(no_project.user_baseline.is_none());
    assert_eq!(no_project.project_root, fixture.repo);

    let nearest = fixture.source("group/repo/subdir/.assura/config.yml");
    fixture.source("group/repo/.assura/config.yml");
    let found = fixture.select(&file, None, None).unwrap();
    assert_eq!(found.project_root, file.parent().unwrap());
    assert_eq!(
        found.project.unwrap(),
        Selection {
            path: nearest,
            origin: SourceOrigin::Discovered
        }
    );

    fixture.git(
        &fixture.repo,
        &["config", "assura.projectConfig", "~/git-project.yml"],
    );
    let explicit = fixture.source("explicit-project.yml");
    assert_eq!(
        fixture
            .select(&file, None, Some(&explicit))
            .unwrap()
            .project
            .unwrap(),
        Selection {
            path: explicit,
            origin: SourceOrigin::Explicit
        }
    );
}

#[test]
fn invalid_explicit_selection_and_malformed_git_config_fail() {
    let fixture = Fixture::new();
    fixture.source("default.yml");
    assert!(matches!(
        fixture.select(&fixture.repo, Some(Path::new("relative.yml")), None),
        Err(SelectionError::InvalidPath {
            layer: "user baseline",
            ..
        })
    ));
    assert!(
        matches!(fixture.select(&fixture.repo, None, Some(&fixture.repo.join("missing.yml"))), Err(SelectionError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound)
    );

    fixture.git(&fixture.repo, &["config", "assura.ownerPolicy", ""]);
    assert!(matches!(
        fixture.select(&fixture.repo, None, None),
        Err(SelectionError::InvalidPath {
            layer: "assura.ownerPolicy",
            ..
        })
    ));
    fs::write(fixture.repo.join(".git/config"), "[broken\n").unwrap();
    assert!(
        matches!(fixture.select(&fixture.repo, None, None), Err(SelectionError::Git { status, diagnostic, .. }) if status.code() == Some(128) && diagnostic.contains("bad config"))
    );
}

#[test]
fn non_repo_is_valid_but_inaccessible_git_config_is_an_error() {
    let mut fixture = Fixture::new();
    let outside = fixture.tmp.path().canonicalize().unwrap();
    let local_project = fixture.source(".assura/config.yml");
    let found = fixture.select(&outside, None, None).unwrap();
    assert!(found.checkout.is_none());
    assert!(found.scoped_owner.is_none());
    assert_eq!(found.project_root, outside);
    assert_eq!(found.project.unwrap().path, local_project);
    fixture.global = fixture.repo.clone();
    assert!(
        matches!(fixture.select(&fixture.repo, None, None), Err(SelectionError::Git { diagnostic, .. }) if diagnostic.contains("unable to access"))
    );
}

#[cfg(unix)]
#[test]
fn nul_framing_preserves_tabs_newlines_and_trailing_spaces() {
    let fixture = Fixture::new();
    let path = fixture.source("owner\tpolicy\n.yml ");
    let global = fixture.source("git\tconfig\n ");
    let fixture = Fixture { global, ..fixture };
    fs::write(&fixture.global, "").unwrap();
    fixture.git(
        &fixture.repo,
        &[
            "config",
            "--global",
            "assura.ownerPolicy",
            path.to_str().unwrap(),
        ],
    );
    let selection = fixture
        .select(&fixture.repo, None, None)
        .unwrap()
        .scoped_owner
        .unwrap();
    assert_eq!(selection.path, path);
    assert_eq!(
        selection.origin,
        SourceOrigin::Git(format!("file:{}", fixture.global.display()))
    );
}
