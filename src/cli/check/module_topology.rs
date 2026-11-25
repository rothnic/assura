//! Rust module topology policy validation.

use super::rust_exports::rust_public_export_families;
use super::{CheckError, StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::{ModuleTopologyConfig, ModuleTopologyModuleConfig};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

impl StructureChecker {
    pub(super) fn validate_module_topologies(
        &self,
        policies: &[ModuleTopologyConfig],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for policy in policies {
            self.validate_module_topology_policy(policy, report)?;
        }
        Ok(())
    }

    fn validate_module_topology_policy(
        &self,
        policy: &ModuleTopologyConfig,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        self.validate_module_roots(policy, report)?;
        self.validate_public_exports(policy, report)?;
        Ok(())
    }

    fn validate_module_roots(
        &self,
        policy: &ModuleTopologyConfig,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for module in &policy.modules {
            for root in &module.roots {
                let rel = safe_module_topology_path(root)?;
                if !self.project_root.join(&rel).exists() {
                    self.push_module_topology_violation(
                        report,
                        policy,
                        rel,
                        format!(
                            "Module topology `{}` declares module family `{}` root `{root}` but it does not exist",
                            policy.id, module.family
                        ),
                    );
                }
            }
        }
        Ok(())
    }

    fn validate_public_exports(
        &self,
        policy: &ModuleTopologyConfig,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let modules = module_rows_by_export(&policy.modules);
        for export_path in &policy.rust_exports {
            let rel = safe_module_topology_path(export_path)?;
            let path = self.project_root.join(&rel);
            if !path.exists() {
                self.push_module_topology_violation(
                    report,
                    policy,
                    rel,
                    format!(
                        "Module topology `{}` configured Rust export file `{export_path}` does not exist",
                        policy.id
                    ),
                );
                continue;
            }
            let content = fs::read_to_string(&path)?;
            for export in rust_public_export_families(&content) {
                let Some(module) = modules.get(export.as_str()) else {
                    self.push_module_topology_violation(
                        report,
                        policy,
                        rel.clone(),
                        format!(
                            "Module topology `{}` does not classify public module/export `{export}` from `{}`",
                            policy.id,
                            display_rel_path(&rel)
                        ),
                    );
                    continue;
                };
                if public_export_conflicts(module) {
                    self.push_module_topology_violation(
                        report,
                        policy,
                        rel.clone(),
                        format!(
                            "Module topology `{}` marks module family `{}` as `{}` but `{}` publicly exports `{export}`",
                            policy.id,
                            module.family,
                            module_visibility_label(module),
                            display_rel_path(&rel)
                        ),
                    );
                }
            }
        }
        Ok(())
    }

    fn push_module_topology_violation(
        &self,
        report: &mut StructureCheckReport,
        policy: &ModuleTopologyConfig,
        path: PathBuf,
        message: String,
    ) {
        report.violations.push(StructureViolation::new(
            path,
            format!("module_topology:{}", policy.id),
            message,
            policy.severity.as_deref().unwrap_or("medium"),
        ));
    }
}

fn module_rows_by_export(
    modules: &[ModuleTopologyModuleConfig],
) -> HashMap<&str, &ModuleTopologyModuleConfig> {
    let mut rows = HashMap::new();
    for module in modules {
        rows.insert(module.family.as_str(), module);
        for public_export in &module.public_exports {
            rows.insert(public_export.as_str(), module);
        }
    }
    rows
}

fn public_export_conflicts(module: &ModuleTopologyModuleConfig) -> bool {
    module.visibility.as_deref() == Some("internal") || module.status == "unsupported"
}

fn module_visibility_label(module: &ModuleTopologyModuleConfig) -> &str {
    if module.visibility.as_deref() == Some("internal") {
        "internal-only"
    } else {
        module.status.as_str()
    }
}

fn safe_module_topology_path(configured_path: &str) -> Result<PathBuf, CheckError> {
    let rel = PathBuf::from(configured_path);
    if rel.as_os_str().is_empty()
        || rel.is_absolute()
        || !rel
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(CheckError::Config(
            crate::cli::config::ConfigError::Invalid(format!(
                "module topology path `{configured_path}` must be project-relative and must not use parent traversal"
            )),
        ));
    }
    Ok(rel)
}

fn display_rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
