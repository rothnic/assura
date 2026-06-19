//! Relationship-specific helpers for first-party custom constraints.

use crate::cli::check::rules::display_rel;
use crate::config::config::{RelationshipConstraintConfig, RelationshipProviderConfig};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(super) fn relationship_provider_expectation(
    project_root: &Path,
    provider: &RelationshipProviderConfig,
    captures: &HashMap<String, String>,
) -> ProviderExpectation {
    let kind = provider.kind.clone().unwrap_or_else(|| {
        if provider.section.is_some() {
            "section".to_string()
        } else {
            "file".to_string()
        }
    });
    let path = expand_named_path_template(&provider.path, captures);
    let section = provider
        .section
        .as_ref()
        .and_then(|section| expand_named_text_template(section, captures));
    let path_exists = path
        .as_ref()
        .is_some_and(|path| project_root.join(path).exists());

    let satisfied = if !path_exists {
        false
    } else if provider.section.is_none() {
        true
    } else {
        path.as_ref()
            .zip(section.as_ref())
            .is_some_and(|(path, section)| {
                let full_path = project_root.join(path);
                fs::read_to_string(full_path)
                    .is_ok_and(|content| markdown_contains_heading(&content, section))
            })
    };

    ProviderExpectation {
        kind,
        declaration: provider.declaration.clone(),
        path_template: provider.path.clone(),
        section_template: provider.section.clone(),
        path,
        section,
        path_exists,
        satisfied,
    }
}

pub(super) fn relationship_missing_message(
    relationship: &RelationshipConstraintConfig,
    entry: &str,
    expectations: &[ProviderExpectation],
) -> String {
    let source_declaration = relationship
        .source_declaration
        .as_deref()
        .unwrap_or(&relationship.source);
    let expected = expectations
        .iter()
        .map(ProviderExpectation::describe)
        .collect::<Vec<_>>()
        .join(" or ");
    format!(
        "Relationship '{}' declared at structure entry '{}' is missing {} for producer '{}' from source pattern '{}'. Expected {}.",
        relationship.need,
        source_declaration,
        missing_relationship_label(&relationship.need),
        display_rel(Path::new(entry)),
        relationship.source,
        expected
    )
}

fn missing_relationship_label(need: &str) -> String {
    if need.starts_with("counterpart") {
        "counterpart".to_string()
    } else {
        format!("provider kind '{need}'")
    }
}

#[derive(Debug)]
pub(super) struct ProviderExpectation {
    kind: String,
    declaration: Option<String>,
    path_template: String,
    section_template: Option<String>,
    path: Option<PathBuf>,
    section: Option<String>,
    path_exists: bool,
    satisfied: bool,
}

impl ProviderExpectation {
    pub(super) fn is_satisfied(&self) -> bool {
        self.satisfied
    }

    fn describe(&self) -> String {
        let declaration = self.declaration.as_deref().unwrap_or(&self.path_template);
        let target = match (&self.path, &self.section) {
            (Some(path), Some(section)) => {
                format!("'{}#{}'", display_rel(path), section)
            }
            (Some(path), None) => format!("'{}'", display_rel(path)),
            (None, _) => format!("template '{}'", self.path_template),
        };
        let state = if self.path.is_none() {
            "could not expand"
        } else if !self.path_exists {
            "missing path"
        } else if self.section_template.is_some() && self.section.is_none() {
            "could not expand section"
        } else if self.section_template.is_some() {
            "missing section"
        } else {
            "missing path"
        };
        format!(
            "{} provider {} ({state}, declared at '{}')",
            self.kind, target, declaration
        )
    }
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

fn is_safe_relative_path(path: &Path) -> bool {
    !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}
