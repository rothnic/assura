use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn git(project: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(project)
        .env("GIT_AUTHOR_NAME", "Assura Test")
        .env("GIT_AUTHOR_EMAIL", "assura-test@example.com")
        .env("GIT_COMMITTER_NAME", "Assura Test")
        .env("GIT_COMMITTER_EMAIL", "assura-test@example.com")
        .output()
        .expect("git command runs");
    assert!(
        output.status.success(),
        "git {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

fn write(path: &Path, content: impl AsRef<[u8]>) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write fixture file");
}

fn fixture() -> TempDir {
    let project = tempfile::tempdir().expect("temp project");
    write(
        &project.path().join(".assura/config.yml"),
        "structure:\n  ./:\n    extra: true\nexclude:\n  - .assura/**\n",
    );
    write(
        &project.path().join(".gitignore"),
        ".assura/agent-sessions/\n",
    );
    write(&project.path().join("src/base.rs"), "fn base() {}\n");
    write(
        &project.path().join("src/remove-me.rs"),
        "fn remove_me() {}\n",
    );
    git(project.path(), &["init"]);
    git(project.path(), &["branch", "-M", "master"]);
    git(project.path(), &["add", "."]);
    git(project.path(), &["commit", "-m", "initial"]);
    project
}

fn inspect(project: &Path) -> Value {
    let output = Command::new(assura_bin())
        .args([
            "agent",
            "nudge",
            project.to_str().expect("project path"),
            "--delivery",
            "inspect",
            "--format",
            "json",
        ])
        .output()
        .expect("assura inspect runs");
    json_success(output)
}

fn json_success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("inspect emits JSON")
}

#[test]
fn explicit_inspect_reports_categories_and_reuses_the_snapshot() {
    let project = fixture();
    git(project.path(), &["switch", "-c", "feature/trajectory"]);
    git(
        project.path(),
        &["mv", "src/base.rs", "src/renamed-base.rs"],
    );
    fs::remove_file(project.path().join("src/remove-me.rs")).expect("delete tracked file");
    write(
        &project.path().join("src/feature.rs"),
        "fn feature() {\n    println!(\"feature\");\n}\n",
    );
    write(
        &project.path().join("tests/feature.rs"),
        "#[test]\nfn feature() {}\n",
    );
    write(
        &project.path().join(".trellis/trajectory.md"),
        "# coordination\n",
    );
    write(
        &project.path().join("dist/generated.js"),
        "console.log('generated');\n",
    );
    write(&project.path().join("src/binary.bin"), [0, 159, 146, 150]);
    git(project.path(), &["add", "."]);
    git(project.path(), &["commit", "-m", "trajectory changes"]);
    write(
        &project.path().join("src/feature.rs"),
        "fn feature() {\n    println!(\"feature\");\n    println!(\"dirty\");\n}\n",
    );
    write(&project.path().join("scratch.txt"), "untracked\n");
    git(project.path(), &["config", "diff.external", "false"]);
    git(project.path(), &["config", "diff.rs.textconv", "false"]);

    let first = inspect(project.path());
    let trajectory = &first["trajectory"];
    assert_eq!(first["delivery"], "inspect");
    assert_eq!(trajectory["schema"], "assura.agent-trajectory.v1");
    assert_eq!(trajectory["coverage"], "complete");
    assert_eq!(trajectory["window"]["kind"], "minutes");
    assert_eq!(trajectory["integration"]["resolved_ref"], "master");
    assert_eq!(trajectory["pending"]["commits"], 1);
    assert!(trajectory["pending"]["dirty_files"].as_u64().unwrap() >= 2);
    assert!(trajectory["pending"]["source"]["files"].as_u64().unwrap() >= 1);
    assert!(trajectory["pending"]["tests"]["files"].as_u64().unwrap() >= 1);
    assert!(
        trajectory["pending"]["coordination"]["files"]
            .as_u64()
            .unwrap()
            >= 1
    );
    assert!(
        trajectory["pending"]["generated"]["files"]
            .as_u64()
            .unwrap()
            >= 1
    );
    assert!(trajectory["pending"]["source"]["additions"].is_null());
    assert!(trajectory["pending"]["files"].as_u64().unwrap() >= 6);
    assert!(trajectory["pending"]["other"]["files"].as_u64().unwrap() >= 1);
    assert!(trajectory["pending"]["other"]["additions"].is_null());

    let second = inspect(project.path());
    assert_eq!(second["trajectory"]["generation"], trajectory["generation"]);
    assert_eq!(second["trajectory"]["cache"]["status"], "hit");
    assert_eq!(second["trajectory"]["freshness"]["snapshot"], "cache_hit");

    let automatic = Command::new(assura_bin())
        .args([
            "agent",
            "nudge",
            project.path().to_str().expect("project path"),
            "--format",
            "json",
        ])
        .output()
        .expect("automatic nudge runs");
    let automatic = json_success(automatic);
    assert_eq!(automatic["delivery"], "automatic");
    assert!(automatic.get("trajectory").is_none());
}

#[test]
fn missing_integration_ref_is_partial_and_detached_head_is_explicit() {
    let project = fixture();
    git(project.path(), &["branch", "-M", "trunk"]);
    let missing = inspect(project.path());
    assert_eq!(missing["trajectory"]["coverage"], "partial");
    assert_eq!(
        missing["trajectory"]["integration"]["requested_ref"],
        "origin/master"
    );
    assert!(missing["trajectory"]["integration"]["sha"].is_null());
    assert!(missing["trajectory"]["coverage_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason == "integration_ref_missing"));
    assert!(missing["trajectory"].get("pending").is_none());

    git(project.path(), &["switch", "--detach", "HEAD"]);
    let detached = inspect(project.path());
    assert!(detached["trajectory"]["worktree"].get("branch").is_none());
    assert_eq!(detached["trajectory"]["coverage"], "partial");
    assert!(detached["trajectory"]["coverage_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason == "integration_ref_missing"));
}

#[test]
fn integration_ref_changes_are_not_reported_as_zero_progress() {
    let project = fixture();
    git(project.path(), &["switch", "-c", "feature/trajectory"]);
    write(&project.path().join("src/feature.rs"), "fn feature() {}\n");
    git(project.path(), &["add", "."]);
    git(project.path(), &["commit", "-m", "feature"]);
    let first = inspect(project.path());
    let first_sha = first["trajectory"]["integration"]["sha"].clone();

    git(project.path(), &["switch", "master"]);
    write(
        &project.path().join("src/base.rs"),
        "fn base() { /* updated */ }\n",
    );
    git(project.path(), &["add", "."]);
    git(project.path(), &["commit", "-m", "move integration ref"]);
    let second_sha = git(project.path(), &["rev-parse", "master"]);
    assert_ne!(first_sha.as_str(), Some(second_sha.as_str()));
    git(project.path(), &["switch", "feature/trajectory"]);

    let changed = inspect(project.path());
    assert_eq!(changed["trajectory"]["integration"]["sha"], second_sha);
    assert_eq!(
        changed["trajectory"]["freshness"]["integration_ref"],
        "changed_local"
    );
    assert!(changed["trajectory"]["coverage_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason == "integration_ref_changed"));
    assert!(changed["trajectory"]["pending"]["commits"]
        .as_u64()
        .is_some());
}

#[test]
fn resolved_integration_ref_changes_refreshes_provenance() {
    let project = fixture();
    let master_sha = git(project.path(), &["rev-parse", "master"]);
    git(
        project.path(),
        &["update-ref", "refs/remotes/origin/master", &master_sha],
    );

    let remote_snapshot = inspect(project.path());
    assert_eq!(
        remote_snapshot["trajectory"]["integration"]["resolved_ref"],
        "origin/master"
    );

    git(
        project.path(),
        &["update-ref", "-d", "refs/remotes/origin/master"],
    );
    let local_snapshot = inspect(project.path());
    assert_eq!(
        local_snapshot["trajectory"]["integration"]["resolved_ref"],
        "master"
    );
    assert_eq!(local_snapshot["trajectory"]["cache"]["status"], "refreshed");
}

#[test]
fn merge_and_squash_reconciliation_do_not_create_false_pending_work() {
    let merged = fixture();
    git(merged.path(), &["switch", "-c", "candidate"]);
    write(&merged.path().join("src/merged.rs"), "fn merged() {}\n");
    git(merged.path(), &["add", "."]);
    git(merged.path(), &["commit", "-m", "candidate"]);
    git(merged.path(), &["switch", "master"]);
    git(
        merged.path(),
        &["merge", "--no-ff", "candidate", "-m", "merge candidate"],
    );
    let merged_snapshot = inspect(merged.path());
    assert_eq!(merged_snapshot["trajectory"]["pending"]["commits"], 0);
    assert_eq!(merged_snapshot["trajectory"]["pending"]["files"], 0);

    let squashed = fixture();
    git(squashed.path(), &["switch", "-c", "squashed-candidate"]);
    write(
        &squashed.path().join("src/squashed.rs"),
        "fn squashed() {}\n",
    );
    git(squashed.path(), &["add", "."]);
    git(squashed.path(), &["commit", "-m", "squashed candidate"]);
    git(squashed.path(), &["switch", "master"]);
    git(
        squashed.path(),
        &["merge", "--squash", "squashed-candidate"],
    );
    git(squashed.path(), &["commit", "-m", "squash candidate"]);
    let squashed_snapshot = inspect(squashed.path());
    assert_eq!(squashed_snapshot["trajectory"]["pending"]["commits"], 0);
    assert_eq!(squashed_snapshot["trajectory"]["pending"]["files"], 0);
    git(squashed.path(), &["switch", "squashed-candidate"]);
    let retained_branch_snapshot = inspect(squashed.path());
    assert_eq!(
        retained_branch_snapshot["trajectory"]["pending"]["commits"],
        0
    );
    assert_eq!(
        retained_branch_snapshot["trajectory"]["pending"]["files"],
        0
    );
}

#[test]
fn shallow_clones_and_multiple_worktrees_keep_coverage_honest() {
    let source = fixture();
    let clone = tempfile::tempdir().expect("clone destination");
    let source_url = format!("file://{}", source.path().display());
    let clone_output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            &source_url,
            clone.path().to_str().expect("clone path"),
        ])
        .output()
        .expect("shallow clone runs");
    assert!(clone_output.status.success());
    let shallow = inspect(clone.path());
    assert_eq!(shallow["trajectory"]["coverage"], "partial");
    assert!(shallow["trajectory"]["coverage_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason == "shallow_history"));

    let worktree = tempfile::tempdir().expect("worktree destination");
    let worktree_path = worktree.path().to_path_buf();
    fs::remove_dir(&worktree_path).expect("remove empty worktree destination");
    git(
        source.path(),
        &[
            "worktree",
            "add",
            "--detach",
            worktree_path.to_str().expect("worktree path"),
            "master",
        ],
    );
    let primary = inspect(source.path());
    let secondary = inspect(&worktree_path);
    assert_ne!(
        primary["trajectory"]["worktree"]["git_dir"],
        secondary["trajectory"]["worktree"]["git_dir"]
    );
    assert_eq!(secondary["trajectory"]["cache"]["status"], "refreshed");
    git(
        source.path(),
        &[
            "worktree",
            "remove",
            "--force",
            worktree_path.to_str().unwrap(),
        ],
    );
}

#[test]
fn corrupt_snapshot_is_replaced_without_inventing_facts() {
    let project = fixture();
    let first = inspect(project.path());
    assert_eq!(first["trajectory"]["cache"]["status"], "refreshed");
    let cache_dir = project.path().join(".git/assura/trajectory");
    let cache_file = fs::read_dir(&cache_dir)
        .expect("trajectory cache directory")
        .map(|entry| entry.expect("cache entry").path())
        .find(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .expect("trajectory cache file");
    fs::write(&cache_file, b"not-json").expect("corrupt cache");
    let repaired = inspect(project.path());
    assert_eq!(repaired["trajectory"]["cache"]["source"], "corrupt");
    assert_eq!(repaired["trajectory"]["coverage"], "complete");
    assert!(repaired["trajectory"]["integrated"]["commits"]
        .as_u64()
        .is_some());

    let schema_mismatch = fs::read_to_string(&cache_file)
        .expect("read cache")
        .replace("assura.agent-trajectory.v1", "assura.agent-trajectory.old");
    fs::write(&cache_file, schema_mismatch).expect("write schema mismatch");
    let schema_mismatch = inspect(project.path());
    assert_eq!(
        schema_mismatch["trajectory"]["cache"]["source"],
        "schema_mismatch"
    );

    fs::write(&cache_file, vec![b'x'; 1024 * 1024 + 1]).expect("write oversized cache");
    let oversized = inspect(project.path());
    assert_eq!(oversized["trajectory"]["cache"]["source"], "oversized");
}

#[test]
fn truncated_status_does_not_invent_a_zero_dirty_file_count() {
    let project = fixture();
    for index in 0..12_000 {
        write(
            &project
                .path()
                .join(format!("untracked-{index:05}-file.txt")),
            "untracked\n",
        );
    }

    let snapshot = inspect(project.path());
    assert_eq!(snapshot["trajectory"]["coverage"], "partial");
    assert!(snapshot["trajectory"]["coverage_reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|reason| reason == "status_output_truncated"));
    assert!(snapshot["trajectory"]["pending"]["dirty_files"].is_null());
}
