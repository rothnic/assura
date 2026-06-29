use assura::config::loader::ConfigLoader;
use assura::content_repository::ContentRepository;
use serde_json::Value;
use std::fs;
use std::path::Path;

const GUIDE: &str = "docs/content-runtime-inspection.md";
const RUNTIME_GUIDE: &str = "docs/content-runtime.md";
const WEBSITE_CONFIG: &str = "website/astro.config.mjs";
const WEBSITE_EXAMPLE: &str = "website/src/content/docs/examples/content-runtime.md";
const WEBSITE_PROJECT_INTELLIGENCE_DEMO: &str =
    "website/src/content/docs/examples/project-intelligence-demo.md";
const WEBSITE_CONTENT_MODELS: &str = "website/src/content/docs/product/content-models.md";
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
    let content_models =
        fs::read_to_string(WEBSITE_CONTENT_MODELS).expect("website content models page");

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
    for required in [
        "label: 'Product Layers'",
        "slug: 'product/structure-validation'",
        "slug: 'product/markdown-validation'",
        "Content Runtime And Models",
        "slug: 'product/content-models'",
        "slug: 'product/query-search'",
        "slug: 'product/code-intelligence'",
        "slug: 'product/agent-editor-surfaces'",
    ] {
        assert!(
            website_config.contains(required),
            "website sidebar should contain {required}"
        );
    }
    for required in [
        "The content runtime makes ordinary repository files addressable as typed",
        "objects. Content models define those objects",
        "markdown_frontmatter",
        "JSON/YAML/JSONL adapters",
        "Agent create/update operations",
        "docs/content-runtime.md",
    ] {
        assert!(
            content_models.contains(required),
            "content models page should contain {required}"
        );
    }

    for path in [
        WEBSITE_CONFIG,
        WEBSITE_EXAMPLE,
        WEBSITE_CONTENT_MODELS,
        "website/src/content/docs/product/structure-validation.md",
        "website/src/content/docs/product/markdown-validation.md",
        "website/src/content/docs/product/query-search.md",
        "website/src/content/docs/product/code-intelligence.md",
        "website/src/content/docs/product/agent-editor-surfaces.md",
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
fn project_intelligence_demo_is_discoverable_and_covers_adoption_commands() {
    let website_config = fs::read_to_string(WEBSITE_CONFIG).expect("website config");
    let demo =
        fs::read_to_string(WEBSITE_PROJECT_INTELLIGENCE_DEMO).expect("project intelligence demo");
    let content_runtime = fs::read_to_string(WEBSITE_EXAMPLE).expect("content runtime example");

    assert!(
        website_config.contains("slug: 'examples/project-intelligence-demo'"),
        "website sidebar should link the project intelligence demo"
    );
    assert!(
        content_runtime.contains("/examples/project-intelligence-demo/"),
        "content runtime example should link the project intelligence demo"
    );

    for required in [
        "pi-demo-flow",
        "pi-demo-board",
        "assura init --project-intelligence --no-git-hooks .",
        "schemas/project-intelligence-starter.schema.json",
        "assura content search \"Adopt Project Intelligence\" . --format json",
        "assura content expand goals goal-project-intelligence-starter . --format json",
        "docs/examples/project-intelligence-broken-goal.md",
        "assura content missing-relations . --format json",
        "assura content context-pack tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid --text checkout --limit 5 --format json",
        "assura.project-intelligence.context-pack.v1",
        "assura content context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program",
        "assura content session .",
        "assura.project-intelligence.session.response.v1",
        "\"type\":\"context-pack\"",
        "`agent-context`, `collections`",
        "`request_failed`",
        "`reload.state` as `initial_load`, `reused`, or `reloaded`",
        "Agent Editing Handoff",
        "Inspect these response fields before editing",
        "do not apply safe fixes automatically",
        "Expected evidence: validation succeeds",
        "assura check --format json tests/fixtures/content_runtime/valid",
        "assura content search \"Portable Structure\" tests/fixtures/content_runtime/valid --format json",
        "assura content expand goals goal-portable-structure tests/fixtures/content_runtime/valid --format json",
        "assura content missing-relations tests/fixtures/content_runtime/missing_reference --format json",
        "assura content agent-query diagnostics tests/fixtures/content_runtime/missing_reference --format json",
        "assura content agent-context",
        "assura.safe-fix.markdown.v1",
        "files_would_change",
        "assura fix markdown --rule trailing-spaces --apply --format json",
        "applied_fix_ids",
        "\"audit_id\"",
        "goal-assura-project-intelligence-usability-program",
        "tests/fixtures/project_intelligence_real_repo/beacon_crm/valid",
        "docs/analysis/2026-06-29-project-intelligence-real-repo-proof.md",
        "does not require a daemon",
    ] {
        assert!(
            demo.contains(required),
            "project intelligence demo should contain {required}"
        );
    }

    for path in [
        WEBSITE_PROJECT_INTELLIGENCE_DEMO,
        "tests/fixtures/content_runtime/valid/.assura/config.yml",
        "tests/fixtures/content_runtime/missing_reference/.assura/config.yml",
    ] {
        assert!(
            Path::new(path).exists(),
            "documented path should exist: {path}"
        );
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
