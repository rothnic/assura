use std::fs;
use std::path::Path;
use std::process::Command;

fn write_project(root: &Path) {
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
        r#"extensions:
  requirements_traceability:
    - id: document_traceability
      severity: high
      requirements_collection: requirements
      priority_field: priority
      high_priority_values:
        - high
      coverage_collections:
        - claims
      claim_collections:
        - claims
      evidence_collections:
        - evidence
      source_document_collections:
        - source_documents
models:
  validation_artifact: "schemas/traceability.schema.json"
collections:
  requirements:
    class: Requirement
    path: "docs/requirements/*.md"
    adapter: markdown_frontmatter
    id: id
  claims:
    class: Claim
    path: "records/claim.json"
    adapter: json_record
    id: id
  evidence:
    class: Evidence
    path: "records/evidence.yml"
    adapter: yaml_record
    id: id
  source_documents:
    class: SourceDocument
    path: "source-documents/*.md"
    adapter: markdown_frontmatter
    id: id
relations:
  claims.requirements:
    target: requirements
    many: true
  claims.evidence:
    target: evidence
    many: true
  evidence.source_documents:
    target: source_documents
    many: true
structure:
  ./:
    required: false
exclude:
  - ".assura/**"
"#,
    )
    .unwrap();
    fs::write(root.join("schemas/traceability.schema.json"), schema()).unwrap();
    fs::write(
        root.join("docs/requirements/req-auth.md"),
        "---\nid: req-auth\ntitle: Auth Requirement\npriority: high\n---\n# Auth\n",
    )
    .unwrap();
    fs::write(
        root.join("records/claim.json"),
        r#"{"id":"claim-auth","title":"Auth claim","requirements":["req-auth"],"evidence":["evidence-auth"]}"#,
    )
    .unwrap();
    fs::write(
        root.join("records/evidence.yml"),
        "id: evidence-auth\ntitle: Auth evidence\nsource_documents:\n  - source-auth\n",
    )
    .unwrap();
    fs::write(
        root.join("source-documents/source-auth.md"),
        "---\nid: source-auth\ntitle: Source Auth\n---\n# Source Auth\n",
    )
    .unwrap();
}

fn schema() -> &'static str {
    r##"{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Requirement": { "$ref": "#/$defs/ProjectRecord" },
    "Claim": { "$ref": "#/$defs/ProjectRecord" },
    "Evidence": { "$ref": "#/$defs/ProjectRecord" },
    "SourceDocument": { "$ref": "#/$defs/ProjectRecord" },
    "ProjectRecord": {
      "type": "object",
      "required": ["id", "title"],
      "properties": {
        "id": { "type": "string" },
        "title": { "type": "string" },
        "priority": { "type": "string" },
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
}"##
}

#[test]
fn compiled_config_cli_enforces_valid_requirements_traceability() {
    let temp = tempfile::tempdir().unwrap();
    let project = temp.path().join("compiled-traceability-project");
    let compiled_config = temp.path().join("check-config.bin");
    write_project(&project);

    let compile = Command::new(env!("CARGO_BIN_EXE_assura-check-compile-config"))
        .arg("--config")
        .arg(project.join(".assura/config.yml"))
        .arg("--output")
        .arg(&compiled_config)
        .output()
        .unwrap();
    assert!(
        compile.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&compile.stdout),
        String::from_utf8_lossy(&compile.stderr)
    );

    let check = Command::new(env!("CARGO_BIN_EXE_assura-check-compiled"))
        .arg("--compiled-config")
        .arg(&compiled_config)
        .arg("--quiet")
        .arg(&project)
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(!String::from_utf8_lossy(&check.stdout).contains("requirements_traceability:"));
}
