//! Validated mutation operations for repo-native content records.

use super::adapters::{parse_object, parse_objects, serialize_jsonl_records, serialize_record};
use super::io::{read_record, replace_or_write_record, replace_record, write_record};
use super::model::{CollectionSpec, ContentFinding};
use super::operations::{
    CreateRecordRequest, CreateRecordResult, UpdateRecordDryRun, UpdateRecordRequest,
    UpdateRecordResult,
};
use super::validation::{validate_object_data, validate_placement, validate_references};
use super::{
    collect_object_edges, normalize_slashes, normalize_slashes_path, schema_validator_for,
    ContentRepository,
};
use serde_json::{Map, Value};
use std::path::{Component, Path, PathBuf};

impl ContentRepository {
    /// Create one validated file-backed content record.
    ///
    /// The operation validates the destination path, collection path policy,
    /// duplicate ID policy, payload shape, and outgoing references before
    /// writing the record. If `request.data` omits the configured ID field, the
    /// request ID is inserted before validation.
    pub fn create_record(
        &self,
        root: &Path,
        request: CreateRecordRequest,
    ) -> Result<CreateRecordResult, Vec<ContentFinding>> {
        let mut findings = Vec::new();
        let rel_path =
            project_relative_create_path(&request.path).map_err(|finding| vec![*finding])?;
        let Some(collection) = self.collection(&request.collection) else {
            return Err(vec![ContentFinding::new(
                "unknown_collection",
                Some(rel_path),
                format!(
                    "Cannot create object in unknown collection '{}'",
                    request.collection
                ),
            )]);
        };
        if !collection_matches_path(collection, &rel_path) {
            findings.push(
                ContentFinding::new(
                    "invalid_object_path",
                    Some(rel_path.clone()),
                    format!(
                        "Destination '{}' does not match collection '{}' path pattern '{}'",
                        rel_path.display(),
                        collection.name,
                        collection.path_pattern
                    ),
                )
                .with_object_type(collection.object_type.clone()),
            );
        }

        let destination = root.join(&rel_path);
        if destination.exists() && !collection.adapter.is_multi_record() {
            findings.push(ContentFinding::new(
                "content_create_path_exists",
                Some(rel_path.clone()),
                format!("Destination '{}' already exists", rel_path.display()),
            ));
        }

        let mut data = request.data;
        match data.get(&collection.id_field).and_then(Value::as_str) {
            Some(existing_id) if existing_id == request.id => {}
            Some(existing_id) => findings.push(
                ContentFinding::new(
                    "content_create_id_mismatch",
                    Some(rel_path.clone()),
                    format!(
                        "Create payload id field '{}' is '{}' but request id is '{}'",
                        collection.id_field, existing_id, request.id
                    ),
                )
                .with_field(collection.id_field.clone()),
            ),
            None if data.contains_key(&collection.id_field) => findings.push(
                ContentFinding::new(
                    "content_create_id_mismatch",
                    Some(rel_path.clone()),
                    format!(
                        "Create payload id field '{}' must be a string matching request id '{}'",
                        collection.id_field, request.id
                    ),
                )
                .with_field(collection.id_field.clone()),
            ),
            None => {
                data.insert(
                    collection.id_field.clone(),
                    Value::String(request.id.clone()),
                );
            }
        }

        let mut existing = self.validate(root);
        if !existing.findings.is_empty() {
            findings.append(&mut existing.findings);
        }
        if existing
            .snapshot
            .objects
            .contains_key(&(collection.name.clone(), request.id.clone()))
        {
            findings.push(ContentFinding::new(
                "duplicate_object_id",
                Some(rel_path.clone()),
                format!(
                    "Collection '{}' already contains object id '{}'",
                    collection.name, request.id
                ),
            ));
        }
        if !findings.is_empty() {
            return Err(findings);
        }

        let content = serialize_record(collection, &rel_path, &data, request.body.as_deref())
            .map_err(|finding| vec![*finding])?;
        let object =
            parse_object(collection, &rel_path, &content).map_err(|finding| vec![*finding])?;

        validate_placement(&self.model, &object, &mut findings);
        validate_object_data(
            collection,
            &object,
            schema_validator_for(collection, &self.schema_validators),
            &mut findings,
        );
        if findings.is_empty() {
            existing.snapshot.objects.insert(
                (collection.name.clone(), request.id.clone()),
                object.clone(),
            );
            collect_object_edges(
                collection,
                &object,
                &mut existing.snapshot.edges,
                &mut findings,
            );
            validate_references(&existing.snapshot, &mut findings);
        }
        if !findings.is_empty() {
            return Err(findings);
        }

        if collection.adapter.is_multi_record() {
            let content = content_with_created_record(root, collection, &rel_path, data)
                .map_err(|finding| vec![*finding])?;
            replace_or_write_record(root, &rel_path, &content).map_err(|finding| vec![*finding])?;
        } else {
            write_record(root, &rel_path, &content).map_err(|finding| vec![*finding])?;
        }
        Ok(CreateRecordResult {
            path: rel_path,
            validation: self.validate(root),
        })
    }

    /// Update one existing file-backed content record.
    ///
    /// The operation loads the current repository snapshot, applies field-level
    /// changes in memory, validates the updated object and affected references,
    /// and either returns deterministic proposed bytes for a dry run or
    /// atomically replaces the existing record file.
    pub fn update_record(
        &self,
        root: &Path,
        request: UpdateRecordRequest,
    ) -> Result<UpdateRecordResult, Vec<ContentFinding>> {
        let mut findings = Vec::new();
        let requested_path = match request.path.as_deref() {
            Some(path) => Some(
                project_relative_file_path(
                    path,
                    "content_update_path_escape",
                    "Update destination must be a non-empty project-relative file path",
                )
                .map_err(|finding| vec![*finding])?,
            ),
            None => None,
        };
        let Some(collection) = self.collection(&request.collection) else {
            return Err(vec![ContentFinding::new(
                "unknown_collection",
                requested_path,
                format!(
                    "Cannot update object in unknown collection '{}'",
                    request.collection
                ),
            )]);
        };

        let mut existing = self.validate(root);
        if !existing.findings.is_empty() {
            return Err(existing.findings);
        }
        let key = (collection.name.clone(), request.id.clone());
        let Some(current_object) = existing.snapshot.objects.get(&key).cloned() else {
            return Err(vec![ContentFinding::new(
                "content_update_missing_record",
                requested_path,
                format!(
                    "Collection '{}' does not contain object id '{}'",
                    collection.name, request.id
                ),
            )]);
        };
        if let Some(requested_path) = requested_path {
            if requested_path != current_object.rel_path {
                return Err(vec![ContentFinding::new(
                    "content_update_path_mismatch",
                    Some(requested_path.clone()),
                    format!(
                        "Update path '{}' does not match existing record path '{}'",
                        requested_path.display(),
                        current_object.rel_path.display()
                    ),
                )]);
            }
        }

        let mut data = current_object.data.clone();
        for (field, value) in request.changes {
            if field == collection.id_field {
                match value.as_str() {
                    Some(id) if id == current_object.id => {}
                    _ => findings.push(
                        ContentFinding::new(
                            "content_update_identity_change",
                            Some(current_object.rel_path.clone()),
                            format!(
                                "Update cannot change id field '{}' for '{}:{}'",
                                collection.id_field, collection.name, current_object.id
                            ),
                        )
                        .with_field(collection.id_field.clone()),
                    ),
                }
            }
            data.insert(field, value);
        }
        if !findings.is_empty() {
            return Err(findings);
        }

        let content = serialize_record(
            collection,
            &current_object.rel_path,
            &data,
            current_object.body.as_deref(),
        )
        .map_err(|finding| vec![*finding])?;
        let updated_object = parse_object(collection, &current_object.rel_path, &content)
            .map_err(|finding| vec![*finding])?;
        if updated_object.id != current_object.id {
            return Err(vec![ContentFinding::new(
                "content_update_identity_change",
                Some(current_object.rel_path.clone()),
                format!(
                    "Update cannot change object id from '{}' to '{}'",
                    current_object.id, updated_object.id
                ),
            )]);
        }

        validate_placement(&self.model, &updated_object, &mut findings);
        validate_object_data(
            collection,
            &updated_object,
            schema_validator_for(collection, &self.schema_validators),
            &mut findings,
        );
        if findings.is_empty() {
            existing
                .snapshot
                .objects
                .insert(key.clone(), updated_object.clone());
            existing
                .snapshot
                .edges
                .retain(|edge| !(edge.source.collection == key.0 && edge.source.id == key.1));
            collect_object_edges(
                collection,
                &updated_object,
                &mut existing.snapshot.edges,
                &mut findings,
            );
            validate_references(&existing.snapshot, &mut findings);
        }
        if !findings.is_empty() {
            return Err(findings);
        }

        let path = current_object.rel_path;
        let write_content = if collection.adapter.is_multi_record() {
            content_with_updated_record(root, collection, &path, &current_object.id, data)
                .map_err(|finding| vec![*finding])?
        } else {
            content
        };
        if request.dry_run {
            return Ok(UpdateRecordResult {
                path: path.clone(),
                validation: existing,
                dry_run: Some(UpdateRecordDryRun {
                    path,
                    content: write_content,
                }),
            });
        }

        replace_record(root, &path, &write_content).map_err(|finding| vec![*finding])?;
        Ok(UpdateRecordResult {
            path,
            validation: self.validate(root),
            dry_run: None,
        })
    }

    fn collection(&self, name: &str) -> Option<&CollectionSpec> {
        self.model
            .collections
            .iter()
            .find(|collection| collection.name == name)
    }
}

fn project_relative_create_path(path: &Path) -> Result<PathBuf, Box<ContentFinding>> {
    project_relative_file_path(
        path,
        "content_create_path_escape",
        "Create destination must be a non-empty project-relative file path",
    )
}

fn project_relative_file_path(
    path: &Path,
    code: &'static str,
    message: &'static str,
) -> Result<PathBuf, Box<ContentFinding>> {
    if path.as_os_str().is_empty()
        || path.components().any(|component| {
            !matches!(component, Component::Normal(_))
                || component
                    .as_os_str()
                    .to_str()
                    .is_some_and(|part| part.contains('\\'))
        })
        || path.file_name().is_none()
    {
        return Err(Box::new(ContentFinding::new(code, None, message)));
    }
    Ok(path.to_path_buf())
}

fn collection_matches_path(collection: &CollectionSpec, rel_path: &Path) -> bool {
    glob::Pattern::new(&normalize_slashes(&collection.path_pattern))
        .map(|pattern| pattern.matches(&normalize_slashes_path(rel_path)))
        .unwrap_or(false)
}

fn content_with_created_record(
    root: &Path,
    collection: &CollectionSpec,
    rel_path: &Path,
    data: Map<String, Value>,
) -> Result<String, Box<ContentFinding>> {
    let mut records = if root.join(rel_path).exists() {
        read_record_maps(root, collection, rel_path)?
    } else {
        Vec::new()
    };
    records.push(data);
    serialize_jsonl_records(collection, rel_path, records)
}

fn content_with_updated_record(
    root: &Path,
    collection: &CollectionSpec,
    rel_path: &Path,
    id: &str,
    data: Map<String, Value>,
) -> Result<String, Box<ContentFinding>> {
    let mut replaced = false;
    let records = read_record_maps(root, collection, rel_path)?
        .into_iter()
        .map(|record| {
            if record
                .get(&collection.id_field)
                .and_then(Value::as_str)
                .is_some_and(|record_id| record_id == id)
            {
                replaced = true;
                data.clone()
            } else {
                record
            }
        })
        .collect::<Vec<_>>();
    if !replaced {
        return Err(Box::new(ContentFinding::new(
            "content_update_missing_record",
            Some(rel_path.to_path_buf()),
            format!(
                "Collection '{}' file '{}' does not contain object id '{}'",
                collection.name,
                rel_path.display(),
                id
            ),
        )));
    }
    serialize_jsonl_records(collection, rel_path, records)
}

fn read_record_maps(
    root: &Path,
    collection: &CollectionSpec,
    rel_path: &Path,
) -> Result<Vec<Map<String, Value>>, Box<ContentFinding>> {
    let content = read_record(root, rel_path)?;
    parse_objects(collection, rel_path, &content)
        .map(|objects| objects.into_iter().map(|object| object.data).collect())
}
