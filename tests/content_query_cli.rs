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

#[test]
fn content_query_semantic_search_is_opt_in_and_returns_context() {
    let disabled = json_output(run_content(&[
        "semantic-search",
        "goal-portable-structure",
        "tests/fixtures/content_runtime/valid",
        "--format",
        "json",
    ]));
    assert_eq!(disabled["query"], "goal-portable-structure");
    assert_eq!(disabled["enabled"], false);
    assert!(disabled["provider"].is_null());
    assert!(disabled["message"]
        .as_str()
        .expect("disabled message")
        .contains("--enable-local"));
    assert!(disabled["matches"]
        .as_array()
        .expect("matches array")
        .is_empty());

    let enabled = json_output(run_content(&[
        "semantic-search",
        "goal-portable-structure",
        "tests/fixtures/content_runtime/valid",
        "--enable-local",
        "--limit",
        "3",
        "--format",
        "json",
    ]));

    assert_eq!(enabled["enabled"], true);
    assert_eq!(enabled["provider"], "local-hash-embedding-v1");
    let first = &enabled["matches"][0];
    assert_eq!(first["source_kind"], "model_instance");
    assert_eq!(first["collection"], "goals");
    assert_eq!(first["instance_id"], "goal-portable-structure");
    assert!(first["score"].as_f64().expect("score") > 0.0);
    assert_eq!(first["text_hash"].as_str().expect("text hash").len(), 16);
    assert_eq!(first["related"][0]["relationship"], "outgoing_relation");
    assert_eq!(
        first["related"][0]["path"],
        "specs/spec_portable_structure.json"
    );

    let no_signal = json_output(run_content(&[
        "semantic-search",
        "zzzzzzzzzzzz",
        "tests/fixtures/content_runtime/valid",
        "--enable-local",
        "--format",
        "json",
    ]));
    assert!(no_signal["matches"]
        .as_array()
        .expect("matches array")
        .is_empty());
}

#[test]
fn content_query_reports_code_symbols_in_both_directions() {
    let symbols = json_output(run_content(&[
        "symbols",
        "components",
        "component-config",
        "tests/fixtures/content_runtime/code_symbols",
        "--format",
        "json",
    ]));

    assert_eq!(symbols["collection"], "components");
    assert_eq!(symbols["id"], "component-config");
    let symbol_refs = symbols["symbols"].as_array().expect("symbols array");
    assert_eq!(symbol_refs.len(), 2);
    assert!(symbol_refs.iter().any(|item| {
        item["field"] == "implementation"
            && item["symbol"] == "crate::sample::Config"
            && item["provider"] == "rust-token-baseline-v1"
            && item["resolved"] == true
            && item["target_symbol"] == "Config"
            && item["target_path"] == "src/sample.rs"
            && item["evidence"] == "baseline"
    }));
    assert!(symbol_refs.iter().any(|item| {
        item["field"] == "external_symbol"
            && item["symbol"] == "external::Runtime"
            && item["provider"] == "external-index-v1"
            && item["resolved"] == false
            && item["target_id"].is_null()
    }));

    let full_refs = json_output(run_content(&[
        "symbol-refs",
        "crate::sample::Config",
        "tests/fixtures/content_runtime/code_symbols",
        "--format",
        "json",
    ]));
    assert_eq!(full_refs["symbol"], "crate::sample::Config");
    assert_eq!(full_refs["references"][0]["collection"], "components");
    assert_eq!(
        full_refs["references"][0]["instance_id"],
        "component-config"
    );
    assert_eq!(full_refs["references"][0]["resolved"], true);

    let unresolved_refs = json_output(run_content(&[
        "symbol-refs",
        "Runtime",
        "tests/fixtures/content_runtime/code_symbols",
        "--format",
        "json",
    ]));
    assert_eq!(
        unresolved_refs["references"][0]["symbol"],
        "external::Runtime"
    );
    assert_eq!(unresolved_refs["references"][0]["resolved"], false);
}

#[test]
fn content_query_reports_generic_agent_context() {
    let context = json_output(run_content(&[
        "agent-context",
        "tests/fixtures/content_runtime/code_symbols",
        "--format",
        "json",
    ]));

    assert_eq!(
        context["schema"],
        "assura.project-intelligence.agent-context.v1"
    );
    assert_eq!(context["summary"]["model_instances"], 1);
    assert_eq!(context["summary"]["symbol_refs"], 2);
    assert_eq!(context["summary"]["resolved_symbol_refs"], 1);
    let capabilities = context["capabilities"]
        .as_array()
        .expect("capabilities array");
    assert!(capabilities.iter().any(|item| {
        item["name"] == "diagnostics" && item["cli"] == "assura check --format agent"
    }));
    assert!(capabilities
        .iter()
        .any(|item| { item["name"] == "code_symbols" && item["cli"] == "assura content symbols" }));
}
