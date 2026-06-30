use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

const VALID_ROOT: &str = "tests/fixtures/project_intelligence_real_repo/beacon_crm/valid";
const INVALID_ROOT: &str = "tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid";

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn run(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(args)
        .output()
        .expect("assura command runs")
}

fn json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "stdout parses as JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn add_trailing_spaces_after_heading(markdown: &str) -> String {
    let newline = if markdown.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let marker = format!("# Checkout Onboarding{newline}{newline}");
    let replacement = format!("# Checkout Onboarding{newline}   {newline}");
    markdown.replace(&marker, &replacement)
}

#[test]
fn beacon_crm_valid_fixture_passes_check_and_project_intelligence_queries() {
    let check = run(&["check", "--format", "json", VALID_ROOT]);
    assert_success(&check);
    let check_json = json_output(&check);
    assert_eq!(check_json["success"], true);
    assert!(check_json["violations"].as_array().unwrap().is_empty());

    let search = run(&[
        "content",
        "search",
        "checkout onboarding",
        VALID_ROOT,
        "--format",
        "json",
    ]);
    assert_success(&search);
    let search_json = json_output(&search);
    let matches = search_json["matches"].as_array().expect("matches array");
    assert!(matches.iter().any(|item| {
        item["source_kind"] == "model_instance"
            && item["collection"] == "epics"
            && item["instance_id"] == "epic-checkout"
    }));
    assert!(matches.iter().any(|item| {
        item["source_kind"] == "markdown_section" && item["path"] == "docs/epics/epic_checkout.md"
    }));

    let expanded = run(&[
        "content",
        "expand",
        "epics",
        "epic-checkout",
        VALID_ROOT,
        "--format",
        "json",
    ]);
    assert_success(&expanded);
    let expanded_json = json_output(&expanded);
    let related = expanded_json["related"].as_array().expect("related array");
    assert!(related.iter().any(|item| {
        item["relationship"] == "outgoing_relation"
            && item["path"] == "docs/decisions/adr_ui_boundary.json"
    }));
    assert!(related.iter().any(|item| {
        item["relationship"] == "outgoing_relation"
            && item["path"] == "packages/ui/package.assura.json"
    }));

    let agent = run(&[
        "content",
        "agent-query",
        "graph-expand",
        VALID_ROOT,
        "--collection",
        "epics",
        "--id",
        "epic-checkout",
        "--format",
        "json",
    ]);
    assert_success(&agent);
    let agent_json = json_output(&agent);
    assert_eq!(
        agent_json["schema"],
        "assura.project-intelligence.agent-query.v1"
    );
    assert_eq!(agent_json["request"]["capability"], "graph_queries");
}

#[test]
fn beacon_crm_invalid_fixture_reports_model_and_relation_drift() {
    let check = run(&["check", "--format", "json", INVALID_ROOT]);
    assert_eq!(check.status.code(), Some(1));
    let check_json = json_output(&check);
    let violations = check_json["violations"].as_array().unwrap();
    assert!(violations.iter().any(|item| {
        item["rule"] == "content_runtime:invalid_object_shape"
            && item["path"] == "docs/epics/epic_checkout.md"
            && item["message"].as_str().unwrap().contains("owner")
    }));
    assert!(violations.iter().any(|item| {
        item["rule"] == "content_runtime:missing_reference"
            && item["path"] == "docs/epics/epic_checkout.md"
            && item["message"]
                .as_str()
                .unwrap()
                .contains("adr-missing-payment-risk")
    }));

    let missing = run(&[
        "content",
        "missing-relations",
        INVALID_ROOT,
        "--format",
        "json",
    ]);
    assert_success(&missing);
    let missing_json = json_output(&missing);
    assert_eq!(
        missing_json["missing_relations"][0]["target_instance_id"],
        "adr-missing-payment-risk"
    );

    let agent = run(&[
        "content",
        "agent-query",
        "diagnostics",
        INVALID_ROOT,
        "--format",
        "json",
    ]);
    assert_success(&agent);
    let agent_json = json_output(&agent);
    let diagnostics = agent_json["response"]["diagnostics"]
        .as_array()
        .expect("diagnostics array");
    assert!(diagnostics.iter().any(|item| {
        item["rule"] == "content_runtime:missing_reference"
            && item["path"] == "docs/epics/epic_checkout.md"
    }));
    assert!(diagnostics.iter().any(|item| {
        item["rule"] == "content_runtime:invalid_object_shape"
            && item["path"] == "docs/epics/epic_checkout.md"
            && item["message"].as_str().unwrap().contains("owner")
            && item["severity"] == "high"
    }));
}

#[test]
fn beacon_crm_materialized_markdown_drift_previews_safe_fix_without_writing() {
    let project = copy_fixture_to_temp(INVALID_ROOT);
    let epic_path = project.path().join("docs/epics/epic_checkout.md");
    let before = fs::read_to_string(&epic_path).expect("epic markdown");
    let drifted = add_trailing_spaces_after_heading(&before);
    assert_ne!(
        drifted, before,
        "fixture mutation should add trailing spaces"
    );
    fs::write(&epic_path, &drifted).expect("write deterministic markdown drift");

    let fix = run(&[
        "fix",
        "markdown",
        project.path().to_str().expect("utf-8 temp path"),
        "--dry-run",
        "--format",
        "json",
    ]);
    assert_success(&fix);
    let fix_json = json_output(&fix);
    assert_eq!(fix_json["schema"], "assura.safe-fix.markdown.v1");
    assert_eq!(fix_json["dry_run"], true);
    assert_eq!(fix_json["files_checked"], 1);
    assert_eq!(fix_json["files_changed"], 0);
    assert_eq!(fix_json["fixes_applied"], 0);
    assert_eq!(fix_json["files_would_change"], 1);
    assert_eq!(fix_json["fixes_would_apply"], 1);
    assert_eq!(fix_json["fixes"][0]["status"], "planned");
    assert_eq!(
        fix_json["fixes"][0]["operation"],
        "remove_blank_line_trailing_spaces"
    );
    assert!(fix_json["fixes"][0]["id"]
        .as_str()
        .expect("fix id")
        .starts_with("markdown.safe_fix."));

    let safe_fixes = run(&[
        "content",
        "agent-query",
        "safe-fixes",
        project.path().to_str().expect("utf-8 temp path"),
        "--format",
        "json",
    ]);
    assert_success(&safe_fixes);
    let safe_fixes_json = json_output(&safe_fixes);
    let preview = &safe_fixes_json["response"]["safe_fixes"][0];
    assert_eq!(preview["path"], "docs/epics/epic_checkout.md");
    assert_eq!(preview["audit_id"], fix_json["fixes"][0]["id"]);

    let after = fs::read_to_string(&epic_path).expect("epic markdown after dry-run");
    assert_eq!(after, drifted, "dry-run must not write the markdown file");
}

#[test]
fn assura_repository_project_intelligence_goals_are_queryable() {
    let search = run(&[
        "content",
        "search",
        "Project Intelligence Usability",
        ".",
        "--format",
        "json",
    ]);
    assert_success(&search);
    let search_json = json_output(&search);
    let matches = search_json["matches"].as_array().expect("matches array");
    assert!(matches.iter().any(|item| {
        item["source_kind"] == "model_instance"
            && item["collection"] == "assura_goals"
            && item["instance_id"] == "goal-assura-project-intelligence-usability-program"
    }));

    let expanded = run(&[
        "content",
        "expand",
        "assura_goals",
        "goal-assura-project-intelligence-usability-program",
        ".",
        "--format",
        "json",
    ]);
    assert_success(&expanded);
    let expanded_json = json_output(&expanded);
    assert!(
        expanded_json["root_id"]
            .as_str()
            .expect("root id")
            .starts_with("instance:"),
        "graph expansion should return an ingested Assura goal instance root"
    );
}

fn copy_fixture_to_temp(source: &str) -> TempDir {
    let temp = TempDir::new().expect("temp project");
    copy_dir(Path::new(source), temp.path());
    temp
}

fn copy_dir(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("create target dir");
    for entry in fs::read_dir(source).expect("read source dir") {
        let entry = entry.expect("dir entry");
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &target_path);
        } else {
            fs::copy(&source_path, &target_path).unwrap_or_else(|error| {
                panic!(
                    "copy {} -> {}: {error}",
                    source_path.display(),
                    target_path.display()
                )
            });
        }
    }
}
