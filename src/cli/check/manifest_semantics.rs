//! Cargo manifest semantic policy validation.

use super::{CheckError, StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::{ManifestSemanticsConfig, ManifestSemanticsManifestConfig};
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
use toml::Value;

impl StructureChecker {
    pub(super) fn validate_manifest_semantics(
        &self,
        policies: &[ManifestSemanticsConfig],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for policy in policies {
            self.validate_manifest_semantic_policy(policy, report)?;
        }
        Ok(())
    }

    fn validate_manifest_semantic_policy(
        &self,
        policy: &ManifestSemanticsConfig,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let workspace_package = workspace_package_table(&self.project_root)?;
        for manifest in &policy.manifests {
            let rel = safe_manifest_semantics_path(&manifest.path)?;
            let path = self.project_root.join(&rel);
            if !path.exists() {
                self.push_manifest_semantics_violation(
                    report,
                    policy,
                    rel,
                    format!(
                        "Manifest semantics `{}` configured manifest `{}` does not exist",
                        policy.id, manifest.path
                    ),
                );
                continue;
            }

            let content = fs::read_to_string(&path)?;
            let parsed = match content.parse::<Value>() {
                Ok(parsed) => parsed,
                Err(error) => {
                    self.push_manifest_semantics_violation(
                        report,
                        policy,
                        rel,
                        format!(
                            "Manifest semantics `{}` could not parse `{}` as TOML: {error}",
                            policy.id, manifest.path
                        ),
                    );
                    continue;
                }
            };
            self.validate_manifest(
                policy,
                manifest,
                &rel,
                &parsed,
                workspace_package.as_ref(),
                report,
            );
        }
        Ok(())
    }

    fn validate_manifest(
        &self,
        policy: &ManifestSemanticsConfig,
        manifest: &ManifestSemanticsManifestConfig,
        rel: &Path,
        parsed: &Value,
        workspace_package: Option<&toml::map::Map<String, Value>>,
        report: &mut StructureCheckReport,
    ) {
        let Some(package) = parsed.get("package").and_then(Value::as_table) else {
            self.push_manifest_semantics_violation(
                report,
                policy,
                rel.to_path_buf(),
                format!(
                    "Manifest semantics `{}` expects `{}` to declare a [package] table",
                    policy.id, manifest.path
                ),
            );
            return;
        };
        let package_name = manifest
            .package
            .as_deref()
            .or_else(|| package.get("name").and_then(Value::as_str))
            .unwrap_or("<unknown>");
        let context = ManifestCheckContext {
            policy,
            rel,
            package_name,
            workspace_package,
        };

        self.validate_expected_string(&context, package, "name", &manifest.package, report);
        self.validate_expected_string(&context, package, "version", &manifest.version, report);
        self.validate_expected_string(
            &context,
            package,
            "rust-version",
            &manifest.rust_version,
            report,
        );
        self.validate_expected_string(&context, package, "license", &manifest.license, report);
        self.validate_description_terms(policy, manifest, rel, package, package_name, report);
        self.validate_keywords(policy, manifest, rel, package, package_name, report);
        self.validate_publish_policy(policy, manifest, rel, package, package_name, report);
        self.validate_binaries(policy, manifest, rel, parsed, package_name, report);
    }

    fn validate_expected_string(
        &self,
        context: &ManifestCheckContext<'_>,
        package: &toml::map::Map<String, Value>,
        field: &str,
        expected: &Option<String>,
        report: &mut StructureCheckReport,
    ) {
        let Some(expected) = expected else {
            return;
        };
        let actual = package_string_field(package, context.workspace_package, field);
        if actual.as_deref() != Some(expected.as_str()) {
            self.push_manifest_semantics_violation(
                report,
                context.policy,
                context.rel.to_path_buf(),
                format!(
                    "Manifest semantics `{id}` expects package `{package_name}` field `{field}` in `{path}` to be `{expected}`, found {actual}",
                    id = context.policy.id,
                    package_name = context.package_name,
                    path = display_rel_path(context.rel),
                    actual = display_value(actual.as_deref())
                ),
            );
        }
    }

    fn validate_description_terms(
        &self,
        policy: &ManifestSemanticsConfig,
        manifest: &ManifestSemanticsManifestConfig,
        rel: &Path,
        package: &toml::map::Map<String, Value>,
        package_name: &str,
        report: &mut StructureCheckReport,
    ) {
        let description = package
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default();
        let normalized = description.to_ascii_lowercase();
        for term in &manifest.description_required_terms {
            if !normalized.contains(&term.to_ascii_lowercase()) {
                self.push_manifest_semantics_violation(
                    report,
                    policy,
                    rel.to_path_buf(),
                    format!(
                        "Manifest semantics `{}` expects package `{package_name}` description in `{}` to contain term `{term}`",
                        policy.id,
                        display_rel_path(rel)
                    ),
                );
            }
        }
        for term in &manifest.description_forbidden_terms {
            if normalized.contains(&term.to_ascii_lowercase()) {
                self.push_manifest_semantics_violation(
                    report,
                    policy,
                    rel.to_path_buf(),
                    format!(
                        "Manifest semantics `{}` forbids package `{package_name}` description in `{}` from containing term `{term}`",
                        policy.id,
                        display_rel_path(rel)
                    ),
                );
            }
        }
    }

    fn validate_keywords(
        &self,
        policy: &ManifestSemanticsConfig,
        manifest: &ManifestSemanticsManifestConfig,
        rel: &Path,
        package: &toml::map::Map<String, Value>,
        package_name: &str,
        report: &mut StructureCheckReport,
    ) {
        if manifest.keywords.is_empty() {
            return;
        }
        let declared = package
            .get("keywords")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        for keyword in &manifest.keywords {
            if !declared.contains(keyword.as_str()) {
                self.push_manifest_semantics_violation(
                    report,
                    policy,
                    rel.to_path_buf(),
                    format!(
                        "Manifest semantics `{}` expects package `{package_name}` keywords in `{}` to include `{keyword}`",
                        policy.id,
                        display_rel_path(rel)
                    ),
                );
            }
        }
    }

    fn validate_publish_policy(
        &self,
        policy: &ManifestSemanticsConfig,
        manifest: &ManifestSemanticsManifestConfig,
        rel: &Path,
        package: &toml::map::Map<String, Value>,
        package_name: &str,
        report: &mut StructureCheckReport,
    ) {
        let Some(expected) = manifest.publish.as_deref().or(manifest.role.as_deref()) else {
            return;
        };
        let Some(actual) = publish_state(package.get("publish")) else {
            self.push_manifest_semantics_violation(
                report,
                policy,
                rel.to_path_buf(),
                format!(
                    "Manifest semantics `{}` expects package `{package_name}` publish field in `{}` to be boolean or registry array",
                    policy.id,
                    display_rel_path(rel)
                ),
            );
            return;
        };
        if actual != expected {
            self.push_manifest_semantics_violation(
                report,
                policy,
                rel.to_path_buf(),
                format!(
                    "Manifest semantics `{}` expects package `{package_name}` publish policy in `{}` to be `{expected}`, found `{actual}`",
                    policy.id,
                    display_rel_path(rel)
                ),
            );
        }
    }

    fn validate_binaries(
        &self,
        policy: &ManifestSemanticsConfig,
        manifest: &ManifestSemanticsManifestConfig,
        rel: &Path,
        parsed: &Value,
        package_name: &str,
        report: &mut StructureCheckReport,
    ) {
        if manifest.binaries.is_empty() {
            return;
        }
        let declared = parsed
            .get("bin")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(|value| value.get("name").and_then(Value::as_str))
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        for binary in &manifest.binaries {
            if !declared.contains(binary.as_str()) {
                self.push_manifest_semantics_violation(
                    report,
                    policy,
                    rel.to_path_buf(),
                    format!(
                        "Manifest semantics `{}` expects package `{package_name}` in `{}` to declare binary `{binary}`",
                        policy.id,
                        display_rel_path(rel)
                    ),
                );
            }
        }
    }

    fn push_manifest_semantics_violation(
        &self,
        report: &mut StructureCheckReport,
        policy: &ManifestSemanticsConfig,
        path: PathBuf,
        message: String,
    ) {
        report.violations.push(StructureViolation::new(
            path,
            format!("manifest_semantics:{}", policy.id),
            message,
            policy.severity.as_deref().unwrap_or("medium"),
        ));
    }
}

struct ManifestCheckContext<'a> {
    policy: &'a ManifestSemanticsConfig,
    rel: &'a Path,
    package_name: &'a str,
    workspace_package: Option<&'a toml::map::Map<String, Value>>,
}

fn workspace_package_table(
    project_root: &Path,
) -> Result<Option<toml::map::Map<String, Value>>, CheckError> {
    let path = project_root.join("Cargo.toml");
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let Ok(parsed) = content.parse::<Value>() else {
        return Ok(None);
    };
    Ok(parsed
        .get("workspace")
        .and_then(|workspace| workspace.get("package"))
        .and_then(Value::as_table)
        .cloned())
}

fn package_string_field(
    package: &toml::map::Map<String, Value>,
    workspace_package: Option<&toml::map::Map<String, Value>>,
    field: &str,
) -> Option<String> {
    match package.get(field) {
        Some(Value::String(value)) => Some(value.clone()),
        Some(Value::Table(table))
            if table.get("workspace").and_then(Value::as_bool) == Some(true) =>
        {
            workspace_package?
                .get(field)
                .and_then(Value::as_str)
                .map(ToString::to_string)
        }
        _ => None,
    }
}

fn publish_state(value: Option<&Value>) -> Option<&'static str> {
    match value {
        None => Some("public"),
        Some(Value::Boolean(true)) => Some("public"),
        Some(Value::Boolean(false)) => Some("internal"),
        Some(Value::Array(registries)) if registries.is_empty() => Some("internal"),
        Some(Value::Array(_)) => Some("public"),
        Some(_) => None,
    }
}

fn display_value(value: Option<&str>) -> String {
    value
        .map(|value| format!("`{value}`"))
        .unwrap_or_else(|| "nothing".to_string())
}

fn display_rel_path(path: &Path) -> String {
    path.display().to_string()
}

fn safe_manifest_semantics_path(configured_path: &str) -> Result<PathBuf, CheckError> {
    let rel = PathBuf::from(configured_path);
    if rel.as_os_str().is_empty()
        || rel.is_absolute()
        || !rel
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(CheckError::Config(
            crate::cli::config::ConfigError::Invalid(format!(
                "manifest semantics path `{configured_path}` must be project-relative and must not use parent traversal"
            )),
        ));
    }
    Ok(rel)
}
