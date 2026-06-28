use assura::config::loader::ConfigLoader;
use assura::content_repository::ContentRepository;
use serde_json::Value;
use std::fs;
use std::path::Path;

const GUIDE: &str = "docs/content-runtime-inspection.md";
const RUNTIME_GUIDE: &str = "docs/content-runtime.md";
const WEBSITE_CONFIG: &str = "website/astro.config.mjs";
const WEBSITE_EXAMPLE: &str = "website/src/content/docs/examples/content-runtime.md";
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
fn content_runtime_user_docs_cover_examples_writes_and_website_page() {
    let guide = fs::read_to_string(RUNTIME_GUIDE).expect("runtime guide");
    let website_config = fs::read_to_string(WEBSITE_CONFIG).expect("website config");
    let website = fs::read_to_string(WEBSITE_EXAMPLE).expect("website content runtime example");

    for required in [
        "Fixture And Example Matrix",
        "Agent Operation Contract",
        "tests/fixtures/content_runtime/valid",
        "tests/fixtures/content_runtime/adapters/yaml/valid",
        "tests/fixtures/content_runtime/adapters/jsonl/valid",
        "tests/fixtures/content_runtime/references/",
        "tests/content_runtime_create.rs",
        "tests/content_runtime_update.rs",
        "tests/content_runtime_adapters.rs",
        "tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/linkml_profile.runtime.schema.json",
        "cargo test --test content_runtime_create --quiet",
        "cargo test --test content_runtime_update --quiet",
        "cargo test --test content_runtime_adapters --quiet",
        "cargo test --test content_runtime_references --quiet",
        "assura check --format json .",
        "Markdown updates preserve the existing body bytes",
        "Adoption Path",
        "content_runtime:invalid_object_shape",
        "content_runtime:missing_reference",
        "content_runtime:duplicate_object_id",
        "content_runtime:ambiguous_reference",
        "content_runtime:cyclic_reference",
    ] {
        assert!(
            guide.contains(required),
            "runtime guide should contain {required}"
        );
    }

    for required in [
        "adapter: markdown_frontmatter",
        "adapter: json_record",
        "yaml_record",
        "jsonl_record",
        "assura check --format json .",
        "tests/content_runtime_create.rs",
        "tests/content_runtime_update.rs",
        "docs/content-runtime.md",
        "docs/content-runtime-inspection.md",
    ] {
        assert!(
            website.contains(required),
            "website example should contain {required}"
        );
    }

    assert!(
        website_config.contains("slug: 'examples/content-runtime'"),
        "website sidebar should link the content runtime example"
    );

    for path in [
        WEBSITE_CONFIG,
        WEBSITE_EXAMPLE,
        "tests/fixtures/content_runtime/adapters/yaml/valid/.assura/config.yml",
        "tests/fixtures/content_runtime/adapters/jsonl/valid/.assura/config.yml",
        "tests/content_runtime_create.rs",
        "tests/content_runtime_update.rs",
        "tests/content_runtime_adapters.rs",
        "tests/content_runtime_references.rs",
        "tests/fixtures/artifact_modeling_options/authoring_paths/generated_outputs/linkml_profile.runtime.schema.json",
    ] {
        assert!(Path::new(path).exists(), "documented path should exist: {path}");
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
