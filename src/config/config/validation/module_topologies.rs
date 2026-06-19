//! Module-topology semantic validation.

use super::{validate_identifier, validate_relative_path_text, validate_severity};
use crate::config::config::{ModuleTopologyConfig, ModuleTopologyModuleConfig};
use std::collections::{HashMap, HashSet};

const MODULE_STATUSES: &[&str] = &[
    "supported",
    "experimental",
    "internal",
    "roadmap",
    "unsupported",
];
const MODULE_VISIBILITIES: &[&str] = &["public", "internal"];

pub(super) fn validate_module_topology_config(policy: &ModuleTopologyConfig) -> Result<(), String> {
    let context = format!("extensions.module_topologies.{}", policy.id);
    validate_identifier(&policy.id, &format!("{context}.id"))?;
    if policy.modules.is_empty() {
        return Err(format!(
            "{context}.modules: at least one module row is required"
        ));
    }
    if policy.rust_exports.is_empty() {
        return Err(format!(
            "{context}.rust_exports: at least one Rust export file is required"
        ));
    }

    validate_modules(&policy.modules, &context)?;
    for rust_export in &policy.rust_exports {
        validate_relative_path_text(rust_export, &format!("{context}.rust_exports"))?;
    }
    if let Some(severity) = &policy.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

fn validate_modules(modules: &[ModuleTopologyModuleConfig], context: &str) -> Result<(), String> {
    let mut families = HashSet::new();
    let mut export_owners = HashMap::new();
    for module in modules {
        validate_identifier(&module.family, &format!("{context}.modules.family"))?;
        if !families.insert(module.family.as_str()) {
            return Err(format!(
                "{context}.modules.{}: duplicate module family",
                module.family
            ));
        }
        register_export_owner(&mut export_owners, &module.family, &module.family, context)?;
        if !MODULE_STATUSES.contains(&module.status.as_str()) {
            return Err(format!(
                "{context}.modules.{}.status: expected one of {}",
                module.family,
                MODULE_STATUSES.join(", ")
            ));
        }
        validate_text(&module.owner, &format!("{context}.modules.owner"))?;
        validate_text(&module.purpose, &format!("{context}.modules.purpose"))?;
        if module.roots.is_empty() {
            return Err(format!(
                "{context}.modules.{}: at least one root is required",
                module.family
            ));
        }
        for root in &module.roots {
            validate_relative_path_text(root, &format!("{context}.modules.roots"))?;
        }
        let mut row_exports = HashSet::new();
        for public_export in &module.public_exports {
            validate_identifier(public_export, &format!("{context}.modules.public_exports"))?;
            if !row_exports.insert(public_export.as_str()) {
                return Err(format!(
                    "{context}.modules.{}: duplicate public export `{public_export}`",
                    module.family
                ));
            }
            register_export_owner(&mut export_owners, public_export, &module.family, context)?;
        }
        if let Some(visibility) = &module.visibility {
            if !MODULE_VISIBILITIES.contains(&visibility.as_str()) {
                return Err(format!(
                    "{context}.modules.{}.visibility: expected one of {}",
                    module.family,
                    MODULE_VISIBILITIES.join(", ")
                ));
            }
        }
    }
    Ok(())
}

fn register_export_owner<'a>(
    owners: &mut HashMap<&'a str, &'a str>,
    export: &'a str,
    family: &'a str,
    context: &str,
) -> Result<(), String> {
    let Some(previous_family) = owners.insert(export, family) else {
        return Ok(());
    };
    if previous_family != family {
        return Err(format!(
            "{context}.modules.{family}: public export `{export}` already belongs to module family `{previous_family}`"
        ));
    }
    Ok(())
}

fn validate_text(value: &str, context: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{context}: value must not be empty"));
    }
    Ok(())
}
