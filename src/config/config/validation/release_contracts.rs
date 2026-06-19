//! Release-contract semantic validation.

use super::{validate_identifier, validate_relative_path_text, validate_severity};
use crate::config::config::ReleaseContractConfig;
use std::collections::HashSet;

pub(super) fn validate_release_contract_config(
    contract: &ReleaseContractConfig,
) -> Result<(), String> {
    let context = format!("extensions.release_contracts.{}", contract.id);
    validate_identifier(&contract.id, &format!("{context}.id"))?;
    if contract.artifacts.is_empty() {
        return Err(format!(
            "{context}.artifacts: at least one artifact is required"
        ));
    }
    let mut artifact_names = HashSet::new();
    for artifact in &contract.artifacts {
        if artifact.name.trim().is_empty() {
            return Err(format!("{context}.artifacts.name: value must not be empty"));
        }
        if !artifact_names.insert(artifact.name.as_str()) {
            return Err(format!(
                "{context}.artifacts.{}: duplicate artifact name",
                artifact.name
            ));
        }
    }
    if contract.workflow_files.is_empty() {
        return Err(format!(
            "{context}.workflow_files: at least one workflow file is required"
        ));
    }
    for workflow_file in &contract.workflow_files {
        validate_relative_path_text(workflow_file, &format!("{context}.workflow_files"))?;
    }
    if contract.docs_files.is_empty() && contract.installer_files.is_empty() {
        return Err(format!(
            "{context}: at least one docs_files or installer_files entry is required"
        ));
    }
    for docs_file in &contract.docs_files {
        validate_relative_path_text(docs_file, &format!("{context}.docs_files"))?;
    }
    for installer_file in &contract.installer_files {
        validate_relative_path_text(installer_file, &format!("{context}.installer_files"))?;
    }
    for branch in &contract.allowed_url_branches {
        validate_branch_text(branch, &format!("{context}.allowed_url_branches"))?;
    }
    if let Some(severity) = &contract.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

fn validate_branch_text(value: &str, context: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{context}: value must not be empty"));
    }
    if value.contains(char::is_whitespace) {
        return Err(format!("{context}: value must not contain whitespace"));
    }
    Ok(())
}
