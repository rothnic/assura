#[path = "../src/cli/git_snapshot.rs"]
mod git_snapshot;

use git_snapshot::{Change, Scope, Side, Snapshot};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Repo(PathBuf);

impl Repo {
    fn init(format: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "assura-git-snapshot-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        fs::create_dir(&dir).unwrap();
        let repo = Self(dir.canonicalize().unwrap());
        repo.git(&["init", "--initial-branch=main", "--object-format", format]);
        repo.git(&["config", "user.name", "Assura fixture"]);
        repo.git(&["config", "user.email", "fixture@example.invalid"]);
        repo.git(&["config", "core.hooksPath", ".git/hooks"]);
        repo.git(&["config", "core.autocrlf", "false"]);
        repo
    }

    fn command(&self) -> Command {
        let mut cmd = Command::new("git");
        cmd.current_dir(&self.0)
            .env_remove("GIT_INDEX_FILE")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env(
                "GIT_CONFIG_GLOBAL",
                if cfg!(windows) { "NUL" } else { "/dev/null" },
            );
        cmd
    }

    fn git(&self, args: &[&str]) -> String {
        let output = self.command().args(args).output().unwrap();
        assert_ok(&output);
        String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned()
    }

    fn write(&self, path: impl AsRef<Path>, content: impl AsRef<[u8]>) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }

    fn commit(&self, message: &str) {
        self.git(&["add", "-A"]);
        self.git(&["commit", "-m", message]);
    }

    fn staged(&self) -> Snapshot {
        Snapshot::resolve(&self.0, Scope::Staged).unwrap()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn assert_ok(output: &Output) {
    assert!(
        output.status.success(),
        "{}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

fn read(snapshot: &Snapshot, side: Side, path: &str) -> Option<Vec<u8>> {
    snapshot.read_file(side, Path::new(path)).unwrap()
}

#[test]
fn staged_paths_policy_and_content_share_a_pinned_tree() {
    let repo = Repo::init("sha1");
    repo.write(".assura/config.yml", "base policy");
    repo.write("changed", "base");
    repo.write("worktree-only", "base");
    repo.write("legacy", "inherited");
    repo.write("deleted", "base");
    repo.commit("base");
    repo.write(".assura/config.yml", "staged policy");
    repo.write("changed", "staged");
    fs::remove_file(repo.0.join("deleted")).unwrap();
    repo.git(&["add", "-u"]);
    repo.write(".assura/config.yml", "worktree policy");
    repo.write("changed", "base");
    repo.write("worktree-only", "worktree");
    repo.write("untracked", "worktree");

    let snapshot = Snapshot::resolve(&repo.0.join(".assura"), Scope::Staged).unwrap();
    assert_eq!(snapshot.root, repo.0);
    assert_eq!(
        snapshot.baseline.commit,
        Some(repo.git(&["rev-parse", "HEAD"]))
    );
    assert_eq!(snapshot.selected.oid, repo.git(&["write-tree"]));
    assert_eq!(
        snapshot.changes,
        [
            Change::Modified(".assura/config.yml".into()),
            Change::Modified("changed".into()),
            Change::Deleted("deleted".into()),
        ]
    );

    repo.git(&["add", "-A"]);
    assert_eq!(
        read(&snapshot, Side::Baseline, "changed"),
        Some(b"base".to_vec())
    );
    assert_eq!(
        read(&snapshot, Side::Selected, "changed"),
        Some(b"staged".to_vec())
    );
    assert_eq!(
        read(&snapshot, Side::Selected, ".assura/config.yml"),
        Some(b"staged policy".to_vec())
    );
    assert_eq!(
        read(&snapshot, Side::Selected, "legacy"),
        Some(b"inherited".to_vec())
    );
    assert_eq!(read(&snapshot, Side::Selected, "deleted"), None);
    assert_eq!(read(&snapshot, Side::Selected, "untracked"), None);
}

#[test]
fn initial_commit_uses_repository_format_empty_tree() {
    for format in ["sha1", "sha256"] {
        let repo = Repo::init(format);
        repo.write("first file", "first");
        repo.git(&["add", "--", "first file"]);
        let snapshot = repo.staged();
        assert_eq!(snapshot.baseline.commit, None);
        assert_eq!(snapshot.baseline.oid, repo.git(&["mktree"]));
        assert_eq!(
            snapshot.baseline.oid.len(),
            if format == "sha1" { 40 } else { 64 }
        );
        assert_eq!(snapshot.changes, [Change::Added("first file".into())]);
        assert_eq!(read(&snapshot, Side::Baseline, "first file"), None);
        assert_eq!(
            read(&snapshot, Side::Selected, "first file"),
            Some(b"first".to_vec())
        );
    }
}

#[test]
fn rejects_unmerged_index() {
    let repo = Repo::init("sha1");
    repo.write("conflict", "base");
    repo.commit("base");
    repo.git(&["checkout", "-b", "other"]);
    repo.write("conflict", "other");
    repo.commit("other");
    repo.git(&["checkout", "main"]);
    repo.write("conflict", "main");
    repo.commit("main");
    let output = repo.command().args(["merge", "other"]).output().unwrap();
    assert_eq!(output.status.code(), Some(1));

    let error = Snapshot::resolve(&repo.0, Scope::Staged).unwrap_err();
    assert!(
        error.contains("unmerged") && error.contains("conflict"),
        "{error}"
    );
}

#[cfg(unix)]
#[test]
fn reads_literal_paths_with_control_and_non_utf8_bytes() {
    use std::ffi::OsStr;
    use std::io::Write;
    use std::os::unix::ffi::OsStrExt;
    use std::process::Stdio;

    let repo = Repo::init("sha1");
    let raw_path = b":tab\tline\n[*] \xff";
    let name = Path::new(OsStr::from_bytes(raw_path));
    repo.write("source", b"raw\0blob\n");
    let blob = repo.git(&["hash-object", "-w", "source"]);
    // Some worktree filesystems forbid non-UTF-8 names. Git's index permits them.
    let mut child = repo
        .command()
        .args(["update-index", "-z", "--index-info"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut input = child.stdin.take().unwrap();
        write!(input, "100644 {blob}\t").unwrap();
        input.write_all(raw_path).unwrap();
        input.write_all(b"\0").unwrap();
    }
    assert_ok(&child.wait_with_output().unwrap());
    let snapshot = repo.staged();
    assert_eq!(snapshot.changes, [Change::Added(name.to_owned())]);
    assert_eq!(
        snapshot.read_file(Side::Selected, name).unwrap(),
        Some(b"raw\0blob\n".to_vec()),
    );
}

#[cfg(unix)]
#[test]
fn path_limited_commit_uses_the_active_hook_index() {
    use std::os::unix::fs::PermissionsExt;

    if std::env::var_os("ASSURA_GIT_SNAPSHOT_PROBE").is_some() {
        path_limited_hook_probe();
        return;
    }

    let repo = Repo::init("sha1");
    repo.write("elsewhere", "base");
    repo.write("chosen path", "base");
    repo.commit("base");
    repo.write("elsewhere", "already staged");
    repo.git(&["add", "elsewhere"]);
    repo.write("chosen path", "commit me");

    let hook = repo.0.join(".git/hooks/pre-commit");
    fs::write(
        &hook,
        "#!/bin/sh\nexec \"$ASSURA_GIT_SNAPSHOT_PROBE\" --exact path_limited_commit_uses_the_active_hook_index --nocapture\n",
    ).unwrap();
    fs::set_permissions(&hook, fs::Permissions::from_mode(0o700)).unwrap();
    let output = repo
        .command()
        .args(["commit", "-m", "chosen only", "--", "chosen path"])
        .env(
            "ASSURA_GIT_SNAPSHOT_PROBE",
            std::env::current_exe().unwrap(),
        )
        .output()
        .unwrap();
    assert_ok(&output);
    let hook_tree = format!(
        "assura-hook-tree: {}",
        repo.git(&["rev-parse", "HEAD^{tree}"])
    );
    assert!(String::from_utf8_lossy(&output.stderr)
        .lines()
        .any(|line| line == hook_tree));
    assert_eq!(repo.git(&["show", "HEAD:chosen path"]), "commit me");
    assert_eq!(repo.git(&["show", "HEAD:elsewhere"]), "base");
    assert_eq!(repo.git(&["show", ":elsewhere"]), "already staged");
}

#[cfg(unix)]
fn path_limited_hook_probe() {
    assert!(std::env::var_os("GIT_INDEX_FILE").is_some());
    let snapshot = Snapshot::resolve(Path::new("."), Scope::Staged).unwrap();
    assert_eq!(snapshot.changes, [Change::Modified("chosen path".into())]);
    eprintln!("assura-hook-tree: {}", snapshot.selected.oid);
    assert_eq!(
        read(&snapshot, Side::Selected, "chosen path"),
        Some(b"commit me".to_vec())
    );
    assert_eq!(
        read(&snapshot, Side::Selected, "elsewhere"),
        Some(b"base".to_vec())
    );
}

#[test]
fn rename_is_conservative_when_candidates_compete() {
    let repo = Repo::init("sha1");
    let content = "keep this line\n".repeat(100);
    repo.write("old", &content);
    repo.commit("base");
    repo.git(&["config", "diff.renames", "false"]);
    repo.git(&["mv", "old", "new"]);
    assert_eq!(
        repo.staged().changes,
        [Change::Renamed {
            from: "old".into(),
            to: "new".into()
        }]
    );
    repo.write("new", format!("{content}extra line\n"));
    repo.git(&["add", "-A"]);
    let snapshot = repo.staged();
    assert_eq!(
        snapshot.changes,
        [Change::Renamed {
            from: "old".into(),
            to: "new".into()
        }]
    );
    assert_eq!(
        read(&snapshot, Side::Baseline, "old").unwrap().len(),
        content.len()
    );
    assert_eq!(
        read(&snapshot, Side::Selected, "new").unwrap().len(),
        content.len() + 11
    );
    repo.write("also new", &content);
    repo.git(&["add", "-A"]);
    let changes = repo.staged().changes;
    assert_eq!(changes.len(), 3);
    assert!(changes.contains(&Change::Deleted("old".into())));
    assert!(changes.contains(&Change::Added("new".into())));
    assert!(changes.contains(&Change::Added("also new".into())));
}

#[test]
fn committed_comparison_can_select_merge_base_or_direct_trees() {
    let repo = Repo::init("sha1");
    repo.write("common", "base");
    repo.commit("common");
    let common = repo.git(&["rev-parse", "HEAD"]);
    repo.git(&["checkout", "-b", "topic"]);
    repo.write("topic", "topic");
    repo.commit("topic");
    repo.git(&["checkout", "main"]);
    repo.write("main", "main");
    repo.commit("main");
    let merge = Snapshot::resolve(
        &repo.0,
        Scope::MergeBase {
            target: "main",
            head: "topic",
        },
    )
    .unwrap();
    assert_eq!(merge.baseline.commit, Some(common));
    assert_eq!(merge.changes, [Change::Added("topic".into())]);

    let base = repo.git(&["rev-parse", "main^{tree}"]);
    let direct = Snapshot::resolve(
        &repo.0,
        Scope::Direct {
            base: &base,
            head: "topic",
        },
    )
    .unwrap();
    assert_eq!(direct.baseline.oid, base);
    assert_eq!(direct.baseline.commit, None);
    assert_eq!(
        direct.changes,
        [
            Change::Deleted("main".into()),
            Change::Added("topic".into())
        ]
    );
    assert!(Snapshot::resolve(
        &repo.0,
        Scope::Direct {
            base: "missing",
            head: "topic"
        }
    )
    .is_err());
}

#[test]
fn linked_worktrees_keep_independent_staged_trees() {
    let repo = Repo::init("sha1");
    repo.write("file", "base");
    repo.commit("base");
    repo.git(&["worktree", "add", "-b", "linked", "linked"]);
    repo.write("file", "main index");
    repo.git(&["add", "file"]);
    repo.write("linked/file", "linked index");
    repo.git(&["-C", "linked", "add", "file"]);

    let first = repo.staged();
    let linked = Snapshot::resolve(&repo.0.join("linked"), Scope::Staged).unwrap();
    assert_eq!(first.baseline.oid, linked.baseline.oid);
    assert_ne!(first.selected.oid, linked.selected.oid);
    assert_eq!(
        read(&first, Side::Selected, "file"),
        Some(b"main index".to_vec())
    );
    assert_eq!(
        read(&linked, Side::Selected, "file"),
        Some(b"linked index".to_vec())
    );
}

#[test]
fn merge_base_requires_unique_shared_history() {
    let repo = Repo::init("sha1");
    repo.write("file", "base");
    repo.commit("base");
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    let unrelated = repo.git(&["commit-tree", &tree, "-m", "unrelated"]);
    let no_base = Snapshot::resolve(
        &repo.0,
        Scope::MergeBase {
            target: "HEAD",
            head: &unrelated,
        },
    )
    .unwrap_err();
    assert!(no_base.contains("no merge base") && no_base.contains("direct comparison"));

    let left = repo.git(&["commit-tree", &tree, "-p", "HEAD", "-m", "left"]);
    let right = repo.git(&["commit-tree", &tree, "-p", "HEAD", "-m", "right"]);
    let first = repo.git(&[
        "commit-tree",
        &tree,
        "-p",
        &left,
        "-p",
        &right,
        "-m",
        "first",
    ]);
    let second = repo.git(&[
        "commit-tree",
        &tree,
        "-p",
        &right,
        "-p",
        &left,
        "-m",
        "second",
    ]);
    let ambiguous = Snapshot::resolve(
        &repo.0,
        Scope::MergeBase {
            target: &first,
            head: &second,
        },
    )
    .unwrap_err();
    assert!(ambiguous.contains("multiple merge bases") && ambiguous.contains("direct comparison"));
}
