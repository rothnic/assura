//! Cargo manifest semantic validation.

use super::{validate_identifier, validate_relative_path_text, validate_severity};
use crate::config::config::ManifestSemanticsConfig;
use std::collections::HashSet;

const CRATE_ROLES: &[&str] = &["public", "internal"];
const PUBLISH_POLICIES: &[&str] = &["public", "internal"];

pub(super) fn validate_manifest_semantics_config(
    policy: &ManifestSemanticsConfig,
) -> Result<(), String> {
    let context = format!("extensions.manifest_semantics.{}", policy.id);
    validate_identifier(&policy.id, &format!("{context}.id"))?;
    if policy.manifests.is_empty() {
        return Err(format!(
            "{context}.manifests: at least one manifest is required"
        ));
    }

    let mut paths = HashSet::new();
    for manifest in &policy.manifests {
        validate_relative_path_text(&manifest.path, &format!("{context}.manifests.path"))?;
        if !paths.insert(manifest.path.as_str()) {
            return Err(format!(
                "{context}.manifests.{}: duplicate manifest path",
                manifest.path
            ));
        }
        validate_optional_text(&manifest.package, &format!("{context}.manifests.package"))?;
        validate_optional_text(&manifest.version, &format!("{context}.manifests.version"))?;
        validate_optional_text(
            &manifest.rust_version,
            &format!("{context}.manifests.rust_version"),
        )?;
        validate_optional_text(&manifest.license, &format!("{context}.manifests.license"))?;
        validate_optional_choice(
            &manifest.role,
            CRATE_ROLES,
            &format!("{context}.manifests.role"),
        )?;
        validate_optional_choice(
            &manifest.publish,
            PUBLISH_POLICIES,
            &format!("{context}.manifests.publish"),
        )?;
        validate_text_list(
            &manifest.description_required_terms,
            &format!("{context}.manifests.description_required_terms"),
        )?;
        validate_text_list(
            &manifest.description_forbidden_terms,
            &format!("{context}.manifests.description_forbidden_terms"),
        )?;
        validate_text_list(&manifest.keywords, &format!("{context}.manifests.keywords"))?;
        validate_text_list(&manifest.binaries, &format!("{context}.manifests.binaries"))?;
        if !manifest_has_policy(manifest) {
            return Err(format!(
                "{context}.manifests.{}: at least one expected manifest field is required",
                manifest.path
            ));
        }
    }
    if let Some(severity) = &policy.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

fn manifest_has_policy(manifest: &crate::config::config::ManifestSemanticsManifestConfig) -> bool {
    manifest.package.is_some()
        || manifest.role.is_some()
        || manifest.version.is_some()
        || manifest.rust_version.is_some()
        || manifest.license.is_some()
        || manifest.publish.is_some()
        || !manifest.description_required_terms.is_empty()
        || !manifest.description_forbidden_terms.is_empty()
        || !manifest.keywords.is_empty()
        || !manifest.binaries.is_empty()
}

fn validate_optional_text(value: &Option<String>, context: &str) -> Result<(), String> {
    if value.as_ref().is_some_and(|text| text.trim().is_empty()) {
        return Err(format!("{context}: value must not be empty"));
    }
    Ok(())
}

fn validate_optional_choice(
    value: &Option<String>,
    allowed: &[&str],
    context: &str,
) -> Result<(), String> {
    if let Some(value) = value {
        if !allowed.contains(&value.as_str()) {
            return Err(format!("{context}: expected one of {}", allowed.join(", ")));
        }
    }
    Ok(())
}

fn validate_text_list(values: &[String], context: &str) -> Result<(), String> {
    let mut seen = HashSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(format!("{context}: value must not be empty"));
        }
        if !seen.insert(value.as_str()) {
            return Err(format!("{context}.{value}: duplicate value"));
        }
    }
    Ok(())
}
