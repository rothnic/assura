use serde_json::Value;
use std::process::{Command, Output};

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn run_content(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .arg("content")
        .args(args)
        .output()
        .expect("content command runs")
}

fn json_output(output: Output) -> Value {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("content command emits JSON")
}

#[test]
fn content_query_lists_collections_and_instances() {
    let collections = json_output(run_content(&[
        "collections",
        "tests/fixtures/content_runtime/valid",
        "--format",
        "json",
    ]));

    assert_eq!(collections["collections"][0]["collection"], "goals");
    assert_eq!(collections["collections"][0]["instances"], 1);
    assert_eq!(collections["collections"][1]["collection"], "specs");

    let instances = json_output(run_content(&[
        "instances",
        "goals",
        "tests/fixtures/content_runtime/valid",
        "--format",
        "json",
    ]));
    assert_eq!(instances["collection"], "goals");
    assert_eq!(instances["instances"][0]["id"], "goal-portable-structure");
    assert_eq!(
        instances["instances"][0]["path"],
        "docs/goals/goal_portable_structure.md"
    );
}

#[test]
fn content_query_shows_instance_and_expands_graph() {
    let shown = json_output(run_content(&[
        "show",
        "goals",
        "goal-portable-structure",
        "tests/fixtures/content_runtime/valid",
        "--format",
        "json",
    ]));

    assert_eq!(shown["collection"], "goals");
    assert_eq!(shown["id"], "goal-portable-structure");
    assert_eq!(shown["outgoing_relations"][0]["field"], "specs");
    assert_eq!(shown["outgoing_relations"][0]["missing"], false);
    assert_eq!(shown["sections"][0]["title"], "Portable Structure Policy");

    let expanded = json_output(run_content(&[
        "expand",
        "goals",
        "goal-portable-structure",
        "tests/fixtures/content_runtime/valid",
        "--format",
        "json",
    ]));
    assert_eq!(expanded["related"][0]["kind"], "model_instance");
    assert_eq!(expanded["related"][0]["relationship"], "outgoing_relation");
    assert_eq!(
        expanded["related"][0]["path"],
        "specs/spec_portable_structure.json"
    );
}

#[test]
fn content_query_searches_and_reports_missing_relations() {
    let search = json_output(run_content(&[
        "search",
        "Portable",
        "tests/fixtures/content_runtime/valid",
        "--format",
        "json",
    ]));

    let matches = search["matches"].as_array().expect("matches array");
    assert!(matches.iter().any(|item| {
        item["source_kind"] == "model_instance"
            && item["collection"] == "goals"
            && item["instance_id"] == "goal-portable-structure"
    }));
    assert!(matches
        .iter()
        .any(|item| item["source_kind"] == "markdown_section"));

    let missing = json_output(run_content(&[
        "missing-relations",
        "tests/fixtures/content_runtime/missing_reference",
        "--format",
        "json",
    ]));
    assert_eq!(missing["missing_relations"][0]["field"], "specs");
    assert_eq!(
        missing["missing_relations"][0]["target_instance_id"],
        "missing-spec"
    );
    assert_eq!(missing["missing_relations"][0]["missing"], true);

    let diagnostic_search = json_output(run_content(&[
        "search",
        "missing-spec",
        "tests/fixtures/content_runtime/missing_reference",
        "--format",
        "json",
    ]));
    assert!(diagnostic_search["matches"]
        .as_array()
        .expect("matches array")
        .iter()
        .any(|item| item["source_kind"] == "diagnostic"));
}
