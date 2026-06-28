use assura::config::config::ContentRelationConfig;
use assura::config::loader::ConfigLoader;
use assura::content_repository::{ContentRepository, RepositoryValidation};
use std::path::{Path, PathBuf};

const FIXTURE_ROOT: &str = "tests/fixtures/content_runtime";

#[test]
fn validates_configured_markdown_and_json_collections() {
    let validation = validate_fixture("valid");

    assert_eq!(validation.findings, Vec::new());
    assert!(validation
        .snapshot
        .objects
        .contains_key(&("goals".to_string(), "goal-portable-structure".to_string())));
    assert!(validation
        .snapshot
        .objects
        .contains_key(&("specs".to_string(), "spec-portable-structure".to_string())));

    let goal = validation
        .snapshot
        .objects
        .get(&("goals".to_string(), "goal-portable-structure".to_string()))
        .expect("goal object exists");
    assert_eq!(goal.object_type, "Goal");
    assert_eq!(
        goal.rel_path,
        Path::new("docs/goals/goal_portable_structure.md")
    );
    assert_eq!(
        goal.body.as_deref().map(normalize_newlines).as_deref(),
        Some("# Portable Structure Policy\n\nAssura constrains markdown and frontmatter without requiring a project language.\n")
    );
}

#[test]
fn reports_shape_errors_with_source_path_and_field() {
    let validation = validate_fixture("invalid_shape");
    let finding = validation
        .findings
        .iter()
        .find(|finding| finding.code == "invalid_object_shape")
        .expect("shape violation is reported");

    assert_eq!(
        finding.path.as_deref(),
        Some(Path::new("docs/goals/goal_portable_structure.md"))
    );
    assert_eq!(finding.object_type.as_deref(), Some("Goal"));
    assert_eq!(finding.field.as_deref(), Some("status"));
}

#[test]
fn reports_json_shape_errors_with_source_path_and_field() {
    let validation = validate_fixture("invalid_shape");
    let finding = validation
        .findings
        .iter()
        .find(|finding| {
            finding.code == "invalid_object_shape" && finding.object_type.as_deref() == Some("Spec")
        })
        .expect("json shape violation is reported");

    assert_eq!(
        finding.path.as_deref(),
        Some(Path::new("specs/spec_portable_structure.json"))
    );
    assert_eq!(finding.field.as_deref(), Some("status"));
}

#[test]
fn reports_missing_references_with_source_field_and_target() {
    let validation = validate_fixture("missing_reference");
    let finding = validation
        .findings
        .iter()
        .find(|finding| finding.code == "missing_reference")
        .expect("missing reference is reported");

    assert_eq!(
        finding.path.as_deref(),
        Some(Path::new("docs/goals/goal_portable_structure.md"))
    );
    assert_eq!(finding.object_type.as_deref(), Some("Goal"));
    assert_eq!(finding.field.as_deref(), Some("specs"));
    assert_eq!(
        finding.referenced_object.as_deref(),
        Some("specs:missing-spec")
    );
    assert!(finding.message.contains(
        "goals:goal-portable-structure field 'specs' references missing specs:missing-spec"
    ));
}

fn validate_fixture(name: &str) -> RepositoryValidation {
    let root = PathBuf::from(FIXTURE_ROOT).join(name);
    let config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    let repository =
        ContentRepository::from_config(&root, &config).expect("content repository compiles");
    repository.validate(&root)
}

fn normalize_newlines(value: &str) -> String {
    value.replace("\r\n", "\n")
}

#[test]
fn rejects_configured_collections_without_runtime_schema() {
    let root = PathBuf::from(FIXTURE_ROOT).join("missing_schema");
    let config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    let error = match ContentRepository::from_config(&root, &config) {
        Ok(_) => panic!("schema artifact is required"),
        Err(error) => error,
    };
    assert!(error
        .iter()
        .any(|finding| finding.code == "content_schema_missing"));
}

#[test]
fn rejects_runtime_schema_paths_outside_project() {
    let root = PathBuf::from(FIXTURE_ROOT).join("valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config
        .models
        .as_mut()
        .expect("fixture declares runtime model")
        .validation_artifact = "../content_runtime.schema.json".to_string();

    let error = match ContentRepository::from_config(&root, &config) {
        Ok(_) => panic!("schema artifact path escapes project root"),
        Err(error) => error,
    };
    assert!(error
        .iter()
        .any(|finding| finding.code == "content_schema_path_escape"));
}

#[test]
fn rejects_malformed_or_unknown_relation_config() {
    let root = PathBuf::from(FIXTURE_ROOT).join("valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config.relations.insert(
        "goal.specs".to_string(),
        ContentRelationConfig {
            target: "specs".to_string(),
            many: true,
        },
    );
    config.relations.insert(
        "goals.unknown_target".to_string(),
        ContentRelationConfig {
            target: "specz".to_string(),
            many: true,
        },
    );

    let error = match ContentRepository::from_config(&root, &config) {
        Ok(_) => panic!("relation config must reject typos"),
        Err(error) => error,
    };
    assert!(error
        .iter()
        .any(|finding| finding.code == "unknown_content_relation_source"));
    assert!(error
        .iter()
        .any(|finding| finding.code == "unknown_content_relation_target"));
}
