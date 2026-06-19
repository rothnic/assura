//! Support-matrix semantic validation.

use super::{validate_identifier, validate_relative_path_text, validate_severity};
use crate::config::config::SupportMatrixConfig;
use std::collections::HashSet;

const SUPPORT_STATUSES: &[&str] = &[
    "supported",
    "experimental",
    "internal",
    "roadmap",
    "unsupported",
];

pub(super) fn validate_support_matrix_config(
    matrix: &SupportMatrixConfig,
    manifest_semantics_ids: &HashSet<&str>,
) -> Result<(), String> {
    let context = format!("extensions.support_matrices.{}", matrix.id);
    validate_identifier(&matrix.id, &format!("{context}.id"))?;
    if matrix.entries.is_empty() {
        return Err(format!("{context}.entries: at least one entry is required"));
    }
    if matrix.command_contracts.is_empty()
        && matrix.rust_exports.is_empty()
        && matrix.docs_claim_sources.is_empty()
        && matrix.manifest_policies.is_empty()
    {
        return Err(format!(
            "{context}: at least one command_contracts, rust_exports, docs_claim_sources, or manifest_policies entry is required"
        ));
    }

    let mut surfaces = HashSet::new();
    for entry in &matrix.entries {
        if entry.surface.trim().is_empty() {
            return Err(format!(
                "{context}.entries.surface: value must not be empty"
            ));
        }
        if !surfaces.insert(entry.surface.as_str()) {
            return Err(format!(
                "{context}.entries.{}: duplicate support surface",
                entry.surface
            ));
        }
        if !SUPPORT_STATUSES.contains(&entry.status.as_str()) {
            return Err(format!(
                "{context}.entries.{}.status: expected one of {}",
                entry.surface,
                SUPPORT_STATUSES.join(", ")
            ));
        }
    }
    for command_contract in &matrix.command_contracts {
        validate_relative_path_text(command_contract, &format!("{context}.command_contracts"))?;
    }
    for rust_export in &matrix.rust_exports {
        validate_relative_path_text(rust_export, &format!("{context}.rust_exports"))?;
    }
    for docs_claim_source in &matrix.docs_claim_sources {
        validate_relative_path_text(
            &docs_claim_source.path,
            &format!("{context}.docs_claim_sources.path"),
        )?;
    }
    let mut manifest_policies = HashSet::new();
    for manifest_policy in &matrix.manifest_policies {
        validate_identifier(manifest_policy, &format!("{context}.manifest_policies"))?;
        if !manifest_policies.insert(manifest_policy.as_str()) {
            return Err(format!(
                "{context}.manifest_policies.{manifest_policy}: duplicate manifest semantics policy reference"
            ));
        }
        if !manifest_semantics_ids.contains(manifest_policy.as_str()) {
            return Err(format!(
                "{context}.manifest_policies.{manifest_policy}: unknown manifest semantics policy"
            ));
        }
    }
    if let Some(severity) = &matrix.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}
