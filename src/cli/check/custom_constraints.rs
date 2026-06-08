//! First-party custom constraints for the structure checker.

use super::rules::{display_rel, is_excluded_rel_with, rel_to_string};
use super::{CheckError, StructureCheckReport, StructureChecker};
use crate::cli::check::command_surface_docs::{
    command_surface_problems, load_command_surface_contract,
};
use crate::config::config::{CommandSurfaceContract, CustomConstraintConfig};
use glob::Pattern;
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

impl StructureChecker {
    pub(super) fn validate_custom_constraints(
        &self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let Some(extensions) = &self.config.extensions else {
            return Ok(());
        };
        if extensions.custom_constraints.is_empty() {
            return Ok(());
        }

        let mut command_surface_constraints = Vec::new();
        for constraint in &extensions.custom_constraints {
            match constraint.kind.as_str() {
                "paired_file_exists" => {
                    self.validate_paired_file_exists(constraint, checked_path, report)?;
                }
                "command_surface_docs" => {
                    command_surface_constraints.push(constraint);
                }
                _ => {}
            }
        }
        self.validate_command_surface_docs_constraints(
            &command_surface_constraints,
            checked_path,
            report,
        )?;

        Ok(())
    }

    fn validate_paired_file_exists(
        &self,
        constraint: &CustomConstraintConfig,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let sources = self.matching_custom_constraint_sources(constraint, checked_path)?;

        for source_rel in sources {
            let Some(target_rel) = expand_target_template(&constraint.target, &source_rel) else {
                continue;
            };
            if self.is_excluded_rel(&target_rel) {
                continue;
            }
            if !self.project_root.join(&target_rel).exists() {
                self.push_violation(
                    report,
                    source_rel.clone(),
                    format!("custom:{}", constraint.id),
                    format!(
                        "File '{}' requires paired file '{}'",
                        display_rel(&source_rel),
                        display_rel(&target_rel)
                    ),
                    constraint.severity.as_deref().unwrap_or("medium"),
                );
            }
        }

        Ok(())
    }

    fn validate_command_surface_docs_constraints(
        &self,
        constraints: &[&CustomConstraintConfig],
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        if constraints.is_empty() {
            return Ok(());
        }

        let compiled_constraints = constraints
            .iter()
            .filter_map(|constraint| {
                Pattern::new(&constraint.source)
                    .ok()
                    .map(|pattern| (*constraint, pattern))
            })
            .collect::<Vec<_>>();
        let mut contract_cache: HashMap<String, CommandSurfaceContract> = HashMap::new();
        let project_root = self.project_root.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let walker = walkdir::WalkDir::new(checked_path)
            .into_iter()
            .filter_entry(move |entry| {
                let path = entry.path();
                if path == checked_path {
                    return true;
                }
                let rel = path.strip_prefix(&project_root).unwrap_or(path);
                !is_excluded_rel_with(&exclude_patterns, rel)
            });

        for entry in walker {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let source_rel = self.relative_path(entry.path());
            if self.is_excluded_rel(&source_rel) {
                continue;
            }
            let matching_constraints = compiled_constraints
                .iter()
                .filter(|(constraint, pattern)| {
                    pattern.matches_path(&source_rel)
                        && source_pattern_depth_allows(&constraint.source, &source_rel)
                })
                .collect::<Vec<_>>();
            if matching_constraints.is_empty() {
                continue;
            }

            let content = fs::read_to_string(self.project_root.join(&source_rel))?;
            for (constraint, _) in matching_constraints {
                if !contract_cache.contains_key(&constraint.target) {
                    let contract_path = self.project_root.join(&constraint.target);
                    contract_cache.insert(
                        constraint.target.clone(),
                        load_command_surface_contract(&contract_path)?,
                    );
                }
                let contract = contract_cache
                    .get(&constraint.target)
                    .expect("contract cache was populated");
                for problem in command_surface_problems(contract, &content) {
                    self.push_violation(
                        report,
                        source_rel.clone(),
                        format!("custom:{}", constraint.id),
                        format!(
                            "Documented command `{}` {}",
                            problem.example, problem.message
                        ),
                        constraint.severity.as_deref().unwrap_or("medium"),
                    );
                }
            }
        }

        Ok(())
    }

    fn matching_custom_constraint_sources(
        &self,
        constraint: &CustomConstraintConfig,
        checked_path: &Path,
    ) -> Result<Vec<PathBuf>, CheckError> {
        let Ok(source_pattern) = Pattern::new(&constraint.source) else {
            return Ok(Vec::new());
        };

        let mut sources = Vec::new();
        let project_root = self.project_root.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let walker = walkdir::WalkDir::new(checked_path)
            .into_iter()
            .filter_entry(move |entry| {
                let path = entry.path();
                if path == checked_path {
                    return true;
                }
                let rel = path.strip_prefix(&project_root).unwrap_or(path);
                !is_excluded_rel_with(&exclude_patterns, rel)
            });

        for entry in walker {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = self.relative_path(entry.path());
            if self.is_excluded_rel(&rel) {
                continue;
            }
            if source_pattern.matches_path(&rel)
                && source_pattern_depth_allows(&constraint.source, &rel)
            {
                sources.push(rel);
            }
        }
        sources.sort();
        sources.dedup();
        Ok(sources)
    }
}

fn expand_target_template(template: &str, source_rel: &Path) -> Option<PathBuf> {
    let source_name = source_rel.file_name()?.to_string_lossy();
    let source_stem = source_rel.file_stem()?.to_string_lossy();
    let source_parent = source_rel
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(rel_to_string)
        .unwrap_or_default();

    let expanded = template
        .replace("{source}", &rel_to_string(source_rel))
        .replace("{source_name}", &source_name)
        .replace("{source_stem}", &source_stem)
        .replace("{stem}", &source_stem);

    let expanded = if source_parent.is_empty() {
        expanded
            .replace("{source_parent}/", "")
            .replace("{source_parent}", "")
    } else {
        expanded.replace("{source_parent}", &source_parent)
    };

    let path = PathBuf::from(expanded);
    is_safe_relative_path(&path).then_some(path)
}

fn source_pattern_depth_allows(pattern: &str, path: &Path) -> bool {
    if pattern.contains("**") {
        return true;
    }
    pattern.split('/').filter(|part| !part.is_empty()).count() == path.components().count()
}

fn is_safe_relative_path(path: &Path) -> bool {
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_template_expands_source_stem_and_parent() {
        assert_eq!(
            expand_target_template(
                "tests/{source_parent}/{stem}_test.rs",
                Path::new("src/core/config.rs")
            )
            .unwrap(),
            PathBuf::from("tests/src/core/config_test.rs")
        );
    }

    #[test]
    fn target_template_drops_empty_source_parent_prefix() {
        assert_eq!(
            expand_target_template("{source_parent}/{stem}_test.rs", Path::new("README.md"))
                .unwrap(),
            PathBuf::from("README_test.rs")
        );
    }

    #[test]
    fn target_template_rejects_parent_escape() {
        assert!(
            expand_target_template("../tests/{stem}_test.rs", Path::new("src/config.rs")).is_none()
        );
    }
}
