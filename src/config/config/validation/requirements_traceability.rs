//! Requirements/evidence traceability semantic validation.

use super::{validate_identifier, validate_severity};
use crate::config::config::RequirementsTraceabilityConfig;
use std::collections::HashSet;

pub(super) fn validate_requirements_traceability_configs(
    policies: &[RequirementsTraceabilityConfig],
) -> Result<(), String> {
    let mut ids = HashSet::new();
    for policy in policies {
        validate_requirements_traceability_config(policy)?;
        if !ids.insert(policy.id.as_str()) {
            return Err(format!(
                "extensions.requirements_traceability.{}: duplicate requirements traceability id",
                policy.id
            ));
        }
    }
    Ok(())
}

fn validate_requirements_traceability_config(
    policy: &RequirementsTraceabilityConfig,
) -> Result<(), String> {
    let context = format!("extensions.requirements_traceability.{}", policy.id);
    validate_identifier(&policy.id, &format!("{context}.id"))?;
    validate_identifier(
        &policy.requirements_collection,
        &format!("{context}.requirements_collection"),
    )?;
    validate_identifier(&policy.priority_field, &format!("{context}.priority_field"))?;
    if policy.high_priority_values.is_empty()
        && policy.claim_collections.is_empty()
        && policy.evidence_collections.is_empty()
        && policy.finding_collections.is_empty()
    {
        return Err(format!(
            "{context}: at least one requirement coverage, claim, evidence, or finding check is required"
        ));
    }
    if !policy.high_priority_values.is_empty() && policy.coverage_collections.is_empty() {
        return Err(format!(
            "{context}.coverage_collections: at least one coverage collection is required when high_priority_values are configured"
        ));
    }
    if !policy.claim_collections.is_empty() && policy.evidence_collections.is_empty() {
        return Err(format!(
            "{context}.evidence_collections: at least one evidence collection is required when claim_collections are configured"
        ));
    }
    if !policy.evidence_collections.is_empty() && policy.source_document_collections.is_empty() {
        return Err(format!(
            "{context}.source_document_collections: at least one source-document collection is required when evidence_collections are configured"
        ));
    }
    if !policy.finding_collections.is_empty() && policy.owner_fields.is_empty() {
        return Err(format!(
            "{context}.owner_fields: at least one owner field is required when finding_collections are configured"
        ));
    }
    if !policy.finding_collections.is_empty() && policy.status_fields.is_empty() {
        return Err(format!(
            "{context}.status_fields: at least one status field is required when finding_collections are configured"
        ));
    }
    validate_identifiers(
        &policy.high_priority_values,
        &format!("{context}.high_priority_values"),
    )?;
    validate_identifiers(
        &policy.coverage_collections,
        &format!("{context}.coverage_collections"),
    )?;
    validate_identifiers(
        &policy.claim_collections,
        &format!("{context}.claim_collections"),
    )?;
    validate_identifiers(
        &policy.evidence_collections,
        &format!("{context}.evidence_collections"),
    )?;
    validate_identifiers(
        &policy.source_document_collections,
        &format!("{context}.source_document_collections"),
    )?;
    validate_identifiers(
        &policy.finding_collections,
        &format!("{context}.finding_collections"),
    )?;
    validate_identifiers(&policy.owner_fields, &format!("{context}.owner_fields"))?;
    validate_identifiers(&policy.status_fields, &format!("{context}.status_fields"))?;
    if let Some(severity) = &policy.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

fn validate_identifiers(values: &[String], context: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for value in values {
        validate_identifier(value, context)?;
        if !seen.insert(value.as_str()) {
            return Err(format!("{context}.{value}: duplicate value"));
        }
    }
    Ok(())
}
