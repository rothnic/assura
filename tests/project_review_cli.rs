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

fn run_review(args: &[&str]) -> Output {
    let mut command = vec!["review"];
    command.extend_from_slice(args);
    run_assura(&command)
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

    assert_eq!(review["schema"], "assura.project-review.v1");
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
    assert_eq!(agent["schema"], "assura.project-review.agent.v1");
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
fn review_exits_nonzero_for_required_file_mismatch() {
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
    assert_eq!(output.status.code(), Some(1));
    let review = json_from_output(&output);

    assert_eq!(review["status"], "fail");
    assert_eq!(review["summary"]["blocking"], 1);
    let blocking = finding(&review, "blocking:exists_count");
    assert_eq!(blocking["category"], "structure");
    assert_eq!(blocking["action_kind"], "fix-now");
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
    assert_eq!(output.status.code(), Some(1));
    let review = json_from_output(&output);

    let blocking = finding(&review, "blocking:unexpected_directory");
    assert_eq!(blocking["category"], "structure");
    assert_eq!(
        finding(&review, "structure-fit:inspect-before-changing")["action_kind"],
        "inspect-before-changing"
    );

    let text = run_review(&[project.path().to_str().unwrap(), "--format", "text"]);
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("assura explain <path> --format json"));
    assert!(stdout.contains(".assura/config.yml"));

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
    assert_eq!(output.status.code(), Some(1));
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
    run_git(&project, &["checkout", "-b", "feature/heat"]);
    fs::write(project.path().join("docs/branch.md"), "# Branch doc\n").expect("branch doc");
    run_git(&project, &["add", "docs/branch.md"]);
    run_git(&project, &["commit", "-m", "branch doc"]);

    fs::write(
        project.path().join("src/good.rs"),
        "fn good() {}\nfn changed() {}\n",
    )
    .expect("modify good rs");
    fs::write(project.path().join("src/BadName.rs"), "fn bad_name() {}\n").expect("bad rs");

    let output = run_review(&[project.path().to_str().unwrap(), "--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    let review = json_from_output(&output);

    assert_eq!(review["heatmap"]["git_available"], true);
    assert_eq!(review["heatmap"]["branch"]["name"], "feature/heat");
    assert_eq!(review["heatmap"]["branch"]["commits_on_branch"], 1);
    assert_eq!(review["heatmap"]["totals"]["untracked_files"], 1);
    assert_eq!(review["heatmap"]["totals"]["modified_files"], 1);
    assert_eq!(review["heatmap"]["totals"]["branch_changed_files"], 1);
    assert_eq!(review["heatmap"]["totals"]["validation_violations"], 1);
    assert_eq!(review["heatmap"]["totals"]["naming_violations"], 1);

    let hot_dirs = review["heatmap"]["hot_dirs"].as_array().expect("hot dirs");
    let src_heat = hot_dirs
        .iter()
        .find(|dir| dir["path"] == "src")
        .unwrap_or_else(|| panic!("missing src heat: {review:#}"));
    assert_eq!(src_heat["validation_violations"], 1);
    assert_eq!(src_heat["naming_violations"], 1);
    assert_eq!(src_heat["untracked_files"], 1);
    assert_eq!(src_heat["modified_files"], 1);

    let text = run_review(&[project.path().to_str().unwrap(), "--format", "text"]);
    let stdout = String::from_utf8_lossy(&text.stdout);
    assert!(stdout.contains("heat: !1 chg=1 ?1 branch_files=1 commits=1"));
    assert!(stdout.contains("hot: src !1 chg=1 ?1"));
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
    assert_eq!(review["heatmap"]["totals"]["branch_changed_files"], 0);
    assert!(
        review["heatmap"]["hot_dirs"]
            .as_array()
            .expect("hot dirs")
            .is_empty(),
        "nested project heat should ignore sibling repo changes: {review:#}"
    );
}
