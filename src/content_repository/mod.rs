//! Experimental repo-native content repository prototype.
//!
//! The prototype treats files as typed repository objects while keeping the
//! files themselves as canonical state. It intentionally starts with a small
//! file-native baseline before comparing heavier storage/index backends.

mod adapters;
mod model;
#[cfg(test)]
mod tests;
mod validation;

use adapters::{parse_object, update_json_record, update_markdown_frontmatter, write_atomic};
pub use model::{
    AdapterKind, CollectionSpec, ContentFinding, FieldKind, FieldSpec, MarkdownHeading, ObjectKey,
    PlacementRule, ReferenceSpec, RepositoryModel, RepositorySnapshot, RepositoryValidation,
};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use validation::{
    edge_from, invalid_reference_scalar, validate_object_data, validate_placement,
    validate_references,
};

/// File-native content repository runner.
pub struct ContentRepository<'a> {
    model: &'a RepositoryModel,
}

impl<'a> ContentRepository<'a> {
    /// Create a repository runner for a model.
    pub fn new(model: &'a RepositoryModel) -> Self {
        Self { model }
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

    /// Update one structured field and write it back through the object's
    /// adapter. The updated file is reloaded and field-validated before the
    /// write is considered successful.
    pub fn update_field(
        &self,
        root: &Path,
        key: &ObjectKey,
        field: &str,
        value: Value,
    ) -> Result<(), ContentFinding> {
        let collection = self
            .model
            .collections
            .iter()
            .find(|collection| collection.name == key.collection)
            .ok_or_else(|| {
                ContentFinding::new(
                    "unknown_collection",
                    None,
                    format!("Collection '{}' is not defined", key.collection),
                )
            })?;

        let validation = self.validate(root);
        if let Some(finding) = validation
            .findings
            .iter()
            .find(|finding| finding.code == "parse_error")
        {
            return Err(finding.clone());
        }

        let object = validation
            .snapshot
            .objects
            .get(&(key.collection.clone(), key.id.clone()))
            .ok_or_else(|| {
                ContentFinding::new(
                    "unknown_object",
                    None,
                    format!("Object '{}:{}' was not found", key.collection, key.id),
                )
            })?;

        let absolute = root.join(&object.rel_path);
        let content = fs::read_to_string(&absolute).map_err(|error| {
            ContentFinding::new(
                "read_error",
                Some(object.rel_path.clone()),
                format!("Failed to read '{}': {error}", object.rel_path.display()),
            )
        })?;

        let updated = match collection.adapter {
            AdapterKind::MarkdownFrontmatter => {
                update_markdown_frontmatter(&content, field, value.clone()).ok_or_else(|| {
                    ContentFinding::new(
                        "frontmatter_missing",
                        Some(object.rel_path.clone()),
                        format!(
                            "Markdown object '{}:{}' has no YAML frontmatter",
                            key.collection, key.id
                        ),
                    )
                })?
            }
            AdapterKind::JsonRecord => {
                update_json_record(&content, field, value.clone()).map_err(|message| {
                    ContentFinding::new("parse_error", Some(object.rel_path.clone()), message)
                })?
            }
        };

        let reloaded = parse_object(collection, &object.rel_path, &updated)?;
        if reloaded.id != key.id {
            return Err(ContentFinding::new(
                "object_id_changed",
                Some(object.rel_path.clone()),
                format!(
                    "Update would change object id from '{}' to '{}'",
                    key.id, reloaded.id
                ),
            ));
        }

        let mut candidate = validation.snapshot.clone();
        candidate
            .objects
            .insert((key.collection.clone(), key.id.clone()), reloaded);
        candidate.edges.clear();

        let mut findings = Vec::new();
        let candidate_object = candidate
            .objects
            .get(&(key.collection.clone(), key.id.clone()))
            .expect("candidate object was just inserted");
        validate_placement(self.model, candidate_object, &mut findings);
        validate_object_data(collection, candidate_object, &mut findings);
        self.collect_edges(&mut candidate, &mut findings);
        validate_references(&candidate, &mut findings);
        if let Some(finding) = findings.into_iter().next() {
            return Err(finding);
        }

        write_atomic(&absolute, &updated).map_err(|error| {
            ContentFinding::new(
                "write_error",
                Some(object.rel_path.clone()),
                format!("Failed to write '{}': {error}", object.rel_path.display()),
            )
        })
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
                    validate_placement(self.model, &object, findings);
                    validate_object_data(collection, &object, findings);

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
                Err(finding) => findings.push(finding),
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
                        findings.push(ContentFinding::new(
                            "invalid_reference_field",
                            Some(object.rel_path.clone()),
                            format!(
                                "Reference field '{}' on '{}:{}' must be an array",
                                reference.field, object.collection, object.id
                            ),
                        ));
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
