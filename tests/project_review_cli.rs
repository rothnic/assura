use serde_json::Value;
use std::fs;
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
