//! Validation helpers for the repo-native content repository prototype.

use super::model::{
    CollectionSpec, ContentFinding, ObjectKey, ReferenceSpec, RepoEdge, RepoObject,
    RepositoryModel, RepositorySnapshot,
};

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
}

pub(super) fn validate_references(
    snapshot: &RepositorySnapshot,
    findings: &mut Vec<ContentFinding>,
) {
    for edge in &snapshot.edges {
        if !snapshot
            .objects
            .contains_key(&(edge.target_collection.clone(), edge.target_id.clone()))
        {
            let source_path = snapshot
                .objects
                .get(&(edge.source.collection.clone(), edge.source.id.clone()))
                .map(|object| object.rel_path.clone());
            findings.push(ContentFinding::new(
                "missing_reference",
                source_path,
                format!(
                    "{}:{} field '{}' references missing {}:{}",
                    edge.source.collection,
                    edge.source.id,
                    edge.field,
                    edge.target_collection,
                    edge.target_id
                ),
            ));
        }
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
        target_collection: reference.target_collection.clone(),
        target_id: target_id.to_string(),
    }
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
}
