//! Computed-check semantic validation.

use super::{validate_identifier, validate_severity};
use crate::config::config::ComputedCheckConfig;
use std::collections::HashSet;
use std::path::{Component, Path};

pub(super) fn validate_computed_check_configs(
    policies: &[ComputedCheckConfig],
) -> Result<(), String> {
    let mut ids = HashSet::new();
    for policy in policies {
        validate_computed_check_config(policy)?;
        if !ids.insert(policy.id.as_str()) {
            return Err(format!(
                "extensions.computed_checks.{}: duplicate computed check id",
                policy.id
            ));
        }
    }
    Ok(())
}

fn validate_computed_check_config(policy: &ComputedCheckConfig) -> Result<(), String> {
    let context = format!("extensions.computed_checks.{}", policy.id);
    validate_identifier(&policy.id, &format!("{context}.id"))?;
    validate_project_relative_script(&policy.script, &format!("{context}.script"))?;
    if let Some(windows_script) = &policy.windows_script {
        validate_project_relative_script(windows_script, &format!("{context}.windows_script"))?;
    }
    if policy.timeout_ms == 0 || policy.timeout_ms > 60_000 {
        return Err(format!(
            "{context}.timeout_ms: expected a value from 1 to 60000"
        ));
    }
    for arg in &policy.args {
        if arg.trim().is_empty() {
            return Err(format!("{context}.args: values must not be empty"));
        }
        if arg.contains('\0') {
            return Err(format!("{context}.args: values must not contain NUL"));
        }
    }
    if let Some(severity) = &policy.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

fn validate_project_relative_script(value: &str, context: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{context}: value must not be empty"));
    }
    if value.contains('\0') {
        return Err(format!("{context}: value must not contain NUL"));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(format!(
            "{context}: script must be project-relative and must not contain '..' or a platform prefix"
        ));
    }
    Ok(())
}
