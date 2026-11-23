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
        resolve_snapshot(&self.0, Scope::Staged).unwrap()
    }
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn resolve_snapshot(cwd: &Path, scope: Scope<'_>) -> Result<Snapshot, String> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .unwrap();
    assert_ok(&output);
    let root = String::from_utf8(output.stdout).unwrap();
    Snapshot::resolve(
        cwd,
        scope,
        Path::new(root.trim_end_matches('\n'))
            .canonicalize()
            .unwrap(),
    )
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

    let snapshot = resolve_snapshot(&repo.0.join(".assura"), Scope::Staged).unwrap();
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
fn working_tree_pins_explicit_base_edits_and_nonignored_paths() {
    let repo = Repo::init("sha1");
    repo.write(".gitignore", "ignored*\n");
    repo.write(".assura/config.yml", "base policy");
    repo.write("committed", "base");
    repo.write("deleted", "base");
    repo.write("edited", "base");
    repo.write("inherited", "unchanged");
    repo.commit("base");
    repo.git(&["tag", "baseline"]);
    repo.write("committed", "newer commit");
    repo.commit("head");
    repo.write("edited", "staged");
    repo.git(&["add", "edited"]);
    repo.write("edited", "working edit");
    repo.write(".assura/config.yml", "working policy");
    repo.write("untracked [file]", "working addition");
    repo.write("ignored scratch", "private");
    fs::remove_file(repo.0.join("deleted")).unwrap();

    let index = repo.0.join(".git/index");
    let before = fs::read(&index).unwrap();
    let modified = fs::metadata(&index).unwrap().modified().unwrap();
    let snapshot =
        Snapshot::resolve_working_tree(&repo.0.join(".assura"), "baseline", repo.0.clone())
            .unwrap();
    assert_eq!(fs::read(&index).unwrap(), before);
    assert_eq!(fs::metadata(&index).unwrap().modified().unwrap(), modified);
    assert_eq!(fs::read(repo.0.join("edited")).unwrap(), b"working edit");
    assert_eq!(repo.git(&["show", ":edited"]), "staged");
    assert_eq!(
        snapshot.baseline.commit,
        Some(repo.git(&["rev-parse", "baseline"]))
    );
    assert_eq!(snapshot.selected.commit, None);
    assert_eq!(
        snapshot.changes,
        [
            Change::Modified(".assura/config.yml".into()),
            Change::Modified("committed".into()),
            Change::Deleted("deleted".into()),
            Change::Modified("edited".into()),
            Change::Added("untracked [file]".into()),
        ]
    );

    repo.write("edited", "later edit");
    repo.write(".assura/config.yml", "later policy");
    fs::remove_file(repo.0.join("untracked [file]")).unwrap();
    repo.git(&["tag", "-f", "baseline", "HEAD"]);
    assert_eq!(
        read(&snapshot, Side::Baseline, "committed").unwrap(),
        b"base"
    );
    assert_eq!(
        read(&snapshot, Side::Selected, "edited").unwrap(),
        b"working edit"
    );
    assert_eq!(
        read(&snapshot, Side::Selected, ".assura/config.yml").unwrap(),
        b"working policy"
    );
    assert_eq!(
        read(&snapshot, Side::Selected, "untracked [file]").unwrap(),
        b"working addition"
    );
    assert_eq!(
        read(&snapshot, Side::Selected, "inherited").unwrap(),
        b"unchanged"
    );
    assert_eq!(read(&snapshot, Side::Selected, "ignored scratch"), None);
    assert_eq!(read(&snapshot, Side::Selected, "deleted"), None);
    let error =
        Snapshot::resolve_working_tree(&repo.0, "missing-base", repo.0.clone()).unwrap_err();
    assert!(
        error.contains("resolve Git commit \"missing-base\""),
        "{error}"
    );
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

    let before = fs::read(repo.0.join(".git/index")).unwrap();
    for result in [
        resolve_snapshot(&repo.0, Scope::Staged),
        Snapshot::resolve_working_tree(&repo.0, "HEAD", repo.0.clone()),
    ] {
        let error = result.unwrap_err();
        assert!(
            error.contains("unmerged") && error.contains("conflict"),
            "{error}"
        );
    }
    assert_eq!(fs::read(repo.0.join(".git/index")).unwrap(), before);
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
    let snapshot = resolve_snapshot(Path::new("."), Scope::Staged).unwrap();
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
    let merge = resolve_snapshot(
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
    let direct = resolve_snapshot(
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
    assert!(resolve_snapshot(
        &repo.0,
        Scope::Direct {
            base: "missing",
            head: "topic"
        }
    )
    .is_err());
}

#[test]
fn linked_worktrees_keep_independent_trees() {
    let repo = Repo::init("sha1");
    repo.write("file", "base");
    repo.commit("base");
    repo.git(&["worktree", "add", "-b", "linked", "linked"]);
    repo.write("file", "main index");
    repo.git(&["add", "file"]);
    repo.write("linked/file", "linked index");
    repo.git(&["-C", "linked", "add", "file"]);

    let first = repo.staged();
    let linked = resolve_snapshot(&repo.0.join("linked"), Scope::Staged).unwrap();
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

    repo.write("linked/file", "linked edit");
    let checkout = repo.0.join("linked");
    let working = Snapshot::resolve_working_tree(&checkout, "HEAD", checkout.clone()).unwrap();
    assert_eq!(working.changes, [Change::Modified("file".into())]);
    assert_eq!(
        read(&working, Side::Selected, "file").unwrap(),
        b"linked edit"
    );
    assert_eq!(repo.git(&["-C", "linked", "show", ":file"]), "linked index");
    assert_eq!(repo.git(&["show", ":file"]), "main index");
}

#[test]
fn merge_base_requires_unique_shared_history() {
    let repo = Repo::init("sha1");
    repo.write("file", "base");
    repo.commit("base");
    let tree = repo.git(&["rev-parse", "HEAD^{tree}"]);
    let unrelated = repo.git(&["commit-tree", &tree, "-m", "unrelated"]);
    let no_base = resolve_snapshot(
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
    let ambiguous = resolve_snapshot(
        &repo.0,
        Scope::MergeBase {
            target: &first,
            head: &second,
        },
    )
    .unwrap_err();
    assert!(ambiguous.contains("multiple merge bases") && ambiguous.contains("direct comparison"));
}

#[test]
fn non_regular_entries_are_not_file_content() {
    let repo = Repo::init("sha1");
    repo.write("folder/note", "one\n");
    repo.commit("tree");
    let snapshot = repo.staged();
    assert!(snapshot
        .read_file(Side::Selected, Path::new("folder"))
        .is_err());
    assert_eq!(
        snapshot
            .read_regular_file(Side::Selected, Path::new("folder"))
            .unwrap(),
        None,
    );
}

#[cfg(feature = "full-cli")]
mod cli {
    use super::*;

    fn policy(limit: usize) -> String {
        format!(
            "structure:\n  ./:\n    files:\n      max_lines: {limit}\nexclude:\n  - .assura/**\n  - ignored\n"
        )
    }

    fn check(repo: &Repo, args: &[&str], index: Option<&Path>) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_assura"));
        let home = repo.0.join(".git/fixture-home");
        command
            .current_dir(&repo.0)
            .env_remove("GIT_INDEX_FILE")
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", home.join("gitconfig"))
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .env("APPDATA", &home)
            .env("XDG_CONFIG_HOME", &home);
        if let Some(index) = index {
            command.env("GIT_INDEX_FILE", index);
        }
        command
            .args(["check-change", "--format", "json"])
            .args(args)
            .output()
            .unwrap()
    }

    fn report(output: Output, code: i32) -> serde_json::Value {
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
        serde_json::from_slice(&output.stdout).unwrap()
    }

    #[test]
    fn owner_verdict_survives_project_selection_exclusion_and_advisory_severity() {
        let repo = Repo::init("sha1");
        let owner = tempfile::tempdir().unwrap();
        let baseline = owner.path().join("baseline.yml");
        let scoped = owner.path().join("scoped.yml");
        fs::write(&baseline, "max_lines: 2\nexemptions: ['.assura/**']\n").unwrap();
        fs::write(&scoped, "max_lines: 10\n").unwrap();
        repo.git(&["config", "assura.ownerPolicy", scoped.to_str().unwrap()]);
        repo.write(".assura/config.yml", policy(2));
        repo.write("ignored/new", "a\nb\nc\n");
        repo.git(&["add", "-A"]);
        let args = ["--staged", "--owner-baseline", baseline.to_str().unwrap()];
        let blocked = report(check(&repo, &args, None), 1);
        assert_eq!(blocked["findings"].as_array().unwrap().len(), 1);
        assert_eq!(blocked["findings"][0]["layer"], "user_baseline");
        assert_eq!(blocked["findings"][0]["path"], "ignored/new");
        assert_eq!(blocked["policies"][0]["origin"], "explicit");
        assert!(blocked["policies"][1]["origin"]["git"].is_string());
        assert_eq!(blocked["policies"][2]["tree"], blocked["selected"]["tree"]);

        let alternate = repo.0.join(".assura/alternate.yml");
        repo.write(
            &alternate,
            "structure:\n  ./:
    files:\n      max_lines: 2\n      severity: low\nexclude: ['.assura/**']\n",
        );
        repo.git(&["add", "--", ".assura/alternate.yml"]);
        fs::remove_file(&alternate).unwrap();
        let args = [args.as_slice(), &["--config", alternate.to_str().unwrap()]].concat();
        let blocked = report(check(&repo, &args, None), 1);
        assert_eq!(blocked["findings"][0]["blocking"], true);
        assert_eq!(blocked["findings"][1]["layer"], "project");
        assert_eq!(blocked["findings"][1]["blocking"], false);
        assert_eq!(blocked["policies"][2]["origin"], "explicit");

        #[cfg(unix)]
        {
            let alias = owner.path().join("checkout");
            std::os::unix::fs::symlink(&repo.0, &alias).unwrap();
            let alias = alias.join(".assura/alternate.yml");
            assert_eq!(
                report(
                    check(
                        &repo,
                        &[
                            "--staged",
                            "--owner-baseline",
                            baseline.to_str().unwrap(),
                            "--config",
                            alias.to_str().unwrap(),
                        ],
                        None
                    ),
                    1
                )["findings"],
                blocked["findings"],
            );
        }

        fs::write(
            &baseline,
            "max_lines: 2\nexemptions: ['.assura/**', 'ignored/**']\n",
        )
        .unwrap();
        let exempted = report(check(&repo, &args, None), 0);
        assert_eq!(exempted["findings"].as_array().unwrap().len(), 1);
        assert!(exempted["exemptions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|hit| hit["layer"] == "user_baseline" && hit["pattern"] == "ignored/**"));

        fs::write(&scoped, "max_lines: 2\nexemptions: ['.assura/**']\n").unwrap();
        let blocked = report(check(&repo, &args, None), 1);
        assert_eq!(blocked["findings"][0]["layer"], "scoped_owner");

        repo.git(&["rm", "--cached", "-f", ".assura/alternate.yml"]);
        let missing = check(&repo, &args, None);
        assert_eq!(missing.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&missing.stderr).contains("absent from selected tree"));
    }

    #[test]
    fn required_owner_digest_rejects_missing_and_changed_bytes() {
        use sha2::{Digest, Sha256};

        let repo = Repo::init("sha1");
        repo.write(".assura/config.yml", policy(4));
        repo.commit("project policy");
        repo.write("change.rs", "one\ntwo\n");
        repo.git(&["add", "-A"]);

        let bytes = b"max_lines: 4\n";
        let digest = format!("{:x}", Sha256::digest(bytes));
        let pin = format!("user_baseline={digest}");
        let required = ["--staged", "--require-policy-digest", &pin];
        let absent = check(&repo, &required, None);
        assert!(!absent.status.success());
        assert!(String::from_utf8_lossy(&absent.stderr)
            .contains("required user_baseline policy is missing"));

        let owner = repo.0.join(".git/owner-policy.yml");
        fs::write(&owner, bytes).unwrap();
        let owner = owner.to_str().unwrap();
        let args = [
            "--staged",
            "--owner-baseline",
            owner,
            "--require-policy-digest",
            &pin,
        ];
        let passed = report(check(&repo, &args, None), 0);
        let source = passed["policies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|source| source["layer"] == "user_baseline")
            .unwrap();
        assert_eq!(source["sha256"], digest);
        fs::write(owner, "max_lines: 4\n# Different bytes at the same path\n").unwrap();
        let changed = check(&repo, &args, None);
        assert!(!changed.status.success());
        assert!(String::from_utf8_lossy(&changed.stderr)
            .contains("user_baseline policy digest mismatch"));
    }

    #[test]
    fn owner_only_uses_the_ratchet_and_diagnoses_invalid_selection() {
        let repo = Repo::init("sha1");
        let owner = tempfile::tempdir().unwrap();
        let policy = owner.path().join("owner.yml");
        fs::write(&policy, "max_lines: 2\n").unwrap();
        repo.git(&["config", "assura.ownerPolicy", policy.to_str().unwrap()]);
        repo.write("legacy", "a\nb\nc\n");
        repo.commit("legacy debt");
        repo.write("legacy", "a\nb\nchanged\n");
        repo.git(&["add", "-A"]);
        assert!(report(check(&repo, &["--staged"], None), 0)["policy_path"].is_null());
        repo.write("legacy", "a\nb\nchanged\nd\n");
        repo.git(&["add", "-A"]);
        repo.commit("growth");
        repo.write(".assura/config.yml", "invalid unstaged config");
        let blocked = report(
            check(&repo, &["--base", "HEAD^", "--head", "HEAD"], None),
            1,
        );
        assert_eq!(blocked["findings"][0]["baseline_lines"], 3);
        assert_eq!(blocked["findings"][0]["allowed_lines"], 3);
        fs::remove_file(&policy).unwrap();
        let bad = check(&repo, &["--staged"], None);
        assert_eq!(bad.status.code(), Some(2));
        let diagnostic = String::from_utf8_lossy(&bad.stderr);
        assert!(diagnostic.contains("assura.ownerPolicy"), "{diagnostic}");
        assert!(diagnostic.contains("owner.yml"), "{diagnostic}");
    }

    #[test]
    fn active_index_rules_and_bytes_drive_the_same_verdict_after_commit() {
        let repo = Repo::init("sha1");
        let config = ".assura/config.yml";
        repo.write(config, policy(3));
        repo.write("legacy", "a\nb\nc\nd\n");
        repo.write("flat", "a\nb\nc\nd\n");
        repo.write("shrink", "a\nb\nc\nd\ne\n");
        repo.write("growing debt", "a\nb\nc\nd\n");
        repo.write("formerly compliant", "a\nb\n");
        repo.commit("base");
        let base = repo.git(&["rev-parse", "HEAD"]);

        repo.write(config, policy(100));
        repo.write("ordinary index", "a\nb\nc\nd\n");
        repo.git(&["add", "--", config, "ordinary index"]);

        // Keep the index relative to the fixture root so Git can lock it on Windows.
        let index = Path::new(".git/alternate-index");
        assert_ok(
            &repo
                .command()
                .env("GIT_INDEX_FILE", index)
                .args(["read-tree", "HEAD"])
                .output()
                .unwrap(),
        );
        repo.write(config, policy(2));
        repo.write("flat", "a\nb\nc\nchanged\n");
        repo.write("shrink", "a\nb\nc\nd\n");
        repo.write("growing debt", "a\nb\nc\nd\ne\n");
        repo.write("formerly compliant", "a\nb\nc\n");
        repo.write("new file", "a\nb\nc\n");
        repo.write("ignored/nested", "a\nb\nc\nd\n");
        assert_ok(
            &repo
                .command()
                .env("GIT_INDEX_FILE", index)
                .args([
                    "add",
                    "--",
                    config,
                    "flat",
                    "shrink",
                    "growing debt",
                    "formerly compliant",
                    "new file",
                    "ignored/nested",
                ])
                .output()
                .unwrap(),
        );

        repo.write(config, "broken: [\n");
        repo.write("growing debt", "a\n");
        repo.write("new file", "a\n");
        repo.write("flat", "a\nb\nc\nd\ne\n");
        let ordinary = report(check(&repo, &["--staged"], None), 0);
        assert_eq!(ordinary["files_checked"], 1);

        let staged = report(check(&repo, &["--staged"], Some(index)), 1);
        assert_eq!(staged["scope"], "staged");
        assert_eq!(staged["baseline"]["commit"], base);
        assert_eq!(staged["files_checked"], 5);
        let findings = staged["findings"].as_array().unwrap();
        let names: Vec<_> = findings
            .iter()
            .map(|finding| finding["path"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["formerly compliant", "growing debt", "new file"]);
        assert_eq!(findings[0]["baseline_lines"], 2);
        assert_eq!(findings[1]["limit"], 2);
        assert_eq!(findings[1]["allowed_lines"], 4);
        assert_eq!(findings[1]["actual"], 5);
        assert!(findings[2]["baseline_lines"].is_null());

        assert_ok(
            &repo
                .command()
                .env("GIT_INDEX_FILE", index)
                .args(["commit", "-m", "alternate snapshot"])
                .output()
                .unwrap(),
        );
        let committed = report(check(&repo, &["--base", &base, "--head", "HEAD"], None), 1);
        assert_eq!(committed["scope"], "direct");
        assert_eq!(committed["selected"]["tree"], staged["selected"]["tree"]);
        assert_eq!(committed["findings"], staged["findings"]);
        assert_eq!(
            report(
                check(&repo, &["--merge-base", &base, "--head", "HEAD"], None),
                1
            )["baseline"],
            staged["baseline"],
        );
    }

    #[test]
    fn initial_policy_addition_and_selected_deletion_are_authoritative() {
        let repo = Repo::init("sha1");
        repo.write(".assura/config.yml", policy(2));
        repo.write("new", "a\nb\nc\n");
        repo.git(&["add", "-A"]);
        let initial = report(check(&repo, &["--staged"], None), 1);
        assert!(initial["baseline"]["commit"].is_null());
        assert_eq!(initial["findings"][0]["path"], "new");
        repo.commit("initial");
        repo.git(&["rm", "--cached", ".assura/config.yml"]);
        let missing = check(&repo, &["--staged"], None);
        assert_eq!(missing.status.code(), Some(4));
        assert!(String::from_utf8_lossy(&missing.stderr).contains("absent from selected tree"));
        assert_eq!(
            check(&repo, &["--base", "HEAD"], None).status.code(),
            Some(2)
        );
        assert_eq!(
            check(&repo, &["--base", "HEAD", "--head", "missing"], None)
                .status
                .code(),
            Some(3)
        );
    }

    #[cfg(unix)]
    #[test]
    fn advisory_finding_preserves_the_literal_git_path() {
        let repo = Repo::init("sha1");
        repo.write(
            ".assura/config.yml",
            "structure:\n  ./:\n    files:\n      max_lines: 2\n      severity: low\nexclude:\n  - .assura/**\n",
        );
        let path = ":path\\part\tline\n[*]";
        repo.write(path, "a\nb\nc\n");
        repo.git(&["add", "-A"]);
        let staged = report(check(&repo, &["--staged"], None), 0);
        assert_eq!(staged["success"], true);
        assert_eq!(staged["findings"][0]["path"], path);
        assert_eq!(staged["findings"][0]["blocking"], false);
    }

    #[cfg(unix)]
    #[test]
    fn older_git_fallback_only_accepts_a_missing_head_ref() {
        use std::os::unix::fs::PermissionsExt;

        let repo = Repo::init("sha1");
        repo.write(".assura/config.yml", policy(2));
        repo.git(&["add", "-A"]);

        // Model the older Git option parser while using real Git for ref lookups.
        let tools = tempfile::tempdir().unwrap();
        let shim = tools.path().join("git");
        fs::write(
            &shim,
            "#!/bin/sh\nif [ \"$2\" = show-ref ] && [ \"$3\" = --exists ]; then exit 129; fi\nPATH=$ASSURA_TEST_GIT_PATH exec git \"$@\"\n",
        ).unwrap();
        fs::set_permissions(&shim, fs::Permissions::from_mode(0o700)).unwrap();
        let check = || {
            Command::new(env!("CARGO_BIN_EXE_assura"))
                .current_dir(&repo.0)
                .env_remove("GIT_INDEX_FILE")
                .env("PATH", tools.path())
                .env("ASSURA_TEST_GIT_PATH", std::env::var_os("PATH").unwrap())
                .args(["check-change", "--staged", "--format", "json"])
                .output()
                .unwrap()
        };
        assert!(report(check(), 0)["baseline"]["commit"].is_null());
        repo.write(".git/refs/heads/main", "invalid ref\n");
        assert_eq!(check().status.code(), Some(3));
    }

    #[test]
    fn isolated_rename_can_inherit_debt_and_growth_still_fails() {
        let repo = Repo::init("sha1");
        repo.write(".assura/config.yml", policy(2));
        repo.write("old name", "line 1\nline 2\nline 3\nline 4\nline 5\n");
        repo.commit("base");
        repo.git(&["mv", "old name", "new name"]);
        let renamed = report(check(&repo, &["--staged"], None), 0);
        assert_eq!(renamed["files_checked"], 1);
        assert_eq!(renamed["findings"], serde_json::json!([]));

        repo.write(
            "new name",
            "line 1\nline 2\nline 3\nline 4\nline 5\nline 6\n",
        );
        repo.git(&["add", "--", "new name"]);
        let grown = report(check(&repo, &["--staged"], None), 1);
        assert_eq!(grown["findings"][0]["allowed_lines"], 5);
        assert_eq!(grown["findings"][0]["actual"], 6);

        repo.write(
            ".assura/config.yml",
            "structure:\n  ./:\n    files:\n      max_lines: 2\n      max_lines_patterns:\n        'new name': 3\nexclude:\n  - .assura/**\n",
        );
        repo.git(&["add", "--", ".assura/config.yml"]);
        let moved = report(check(&repo, &["--staged"], None), 1);
        assert_eq!(moved["findings"][0]["allowed_lines"], 3);
        assert!(moved["findings"][0]["baseline_lines"].is_null());
    }
}
