//! Rust module topology policy validation.

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

fn rust_public_export_families(content: &str) -> Vec<String> {
    let mut families = Vec::new();
    let mut brace_depth = 0usize;
    let mut grouped_use_prefix: Option<String> = None;
    let mut in_block_comment = false;
    for line in content.lines() {
        let active_line = rust_code_without_comments(line, &mut in_block_comment);
        if let Some(prefix) = grouped_use_prefix.as_deref() {
            let (grouped, closed) = grouped_use_families(prefix, &active_line);
            families.extend(grouped);
            if closed {
                grouped_use_prefix = None;
            }
        } else if brace_depth == 0 {
            let (line_families, grouped_prefix) = rust_export_families(&active_line);
            families.extend(line_families);
            grouped_use_prefix = grouped_prefix;
        }
        brace_depth = update_brace_depth(brace_depth, &active_line);
    }
    families.sort();
    families.dedup();
    families
}

fn rust_export_families(line: &str) -> (Vec<String>, Option<String>) {
    let trimmed = line.trim_start();
    if let Some(family) = ident_after_prefix(trimmed, "pub mod ") {
        return (vec![family], None);
    }

    let Some(rest) = trimmed.strip_prefix("pub use ") else {
        return (Vec::new(), None);
    };
    if let Some((prefix, after_open)) = rest.split_once('{') {
        let (families, closed) = grouped_use_families(prefix, after_open);
        let grouped_prefix = (!closed).then(|| prefix.trim().to_string());
        return (families, grouped_prefix);
    }

    (
        use_export_family(rest).into_iter().collect::<Vec<_>>(),
        None,
    )
}

fn ident_after_prefix(line: &str, prefix: &str) -> Option<String> {
    let rest = line.strip_prefix(prefix)?;
    let ident = rest
        .trim_start()
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .next()?;
    (!ident.is_empty()).then(|| ident.to_string())
}

fn rust_code_without_comments(line: &str, in_block_comment: &mut bool) -> String {
    let mut output = String::new();
    let mut rest = line;
    loop {
        if *in_block_comment {
            let Some(end) = rest.find("*/") else {
                break;
            };
            *in_block_comment = false;
            rest = &rest[end + 2..];
        }

        let line_comment = rest.find("//");
        let block_comment = rest.find("/*");
        match (line_comment, block_comment) {
            (Some(line_index), Some(block_index)) if line_index < block_index => {
                output.push_str(&rest[..line_index]);
                break;
            }
            (_, Some(block_index)) => {
                output.push_str(&rest[..block_index]);
                rest = &rest[block_index + 2..];
                *in_block_comment = true;
            }
            (Some(line_index), None) => {
                output.push_str(&rest[..line_index]);
                break;
            }
            (None, None) => {
                output.push_str(rest);
                break;
            }
        }
    }
    output
}

fn grouped_use_families(prefix: &str, fragment: &str) -> (Vec<String>, bool) {
    let content = fragment.split("//").next().unwrap_or(fragment);
    let closed = content.contains('}');
    let content = content
        .split('}')
        .next()
        .unwrap_or(content)
        .trim()
        .trim_end_matches(';');
    let prefix_family = use_export_family(prefix);
    let mut families = Vec::new();
    for entry in content.split(',') {
        let entry = entry.trim().trim_end_matches(';');
        if entry.is_empty() {
            continue;
        }
        if let Some(family) = prefix_family.clone().or_else(|| {
            let path = format!("{prefix}{entry}");
            use_export_family(&path)
        }) {
            families.push(family);
        }
    }
    (families, closed)
}

fn use_export_family(path: &str) -> Option<String> {
    path.trim()
        .trim_end_matches(';')
        .split("::")
        .map(clean_use_segment)
        .filter(|segment| !segment.is_empty())
        .find(|segment| !matches!(segment.as_str(), "crate" | "self" | "super"))
}

fn clean_use_segment(segment: &str) -> String {
    segment
        .trim()
        .trim_start_matches('{')
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .next()
        .unwrap_or_default()
        .to_string()
}

fn update_brace_depth(depth: usize, line: &str) -> usize {
    line.chars().fold(depth, |depth, ch| match ch {
        '{' => depth.saturating_add(1),
        '}' => depth.saturating_sub(1),
        _ => depth,
    })
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

#[cfg(test)]
mod tests {
    use super::rust_public_export_families;

    #[test]
    fn rust_public_exports_extract_modules_and_reexports() {
        let content = r#"
            pub mod cli;
            mod private;
            pub use intelligence::{GraphBuilder, GraphResult};
            pub use crate::{
                config::Config,
                validation::Validator,
            };
            pub mod nested {
                pub mod ignored_inside_body;
            }
        "#;

        assert_eq!(
            rust_public_export_families(content),
            vec!["cli", "config", "intelligence", "nested", "validation"]
        );
    }
}
