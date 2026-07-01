//! Semantic validation for repository-reference check policies.

use super::{validate_identifier, validate_relative_pattern, validate_severity};
use crate::config::config::RepositoryReferenceConfig;
use std::collections::HashSet;

pub(super) fn validate_repository_reference_configs(
    policies: &[RepositoryReferenceConfig],
) -> Result<(), String> {
    let mut ids = HashSet::new();
    for policy in policies {
        validate_repository_reference_config(policy)?;
        if !ids.insert(&policy.id) {
            return Err(format!(
                "extensions.repository_references.{}: duplicate repository reference id",
                policy.id
            ));
        }
    }
    Ok(())
}

fn validate_repository_reference_config(policy: &RepositoryReferenceConfig) -> Result<(), String> {
    let context = format!("extensions.repository_references.{}", policy.id);
    validate_identifier(&policy.id, &format!("{context}.id"))?;
    for path in &policy.paths {
        validate_relative_pattern(path, &format!("{context}.paths"))?;
    }
    if let Some(severity) = &policy.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}
