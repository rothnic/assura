use assura::config::loader::ConfigLoader;
use assura::content_repository::ContentRepository;
use serde_json::Value;
use std::fs;
use std::path::Path;

const GUIDE: &str = "docs/content-runtime-inspection.md";
const CONTENT_RUNTIME_FIXTURE: &str = "tests/fixtures/content_runtime/valid";
const AUTHORING_ROOT: &str = "tests/fixtures/artifact_modeling_options/authoring_paths";

#[test]
fn content_runtime_inspection_guide_points_to_real_artifacts_and_commands() {
    let guide = fs::read_to_string(GUIDE).expect("inspection guide");

    for required in [
        "TypeScript Inspection",
        "Python Inspection",
        "Rust Inspection",
        "docs/goals/goal_model_runtime.md",
        "specs/spec_artifact_runtime.json",
        "linkml_profile.runtime.schema.json",
        "selected_authoring_profile.compile.json",
        "assura check --format json tests/fixtures/content_runtime/valid",
        "assura check --format json tests/fixtures/content_runtime/missing_reference",
        "cargo run --quiet -- check --format json tests/fixtures/content_runtime/valid",
        "cargo run --quiet -- check --format json tests/fixtures/content_runtime/missing_reference",
        "cargo test --test artifact_authoring_paths_proof --quiet",
    ] {
        assert!(guide.contains(required), "guide should contain {required}");
    }

    for path in [
        format!("{AUTHORING_ROOT}/fixtures/pass/docs/goals/goal_model_runtime.md"),
        format!("{AUTHORING_ROOT}/fixtures/pass/specs/spec_artifact_runtime.json"),
        format!("{AUTHORING_ROOT}/generated_outputs/linkml_profile.runtime.schema.json"),
        format!("{AUTHORING_ROOT}/generated_outputs/selected_authoring_profile.compile.json"),
    ] {
        assert!(Path::new(&path).exists(), "guide path should exist: {path}");
    }
}

#[test]
fn content_runtime_inspection_schema_exposes_shape_collections_and_relations() {
    let schema_path =
        format!("{AUTHORING_ROOT}/generated_outputs/linkml_profile.runtime.schema.json");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(schema_path).expect("runtime schema fixture"))
            .expect("runtime schema json");

    assert_eq!(schema["$defs"]["Goal"]["required"][0], "id");
    assert_eq!(
        schema["$defs"]["Goal"]["properties"]["specs"]["items"]["type"],
        "string"
    );
    assert!(schema["x-assura"]["collections"]
        .as_array()
        .expect("collections")
        .iter()
        .any(|collection| {
            collection["class"] == "Goal"
                && collection["path"] == "docs/goals/*.md"
                && collection["adapter"] == "markdown_frontmatter"
        }));
    assert!(schema["x-assura"]["relations"]
        .as_array()
        .expect("relations")
        .iter()
        .any(|relation| relation["from"] == "Goal.specs" && relation["to"] == "Spec.id"));
}

#[test]
fn documented_content_runtime_fixture_validates_without_authoring_tools() {
    let root = Path::new(CONTENT_RUNTIME_FIXTURE);
    let config = ConfigLoader::load(&root.join(".assura/config.yml")).expect("fixture config");
    let repository = ContentRepository::from_config(root, &config).expect("content repository");
    let validation = repository.validate(root);

    assert_eq!(validation.findings, Vec::new());
}
