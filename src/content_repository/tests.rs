//! Tests for the repo-native content repository prototype.

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

    let validation = ContentRepository::new(&model()).validate(fixture.path());

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

    let validation = ContentRepository::new(&model()).validate(fixture.path());
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

    let validation = ContentRepository::new(&model()).validate(fixture.path());
    let codes = validation
        .findings
        .iter()
        .map(|finding| finding.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&"missing_field"));
    assert!(codes.contains(&"invalid_field_type"));
}

#[test]
fn updates_markdown_frontmatter_without_changing_body() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "docs/goals/goal-1.md",
        "---\nid: goal-1\ntitle: Goal One\nstatus: active\nspecs: []\n---\n# Goal One\n\nBody stays.\n",
    );

    let model = model();
    let repo = ContentRepository::new(&model);
    repo.update_field(
        fixture.path(),
        &ObjectKey::new("goals", "goal-1"),
        "status",
        json!("completed"),
    )
    .expect("update succeeds");

    let content = fixture.read("docs/goals/goal-1.md");
    assert!(content.contains("status: completed"));
    assert!(content.ends_with("# Goal One\n\nBody stays.\n"));
    let validation = repo.validate(fixture.path());
    assert_eq!(validation.findings, Vec::new());
}

#[test]
fn updates_json_record_and_revalidates() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "specs/spec-1.json",
        r#"{
  "id": "spec-1",
  "title": "Spec One",
  "status": "draft"
}"#,
    );

    let model = model();
    let repo = ContentRepository::new(&model);
    repo.update_field(
        fixture.path(),
        &ObjectKey::new("specs", "spec-1"),
        "status",
        json!("active"),
    )
    .expect("update succeeds");

    let content = fixture.read("specs/spec-1.json");
    assert!(content.contains("\"status\": \"active\""));
    let validation = repo.validate(fixture.path());
    assert_eq!(validation.findings, Vec::new());
}

#[test]
fn rejects_markdown_update_that_breaks_reference_graph() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "docs/goals/goal-1.md",
        "---\nid: goal-1\ntitle: Goal One\nstatus: active\nspecs:\n  - spec-1\n---\n# Goal One\n",
    );
    fixture.write(
        "specs/spec-1.json",
        r#"{
  "id": "spec-1",
  "title": "Spec One",
  "status": "active"
}"#,
    );

    let model = model();
    let repo = ContentRepository::new(&model);
    let error = repo
        .update_field(
            fixture.path(),
            &ObjectKey::new("goals", "goal-1"),
            "specs",
            json!(["missing-spec"]),
        )
        .expect_err("broken reference is rejected");

    assert_eq!(error.code, "missing_reference");
    assert!(fixture.read("docs/goals/goal-1.md").contains("spec-1"));
    let validation = repo.validate(fixture.path());
    assert_eq!(validation.findings, Vec::new());
}

#[test]
fn rejects_update_that_changes_object_identity() {
    let fixture = FixtureRepo::new();
    fixture.write(
        "specs/spec-1.json",
        r#"{
  "id": "spec-1",
  "title": "Spec One",
  "status": "draft"
}"#,
    );

    let model = model();
    let repo = ContentRepository::new(&model);
    let error = repo
        .update_field(
            fixture.path(),
            &ObjectKey::new("specs", "spec-1"),
            "id",
            json!("renamed-spec"),
        )
        .expect_err("id changes are rejected");

    assert_eq!(error.code, "object_id_changed");
    assert!(fixture
        .read("specs/spec-1.json")
        .contains("\"id\": \"spec-1\""));
    let validation = repo.validate(fixture.path());
    assert_eq!(validation.findings, Vec::new());
}

fn model() -> RepositoryModel {
    RepositoryModel {
        collections: vec![goals_collection(), specs_collection()],
        placements: vec![
            PlacementRule::recursive("docs/goals", ["goal"]),
            PlacementRule::recursive("specs", ["spec"]),
        ],
    }
}

fn goals_collection() -> CollectionSpec {
    CollectionSpec {
        name: "goals".to_string(),
        object_type: "goal".to_string(),
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
    }
}

fn specs_collection() -> CollectionSpec {
    CollectionSpec {
        name: "specs".to_string(),
        object_type: "spec".to_string(),
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

    fn read(&self, rel: &str) -> String {
        fs::read_to_string(self.temp.path().join(rel)).expect("read fixture")
    }
}
