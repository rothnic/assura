//! Validation helpers for repo-native content runtime validation.

use super::model::{
    CollectionSpec, ContentFinding, ObjectKey, ReferenceSpec, RepoEdge, RepoObject,
    RepositoryModel, RepositorySnapshot,
};
use jsonschema::Validator;
use serde_json::Value;

pub(super) fn validate_placement(
    model: &RepositoryModel,
    object: &RepoObject,
    findings: &mut Vec<ContentFinding>,
) {
    if model
        .placements
        .iter()
        .any(|rule| rule.allows(&object.rel_path, &object.object_type))
    {
        return;
    }
    findings.push(ContentFinding::new(
        "invalid_object_placement",
        Some(object.rel_path.clone()),
        format!(
            "Object '{}:{}' of type '{}' is not allowed at '{}'",
            object.collection,
            object.id,
            object.object_type,
            object.rel_path.display()
        ),
    ));
}

pub(super) fn validate_object_data(
    collection: &CollectionSpec,
    object: &RepoObject,
    schema_validator: Option<&Validator>,
    findings: &mut Vec<ContentFinding>,
) {
    for field in &collection.fields {
        match object.data.get(&field.name) {
            Some(value) if field.kind.validate(value) => {}
            Some(_) => findings.push(ContentFinding::new(
                "invalid_field_type",
                Some(object.rel_path.clone()),
                format!(
                    "Field '{}' on '{}:{}' has the wrong type",
                    field.name, object.collection, object.id
                ),
            )),
            None if field.required => findings.push(ContentFinding::new(
                "missing_field",
                Some(object.rel_path.clone()),
                format!(
                    "Object '{}:{}' is missing required field '{}'",
                    object.collection, object.id, field.name
                ),
            )),
            None => {}
        }
    }

    let Some(validator) = schema_validator else {
        return;
    };
    let value = Value::Object(object.data.clone());
    for error in validator.iter_errors(&value) {
        let field = schema_error_field(&error);
        let mut finding = ContentFinding::new(
            "invalid_object_shape",
            Some(object.rel_path.clone()),
            format!(
                "Object '{}:{}' of type '{}' does not match runtime schema '{}': {}",
                object.collection,
                object.id,
                object.object_type,
                collection
                    .schema_class
                    .as_deref()
                    .unwrap_or(object.object_type.as_str()),
                error
            ),
        )
        .with_object_type(object.object_type.clone());
        if let Some(field) = field {
            finding = finding.with_field(field);
        }
        findings.push(finding);
    }
}

pub(super) fn validate_references(
    snapshot: &RepositorySnapshot,
    findings: &mut Vec<ContentFinding>,
) {
    for edge in &snapshot.edges {
        let candidates = edge_candidate_collections(snapshot, edge);
        let matches = candidates
            .iter()
            .filter(|collection| {
                snapshot
                    .objects
                    .contains_key(&((*collection).clone(), edge.target_id.clone()))
            })
            .cloned()
            .collect::<Vec<_>>();
        if matches.is_empty() {
            let source = snapshot
                .objects
                .get(&(edge.source.collection.clone(), edge.source.id.clone()));
            let source_path = source.map(|object| object.rel_path.clone());
            let mut finding = ContentFinding::new(
                "missing_reference",
                source_path,
                format!(
                    "{}:{} field '{}' references missing {}:{}",
                    edge.source.collection,
                    edge.source.id,
                    edge.field,
                    target_collection_label(&candidates),
                    edge.target_id
                ),
            )
            .with_field(edge.field.clone())
            .with_referenced_object(format!(
                "{}:{}",
                target_collection_label(&candidates),
                edge.target_id
            ));
            if let Some(source) = source {
                finding = finding.with_object_type(source.object_type.clone());
            }
            findings.push(finding);
        } else if matches.len() > 1 {
            let source = snapshot
                .objects
                .get(&(edge.source.collection.clone(), edge.source.id.clone()));
            let targets = matches
                .iter()
                .map(|collection| format!("{collection}:{}", edge.target_id))
                .collect::<Vec<_>>()
                .join(", ");
            let mut finding = ContentFinding::new(
                "ambiguous_reference",
                source.map(|object| object.rel_path.clone()),
                format!(
                    "{}:{} field '{}' ambiguously references {}",
                    edge.source.collection, edge.source.id, edge.field, targets
                ),
            )
            .with_field(edge.field.clone())
            .with_referenced_object(targets);
            if let Some(source) = source {
                finding = finding.with_object_type(source.object_type.clone());
            }
            findings.push(finding);
        }
    }
    validate_acyclic_references(snapshot, findings);
}

fn edge_candidate_collections(snapshot: &RepositorySnapshot, edge: &RepoEdge) -> Vec<String> {
    if edge.target_collections.is_empty() {
        let mut collections = snapshot
            .objects
            .keys()
            .map(|(collection, _)| collection.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        collections.retain(|collection| collection != &edge.source.collection);
        collections
    } else {
        edge.target_collections.clone()
    }
}

pub(super) fn edge_from(
    object: &RepoObject,
    reference: &ReferenceSpec,
    target_id: &str,
) -> RepoEdge {
    RepoEdge {
        source: ObjectKey::new(object.collection.clone(), object.id.clone()),
        field: reference.field.clone(),
        target_collections: reference.target_collections.clone(),
        target_id: target_id.to_string(),
        acyclic: reference.acyclic,
    }
}

pub(super) fn missing_required_reference(
    object: &RepoObject,
    reference: &ReferenceSpec,
) -> ContentFinding {
    ContentFinding::new(
        "missing_reference_field",
        Some(object.rel_path.clone()),
        format!(
            "Required reference field '{}' on '{}:{}' is missing or empty",
            reference.field, object.collection, object.id
        ),
    )
    .with_object_type(object.object_type.clone())
    .with_field(reference.field.clone())
}

pub(super) fn invalid_reference_scalar(
    object: &RepoObject,
    reference: &ReferenceSpec,
) -> ContentFinding {
    ContentFinding::new(
        "invalid_reference_field",
        Some(object.rel_path.clone()),
        format!(
            "Reference field '{}' on '{}:{}' must contain string target IDs",
            reference.field, object.collection, object.id
        ),
    )
    .with_field(reference.field.clone())
}

fn target_collection_label(collections: &[String]) -> String {
    if collections.is_empty() {
        "*".to_string()
    } else {
        collections.join("|")
    }
}

fn validate_acyclic_references(snapshot: &RepositorySnapshot, findings: &mut Vec<ContentFinding>) {
    for edge in snapshot.edges.iter().filter(|edge| edge.acyclic) {
        for target_collection in &edge.target_collections {
            let target = ObjectKey::new(target_collection.clone(), edge.target_id.clone());
            if path_exists(snapshot, &target, &edge.source, &mut Vec::new()) {
                let source = snapshot
                    .objects
                    .get(&(edge.source.collection.clone(), edge.source.id.clone()));
                let mut finding = ContentFinding::new(
                    "cyclic_reference",
                    source.map(|object| object.rel_path.clone()),
                    format!(
                        "{}:{} field '{}' creates a cycle through {}:{}",
                        edge.source.collection,
                        edge.source.id,
                        edge.field,
                        target.collection,
                        target.id
                    ),
                )
                .with_field(edge.field.clone())
                .with_referenced_object(format!("{}:{}", target.collection, target.id));
                if let Some(source) = source {
                    finding = finding.with_object_type(source.object_type.clone());
                }
                findings.push(finding);
            }
        }
    }
}

fn path_exists(
    snapshot: &RepositorySnapshot,
    current: &ObjectKey,
    target: &ObjectKey,
    visited: &mut Vec<ObjectKey>,
) -> bool {
    if current == target {
        return true;
    }
    if visited.iter().any(|seen| seen == current) {
        return false;
    }
    visited.push(current.clone());
    let found = snapshot
        .edges
        .iter()
        .filter(|edge| edge.acyclic && edge.source == *current)
        .any(|edge| {
            edge.target_collections.iter().any(|collection| {
                let next = ObjectKey::new(collection.clone(), edge.target_id.clone());
                path_exists(snapshot, &next, target, visited)
            })
        });
    visited.pop();
    found
}

fn schema_error_field(error: &jsonschema::ValidationError<'_>) -> Option<String> {
    let path = error.instance_path.to_string();
    let path = path.trim_start_matches('/');
    if path.is_empty() {
        required_field_from_message(&error.to_string())
    } else {
        path.split('/')
            .next()
            .map(|field| field.replace("~1", "/").replace("~0", "~"))
    }
}

fn required_field_from_message(message: &str) -> Option<String> {
    for quote in ['\'', '"', '`'] {
        let mut parts = message.split(quote);
        let _ = parts.next();
        let Some(candidate) = parts.next() else {
            continue;
        };
        if message.contains("required") && !candidate.trim().is_empty() {
            return Some(candidate.to_string());
        }
    }
    None
}
