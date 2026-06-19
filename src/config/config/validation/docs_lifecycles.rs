//! Docs-lifecycle semantic validation.

use super::{
    validate_identifier, validate_relative_path_text, validate_relative_pattern, validate_severity,
};
use crate::config::config::{DocsLifecycleClaimPatternConfig, DocsLifecycleConfig};
use std::collections::HashSet;

pub(super) fn validate_docs_lifecycle_config(policy: &DocsLifecycleConfig) -> Result<(), String> {
    let context = format!("extensions.docs_lifecycles.{}", policy.id);
    validate_identifier(&policy.id, &format!("{context}.id"))?;
    if policy.active.is_empty() {
        return Err(format!(
            "{context}.active: at least one active doc pattern is required"
        ));
    }
    if policy.allowed_statuses.is_empty() {
        return Err(format!(
            "{context}.allowed_statuses: at least one lifecycle status is required"
        ));
    }
    if policy.require_frontmatter_status.is_empty()
        && policy.historical.is_empty()
        && policy.claim_patterns.is_empty()
    {
        return Err(format!(
            "{context}: at least one lifecycle or claim check is required"
        ));
    }

    validate_patterns(&policy.active, &format!("{context}.active"))?;
    validate_patterns(&policy.historical, &format!("{context}.historical"))?;
    validate_patterns(
        &policy.require_frontmatter_status,
        &format!("{context}.require_frontmatter_status"),
    )?;
    validate_patterns(
        &policy.historical_exceptions,
        &format!("{context}.historical_exceptions"),
    )?;
    validate_statuses(&policy.allowed_statuses, &context)?;
    validate_claim_patterns(&policy.claim_patterns, &context)?;
    if let Some(severity) = &policy.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

fn validate_patterns(patterns: &[String], context: &str) -> Result<(), String> {
    for pattern in patterns {
        validate_relative_pattern(pattern, context)?;
    }
    Ok(())
}

fn validate_statuses(statuses: &[String], context: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for status in statuses {
        validate_identifier(status, &format!("{context}.allowed_statuses"))?;
        if !seen.insert(status.as_str()) {
            return Err(format!(
                "{context}.allowed_statuses.{status}: duplicate lifecycle status"
            ));
        }
    }
    Ok(())
}

fn validate_claim_patterns(
    claims: &[DocsLifecycleClaimPatternConfig],
    context: &str,
) -> Result<(), String> {
    let mut ids = HashSet::new();
    for claim in claims {
        validate_identifier(&claim.id, &format!("{context}.claim_patterns.id"))?;
        if !ids.insert(claim.id.as_str()) {
            return Err(format!(
                "{context}.claim_patterns.{}: duplicate claim pattern id",
                claim.id
            ));
        }
        if claim.pattern.trim().is_empty() {
            return Err(format!(
                "{context}.claim_patterns.{}.pattern: value must not be empty",
                claim.id
            ));
        }
        if claim.evidence_files.is_empty() {
            return Err(format!(
                "{context}.claim_patterns.{}: at least one evidence file is required",
                claim.id
            ));
        }
        for evidence_file in &claim.evidence_files {
            validate_relative_path_text(
                evidence_file,
                &format!("{context}.claim_patterns.{}.evidence_files", claim.id),
            )?;
        }
    }
    Ok(())
}
