use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn run_assura(args: &[&str]) -> Output {
    Command::new(assura_bin())
        .args(args)
        .output()
        .expect("assura command runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

fn json_output(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "command emits JSON: {error}\nstdout:\n{}\nstderr:\n{}",
            stdout(output),
            stderr(output)
        )
    })
}

#[test]
fn check_passes_for_mixed_record_traceability() {
    let project = traceability_project(true);
    let output = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report = json_output(&output);
    assert_eq!(report["success"], true);
    assert_eq!(report["violations"].as_array().unwrap().len(), 0);
}

#[test]
fn check_reports_requirement_claim_evidence_and_finding_traceability_gaps() {
    let project = traceability_project(false);
    let output = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report = json_output(&output);
    let violations = report["violations"].as_array().unwrap();
    let messages = violations
        .iter()
        .filter(|violation| violation["rule"] == "requirements_traceability:document_traceability")
        .map(|violation| violation["message"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert!(messages
        .iter()
        .any(|message| message.contains("High-priority requirement `req-auth`")));
    assert!(messages
        .iter()
        .any(|message| message.contains("Claim `claim-auth` must link to evidence")));
    assert!(
        messages
            .iter()
            .any(|message| message
                .contains("Evidence `evidence-auth` must link to a source document"))
    );
    assert!(messages
        .iter()
        .any(|message| message.contains("Finding `finding-auth` must carry owner metadata")));
    assert!(messages
        .iter()
        .any(|message| message.contains("Finding `finding-auth` must carry status metadata")));
}

#[test]
fn agent_query_exposes_traceability_diagnostics_gaps_and_next_actions() {
    let project = traceability_project(false);
    let path = project.path().to_str().unwrap();

    let diagnostics = run_assura(&[
        "content",
        "agent-query",
        "diagnostics",
        path,
        "--format",
        "json",
    ]);
    assert!(
        diagnostics.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&diagnostics),
        stderr(&diagnostics)
    );
    let diagnostics_json = json_output(&diagnostics);
    assert!(diagnostics_json["response"]["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|diagnostic| {
            diagnostic["rule"] == "requirements_traceability:document_traceability"
        }));

    let gaps = run_assura(&["content", "agent-query", "gaps", path, "--format", "json"]);
    assert!(gaps.status.success(), "stderr:\n{}", stderr(&gaps));
    let gaps_json = json_output(&gaps);
    assert_eq!(gaps_json["response"]["requirements_traceability"], 5);

    let next_actions = run_assura(&[
        "content",
        "agent-query",
        "next-actions",
        path,
        "--format",
        "json",
    ]);
    assert!(
        next_actions.status.success(),
        "stderr:\n{}",
        stderr(&next_actions)
    );
    let next_actions_json = json_output(&next_actions);
    assert!(next_actions_json["response"]["actions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|action| action["reason"] == "requirements traceability gaps exist"));
}

#[test]
fn doctor_reports_inactive_traceability_model_as_gap() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            "{}\nstructure:\n  ./:\n    required: false\n",
            traceability_extension_yaml()
        ),
    )
    .unwrap();

    let output = run_assura(&[
        "doctor",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let doctor = json_output(&output);
    assert!(doctor["gaps"].as_array().unwrap().iter().any(|gap| {
        gap["name"] == "requirements_traceability_inactive:document_traceability"
            && gap["status"] == "gap"
    }));
}

#[test]
fn config_rejects_claim_traceability_without_evidence_collections() {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        r#"extensions:
  requirements_traceability:
    - id: document_traceability
      requirements_collection: requirements
      priority_field: priority
      claim_collections:
        - claims
structure: {}
"#,
    )
    .unwrap();

    let output = run_assura(&[
        "check",
        project.path().to_str().unwrap(),
        "--format",
        "json",
    ]);

    assert!(!output.status.success());
    assert!(
        stderr(&output).contains("evidence_collections"),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
}

fn traceability_project(valid: bool) -> TempDir {
    let project = TempDir::new().unwrap();
    write_base_project(project.path());
    if valid {
        write_valid_records(project.path());
    } else {
        write_invalid_records(project.path());
    }
    project
}

fn write_base_project(root: &Path) {
    for dir in [
        ".assura",
        "schemas",
        "docs/requirements",
        "records",
        "source-documents",
    ] {
        fs::create_dir_all(root.join(dir)).unwrap();
    }
    fs::write(
        root.join(".assura/config.yml"),
        format!(
            r#"{}
models:
  validation_artifact: "schemas/traceability.schema.json"

collections:
  requirements:
    class: Requirement
    path: "docs/requirements/*.md"
    adapter: markdown_frontmatter
    id: id
  evidence:
    class: Evidence
    path: "records/evidence.yml"
    adapter: yaml_record
    id: id
  claims:
    class: Claim
    path: "records/claim.json"
    adapter: json_record
    id: id
  findings:
    class: Finding
    path: "records/findings.jsonl"
    adapter: jsonl_record
    id: id
  source_documents:
    class: SourceDocument
    path: "source-documents/*.md"
    adapter: markdown_frontmatter
    id: id

relations:
  evidence.requirements:
    target: requirements
    many: true
  evidence.source_documents:
    target: source_documents
    many: true
  claims.evidence:
    target: evidence
    many: true
  findings.evidence:
    target: evidence
    many: true

structure: {{}}
exclude:
  - ".assura/**"
"#,
            traceability_extension_yaml()
        ),
    )
    .unwrap();
    fs::write(
        root.join("schemas/traceability.schema.json"),
        traceability_schema(),
    )
    .unwrap();
    fs::write(
        root.join("docs/requirements/req-auth.md"),
        r#"---
id: req-auth
title: Auth Requirement
status: active
priority: high
---

# Auth Requirement
"#,
    )
    .unwrap();
    fs::write(
        root.join("source-documents/source-auth.md"),
        r#"---
id: source-auth
title: Source Auth
status: available
---

# Source Auth
"#,
    )
    .unwrap();
}

fn traceability_extension_yaml() -> &'static str {
    r#"extensions:
  requirements_traceability:
    - id: document_traceability
      severity: high
      requirements_collection: requirements
      priority_field: priority
      high_priority_values:
        - high
        - critical
      coverage_collections:
        - evidence
        - claims
      claim_collections:
        - claims
      evidence_collections:
        - evidence
      source_document_collections:
        - source_documents
      finding_collections:
        - findings
      owner_fields:
        - owner
      status_fields:
        - status
"#
}

fn write_valid_records(root: &Path) {
    fs::write(
        root.join("records/evidence.yml"),
        r#"id: evidence-auth
title: Auth evidence
status: active
requirements:
  - req-auth
source_documents:
  - source-auth
"#,
    )
    .unwrap();
    fs::write(
        root.join("records/claim.json"),
        r#"{"id":"claim-auth","title":"Auth claim","status":"active","evidence":["evidence-auth"]}"#,
    )
    .unwrap();
    fs::write(
        root.join("records/findings.jsonl"),
        r#"{"id":"finding-auth","title":"Auth finding","status":"open","owner":"docs-team","evidence":["evidence-auth"]}"#,
    )
    .unwrap();
}

fn write_invalid_records(root: &Path) {
    fs::write(
        root.join("records/evidence.yml"),
        r#"id: evidence-auth
title: Auth evidence
status: active
"#,
    )
    .unwrap();
    fs::write(
        root.join("records/claim.json"),
        r#"{"id":"claim-auth","title":"Auth claim","status":"active"}"#,
    )
    .unwrap();
    fs::write(
        root.join("records/findings.jsonl"),
        r#"{"id":"finding-auth","title":"Auth finding","evidence":["evidence-auth"]}"#,
    )
    .unwrap();
}

fn traceability_schema() -> &'static str {
    r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Requirement": { "$ref": "#/$defs/ProjectRecord" },
    "Evidence": { "$ref": "#/$defs/ProjectRecord" },
    "Claim": { "$ref": "#/$defs/ProjectRecord" },
    "Finding": { "$ref": "#/$defs/ProjectRecord" },
    "SourceDocument": { "$ref": "#/$defs/ProjectRecord" },
    "ProjectRecord": {
      "type": "object",
      "required": ["id", "title"],
      "properties": {
        "id": { "type": "string" },
        "title": { "type": "string" },
        "status": { "type": "string" },
        "priority": { "type": "string" },
        "owner": { "type": "string" },
        "requirements": {
          "type": "array",
          "items": { "type": "string" }
        },
        "evidence": {
          "type": "array",
          "items": { "type": "string" }
        },
        "source_documents": {
          "type": "array",
          "items": { "type": "string" }
        }
      },
      "additionalProperties": true
    }
  }
}
"##
}
