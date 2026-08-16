use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    let _ = env!("CARGO_BIN_EXE_assura-full");
    env!("CARGO_BIN_EXE_assura")
}

fn run_assura(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(args)
        .output()
        .expect("assura command runs")
}

fn run_assura_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(assura_bin());
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("assura command runs")
}

fn run_review(args: &[&str]) -> Output {
    let mut command = vec!["review"];
    command.extend_from_slice(args);
    run_assura(&command)
}

fn run_review_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = vec!["review"];
    command.extend_from_slice(args);
    run_assura_with_env(&command, envs)
}

fn run_git(project: &TempDir, args: &[&str]) {
    run_git_path(project.path(), args);
}

fn run_git_path(project: &Path, args: &[&str]) {
    let output = Command::new("git")
        .arg("-C")
        .arg(project)
        .args(args)
        .output()
        .expect("git command runs");
    assert!(
        output.status.success(),
        "git {:?}\nstdout:\n{}\nstderr:\n{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn json_from_success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    json_from_output(&output)
}

fn json_from_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "command emits JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn write_config(project: &TempDir, config: &str) {
    fs::create_dir_all(project.path().join(".assura")).expect("create assura dir");
    fs::write(project.path().join(".assura/config.yml"), config).expect("write config");
}

fn finding<'a>(review: &'a Value, id: &str) -> &'a Value {
    review["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|finding| finding["id"] == id)
        .unwrap_or_else(|| panic!("missing finding {id}: {review:#}"))
}

#[test]
fn review_clean_repo_reports_inactive_guidance_without_blocking() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
exclude:
  - .assura/**
"#,
    );

    let review = json_from_success(run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));

    assert_eq!(review["schema"], "assura.project-review.v2");
    assert_eq!(review["structure"]["status"], "pass");
    assert_eq!(review["summary"]["blocking"], 0);
    assert!(review["summary"]["inactive"].as_u64().expect("inactive") > 0);
    assert_eq!(
        finding(&review, "inactive:onboarding_packet")["severity"],
        "inactive"
    );

    let agent = json_from_success(run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "agent",
    ]));
    assert_eq!(agent["schema"], "assura.project-review.agent.v2");
    assert!(agent["findings"].as_array().expect("agent findings").len() <= 12);
    assert!(
        agent["omitted_noise"]
            .as_array()
            .expect("agent omitted noise")
            .len()
            <= 4
    );
    assert_eq!(agent["heatmap"]["git_available"], false);
}

#[test]
fn review_reports_blockers_without_becoming_the_policy_gate() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      exists:
        README.md: 1
exclude:
  - .assura/**
"#,
    );

    let output = run_review(&[project.path().to_str().unwrap(), "--format", "json"]);
    assert_eq!(output.status.code(), Some(0));
    let review = json_from_output(&output);

    assert_eq!(review["status"], "fail");
    assert_eq!(review["summary"]["blocking"], 1);
    let blocking = finding(&review, "blocking:exists_count");
    assert_eq!(blocking["category"], "structure");
    assert_eq!(blocking["action_kind"], "fix-now");
}

#[test]
fn review_keeps_distinct_fingerprints_for_multiple_targets_of_one_rule() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      exists:
        README.md: 1
        LICENSE: 1
exclude:
  - .assura/**
"#,
    );

    let review = json_from_success(run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    let fingerprints = review["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .filter(|item| item["id"] == "blocking:exists_count")
        .map(|item| item["fingerprint"].as_str().expect("fingerprint"))
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(fingerprints.len(), 2, "{review:#}");
}

#[test]
fn review_reports_unmodeled_path_pressure_with_structure_fit_guidance() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    directories:
      allowed_names:
        - src
      allow_extra: false
exclude:
  - .assura/**
"#,
    );
    fs::create_dir_all(project.path().join("src")).expect("src dir");
    fs::create_dir_all(project.path().join("experiments")).expect("experiments dir");

    let output = run_review(&[project.path().to_str().unwrap(), "--format", "json"]);
    assert_eq!(output.status.code(), Some(0));
    let review = json_from_output(&output);

    let blocking = finding(&review, "blocking:unexpected_directory");
    assert_eq!(blocking["category"], "structure");
    assert_eq!(
        finding(&review, "structure-fit:inspect-before-changing")["action_kind"],
        "inspect-before-changing"
    );

    let text = run_review(&[project.path().to_str().unwrap(), "--format", "text"]);
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("Fix first"));
    assert!(stdout.contains("experiments"));
    assert!(stdout.contains("assura check --format agent ."));
    assert!(!stdout.contains("\x1b["));

    let verbose = run_review(&[
        "--verbose",
        project.path().to_str().unwrap(),
        "--format",
        "text",
    ]);
    let verbose_stdout = String::from_utf8_lossy(&verbose.stdout);
    assert!(verbose_stdout.contains("assura explain <path> --format json"));
    assert!(verbose_stdout.contains(".assura/config.yml"));

    let experiment_heat = review["heatmap"]["hot_dirs"]
        .as_array()
        .expect("hot dirs")
        .iter()
        .find(|dir| dir["path"] == "experiments")
        .unwrap_or_else(|| panic!("missing experiments heat: {review:#}"));
    assert_eq!(experiment_heat["validation_violations"], 1);
}

#[test]
fn review_classifies_noisy_reference_gaps_as_informational() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
exclude:
  - .assura/**
"#,
    );
    fs::create_dir_all(project.path().join("docs/generated")).expect("generated docs");
    fs::write(
        project.path().join("docs/note.md"),
        "# Note\n\nGenerated link: [missing](generated/missing.md)\n",
    )
    .expect("note");

    let review = json_from_success(run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));

    assert_eq!(
        review["content_gaps"]["unresolved_repository_references"],
        1
    );
    let references = finding(&review, "content:unresolved_repository_references");
    assert_eq!(references["severity"], "informational");
    assert_eq!(references["action_kind"], "informational");
    assert!(review["omitted_noise"]
        .as_array()
        .expect("omitted noise")
        .iter()
        .any(|item| item["category"] == "generated"));
}

#[test]
fn review_reports_actionable_content_gap_from_content_runtime_fixture() {
    let output = run_review(&[
        "tests/fixtures/content_runtime/missing_reference",
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(0));
    let review = json_from_output(&output);

    assert_eq!(review["content_gaps"]["diagnostics"], 1);
    assert_eq!(review["content_gaps"]["missing_relations"], 1);
    assert_eq!(
        finding(&review, "blocking:content_runtime:missing_reference")["category"],
        "content"
    );
    let content = finding(&review, "content:diagnostics");
    assert_eq!(content["action_kind"], "fix-now");
    assert_eq!(
        content["command"],
        "assura content agent-query diagnostics --format json ."
    );
}

#[test]
fn review_heatmap_rolls_up_git_and_validation_signals_by_directory() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
    children:
      src/:
        files:
          naming: kebab-case
          extensions:
            - rs
exclude:
  - .assura/**
  - .git/**
"#,
    );
    fs::create_dir_all(project.path().join("src")).expect("src dir");
    fs::create_dir_all(project.path().join("docs")).expect("docs dir");
    fs::write(project.path().join("src/good.rs"), "fn good() {}\n").expect("good rs");

    run_git(&project, &["init"]);
    run_git(&project, &["config", "user.email", "assura@example.test"]);
    run_git(&project, &["config", "user.name", "Assura Test"]);
    run_git(&project, &["branch", "-M", "main"]);
    run_git(&project, &["add", "."]);
    run_git(&project, &["commit", "-m", "initial"]);
    let remote = TempDir::new().expect("temp remote");
    run_git_path(remote.path(), &["init", "--bare"]);
    run_git(
        &project,
        &["remote", "add", "origin", remote.path().to_str().unwrap()],
    );
    run_git(&project, &["push", "-u", "origin", "main"]);
    run_git(&project, &["checkout", "-b", "feature/heat"]);
    fs::write(project.path().join("docs/branch.md"), "# Branch doc\n").expect("branch doc");
    run_git(&project, &["add", "docs/branch.md"]);
    run_git(&project, &["commit", "-m", "branch doc"]);
    run_git(&project, &["push", "-u", "origin", "feature/heat"]);

    fs::write(
        project.path().join("src/good.rs"),
        "fn good() {}\nfn changed() {}\n",
    )
    .expect("modify good rs");
    fs::write(project.path().join("src/BadName.rs"), "fn bad_name() {}\n").expect("bad rs");

    let output = run_review(&[project.path().to_str().unwrap(), "--format", "json"]);
    assert_eq!(output.status.code(), Some(0));
    let review = json_from_output(&output);

    assert_eq!(review["heatmap"]["git_available"], true);
    assert_eq!(review["heatmap"]["branch"]["name"], "feature/heat");
    assert_eq!(review["heatmap"]["branch"]["base"], "origin/main");
    assert_eq!(
        review["heatmap"]["branch"]["upstream"],
        "origin/feature/heat"
    );
    assert_eq!(review["heatmap"]["branch"]["commits_on_branch"], 1);
    assert_eq!(review["heatmap"]["totals"]["untracked_files"], 1);
    assert_eq!(review["heatmap"]["totals"]["modified_files"], 1);
    assert_eq!(review["heatmap"]["totals"]["branch_changed_files"], 1);
    assert_eq!(review["heatmap"]["totals"]["branch_line_additions"], 1);
    assert_eq!(review["heatmap"]["totals"]["branch_line_deletions"], 0);
    assert_eq!(review["heatmap"]["totals"]["worktree_line_additions"], 1);
    assert_eq!(review["heatmap"]["totals"]["worktree_line_deletions"], 0);
    assert_eq!(review["heatmap"]["totals"]["line_additions"], 1);
    assert_eq!(review["heatmap"]["totals"]["line_deletions"], 0);
    assert_eq!(review["heatmap"]["totals"]["validation_violations"], 1);
    assert_eq!(review["heatmap"]["totals"]["naming_violations"], 1);
    assert!(review["heatmap"]["hot_dirs"]
        .as_array()
        .expect("hot dirs")
        .iter()
        .all(|dir| dir.get("score").is_none()));
    assert_eq!(review["heatmap"]["thresholds"]["worktree_files"], 10);
    assert_eq!(review["heatmap"]["thresholds"]["line_churn"], 1_000);

    let hot_dirs = review["heatmap"]["hot_dirs"].as_array().expect("hot dirs");
    let src_heat = hot_dirs
        .iter()
        .find(|dir| dir["path"] == "src")
        .unwrap_or_else(|| panic!("missing src heat: {review:#}"));
    assert_eq!(src_heat["validation_violations"], 1);
    assert_eq!(src_heat["naming_violations"], 1);
    assert_eq!(src_heat["untracked_files"], 1);
    assert_eq!(src_heat["modified_files"], 1);
    assert_eq!(src_heat["worktree_line_additions"], 1);
    let docs_heat = hot_dirs
        .iter()
        .find(|dir| dir["path"] == "docs")
        .unwrap_or_else(|| panic!("missing docs heat: {review:#}"));
    assert_eq!(docs_heat["branch_changed_files"], 1);
    assert_eq!(docs_heat["branch_line_additions"], 1);

    let text = run_review(&[project.path().to_str().unwrap(), "--format", "text"]);
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.starts_with("Assura review\n\n"));
    assert!(stdout.contains("Status     needs attention"));
    assert!(stdout.contains("Scope      feature/heat -> origin/main"));
    assert!(stdout.contains("Findings   blocking=1"));
    assert!(stdout.contains("Branch"));
    assert!(stdout.contains("feature/heat files=1 lines=+1/-0 commits=1"));
    assert!(stdout.contains("Worktree"));
    assert!(stdout.contains("files=2 modified=1 untracked=1 lines=+1/-0"));
    assert!(stdout.contains("Watch      blocking-validation=1/1"));
    assert!(stdout.contains("Hot path   src violations=1 changed=2 lines=+1/-0"));
    assert!(stdout.contains("Fix first"));
    assert!(stdout.contains("BadName.rs"));
    assert!(stdout.contains("Next"));
    assert!(stdout.contains("Run        assura check --format agent ."));
    assert!(!stdout.contains("Check      "));
    assert!(!stdout.contains("Heat       "));
    assert!(!stdout.contains("Content    "));
    assert!(!stdout.contains("Configure  "));
    assert!(!stdout.contains("Inspect    "));
    assert!(!stdout.contains("Policy     "));
    assert!(!stdout.contains("Details    "));
    assert!(!stdout.contains("worktree=2/10"));

    let verbose = run_review(&[
        "--verbose",
        project.path().to_str().unwrap(),
        "--format",
        "text",
    ]);
    let verbose_stdout = String::from_utf8_lossy(&verbose.stdout);
    assert!(verbose_stdout.contains("Diagnostics"));
    assert!(verbose_stdout.contains("Check      "));
    assert!(verbose_stdout.contains("Heat       !1 hot_dirs=2 risks=1"));
    assert!(verbose_stdout.contains("Thresholds"));
    assert!(verbose_stdout.contains("worktree=2/10"));
    assert!(verbose_stdout.contains("Hot dirs"));
    assert!(verbose_stdout.contains("|- docs v=0 files=b1/m0/u0 lines=b+1/-0,w+0/-0"));
    assert!(verbose_stdout.contains("`- src v=1 files=b0/m1/u1 lines=b+0/-0,w+1/-0 blocking=1"));
    assert!(verbose_stdout.contains("Details    "));

    let explicit = run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "json",
        "--base",
        "main",
    ]);
    let explicit_review = json_from_output(&explicit);
    assert_eq!(explicit_review["heatmap"]["branch"]["base"], "main");
    assert_eq!(
        explicit_review["heatmap"]["totals"]["branch_changed_files"],
        1
    );
}

#[test]
fn review_bounds_agent_output_and_renders_nested_hot_directories_as_a_tree() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
exclude:
  - .assura/**
  - .git/**
        "#,
    );
    fs::create_dir_all(project.path().join("src/cli")).expect("nested source directory");
    fs::write(project.path().join("src/cli/change.txt"), "baseline\n").expect("baseline");
    fs::write(project.path().join("src/cli/second.txt"), "baseline\n").expect("baseline");
    run_git(&project, &["init"]);
    run_git(&project, &["config", "user.email", "assura@example.test"]);
    run_git(&project, &["config", "user.name", "Assura Test"]);
    run_git(&project, &["add", "."]);
    run_git(&project, &["commit", "-m", "initial"]);

    for index in 0..8 {
        let directory = if index == 0 {
            project.path().join("src/cli")
        } else {
            project.path().join(format!("area-{index}"))
        };
        fs::create_dir_all(&directory).expect("hot directory");
        fs::write(directory.join("change.txt"), "change\n").expect("untracked change");
        if index == 0 {
            fs::write(directory.join("second.txt"), "change\n").expect("second change");
        }
    }

    let agent = json_from_success(run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "agent",
    ]));
    assert!(agent["findings"].as_array().expect("findings").len() <= 12);
    assert!(
        agent["heatmap"]["hot_dirs"]
            .as_array()
            .expect("hot dirs")
            .len()
            <= 5
    );
    assert!(
        agent["next_actions"]
            .as_array()
            .expect("next actions")
            .len()
            <= 6
    );

    let text = run_review(&[project.path().to_str().unwrap(), "--format", "text"]);
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("Hot path   src/cli"), "{stdout}");

    let verbose = run_review(&[
        "--verbose",
        project.path().to_str().unwrap(),
        "--format",
        "text",
    ]);
    let verbose_stdout = String::from_utf8_lossy(&verbose.stdout);
    assert!(verbose_stdout.contains("|- src"), "{verbose_stdout}");
    assert!(verbose_stdout.contains("|  `- cli"), "{verbose_stdout}");
}

#[test]
fn review_rejects_an_invalid_explicit_base() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
exclude:
  - .assura/**
  - .git/**
"#,
    );
    run_git(&project, &["init"]);
    run_git(&project, &["config", "user.email", "assura@example.test"]);
    run_git(&project, &["config", "user.name", "Assura Test"]);
    run_git(&project, &["add", "."]);
    run_git(&project, &["commit", "-m", "initial"]);

    let output = run_review(&[project.path().to_str().unwrap(), "--base", "missing-ref"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("review base `missing-ref` is not a valid Git commit ref"));
}

#[test]
fn review_rejects_an_explicit_base_without_a_common_ancestor() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
exclude:
  - .assura/**
  - .git/**
"#,
    );
    run_git(&project, &["init"]);
    run_git(&project, &["config", "user.email", "assura@example.test"]);
    run_git(&project, &["config", "user.name", "Assura Test"]);
    run_git(&project, &["add", "."]);
    run_git(&project, &["commit", "-m", "initial"]);

    let tree = Command::new("git")
        .arg("-C")
        .arg(project.path())
        .args(["mktree"])
        .output()
        .expect("mktree");
    assert!(tree.status.success());
    let tree = String::from_utf8_lossy(&tree.stdout).trim().to_string();
    let orphan = Command::new("git")
        .arg("-C")
        .arg(project.path())
        .env("GIT_AUTHOR_NAME", "Assura Test")
        .env("GIT_AUTHOR_EMAIL", "assura@example.test")
        .env("GIT_COMMITTER_NAME", "Assura Test")
        .env("GIT_COMMITTER_EMAIL", "assura@example.test")
        .args(["commit-tree", &tree, "-m", "orphan"])
        .output()
        .expect("commit-tree");
    assert!(orphan.status.success());
    let orphan = String::from_utf8_lossy(&orphan.stdout).trim().to_string();
    run_git(&project, &["update-ref", "refs/heads/orphan", &orphan]);

    let output = run_review(&[project.path().to_str().unwrap(), "--base", "orphan"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("has no common ancestor with HEAD"));
}

#[test]
fn review_fingerprints_and_tracks_finding_state_without_dirtying_the_repo() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    files:
      exists:
        README.md: 1
exclude:
  - .assura/**
  - .git/**
"#,
    );
    run_git(&project, &["init"]);
    run_git(&project, &["config", "user.email", "assura@example.test"]);
    run_git(&project, &["config", "user.name", "Assura Test"]);
    run_git(&project, &["add", "."]);
    run_git(&project, &["commit", "-m", "initial"]);

    let first = json_from_output(&run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    let first_blocking = finding(&first, "blocking:exists_count");
    let fingerprint = first_blocking["fingerprint"].as_str().expect("fingerprint");
    assert_eq!(fingerprint.len(), 64);
    assert_eq!(first_blocking["state"], "new");
    assert_eq!(first["finding_history"]["cache"]["mode"], "git-worktree");

    let second = json_from_output(&run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(
        finding(&second, "blocking:exists_count")["fingerprint"],
        fingerprint
    );
    assert_eq!(
        finding(&second, "blocking:exists_count")["state"],
        "unchanged"
    );
    assert!(
        second["finding_history"]["unchanged"]
            .as_u64()
            .expect("unchanged")
            > 0
    );
    let second_text = run_review(&[project.path().to_str().unwrap(), "--format", "text"]);
    let second_stdout = String::from_utf8_lossy(&second_text.stdout);
    assert!(second_stdout.contains("Fix first"));
    assert!(second_stdout.contains("README.md"));
    let second_agent = json_from_output(&run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "agent",
    ]));
    assert_eq!(
        finding(&second_agent, "blocking:exists_count")["state"],
        "unchanged"
    );

    fs::write(project.path().join("README.md"), "# Project\n").expect("README");
    let third = json_from_success(run_review(&[
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]));
    assert_eq!(
        finding(&third, "blocking:exists_count")["state"],
        "resolved"
    );
    assert_eq!(third["finding_history"]["resolved"], 1);
    assert!(!project.path().join(".assura/cache").exists());
}

#[test]
fn review_heatmap_scopes_git_signals_to_nested_project_root() {
    let repo = TempDir::new().expect("temp repo");
    let project = repo.path().join("workspace/project");
    fs::create_dir_all(project.join(".assura")).expect("assura dir");
    fs::create_dir_all(project.join("src")).expect("src dir");
    fs::write(
        project.join(".assura/config.yml"),
        r#"
structure:
  ./:
    extra: true
exclude:
  - .assura/**
"#,
    )
    .expect("write config");
    fs::write(project.join("src/good.rs"), "fn good() {}\n").expect("good rs");
    fs::write(repo.path().join("outside.txt"), "outside\n").expect("outside");

    run_git(&repo, &["init"]);
    run_git(&repo, &["config", "user.email", "assura@example.test"]);
    run_git(&repo, &["config", "user.name", "Assura Test"]);
    run_git(&repo, &["branch", "-M", "main"]);
    run_git(&repo, &["add", "."]);
    run_git(&repo, &["commit", "-m", "initial"]);

    fs::write(repo.path().join("outside.txt"), "outside\nchanged\n").expect("outside changed");
    fs::write(repo.path().join("outside-new.txt"), "new outside\n").expect("outside new");

    let review = json_from_success(run_review(&[project.to_str().unwrap(), "--format", "json"]));

    assert_eq!(review["heatmap"]["git_available"], true);
    assert_eq!(review["heatmap"]["totals"]["modified_files"], 0);
    assert_eq!(review["heatmap"]["totals"]["untracked_files"], 0);
    assert_eq!(review["heatmap"]["totals"]["line_additions"], 0);
    assert_eq!(review["heatmap"]["totals"]["branch_line_additions"], 0);
    assert_eq!(review["heatmap"]["totals"]["worktree_line_additions"], 0);
    assert_eq!(review["heatmap"]["totals"]["branch_changed_files"], 0);
    assert!(
        review["heatmap"]["hot_dirs"]
            .as_array()
            .expect("hot dirs")
            .is_empty(),
        "nested project heat should ignore sibling repo changes: {review:#}"
    );

    let sibling = repo.path().join("workspace/sibling");
    fs::create_dir_all(sibling.join(".assura")).expect("sibling assura dir");
    fs::write(
        sibling.join(".assura/config.yml"),
        "structure:\n  ./:\n    extra: true\nexclude:\n  - .assura/**\n",
    )
    .expect("sibling config");
    let sibling_review =
        json_from_success(run_review(&[sibling.to_str().unwrap(), "--format", "json"]));
    assert_ne!(
        review["finding_history"]["cache"]["path"],
        sibling_review["finding_history"]["cache"]["path"],
        "nested projects in one worktree require separate history namespaces"
    );
}

#[test]
fn review_text_output_supports_forced_ansi_color() {
    let project = TempDir::new().expect("temp project");
    write_config(
        &project,
        r#"
structure:
  ./:
    extra: true
exclude:
  - .assura/**
"#,
    );

    let text = run_review_with_env(
        &[project.path().to_str().unwrap(), "--format", "text"],
        &[("ASSURA_FORCE_COLOR", "1")],
    );
    assert!(text.status.success());
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("\x1b["));
    assert!(stdout.contains("Assura review"));
    assert!(stdout.contains("needs attention"));
    assert!(stdout.contains("Status"));
    assert!(stdout.contains("Findings"));
    assert!(!stdout.contains("Details"));
}
