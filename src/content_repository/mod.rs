//! Repo-native content runtime validation.
//!
//! The runtime treats files as typed repository objects while keeping the files
//! themselves as canonical state.

mod adapters;
mod model;
#[cfg(test)]
mod tests;
mod validation;

use adapters::parse_object;
use jsonschema::Validator;
pub use model::{
    AdapterKind, CollectionSpec, ContentFinding, FieldKind, FieldSpec, MarkdownHeading, ObjectKey,
    PlacementRule, ReferenceSpec, RepositoryModel, RepositorySnapshot, RepositoryValidation,
};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use validation::{
    edge_from, invalid_reference_scalar, validate_object_data, validate_placement,
    validate_references,
};

/// File-native content repository runner.
pub struct ContentRepository {
    model: RepositoryModel,
    schema_validators: HashMap<String, Validator>,
}

impl ContentRepository {
    /// Create a repository runner with compiled runtime schema validators.
    pub fn try_new(model: RepositoryModel) -> Result<Self, Vec<ContentFinding>> {
        let schema_validators = compile_schema_validators(&model)?;
        Ok(Self {
            model,
            schema_validators,
        })
    }

    /// Create a repository runner from parsed Assura configuration.
    pub fn from_config(
        project_root: &Path,
        config: &crate::config::config::Config,
    ) -> Result<Self, Vec<ContentFinding>> {
        let model = RepositoryModel::from_config(project_root, config)?;
        Self::try_new(model)
    }

    /// Load and validate all configured collections under `root`.
    pub fn validate(&self, root: &Path) -> RepositoryValidation {
        let mut snapshot = RepositorySnapshot::default();
        let mut findings = Vec::new();

        for collection in &self.model.collections {
            self.load_collection(root, collection, &mut snapshot, &mut findings);
        }

        self.collect_edges(&mut snapshot, &mut findings);
        validate_references(&snapshot, &mut findings);

        RepositoryValidation { snapshot, findings }
    }

    fn load_collection(
        &self,
        root: &Path,
        collection: &CollectionSpec,
        snapshot: &mut RepositorySnapshot,
        findings: &mut Vec<ContentFinding>,
    ) {
        let pattern = match glob::Pattern::new(&normalize_slashes(&collection.path_pattern)) {
            Ok(pattern) => pattern,
            Err(error) => {
                findings.push(ContentFinding::new(
                    "invalid_pattern",
                    None,
                    format!(
                        "Collection '{}' has invalid path pattern '{}': {error}",
                        collection.name, collection.path_pattern
                    ),
                ));
                return;
            }
        };

        for entry in walkdir::WalkDir::new(root)
            .into_iter()
            .filter_entry(|entry| !is_ignored_dir(entry.path()))
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let Ok(rel_path) = entry.path().strip_prefix(root) else {
                continue;
            };
            let rel_key = normalize_slashes_path(rel_path);
            if !pattern.matches(&rel_key) {
                continue;
            }

            let content = match fs::read_to_string(entry.path()) {
                Ok(content) => content,
                Err(error) => {
                    findings.push(ContentFinding::new(
                        "read_error",
                        Some(rel_path.to_path_buf()),
                        format!("Failed to read '{}': {error}", rel_path.display()),
                    ));
                    continue;
                }
            };

            match parse_object(collection, rel_path, &content) {
                Ok(object) => {
                    validate_placement(&self.model, &object, findings);
                    validate_object_data(
                        collection,
                        &object,
                        schema_validator_for(collection, &self.schema_validators),
                        findings,
                    );

                    let key = (object.collection.clone(), object.id.clone());
                    if snapshot.objects.insert(key.clone(), object).is_some() {
                        findings.push(ContentFinding::new(
                            "duplicate_object_id",
                            Some(rel_path.to_path_buf()),
                            format!(
                                "Collection '{}' contains duplicate object id '{}'",
                                key.0, key.1
                            ),
                        ));
                    }
                }
                Err(finding) => findings.push(*finding),
            }
        }
    }

    fn collect_edges(&self, snapshot: &mut RepositorySnapshot, findings: &mut Vec<ContentFinding>) {
        let collections = self
            .model
            .collections
            .iter()
            .map(|collection| (collection.name.as_str(), collection))
            .collect::<HashMap<_, _>>();

        for object in snapshot.objects.values() {
            let Some(collection) = collections.get(object.collection.as_str()) else {
                continue;
            };
            for reference in &collection.references {
                let Some(value) = object.data.get(&reference.field) else {
                    continue;
                };
                if reference.many {
                    let Some(items) = value.as_array() else {
                        findings.push(
                            ContentFinding::new(
                                "invalid_reference_field",
                                Some(object.rel_path.clone()),
                                format!(
                                    "Reference field '{}' on '{}:{}' must be an array",
                                    reference.field, object.collection, object.id
                                ),
                            )
                            .with_field(reference.field.clone()),
                        );
                        continue;
                    };
                    for item in items {
                        if let Some(target_id) = item.as_str() {
                            snapshot.edges.push(edge_from(object, reference, target_id));
                        } else {
                            findings.push(invalid_reference_scalar(object, reference));
                        }
                    }
                } else if let Some(target_id) = value.as_str() {
                    snapshot.edges.push(edge_from(object, reference, target_id));
                } else {
                    findings.push(invalid_reference_scalar(object, reference));
                }
            }
        }
    }
}

fn is_ignored_dir(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| matches!(name, ".git" | "target" | "node_modules"))
}

fn normalize_slashes(value: &str) -> String {
    value.replace('\\', "/")
}

fn normalize_slashes_path(path: &Path) -> String {
    normalize_slashes(&path.to_string_lossy())
}

fn compile_schema_validators(
    model: &RepositoryModel,
) -> Result<HashMap<String, Validator>, Vec<ContentFinding>> {
    let Some(schema) = model.schema_artifact.as_ref() else {
        return Ok(HashMap::new());
    };

    let mut validators = HashMap::new();
    let mut findings = Vec::new();
    for collection in &model.collections {
        let Some(class_name) = collection.schema_class.as_ref() else {
            continue;
        };
        if validators.contains_key(class_name) {
            continue;
        }
        let compiled_schema = json!({
            "$schema": schema["$schema"].clone(),
            "$defs": schema["$defs"].clone(),
            "$ref": format!("#/$defs/{class_name}")
        });
        match jsonschema::validator_for(&compiled_schema) {
            Ok(validator) => {
                validators.insert(class_name.clone(), validator);
            }
            Err(error) => findings.push(
                ContentFinding::new(
                    "content_schema_compile_error",
                    model.schema_artifact_path.clone(),
                    format!(
                        "Failed to compile runtime schema class '{class_name}' from '{}': {error}",
                        model
                            .schema_artifact_path
                            .as_ref()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "<memory>".to_string())
                    ),
                )
                .with_object_type(class_name.clone()),
            ),
        }
    }

    if findings.is_empty() {
        Ok(validators)
    } else {
        Err(findings)
    }
}

fn schema_validator_for<'a>(
    collection: &CollectionSpec,
    validators: &'a HashMap<String, Validator>,
) -> Option<&'a Validator> {
    collection
        .schema_class
        .as_ref()
        .and_then(|class_name| validators.get(class_name))
}
