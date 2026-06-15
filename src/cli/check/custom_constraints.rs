//! First-party custom constraints for the structure checker.

use super::rules::{display_rel, is_excluded_rel_with, rel_to_string};
use super::{CheckError, StructureCheckReport, StructureChecker};
use crate::cli::check::command_surface_docs::{
    command_surface_problems, load_command_surface_contract,
};
use crate::config::config::{
    CommandSurfaceContract, CustomConstraintConfig, RelationshipConstraintConfig,
    RelationshipProviderConfig,
};
use glob::Pattern;
use regex_lite::Regex;
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
        if extensions.custom_constraints.is_empty() && extensions.relationships.is_empty() {
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
        self.validate_relationship_constraints(&extensions.relationships, checked_path, report)?;

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

    fn validate_relationship_constraints(
        &self,
        relationships: &[RelationshipConstraintConfig],
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        if relationships.is_empty() {
            return Ok(());
        }

        let entries = self.relationship_entries(checked_path)?;
        for relationship in relationships {
            let source_pattern = CapturePattern::new(&relationship.source);
            let provider_patterns = relationship
                .providers
                .iter()
                .map(|provider| CapturePattern::new(&provider.path))
                .collect::<Vec<_>>();
            for entry in &entries {
                if provider_patterns
                    .iter()
                    .any(|provider_pattern| provider_pattern.captures(entry).is_some())
                {
                    continue;
                }
                let Some(captures) = source_pattern.captures(entry) else {
                    continue;
                };
                if relationship.providers.iter().any(|provider| {
                    relationship_provider_exists(&self.project_root, provider, &captures)
                }) {
                    continue;
                }

                self.push_violation(
                    report,
                    PathBuf::from(entry),
                    format!("relationship:{}", relationship.id),
                    format!(
                        "'{}' must provide relationship '{}'",
                        display_rel(Path::new(entry)),
                        relationship.need
                    ),
                    relationship.severity.as_deref().unwrap_or("medium"),
                );
            }
        }
        Ok(())
    }

    fn relationship_entries(&self, checked_path: &Path) -> Result<Vec<String>, CheckError> {
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

        let mut entries = Vec::new();
        for entry in walker {
            let entry = entry?;
            if entry.path() == checked_path {
                continue;
            }
            let rel = self.relative_path(entry.path());
            if self.is_excluded_rel(&rel) {
                continue;
            }
            entries.push(rel_to_string(&rel));
        }
        entries.sort();
        entries.dedup();
        Ok(entries)
    }
}

fn relationship_provider_exists(
    project_root: &Path,
    provider: &RelationshipProviderConfig,
    captures: &HashMap<String, String>,
) -> bool {
    let Some(path) = expand_named_path_template(&provider.path, captures) else {
        return false;
    };
    let full_path = project_root.join(&path);
    if !full_path.exists() {
        return false;
    }

    let Some(section) = &provider.section else {
        return true;
    };
    let Some(section) = expand_named_text_template(section, captures) else {
        return false;
    };
    let Ok(content) = fs::read_to_string(full_path) else {
        return false;
    };
    markdown_contains_heading(&content, &section)
}

fn expand_named_path_template(
    template: &str,
    captures: &HashMap<String, String>,
) -> Option<PathBuf> {
    let expanded = expand_named_text_template(template, captures)?;
    let path = PathBuf::from(expanded);
    is_safe_relative_path(&path).then_some(path)
}

fn expand_named_text_template(
    template: &str,
    captures: &HashMap<String, String>,
) -> Option<String> {
    let mut expanded = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        expanded.push_str(&rest[..start]);
        let after_start = &rest[start + 1..];
        let end = after_start.find('}')?;
        let name = &after_start[..end];
        expanded.push_str(captures.get(name)?);
        rest = &after_start[end + 1..];
    }
    expanded.push_str(rest);
    Some(expanded)
}

fn markdown_contains_heading(content: &str, expected: &str) -> bool {
    content.lines().any(|line| {
        let trimmed = line.trim_start();
        let depth = trimmed.chars().take_while(|ch| *ch == '#').count();
        if !(1..=6).contains(&depth) {
            return false;
        }
        let after_marks = &trimmed[depth..];
        after_marks
            .chars()
            .next()
            .is_some_and(|ch| ch.is_whitespace())
            && after_marks.trim().trim_end_matches('#').trim_end().trim() == expected
    })
}

struct CapturePattern {
    regex: Option<Regex>,
    names: Vec<String>,
}

impl CapturePattern {
    fn new(pattern: &str) -> Self {
        let pattern = pattern.trim_end_matches('/');
        let mut regex = String::from("^");
        let mut names = Vec::new();
        let mut rest = pattern;
        while let Some(start) = rest.find('{') {
            push_pattern_literal(&mut regex, &rest[..start]);
            let after_start = &rest[start + 1..];
            let Some(end) = after_start.find('}') else {
                return Self { regex: None, names };
            };
            names.push(after_start[..end].to_string());
            regex.push_str("([^/]+)");
            rest = &after_start[end + 1..];
        }
        push_pattern_literal(&mut regex, rest);
        regex.push('$');
        Self {
            regex: Regex::new(&regex).ok(),
            names,
        }
    }

    fn captures(&self, path: &str) -> Option<HashMap<String, String>> {
        let captures = self.regex.as_ref()?.captures(path.trim_end_matches('/'))?;
        let mut values = HashMap::new();
        for (index, name) in self.names.iter().enumerate() {
            values.insert(name.clone(), captures.get(index + 1)?.as_str().to_string());
        }
        Some(values)
    }
}

fn push_pattern_literal(regex: &mut String, literal: &str) {
    for ch in literal.chars() {
        match ch {
            '*' => regex.push_str("[^/]*"),
            '?' => regex.push_str("[^/]"),
            '.' | '+' | '(' | ')' | '[' | ']' | '^' | '$' | '|' | '\\' => {
                regex.push('\\');
                regex.push(ch);
            }
            other => regex.push(other),
        }
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
