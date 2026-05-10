//! Shared rule helpers for structure-first CLI validation.

use crate::config::config::{DirectoryNode, FileBundle, MarkdownBundle};
use crate::constraints::CaseConvention;
use glob::Pattern;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub(super) struct EffectiveRules {
    pub(super) files: Option<FileBundle>,
    pub(super) markdown: Option<MarkdownBundle>,
}

#[derive(Debug, Clone)]
pub(super) struct CompiledExclusion {
    prefix: Option<String>,
    prefix_with_slash: Option<String>,
    pattern: Option<Pattern>,
}

impl CompiledExclusion {
    pub(super) fn new(pattern: &str) -> Self {
        let prefix = pattern.strip_suffix("/**").map(ToOwned::to_owned);
        let prefix_with_slash = prefix.as_ref().map(|prefix| format!("{}/", prefix));
        Self {
            prefix,
            prefix_with_slash,
            pattern: Pattern::new(pattern).ok(),
        }
    }

    fn matches(&self, rel: &str) -> bool {
        if let Some(prefix) = &self.prefix {
            if rel == prefix {
                return true;
            }
        }

        if let Some(prefix_with_slash) = &self.prefix_with_slash {
            if rel.starts_with(prefix_with_slash) {
                return true;
            }
        }

        self.pattern
            .as_ref()
            .map(|pattern| pattern.matches(rel))
            .unwrap_or(false)
    }
}

pub(super) fn collect_configured_dirs(
    node_rel: PathBuf,
    node: &DirectoryNode,
    configured_dirs: &mut HashSet<PathBuf>,
) {
    configured_dirs.insert(node_rel.clone());

    if let Some(children) = &node.children {
        for (child_name, child) in children {
            let child_rel = join_config_child(&node_rel, child_name);
            collect_configured_dirs(child_rel, child, configured_dirs);
        }
    }
}

pub(super) fn collect_naming_regexes(node: &DirectoryNode, regexes: &mut HashMap<String, Regex>) {
    if let Some(files) = &node.files {
        if let Some(naming) = &files.naming {
            collect_naming_regex(naming, regexes);
        }
    }

    if let Some(children) = &node.children {
        for child in children.values() {
            collect_naming_regexes(child, regexes);
        }
    }
}

fn collect_naming_regex(convention: &str, regexes: &mut HashMap<String, Regex>) {
    let Some(pattern) = convention.strip_prefix("regex:") else {
        return;
    };

    if regexes.contains_key(pattern) {
        return;
    }

    if let Ok(regex) = Regex::new(pattern) {
        regexes.insert(pattern.to_string(), regex);
    }
}

pub(super) fn is_excluded_rel_with(patterns: &[CompiledExclusion], rel: &Path) -> bool {
    if rel.as_os_str().is_empty() {
        return false;
    }

    let rel = rel_to_string(rel);
    patterns.iter().any(|pattern| pattern.matches(&rel))
}

pub(super) fn normalize_config_dir(path: &str) -> PathBuf {
    let trimmed = path.trim_end_matches('/');
    if trimmed.is_empty() || trimmed == "." {
        PathBuf::new()
    } else if let Some(stripped) = trimmed.strip_prefix("./") {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(trimmed)
    }
}

pub(super) fn join_config_child(parent: &Path, child_name: &str) -> PathBuf {
    let child = normalize_config_dir(child_name);
    if parent.as_os_str().is_empty() {
        child
    } else {
        parent.join(child)
    }
}

pub(super) fn dir_contains(node_rel: &Path, target_dir: &Path) -> bool {
    node_rel.as_os_str().is_empty() || target_dir == node_rel || target_dir.starts_with(node_rel)
}

pub(super) fn merge_file_bundle(
    parent: Option<&FileBundle>,
    child: Option<&FileBundle>,
) -> Option<FileBundle> {
    match (parent, child) {
        (None, None) => None,
        (Some(parent), None) => Some(FileBundle {
            required: None,
            allowed_names: None,
            ..parent.clone()
        }),
        (None, Some(child)) => Some(child.clone()),
        (Some(parent), Some(child)) => Some(FileBundle {
            naming: child.naming.clone().or_else(|| parent.naming.clone()),
            max_lines: child.max_lines.or(parent.max_lines),
            max_size: child.max_size.clone().or_else(|| parent.max_size.clone()),
            require_docs: child.require_docs.or(parent.require_docs),
            extensions: child
                .extensions
                .clone()
                .or_else(|| parent.extensions.clone()),
            severity: child.severity.clone().or_else(|| parent.severity.clone()),
            required: child.required.clone(),
            allowed_names: child.allowed_names.clone(),
        }),
    }
}

pub(super) fn merge_markdown_bundle(
    parent: Option<&MarkdownBundle>,
    child: Option<&MarkdownBundle>,
) -> Option<MarkdownBundle> {
    match (parent, child) {
        (None, None) => None,
        (Some(parent), None) => Some(parent.clone()),
        (None, Some(child)) => Some(child.clone()),
        (Some(parent), Some(child)) => Some(MarkdownBundle {
            require_frontmatter: child.require_frontmatter.or(parent.require_frontmatter),
            required_fields: child
                .required_fields
                .clone()
                .or_else(|| parent.required_fields.clone()),
            max_heading_depth: child.max_heading_depth.or(parent.max_heading_depth),
            check_links: child.check_links.or(parent.check_links),
            required_sections: child
                .required_sections
                .clone()
                .or_else(|| parent.required_sections.clone()),
        }),
    }
}

pub(super) fn validate_name(
    name: &str,
    convention: &str,
    regexes: &HashMap<String, Regex>,
) -> bool {
    if let Some(pattern) = convention.strip_prefix("regex:") {
        return regexes
            .get(pattern)
            .map(|regex| regex.is_match(name))
            .unwrap_or(false);
    }

    match convention_to_case(convention) {
        Some(case) => case.validate(name),
        None => false,
    }
}

pub(super) fn validate_file_stem(
    stem: &str,
    convention: &str,
    regexes: &HashMap<String, Regex>,
) -> bool {
    validate_name(stem, convention, regexes)
        || stem
            .split_once('.')
            .map(|(base, _)| validate_name(base, convention, regexes))
            .unwrap_or(false)
}

fn convention_to_case(convention: &str) -> Option<CaseConvention> {
    match convention {
        "snake_case" => Some(CaseConvention::SnakeCase),
        "camelCase" => Some(CaseConvention::CamelCase),
        "PascalCase" => Some(CaseConvention::PascalCase),
        "kebab-case" => Some(CaseConvention::KebabCase),
        "SCREAMING_SNAKE_CASE" => Some(CaseConvention::ScreamingSnakeCase),
        "dot.case" => Some(CaseConvention::DotCase),
        "flatcase" => Some(CaseConvention::FlatCase),
        "FLATCASE" => Some(CaseConvention::ScreamingFlatCase),
        "COBOL-CASE" => Some(CaseConvention::CobolCase),
        "Train-Case" => Some(CaseConvention::TrainCase),
        "lowercase" => Some(CaseConvention::LowerCase),
        "UPPERCASE" => Some(CaseConvention::UpperCase),
        _ => None,
    }
}

pub(super) fn parse_size(size: &str) -> Option<u64> {
    let size = size.trim();
    let split = size
        .find(|ch: char| !ch.is_ascii_digit() && !ch.is_ascii_whitespace())
        .unwrap_or(size.len());
    let amount: u64 = size[..split].trim().parse().ok()?;
    let unit = size[split..].trim().to_ascii_uppercase();

    match unit.as_str() {
        "B" => Some(amount),
        "KB" => Some(amount * 1024),
        "MB" => Some(amount * 1024 * 1024),
        "GB" => Some(amount * 1024 * 1024 * 1024),
        "TB" => Some(amount * 1024 * 1024 * 1024 * 1024),
        _ => None,
    }
}

pub(super) fn parse_frontmatter(content: &str) -> Option<&str> {
    let mut lines = content.lines();
    if lines.next()? != "---" {
        return None;
    }

    let start = 4;
    let end = content[start..].find("\n---")?;
    Some(&content[start..start + end])
}

pub(super) fn rel_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

pub(super) fn display_rel(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        rel_to_string(path)
    }
}

pub(super) fn severity_for_node(node: &DirectoryNode) -> String {
    node.files
        .as_ref()
        .and_then(|files| files.severity.clone())
        .unwrap_or_else(|| "medium".to_string())
}

pub(super) fn severity_for_bundle(files: &FileBundle) -> String {
    files
        .severity
        .clone()
        .unwrap_or_else(|| "medium".to_string())
}
