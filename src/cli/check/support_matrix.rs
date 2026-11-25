//! Public surface support-matrix validation.

use super::command_surface_docs::load_command_surface_contract;
use super::rust_exports::rust_public_export_families;
use super::support_matrix_docs::docs_claim_surfaces;
use super::{CheckError, StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::{ManifestSemanticsConfig, SupportMatrixConfig};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

impl StructureChecker {
    pub(super) fn validate_support_matrices(
        &self,
        matrices: &[SupportMatrixConfig],
        manifest_semantics: &[ManifestSemanticsConfig],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for matrix in matrices {
            self.validate_support_matrix(matrix, manifest_semantics, report)?;
        }
        Ok(())
    }

    fn validate_support_matrix(
        &self,
        matrix: &SupportMatrixConfig,
        manifest_semantics: &[ManifestSemanticsConfig],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let classified = matrix
            .entries
            .iter()
            .map(|entry| (entry.surface.as_str(), entry.status.as_str()))
            .collect::<HashMap<_, _>>();

        for contract_path in &matrix.command_contracts {
            let rel = safe_support_matrix_path(contract_path)?;
            let path = self.project_root.join(&rel);
            if !path.exists() {
                self.push_support_matrix_violation(
                    report,
                    matrix,
                    rel,
                    format!(
                        "Support matrix `{}` configured command contract `{contract_path}` does not exist",
                        matrix.id
                    ),
                );
                continue;
            }
            let contract = load_command_surface_contract(&path)?;
            for command in contract.commands {
                let surface = format!("command:{}", command.name);
                if !classified.contains_key(surface.as_str()) {
                    self.push_support_matrix_violation(
                        report,
                        matrix,
                        rel.clone(),
                        format!(
                            "Support matrix `{}` does not classify command surface `{surface}` from `{}`",
                            matrix.id,
                            display_rel_path(&rel)
                        ),
                    );
                }
            }
        }

        for export_path in &matrix.rust_exports {
            let rel = safe_support_matrix_path(export_path)?;
            let path = self.project_root.join(&rel);
            if !path.exists() {
                self.push_support_matrix_violation(
                    report,
                    matrix,
                    rel,
                    format!(
                        "Support matrix `{}` configured Rust export file `{export_path}` does not exist",
                        matrix.id
                    ),
                );
                continue;
            }
            let content = fs::read_to_string(&path)?;
            for family in rust_public_export_families(&content) {
                let surface = format!("rust:{family}");
                if !classified.contains_key(surface.as_str()) {
                    self.push_support_matrix_violation(
                        report,
                        matrix,
                        rel.clone(),
                        format!(
                            "Support matrix `{}` does not classify Rust export surface `{surface}` from `{}`",
                            matrix.id,
                            display_rel_path(&rel)
                        ),
                    );
                }
            }
        }

        for docs_claim_source in &matrix.docs_claim_sources {
            let rel = safe_support_matrix_path(&docs_claim_source.path)?;
            let path = self.project_root.join(&rel);
            if !path.exists() {
                self.push_support_matrix_violation(
                    report,
                    matrix,
                    rel,
                    format!(
                        "Support matrix `{}` configured docs claim source `{}` does not exist",
                        matrix.id, docs_claim_source.path
                    ),
                );
                continue;
            }
            let content = fs::read_to_string(&path)?;
            for claim in docs_claim_surfaces(&content) {
                let Some(configured_status) = classified.get(claim.surface.as_str()) else {
                    self.push_support_matrix_violation(
                        report,
                        matrix,
                        rel.clone(),
                        format!(
                            "Support matrix `{}` does not classify docs support-claim surface `{}` claimed as `{}` from `{}`",
                            matrix.id,
                            claim.surface,
                            claim.status,
                            display_rel_path(&rel)
                        ),
                    );
                    continue;
                };
                if claim.status == "supported" && *configured_status != "supported" {
                    self.push_support_matrix_violation(
                        report,
                        matrix,
                        rel.clone(),
                        format!(
                            "Support matrix `{}` classifies docs support-claim surface `{}` as `{}`, but `{}` says it is `supported`",
                            matrix.id,
                            claim.surface,
                            configured_status,
                            display_rel_path(&rel)
                        ),
                    );
                }
            }
        }

        let manifest_semantics_by_id = manifest_semantics
            .iter()
            .map(|policy| (policy.id.as_str(), policy))
            .collect::<HashMap<_, _>>();
        for manifest_policy_id in &matrix.manifest_policies {
            let Some(manifest_policy) = manifest_semantics_by_id.get(manifest_policy_id.as_str())
            else {
                self.push_support_matrix_violation(
                    report,
                    matrix,
                    PathBuf::from(".assura/config.yml"),
                    format!(
                        "Support matrix `{}` references unknown manifest semantics policy `{manifest_policy_id}`",
                        matrix.id
                    ),
                );
                continue;
            };
            for manifest in &manifest_policy.manifests {
                let rel = safe_support_matrix_path(&manifest.path)?;
                if let Some(package) = &manifest.package {
                    let surface = format!("package:{package}");
                    if !classified.contains_key(surface.as_str()) {
                        self.push_support_matrix_violation(
                            report,
                            matrix,
                            rel.clone(),
                            format!(
                                "Support matrix `{}` does not classify package surface `{surface}` from manifest policy `{manifest_policy_id}`",
                                matrix.id
                            ),
                        );
                    }
                }
                for binary in &manifest.binaries {
                    let surface = format!("binary:{binary}");
                    if !classified.contains_key(surface.as_str()) {
                        self.push_support_matrix_violation(
                            report,
                            matrix,
                            rel.clone(),
                            format!(
                                "Support matrix `{}` does not classify binary surface `{surface}` from manifest policy `{manifest_policy_id}`",
                                matrix.id
                            ),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    fn push_support_matrix_violation(
        &self,
        report: &mut StructureCheckReport,
        matrix: &SupportMatrixConfig,
        path: PathBuf,
        message: String,
    ) {
        report.violations.push(StructureViolation::new(
            path,
            format!("support_matrix:{}", matrix.id),
            message,
            matrix.severity.as_deref().unwrap_or("medium"),
        ));
    }
}

fn safe_support_matrix_path(configured_path: &str) -> Result<PathBuf, CheckError> {
    let rel = PathBuf::from(configured_path);
    if rel.as_os_str().is_empty()
        || rel.is_absolute()
        || !rel
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(CheckError::Config(
            crate::cli::config::ConfigError::Invalid(format!(
                "support matrix path `{configured_path}` must be project-relative and must not use parent traversal"
            )),
        ));
    }
    Ok(rel)
}

fn display_rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
