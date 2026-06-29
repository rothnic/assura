use assura::config::config::ContentRelationConfig;
use assura::config::loader::ConfigLoader;
use assura::content_repository::{ContentRepository, RepositoryValidation};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

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

#[test]
fn reports_missing_markdown_frontmatter_fields_through_content_model() {
    let validation = validate_fixture("missing_model_frontmatter_field");
    let finding = validation
        .findings
        .iter()
        .find(|finding| {
            finding.code == "invalid_object_shape"
                && finding.object_type.as_deref() == Some("Goal")
                && finding.field.as_deref() == Some("title")
        })
        .expect("missing required frontmatter field is reported by content model");

    assert_eq!(
        finding.path.as_deref(),
        Some(Path::new("docs/goals/goal_portable_structure.md"))
    );
    assert!(finding.message.contains("does not match runtime schema"));
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
fn rejects_assura_root_model_artifacts() {
    let root = PathBuf::from(FIXTURE_ROOT).join("valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config
        .models
        .as_mut()
        .expect("fixture declares runtime model")
        .validation_artifact = ".assura/content_runtime.schema.json".to_string();

    let error = match ContentRepository::from_config(&root, &config) {
        Ok(_) => panic!("root-level .assura model artifact should be rejected"),
        Err(error) => error,
    };
    let finding = error
        .iter()
        .find(|finding| finding.code == "content_model_artifact_outside_models_dir")
        .expect("layout finding is reported");
    assert_eq!(
        finding.path.as_deref(),
        Some(Path::new(".assura/content_runtime.schema.json"))
    );
    assert!(finding.message.contains(".assura/models/**"));
}

#[test]
fn rejects_dot_prefixed_assura_root_model_artifacts() {
    let root = PathBuf::from(FIXTURE_ROOT).join("valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config
        .models
        .as_mut()
        .expect("fixture declares runtime model")
        .validation_artifact = "./.assura/content_runtime.schema.json".to_string();

    let error = match ContentRepository::from_config(&root, &config) {
        Ok(_) => panic!("dot-prefixed root-level .assura model artifact should be rejected"),
        Err(error) => error,
    };
    let finding = error
        .iter()
        .find(|finding| finding.code == "content_model_artifact_outside_models_dir")
        .expect("layout finding is reported");
    assert_eq!(
        finding.path.as_deref(),
        Some(Path::new(".assura/content_runtime.schema.json"))
    );
}

#[test]
fn rejects_assura_root_model_source_artifacts() {
    let root = PathBuf::from(FIXTURE_ROOT).join("valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config
        .models
        .as_mut()
        .expect("fixture declares runtime model")
        .source = Some(".assura/content_runtime.linkml.yaml".to_string());

    let error = match ContentRepository::from_config(&root, &config) {
        Ok(_) => panic!("root-level .assura model source should be rejected"),
        Err(error) => error,
    };
    assert!(error
        .iter()
        .any(|finding| finding.code == "content_model_artifact_outside_models_dir"));
}

#[test]
fn rejects_dot_prefixed_assura_root_model_source_artifacts() {
    let root = PathBuf::from(FIXTURE_ROOT).join("valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config
        .models
        .as_mut()
        .expect("fixture declares runtime model")
        .source = Some("./.assura/content_runtime.linkml.yaml".to_string());

    let error = match ContentRepository::from_config(&root, &config) {
        Ok(_) => panic!("dot-prefixed root-level .assura model source should be rejected"),
        Err(error) => error,
    };
    let finding = error
        .iter()
        .find(|finding| finding.code == "content_model_artifact_outside_models_dir")
        .expect("layout finding is reported");
    assert_eq!(
        finding.path.as_deref(),
        Some(Path::new(".assura/content_runtime.linkml.yaml"))
    );
}

#[test]
fn accepts_nested_assura_models_artifact_paths() {
    let project = TempDir::new().expect("temp project");
    copy_dir_all(&PathBuf::from(FIXTURE_ROOT).join("valid"), project.path())
        .expect("fixture copy succeeds");
    let model_dir = project.path().join(".assura/models/content-runtime/v1");
    fs::create_dir_all(&model_dir).expect("model dir created");
    fs::copy(
        project.path().join("schemas/content_runtime.schema.json"),
        model_dir.join("runtime.schema.json"),
    )
    .expect("schema copied");

    let mut config =
        ConfigLoader::load(&project.path().join(".assura/config.yml")).expect("config loads");
    let models = config
        .models
        .as_mut()
        .expect("fixture declares runtime model");
    models.source = Some(".assura/models/content-runtime/source.linkml.yaml".to_string());
    models.validation_artifact =
        ".assura/models/content-runtime/v1/runtime.schema.json".to_string();

    let repository = ContentRepository::from_config(project.path(), &config)
        .expect("nested .assura/models artifact compiles");
    assert_eq!(repository.validate(project.path()).findings, Vec::new());
}

#[test]
fn rejects_malformed_or_unknown_relation_config() {
    let root = PathBuf::from(FIXTURE_ROOT).join("valid");
    let mut config =
        ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config loads");
    config.relations.insert(
        "goal.specs".to_string(),
        ContentRelationConfig {
            target: Some("specs".to_string()),
            targets: Vec::new(),
            many: true,
            required: false,
            acyclic: false,
        },
    );
    config.relations.insert(
        "goals.unknown_target".to_string(),
        ContentRelationConfig {
            target: Some("specz".to_string()),
            targets: Vec::new(),
            many: true,
            required: false,
            acyclic: false,
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

fn copy_dir_all(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let destination = to.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &destination)?;
        } else {
            fs::copy(entry.path(), destination)?;
        }
    }
    Ok(())
}
