//! Tests for repo-native content runtime validation.

use super::*;
use serde_json::json;
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn validates_markdown_json_placement_and_references() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "docs/goals/goal-1.md",
        "---\nid: goal-1\ntitle: Goal One\nstatus: active\nspecs:\n  - spec-1\n---\n# Goal One\n\n## Validation\n",
    );
    fixture.write(
        "specs/spec-1.json",
        r#"{
  "id": "spec-1",
  "title": "Spec One",
  "status": "active"
}"#,
    );

    let validation = ContentRepository::try_new(model())
        .expect("model compiles")
        .validate(fixture.path());

    assert_eq!(validation.findings, Vec::new());
    assert_eq!(validation.snapshot.objects.len(), 2);
    assert_eq!(validation.snapshot.edges.len(), 1);
    let goal = validation
        .snapshot
        .objects
        .get(&("goals".to_string(), "goal-1".to_string()))
        .expect("goal loaded");
    assert_eq!(
        goal.body.as_deref().expect("body"),
        "# Goal One\n\n## Validation\n"
    );
    assert_eq!(
        goal.headings,
        vec![
            MarkdownHeading {
                level: 1,
                text: "Goal One".to_string(),
                line_number: 1,
            },
            MarkdownHeading {
                level: 2,
                text: "Validation".to_string(),
                line_number: 3,
            },
        ]
    );
}

#[test]
fn reports_missing_reference_and_invalid_placement() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "docs/goals/goal-1.md",
        "---\nid: goal-1\ntitle: Goal One\nstatus: active\nspecs:\n  - missing-spec\n---\n# Goal One\n",
    );
    fixture.write(
        "docs/specs/spec-1.json",
        r#"{
  "id": "spec-1",
  "title": "Spec One",
  "status": "active"
}"#,
    );

    let validation = ContentRepository::try_new(model())
        .expect("model compiles")
        .validate(fixture.path());
    let codes = validation
        .findings
        .iter()
        .map(|finding| finding.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&"missing_reference"));
    assert!(codes.contains(&"invalid_object_placement"));
}

#[test]
fn reports_field_validation_errors() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "docs/goals/goal-1.md",
        "---\nid: goal-1\nstatus: unknown\nspecs: spec-1\n---\n# Goal One\n",
    );

    let validation = ContentRepository::try_new(model())
        .expect("model compiles")
        .validate(fixture.path());
    let codes = validation
        .findings
        .iter()
        .map(|finding| finding.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&"missing_field"));
    assert!(codes.contains(&"invalid_field_type"));
}

#[test]
fn reports_collection_pattern_errors_in_collection_order() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "docs/goals/goal-1.md",
        "---\nid: goal-1\nstatus: active\n---\n# Goal One\n",
    );

    let validation = ContentRepository::try_new(RepositoryModel {
        collections: vec![goals_collection(), invalid_pattern_collection()],
        placements: vec![PlacementRule::recursive("docs/goals", ["goal"])],
        schema_artifact_path: None,
        schema_artifact: None,
    })
    .expect("model compiles")
    .validate(fixture.path());
    let codes = validation
        .findings
        .iter()
        .map(|finding| finding.code)
        .collect::<Vec<_>>();

    assert_eq!(codes, vec!["missing_field", "invalid_pattern"]);
}

#[test]
fn declared_fields_still_validate_when_schema_validator_exists() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "docs/goals/goal-1.md",
        "---\nid: goal-1\nstatus: active\n---\n# Goal One\n",
    );

    let validation = ContentRepository::try_new(RepositoryModel {
        collections: vec![schema_backed_goals_collection()],
        placements: vec![PlacementRule::recursive("docs/goals", ["goal"])],
        schema_artifact_path: None,
        schema_artifact: Some(json!({
            "$schema": "https://json-schema.org/draft/2020-12/schema",
            "$defs": {
                "goal": {
                    "type": "object",
                    "required": ["id", "status"],
                    "properties": {
                        "id": { "type": "string" },
                        "status": {
                            "type": "string",
                            "enum": ["active", "completed"]
                        },
                        "title": { "type": "string" }
                    }
                }
            }
        })),
    })
    .expect("model compiles")
    .validate(fixture.path());
    let codes = validation
        .findings
        .iter()
        .map(|finding| finding.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&"missing_field"));
    assert!(!codes.contains(&"invalid_object_shape"));
}

fn model() -> RepositoryModel {
    RepositoryModel {
        collections: vec![goals_collection(), specs_collection()],
        placements: vec![
            PlacementRule::recursive("docs/goals", ["goal"]),
            PlacementRule::recursive("specs", ["spec"]),
        ],
        schema_artifact_path: None,
        schema_artifact: None,
    }
}

fn goals_collection() -> CollectionSpec {
    CollectionSpec {
        name: "goals".to_string(),
        object_type: "goal".to_string(),
        schema_class: None,
        path_pattern: "docs/goals/*.md".to_string(),
        adapter: AdapterKind::MarkdownFrontmatter,
        id_field: "id".to_string(),
        fields: vec![
            FieldSpec::required("id", FieldKind::String),
            FieldSpec::required("title", FieldKind::String),
            FieldSpec::required(
                "status",
                FieldKind::Enum(vec!["active".to_string(), "completed".to_string()]),
            ),
            FieldSpec::optional("specs", FieldKind::StringArray),
        ],
        references: vec![ReferenceSpec::many("specs", "specs")],
        code_symbols: Vec::new(),
    }
}

fn specs_collection() -> CollectionSpec {
    CollectionSpec {
        name: "specs".to_string(),
        object_type: "spec".to_string(),
        schema_class: None,
        path_pattern: "**/*.json".to_string(),
        adapter: AdapterKind::JsonRecord,
        id_field: "id".to_string(),
        fields: vec![
            FieldSpec::required("id", FieldKind::String),
            FieldSpec::required("title", FieldKind::String),
            FieldSpec::required(
                "status",
                FieldKind::Enum(vec!["draft".to_string(), "active".to_string()]),
            ),
        ],
        references: Vec::new(),
        code_symbols: Vec::new(),
    }
}

fn invalid_pattern_collection() -> CollectionSpec {
    CollectionSpec {
        name: "invalid".to_string(),
        object_type: "invalid".to_string(),
        schema_class: None,
        path_pattern: "[".to_string(),
        adapter: AdapterKind::JsonRecord,
        id_field: "id".to_string(),
        fields: Vec::new(),
        references: Vec::new(),
        code_symbols: Vec::new(),
    }
}

fn schema_backed_goals_collection() -> CollectionSpec {
    CollectionSpec {
        name: "goals".to_string(),
        object_type: "goal".to_string(),
        schema_class: Some("goal".to_string()),
        path_pattern: "docs/goals/*.md".to_string(),
        adapter: AdapterKind::MarkdownFrontmatter,
        id_field: "id".to_string(),
        fields: vec![
            FieldSpec::required("id", FieldKind::String),
            FieldSpec::required("title", FieldKind::String),
            FieldSpec::required(
                "status",
                FieldKind::Enum(vec!["active".to_string(), "completed".to_string()]),
            ),
        ],
        references: Vec::new(),
        code_symbols: Vec::new(),
    }
}

struct FixtureRepo {
    temp: TempDir,
}

impl FixtureRepo {
    fn new() -> Self {
        Self {
            temp: TempDir::new().expect("tempdir"),
        }
    }

    fn path(&self) -> &Path {
        self.temp.path()
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.temp.path().join(rel);
        fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
        fs::write(path, content).expect("write fixture");
    }
}
