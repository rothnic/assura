use assura::config::loader::ConfigLoader;
use assura::content_repository::{ContentRepository, RepositoryModel};
use assura::intelligence::{model_instance_id, FactIngestor, InMemoryFactStore, ProjectFact};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

const CODE_SYMBOL_FIXTURE: &str = "tests/fixtures/content_runtime/code_symbols";

#[test]
fn code_symbol_config_models_fields_and_providers() {
    let root = Path::new(CODE_SYMBOL_FIXTURE);
    let config = ConfigLoader::load(&root.join(".assura/config.yml")).unwrap();
    let model = RepositoryModel::from_config(root, &config).unwrap();
    let components = model
        .collections
        .iter()
        .find(|collection| collection.name == "components")
        .expect("components collection");

    assert_eq!(components.code_symbols.len(), 2);
    assert!(components.code_symbols.iter().any(|symbol| {
        symbol.field == "implementation"
            && symbol.provider.as_deref() == Some("rust-token-baseline-v1")
            && !symbol.many
    }));
    assert!(components.code_symbols.iter().any(|symbol| {
        symbol.field == "external_symbol"
            && symbol.provider.as_deref() == Some("external-index-v1")
            && !symbol.many
    }));
}

#[test]
fn code_symbol_ingestion_resolves_baseline_and_preserves_unresolved_refs() {
    let root = Path::new(CODE_SYMBOL_FIXTURE);
    let config = ConfigLoader::load(&root.join(".assura/config.yml")).unwrap();
    let model = RepositoryModel::from_config(root, &config).unwrap();
    let repository = ContentRepository::try_new(model.clone()).unwrap();
    let validation = repository.validate(root);
    assert_eq!(validation.findings, Vec::new());

    let mut ingestor = FactIngestor::new("code-symbol-fixture");
    ingestor.ingest_repository_model(&model);
    ingestor.ingest_local_rust_code_symbols(root);
    ingestor.ingest_repository_validation(&validation);
    ingestor.ingest_content_code_symbol_refs(&model, &validation);
    let facts = ingestor.finish();
    let store = InMemoryFactStore::load(facts.clone());

    assert_eq!(facts.count_kind("CodeProviderEvidence"), 1);
    assert!(facts.facts.iter().any(|fact| match fact {
        ProjectFact::CodeSymbol(symbol) => {
            symbol.symbol == "Config"
                && symbol.provider == "rust-token-baseline-v1"
                && symbol.evidence == "baseline"
                && symbol
                    .location
                    .as_ref()
                    .is_some_and(|location| location.path == Path::new("src/sample.rs"))
        }
        _ => false,
    }));

    let source_id = model_instance_id("components", "component-config");
    let refs = store
        .symbol_refs()
        .into_iter()
        .filter(|edge| edge.source_id == source_id)
        .collect::<Vec<_>>();
    assert_eq!(refs.len(), 2);
    assert!(refs.iter().any(|edge| {
        edge.symbol == "crate::sample::Config"
            && edge.field.as_deref() == Some("implementation")
            && edge.provider.as_deref() == Some("rust-token-baseline-v1")
            && edge.target_id.is_some()
    }));
    assert!(refs.iter().any(|edge| {
        edge.symbol == "external::Runtime"
            && edge.field.as_deref() == Some("external_symbol")
            && edge.provider.as_deref() == Some("external-index-v1")
            && edge.target_id.is_none()
    }));
}

#[test]
fn duplicate_baseline_symbols_leave_refs_unresolved() {
    let project = temp_code_symbol_project(
        r#"
code_symbols:
  components.implementation:
    provider: rust-token-baseline-v1
"#,
        r#"
{
  "id": "component-config",
  "title": "Configuration component",
  "status": "active",
  "implementation": "crate::sample::Config"
}
"#,
        &["src/sample.rs", "src/alternate.rs"],
    );
    let (facts, store) = ingest_project(project.path());

    let config_symbols = facts
        .facts
        .iter()
        .filter(|fact| match fact {
            ProjectFact::CodeSymbol(symbol) => symbol.symbol == "Config",
            _ => false,
        })
        .count();
    assert_eq!(config_symbols, 2);

    let refs = refs_for_component(&store);
    assert_eq!(refs.len(), 1);
    assert_eq!(refs[0].field.as_deref(), Some("implementation"));
    assert!(refs[0].target_id.is_none());
}

#[test]
fn duplicate_configured_fields_keep_distinct_symbol_refs() {
    let project = temp_code_symbol_project(
        r#"
code_symbols:
  components.implementation:
    provider: rust-token-baseline-v1
  components.secondary_implementation:
    provider: rust-token-baseline-v1
"#,
        r#"
{
  "id": "component-config",
  "title": "Configuration component",
  "status": "active",
  "implementation": "crate::sample::Config",
  "secondary_implementation": "crate::sample::Config"
}
"#,
        &["src/sample.rs"],
    );
    let (_facts, store) = ingest_project(project.path());

    let refs = refs_for_component(&store);
    assert_eq!(refs.len(), 2);
    assert!(refs.iter().any(|edge| {
        edge.field.as_deref() == Some("implementation")
            && edge.symbol == "crate::sample::Config"
            && edge.target_id.is_some()
    }));
    assert!(refs.iter().any(|edge| {
        edge.field.as_deref() == Some("secondary_implementation")
            && edge.symbol == "crate::sample::Config"
            && edge.target_id.is_some()
    }));
}

fn ingest_project(root: &Path) -> (assura::intelligence::FactSet, InMemoryFactStore) {
    let config = ConfigLoader::load(&root.join(".assura/config.yml")).unwrap();
    let model = RepositoryModel::from_config(root, &config).unwrap();
    let repository = ContentRepository::try_new(model.clone()).unwrap();
    let validation = repository.validate(root);
    assert_eq!(validation.findings, Vec::new());

    let mut ingestor = FactIngestor::new("code-symbol-regression");
    ingestor.ingest_repository_model(&model);
    ingestor.ingest_local_rust_code_symbols(root);
    ingestor.ingest_repository_validation(&validation);
    ingestor.ingest_content_code_symbol_refs(&model, &validation);
    let facts = ingestor.finish();
    let store = InMemoryFactStore::load(facts.clone());
    (facts, store)
}

fn refs_for_component(store: &InMemoryFactStore) -> Vec<&assura::intelligence::SymbolRef> {
    let source_id = model_instance_id("components", "component-config");
    store
        .symbol_refs()
        .into_iter()
        .filter(|edge| edge.source_id == source_id)
        .collect()
}

fn temp_code_symbol_project(
    code_symbols: &str,
    component_json: &str,
    rust_files: &[&str],
) -> TempDir {
    let project = TempDir::new().unwrap();
    fs::create_dir_all(project.path().join(".assura")).unwrap();
    fs::create_dir_all(project.path().join("components")).unwrap();
    fs::create_dir_all(project.path().join("schemas")).unwrap();
    fs::create_dir_all(project.path().join("src")).unwrap();
    fs::write(
        project.path().join(".assura/config.yml"),
        format!(
            r#"structure:
  ./:
    required: false

models:
  source: schemas/content-runtime.linkml.yaml
  validation_artifact: schemas/content_runtime.schema.json

collections:
  components:
    class: Component
    path: components/*.json
    adapter: json_record
    id: id
{code_symbols}
"#
        ),
    )
    .unwrap();
    fs::write(
        project.path().join("schemas/content_runtime.schema.json"),
        component_schema(),
    )
    .unwrap();
    fs::write(
        project.path().join("components/component_config.json"),
        component_json,
    )
    .unwrap();
    for file in rust_files {
        fs::write(project.path().join(file), "pub struct Config;\n").unwrap();
    }
    project
}

fn component_schema() -> &'static str {
    r#"
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$defs": {
    "Component": {
      "type": "object",
      "additionalProperties": false,
      "required": ["id", "title", "status", "implementation"],
      "properties": {
        "id": { "type": "string", "minLength": 1 },
        "title": { "type": "string", "minLength": 1 },
        "status": { "type": "string", "enum": ["planned", "active", "complete"] },
        "implementation": { "type": "string", "minLength": 1 },
        "secondary_implementation": { "type": "string", "minLength": 1 }
      }
    }
  }
}
"#
}
