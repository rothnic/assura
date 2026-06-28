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
        if !snapshot
            .objects
            .contains_key(&(edge.target_collection.clone(), edge.target_id.clone()))
        {
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
                    edge.target_collection,
                    edge.target_id
                ),
            )
            .with_field(edge.field.clone())
            .with_referenced_object(format!("{}:{}", edge.target_collection, edge.target_id));
            if let Some(source) = source {
                finding = finding.with_object_type(source.object_type.clone());
            }
            findings.push(finding);
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
    .with_field(reference.field.clone())
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
