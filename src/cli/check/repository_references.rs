//! Repository-reference diagnostics for locally recognizable source references.

use super::rules::display_rel;
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::RepositoryReferenceConfig;
use crate::intelligence::facts::{source_references, SourceReference};
use crate::markdown_links::is_markdown_file;
use glob::Pattern;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub(super) const SOURCE_REFERENCE_FILE_SIZE_LIMIT: u64 = 512 * 1024;

impl StructureChecker {
    pub(super) fn has_repository_reference_diagnostics_config(&self) -> bool {
        self.config
            .extensions
            .as_ref()
            .is_some_and(|extensions| !extensions.repository_references.is_empty())
    }

    pub(super) fn repository_reference_severity_for_path(&self, rel: &Path) -> Option<String> {
        let policies = &self.config.extensions.as_ref()?.repository_references;
        policies
            .iter()
            .find(|policy| repository_reference_policy_matches(policy, rel))
            .map(|policy| policy.severity.as_deref().unwrap_or("medium").to_string())
    }

    pub(super) fn validate_repository_references(
        &self,
        rel: &Path,
        content: &str,
        severity: &str,
        report: &mut StructureCheckReport,
    ) {
        let mut target_cache = HashMap::new();
        for reference in source_references(rel, content) {
            let target_path = self.project_root.join(&reference.target_path);
            if !target_path.is_file() {
                self.push_repository_reference_violation(
                    report,
                    rel,
                    &reference,
                    "repository_reference_target",
                    format!(
                        "File '{}' references missing local target '{}' on line {}, column {} [{}; confidence={}]",
                        display_rel(rel),
                        display_rel(&reference.target_path),
                        reference.line_number,
                        reference.column_number,
                        reference.kind,
                        reference.confidence
                    ),
                    severity,
                );
                continue;
            }

            if reference.target_line_start.is_some() {
                self.validate_repository_reference_line(
                    rel,
                    &reference,
                    &target_path,
                    severity,
                    report,
                    &mut target_cache,
                );
            } else if reference.target_anchor.is_some() && is_markdown_file(&reference.target_path)
            {
                self.validate_repository_reference_markdown_anchor(
                    rel,
                    &reference,
                    &target_path,
                    severity,
                    report,
                    &mut target_cache,
                );
            }
        }
    }

    fn validate_repository_reference_line(
        &self,
        rel: &Path,
        reference: &SourceReference,
        target_path: &Path,
        severity: &str,
        report: &mut StructureCheckReport,
        target_cache: &mut HashMap<std::path::PathBuf, Option<String>>,
    ) {
        let Some(content) = cached_target_content(target_cache, target_path) else {
            return;
        };
        let line_count = content.lines().count();
        let start = reference.target_line_start.unwrap_or(0);
        let end = reference.target_line_end.unwrap_or(start);
        if start == 0 || start > line_count || end < start || end > line_count {
            self.push_repository_reference_violation(
                report,
                rel,
                reference,
                "repository_reference_line_anchor",
                format!(
                    "File '{}' references invalid line range '{}' in '{}' on line {}, column {}; target has {} line(s)",
                    display_rel(rel),
                    target_line_label(reference),
                    display_rel(&reference.target_path),
                    reference.line_number,
                    reference.column_number,
                    line_count
                ),
                severity,
            );
        }
    }

    fn validate_repository_reference_markdown_anchor(
        &self,
        rel: &Path,
        reference: &SourceReference,
        target_path: &Path,
        severity: &str,
        report: &mut StructureCheckReport,
        target_cache: &mut HashMap<std::path::PathBuf, Option<String>>,
    ) {
        let Some(content) = cached_target_content(target_cache, target_path) else {
            return;
        };
        let anchor = reference.target_anchor.as_deref().unwrap_or_default();
        if !github_heading_slugs(content).contains(anchor) {
            self.push_repository_reference_violation(
                report,
                rel,
                reference,
                "repository_reference_anchor",
                format!(
                    "File '{}' references missing Markdown heading anchor '#{}' in '{}' on line {}, column {}",
                    display_rel(rel),
                    anchor,
                    display_rel(&reference.target_path),
                    reference.line_number,
                    reference.column_number
                ),
                severity,
            );
        }
    }

    fn push_repository_reference_violation(
        &self,
        report: &mut StructureCheckReport,
        rel: &Path,
        reference: &SourceReference,
        rule: &str,
        message: String,
        severity: &str,
    ) {
        let _ = reference;
        self.push_violation(report, rel.to_path_buf(), rule, message, severity);
    }
}

pub(super) fn is_source_reference_file(path: &Path) -> bool {
    let Some(extension) = path.extension().and_then(|extension| extension.to_str()) else {
        return false;
    };
    matches!(
        extension.to_ascii_lowercase().as_str(),
        "rs" | "py"
            | "js"
            | "jsx"
            | "ts"
            | "tsx"
            | "go"
            | "java"
            | "kt"
            | "swift"
            | "c"
            | "h"
            | "hpp"
            | "cpp"
            | "cs"
            | "rb"
            | "php"
            | "sh"
            | "bash"
            | "zsh"
            | "fish"
            | "toml"
            | "yaml"
            | "yml"
            | "json"
            | "jsonl"
    )
}

fn repository_reference_policy_matches(policy: &RepositoryReferenceConfig, rel: &Path) -> bool {
    if policy.paths.is_empty() {
        return true;
    }
    let rel = rel.to_string_lossy().replace('\\', "/");
    policy
        .paths
        .iter()
        .any(|pattern| Pattern::new(pattern).is_ok_and(|pattern| pattern.matches(&rel)))
}

fn cached_target_content<'a>(
    cache: &'a mut HashMap<std::path::PathBuf, Option<String>>,
    target_path: &Path,
) -> Option<&'a str> {
    cache
        .entry(target_path.to_path_buf())
        .or_insert_with(|| std::fs::read_to_string(target_path).ok())
        .as_deref()
}

fn target_line_label(reference: &SourceReference) -> String {
    match (reference.target_line_start, reference.target_line_end) {
        (Some(start), Some(end)) if start == end => format!("L{start}"),
        (Some(start), Some(end)) => format!("L{start}-L{end}"),
        (Some(start), None) => format!("L{start}"),
        _ => "-".to_string(),
    }
}

fn github_heading_slugs(content: &str) -> HashSet<String> {
    let mut counts = HashMap::<String, usize>::new();
    let mut slugs = HashSet::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let depth = trimmed.chars().take_while(|ch| *ch == '#').count();
        if depth == 0 || depth > 6 || !trimmed.chars().nth(depth).is_some_and(char::is_whitespace) {
            continue;
        }
        let base = github_heading_slug(trimmed[depth..].trim());
        let count = counts.entry(base.clone()).or_insert(0);
        let slug = if *count == 0 {
            base
        } else {
            format!("{base}-{count}")
        };
        *count += 1;
        slugs.insert(slug);
    }
    slugs
}

fn github_heading_slug(text: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in text.to_ascii_lowercase().chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            slug.push(ch);
            previous_dash = false;
        } else if ch.is_whitespace() || ch == '-' {
            if !previous_dash && !slug.is_empty() {
                slug.push('-');
                previous_dash = true;
            }
        }
    }
    slug.trim_matches('-').to_string()
}
