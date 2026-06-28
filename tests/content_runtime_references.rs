use assura::config::config::ContentRelationConfig;
use assura::config::loader::ConfigLoader;
use assura::content_repository::{
    AdapterKind, CollectionSpec, ContentFinding, ContentRepository, PlacementRule, ReferenceSpec,
    RepositoryModel, RepositoryValidation,
};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const FIXTURE_ROOT: &str = "tests/fixtures/content_runtime/references";

#[test]
fn validates_cross_adapter_required_optional_and_many_references() {
    let validation = validate_fixture("valid");

    assert_eq!(validation.findings, Vec::new());
    assert_object_exists(&validation, "goals", "goal-portable-structure");
    assert_object_exists(&validation, "specs", "spec-portable-structure");
    assert_object_exists(&validation, "decisions", "decision-runtime-shape");
    assert_object_exists(&validation, "events", "event-runtime-proof");
    assert_edge(&validation, "goals", "goal-portable-structure", "specs");
    assert_edge(&validation, "specs", "spec-portable-structure", "decision");
    assert_edge(&validation, "decisions", "decision-runtime-shape", "events");
    assert_edge(&validation, "events", "event-runtime-proof", "related");
}

#[test]
fn reports_missing_targets_and_required_reference_fields() {
    let validation = validate_fixture("missing");

    let missing_target = find_code(&validation, "missing_reference");
    assert_eq!(
        missing_target.path.as_deref(),
        Some(Path::new("docs/goals/goal_portable_structure.md"))
    );
    assert_eq!(missing_target.object_type.as_deref(), Some("Goal"));
    assert_eq!(missing_target.field.as_deref(), Some("specs"));
    assert_eq!(
        missing_target.referenced_object.as_deref(),
        Some("specs:missing-spec")
    );

    let missing_field = validation
        .findings
        .iter()
        .find(|finding| {
            finding.code == "missing_reference_field"
                && finding.path.as_deref()
                    == Some(Path::new("decisions/decision_runtime_shape.yml"))
        })
        .expect("required empty many reference is reported");
    assert_eq!(missing_field.object_type.as_deref(), Some("Decision"));
    assert_eq!(missing_field.field.as_deref(), Some("events"));
}

#[test]
fn reports_duplicate_ids_without_overwriting_first_record() {
    let validation = validate_fixture("duplicate");

    let duplicate = find_code(&validation, "duplicate_object_id");
    assert_eq!(
        duplicate.path.as_deref(),
        Some(Path::new("specs/z_spec_duplicate.json"))
    );
    assert_eq!(duplicate.object_type.as_deref(), Some("Spec"));
    assert_eq!(duplicate.field.as_deref(), Some("id"));

    let spec = validation
        .snapshot
        .objects
        .get(&("specs".to_string(), "spec-portable-structure".to_string()))
        .expect("original spec remains indexed");
    assert_eq!(
        spec.rel_path,
        Path::new("specs/spec_portable_structure.json")
    );
    assert_eq!(
        spec.data.get("title").and_then(Value::as_str),
        Some("Portable structure and frontmatter")
    );
}

#[test]
fn treats_optional_empty_scalar_references_as_absent() {
    let fixture = tempfile::tempdir().expect("tempdir");
    let root = fixture.path();
    std::fs::create_dir_all(root.join("events")).expect("events dir");
    std::fs::write(
        root.join("events/event.json"),
        r#"{"id":"event-1","related":""}"#,
    )
    .expect("event fixture");

    let repository = ContentRepository::try_new(RepositoryModel {
        collections: vec![CollectionSpec {
            name: "events".to_string(),
            object_type: "Event".to_string(),
            schema_class: None,
            path_pattern: "events/*.json".to_string(),
            adapter: AdapterKind::JsonRecord,
            id_field: "id".to_string(),
            fields: Vec::new(),
            references: vec![ReferenceSpec {
                field: "related".to_string(),
                target_collections: vec!["goals".to_string()],
                many: false,
                required: false,
                acyclic: false,
            }],
        }],
        placements: vec![PlacementRule::recursive("events", ["Event"])],
        schema_artifact_path: None,
        schema_artifact: None,
    })
    .expect("repository compiles");

    let validation = repository.validate(root);

    assert_eq!(validation.findings, Vec::new());
    assert_eq!(validation.snapshot.edges, Vec::new());
}

#[test]
fn reports_ambiguous_multi_target_references() {
    let validation = validate_fixture("ambiguous");

    let ambiguous = find_code(&validation, "ambiguous_reference");
    assert_eq!(
        ambiguous.path.as_deref(),
        Some(Path::new("events/events.jsonl"))
    );
    assert_eq!(ambiguous.object_type.as_deref(), Some("Event"));
    assert_eq!(ambiguous.field.as_deref(), Some("related"));
    assert_eq!(
        ambiguous.referenced_object.as_deref(),
        Some("goals:shared, specs:shared")
    );
}

#[test]
fn reports_configured_acyclic_reference_cycles() {
    let validation = validate_fixture("cycle");

    let cycle = find_code(&validation, "cyclic_reference");
    assert_eq!(
        cycle.path.as_deref(),
        Some(Path::new("events/events.jsonl"))
    );
    assert_eq!(cycle.object_type.as_deref(), Some("Event"));
    assert_eq!(cycle.field.as_deref(), Some("parent"));
    assert!(
        matches!(
            cycle.referenced_object.as_deref(),
            Some("events:event-a") | Some("events:event-b")
        ),
        "{cycle:?}"
    );
}

#[test]
fn rejects_ambiguous_relation_configuration() {
    let root = fixture_path("valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config.relations.insert(
        "events.bad".to_string(),
        ContentRelationConfig {
            target: Some("goals".to_string()),
            targets: vec!["specs".to_string()],
            many: false,
            required: false,
            acyclic: false,
        },
    );
    config.relations.insert(
        "events.inferred_parent".to_string(),
        ContentRelationConfig {
            target: None,
            targets: Vec::new(),
            many: false,
            required: false,
            acyclic: true,
        },
    );

    let error = match ContentRepository::from_config(&root, &config) {
        Ok(_) => panic!("invalid relation config must be rejected"),
        Err(error) => error,
    };
    let messages = error
        .iter()
        .filter(|finding| finding.code == "invalid_content_relation")
        .map(|finding| finding.message.as_str())
        .collect::<Vec<_>>();
    assert!(messages
        .iter()
        .any(|message| message.contains("must use either target or targets")));
    assert!(messages
        .iter()
        .any(|message| message.contains("must declare target or targets")));
}

#[test]
fn check_json_reports_ambiguous_reference_diagnostics() {
    let output = check_fixture("ambiguous", &["--format", "json"]);

    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout:\n{}\nstderr:\n{}",
        stdout(&output),
        stderr(&output)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let violation = report["violations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|violation| violation["rule"] == "content_runtime:ambiguous_reference")
        .expect("ambiguous-reference violation is emitted");

    assert_eq!(json_path(&violation["path"]), "events/events.jsonl");
    let message = violation["message"].as_str().unwrap();
    assert!(message.contains("object_type=Event"), "{message}");
    assert!(message.contains("field=related"), "{message}");
    assert!(
        message.contains("referenced_object=goals:shared, specs:shared"),
        "{message}"
    );
}

fn validate_fixture(name: &str) -> RepositoryValidation {
    let root = fixture_path(name);
    let config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    let repository =
        ContentRepository::from_config(&root, &config).expect("content repository compiles");
    repository.validate(&root)
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(FIXTURE_ROOT).join(name)
}

fn assert_object_exists(validation: &RepositoryValidation, collection: &str, id: &str) {
    assert!(
        validation
            .snapshot
            .objects
            .contains_key(&(collection.to_string(), id.to_string())),
        "{collection}:{id} exists"
    );
}

fn assert_edge(validation: &RepositoryValidation, collection: &str, id: &str, field: &str) {
    assert!(
        validation.snapshot.edges.iter().any(|edge| {
            edge.source.collection == collection && edge.source.id == id && edge.field == field
        }),
        "{collection}:{id}.{field} edge exists"
    );
}

fn find_code<'a>(validation: &'a RepositoryValidation, code: &str) -> &'a ContentFinding {
    validation
        .findings
        .iter()
        .find(|finding| finding.code == code)
        .unwrap_or_else(|| panic!("{code} finding exists: {:#?}", validation.findings))
}

fn assura_bin() -> &'static str {
    env!("CARGO_BIN_EXE_assura")
}

fn check_fixture(fixture: &str, args: &[&str]) -> Output {
    let mut command = Command::new(assura_bin());
    command
        .arg("check")
        .arg(format!("{FIXTURE_ROOT}/{fixture}"));
    for arg in args {
        command.arg(arg);
    }
    command.output().unwrap()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).replace("\r\n", "\n")
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).replace("\r\n", "\n")
}

fn json_path(value: &Value) -> String {
    value.as_str().unwrap().replace('\\', "/")
}
