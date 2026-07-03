use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const BEACON_INVALID: &str = "tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid";
const BEACON_VALID: &str = "tests/fixtures/project_intelligence_real_repo/beacon_crm/valid";

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn run_assura(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(args)
        .output()
        .expect("assura command runs")
}

fn json_from_success(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("command emits JSON")
}

fn copy_dir(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("create destination");
    for entry in fs::read_dir(source).expect("read source dir") {
        let entry = entry.expect("dir entry");
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_dir(&source_path, &destination_path);
        } else {
            fs::copy(&source_path, &destination_path).expect("copy file");
        }
    }
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
fn context_pack_wraps_beacon_diagnostics_relations_search_and_safe_fixes() {
    let pack = json_from_success(run_assura(&[
        "content",
        "context-pack",
        BEACON_INVALID,
        "--text",
        "checkout",
        "--limit",
        "5",
        "--format",
        "json",
    ]));
    assert_eq!(
        pack["schema"],
        "assura.project-intelligence.context-pack.v1"
    );
    assert_eq!(pack["request"]["mode"], "diagnostics");
    assert_eq!(pack["request"]["cli"], "assura content context-pack");
    assert_eq!(pack["bounds"]["limit"], 5);
    assert!(pack["bounds"]["omissions"]
        .as_array()
        .expect("omissions array")
        .iter()
        .any(|item| item["field"] == "instance"));
    assert!(pack["bounds"]["omissions"]
        .as_array()
        .expect("omissions array")
        .iter()
        .any(|item| item["field"] == "repository_references"));

    let diagnostics = pack["diagnostics"].as_array().expect("diagnostics");
    assert!(diagnostics
        .iter()
        .any(|item| item["rule"] == "content_runtime:invalid_object_shape"));
    assert!(diagnostics
        .iter()
        .any(|item| item["rule"] == "content_runtime:missing_reference"));
    assert!(pack["missing_relations"]
        .as_array()
        .expect("missing relations")
        .iter()
        .any(|item| item["target_instance_id"] == "adr-missing-payment-risk"));
    assert!(pack["search"]["matches"]
        .as_array()
        .expect("search matches")
        .iter()
        .any(|item| item["instance_id"] == "epic-checkout"));
    assert_eq!(pack["safe_fixes"].as_array().expect("safe fixes").len(), 0);

    let lower_diagnostics = json_from_success(run_assura(&[
        "content",
        "agent-query",
        "diagnostics",
        BEACON_INVALID,
        "--format",
        "json",
    ]));
    assert_eq!(
        diagnostics.len(),
        lower_diagnostics["response"]["diagnostics"]
            .as_array()
            .expect("lower diagnostics")
            .len()
    );

    let lower_missing = json_from_success(run_assura(&[
        "content",
        "missing-relations",
        BEACON_INVALID,
        "--format",
        "json",
    ]));
    assert_eq!(
        pack["missing_relations"][0]["target_instance_id"],
        lower_missing["missing_relations"][0]["target_instance_id"]
    );
}

#[test]
fn context_pack_wraps_repository_references_for_object_context() {
    let pack = json_from_success(run_assura(&[
        "content",
        "context-pack",
        "tests/fixtures/content_runtime/code_symbols",
        "--collection",
        "components",
        "--id",
        "component-config",
        "--limit",
        "5",
        "--format",
        "json",
    ]));

    assert_eq!(pack["request"]["mode"], "object");
    assert_eq!(
        pack["repository_references"]["path"],
        "components/component_config.json"
    );

    let inbound = pack["repository_references"]["inbound"]
        .as_array()
        .expect("inbound references");
    assert_eq!(inbound.len(), 1);
    assert_eq!(inbound[0]["source_path"], "src/sample.rs");
    assert_eq!(
        inbound[0]["target_path"],
        "components/component_config.json"
    );
    assert_eq!(inbound[0]["reference_kind"], "comment_reference");
    assert_eq!(inbound[0]["target_exists"], true);

    let outbound = pack["repository_references"]["outbound"]
        .as_array()
        .expect("outbound references");
    assert_eq!(outbound.len(), 0);

    let lower_refs = json_from_success(run_assura(&[
        "content",
        "references",
        "tests/fixtures/content_runtime/code_symbols",
        "--target",
        "components/component_config.json",
        "--format",
        "json",
    ]));
    assert_eq!(inbound[0]["id"], lower_refs["references"][0]["id"]);
}

#[test]
fn context_pack_wraps_configured_frontmatter_repository_references() {
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(
        Path::new("tests/fixtures/content_runtime/valid"),
        temp.path(),
    );
    let config_path = temp.path().join(".assura/config.yml");
    let mut config = fs::read_to_string(&config_path).expect("config");
    config.push_str(
        r#"
extensions:
  repository_references:
    - id: source_docs
      paths:
        - "docs/goals/*.md"
      frontmatter_fields:
        - source_documents
"#,
    );
    fs::write(&config_path, config).expect("write config");

    let schema_path = temp.path().join("schemas/content_runtime.schema.json");
    let mut schema = fs::read_to_string(&schema_path).expect("schema");
    schema = schema.replace(
        r#""specs": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        }"#,
        r#""specs": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        },
        "source_documents": {
          "type": "array",
          "items": { "type": "string", "minLength": 1 }
        }"#,
    );
    fs::write(&schema_path, schema).expect("write schema");

    let goal_path = temp.path().join("docs/goals/goal_portable_structure.md");
    let mut goal = fs::read_to_string(&goal_path).expect("goal");
    goal = goal.replace(
        "specs:\n  - spec-portable-structure\n---",
        "specs:\n  - spec-portable-structure\nsource_documents:\n  - specs/spec_portable_structure.json\n  - docs/missing_source.md\n---",
    );
    fs::write(&goal_path, goal).expect("write goal");

    let pack = json_from_success(run_assura(&[
        "content",
        "context-pack",
        temp.path().to_str().expect("temp path"),
        "--collection",
        "goals",
        "--id",
        "goal-portable-structure",
        "--limit",
        "5",
        "--format",
        "json",
    ]));

    let outbound = pack["repository_references"]["outbound"]
        .as_array()
        .expect("outbound references");
    assert!(outbound.iter().any(|item| {
        item["reference_kind"] == "frontmatter_reference"
            && item["target_path"] == "specs/spec_portable_structure.json"
            && item["target_exists"] == true
    }));
    assert!(outbound.iter().any(|item| {
        item["reference_kind"] == "frontmatter_reference"
            && item["target_path"] == "docs/missing_source.md"
            && item["target_exists"] == false
    }));
}

#[test]
fn context_pack_reports_repository_reference_truncation() {
    let pack = json_from_success(run_assura(&[
        "content",
        "context-pack",
        "tests/fixtures/content_runtime/code_symbols",
        "--collection",
        "components",
        "--id",
        "component-config",
        "--limit",
        "0",
        "--format",
        "json",
    ]));

    assert_eq!(
        pack["repository_references"]["inbound"]
            .as_array()
            .expect("inbound refs")
            .len(),
        0
    );
    assert!(pack["bounds"]["truncated"]
        .as_array()
        .expect("truncated")
        .iter()
        .any(|item| {
            item["field"] == "repository_references.inbound"
                && item["original_count"] == 1
                && item["returned_count"] == 0
        }));
}

#[test]
fn context_pack_safe_fixes_include_cli_audit_id() {
    let temp = tempfile::tempdir().expect("tempdir");
    copy_dir(Path::new(BEACON_INVALID), temp.path());
    let epic_path = temp.path().join("docs/epics/epic_checkout.md");
    let original = fs::read_to_string(&epic_path).expect("epic markdown");
    let drifted = add_trailing_spaces_after_heading(&original);
    assert_ne!(
        drifted, original,
        "fixture mutation should add trailing spaces"
    );
    fs::write(&epic_path, drifted).expect("write deterministic markdown drift");

    let dry_run = json_from_success(run_assura(&[
        "fix",
        "markdown",
        temp.path().to_str().expect("temp path"),
        "--dry-run",
        "--format",
        "json",
    ]));
    let dry_run_id = dry_run["fixes"][0]["id"].as_str().expect("fix id");

    let pack = json_from_success(run_assura(&[
        "content",
        "context-pack",
        temp.path().to_str().expect("temp path"),
        "--text",
        "checkout",
        "--limit",
        "5",
        "--format",
        "json",
    ]));
    let safe_fix = &pack["safe_fixes"][0];
    assert_eq!(safe_fix["path"], "docs/epics/epic_checkout.md");
    assert_eq!(safe_fix["audit_id"], dry_run_id);
}

#[test]
fn context_pack_wraps_assura_goal_object_context_with_bounds() {
    let pack = json_from_success(run_assura(&[
        "content",
        "context-pack",
        ".",
        "--collection",
        "assura_goals",
        "--id",
        "goal-assura-project-intelligence-usability-program",
        "--text",
        "Project Intelligence Usability",
        "--limit",
        "5",
        "--format",
        "json",
    ]));
    assert_eq!(pack["request"]["mode"], "object");
    assert_eq!(
        pack["instance"]["id"],
        "goal-assura-project-intelligence-usability-program"
    );
    assert_eq!(
        pack["instance"]["path"],
        "docs/goals/assura-project-intelligence-usability-program.md"
    );
    assert!(
        pack["instance"]["sections"]
            .as_array()
            .expect("sections")
            .len()
            <= 5
    );
    assert!(pack["bounds"]["truncated"]
        .as_array()
        .expect("truncated")
        .iter()
        .any(|item| item["field"] == "instance.sections"));
    assert!(pack["search"]["matches"]
        .as_array()
        .expect("search matches")
        .iter()
        .any(|item| item["instance_id"] == "goal-assura-project-intelligence-usability-program"));

    let lower_expand = json_from_success(run_assura(&[
        "content",
        "expand",
        "assura_goals",
        "goal-assura-project-intelligence-usability-program",
        ".",
        "--limit",
        "5",
        "--format",
        "json",
    ]));
    assert_eq!(pack["related"]["root_id"], lower_expand["root_id"]);
}

#[test]
fn context_pack_reports_related_graph_truncation() {
    let pack = json_from_success(run_assura(&[
        "content",
        "context-pack",
        BEACON_VALID,
        "--collection",
        "epics",
        "--id",
        "epic-checkout",
        "--limit",
        "1",
        "--format",
        "json",
    ]));
    assert_eq!(pack["request"]["mode"], "object");
    assert_eq!(
        pack["related"]["related"]
            .as_array()
            .expect("related facts")
            .len(),
        1
    );
    assert!(pack["bounds"]["truncated"]
        .as_array()
        .expect("truncated")
        .iter()
        .any(|item| {
            item["field"] == "related.related"
                && item["original_count"].as_u64().unwrap_or_default() > 1
                && item["returned_count"] == 1
        }));
}

#[test]
fn context_pack_rejects_partial_object_scope() {
    let output = run_assura(&[
        "content",
        "context-pack",
        ".",
        "--collection",
        "assura_goals",
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("requires --collection and --id together")
    );
}
