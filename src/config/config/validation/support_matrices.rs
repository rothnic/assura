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

pub(super) fn validate_support_matrix_config(matrix: &SupportMatrixConfig) -> Result<(), String> {
    let context = format!("extensions.support_matrices.{}", matrix.id);
    validate_identifier(&matrix.id, &format!("{context}.id"))?;
    if matrix.entries.is_empty() {
        return Err(format!("{context}.entries: at least one entry is required"));
    }
    if matrix.command_contracts.is_empty() && matrix.rust_exports.is_empty() {
        return Err(format!(
            "{context}: at least one command_contracts or rust_exports entry is required"
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
    if let Some(severity) = &matrix.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}
