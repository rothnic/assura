//! Shared rule helpers for structure-first CLI validation.

use crate::config::config::{
    split_naming_conventions, DirectoryBundle, DirectoryNode, FileBundle, MarkdownBundle,
};
use crate::constraints::CaseConvention;
use glob::Pattern;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub(super) struct EffectiveRules {
    pub(super) files: Option<FileBundle>,
    pub(super) directories: Option<DirectoryBundle>,
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
        if let Some(naming_patterns) = &files.naming_patterns {
            for naming in naming_patterns.values() {
                collect_naming_regex(naming, regexes);
            }
        }
    }

    if let Some(directories) = &node.directories {
        if let Some(naming) = &directories.naming {
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
    let alternatives = split_naming_conventions(convention);
    if alternatives.len() > 1 {
        for part in alternatives {
            collect_naming_regex(part, regexes);
        }
        return;
    }

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
            naming_patterns: None,
            required: None,
            allowed_names: None,
            allowed_patterns: None,
            forbidden_patterns: None,
            allow_extra: None,
            exists: None,
            ..parent.clone()
        }),
        (None, Some(child)) => Some(child.clone()),
        (Some(parent), Some(child)) => Some(FileBundle {
            naming: child.naming.clone().or_else(|| parent.naming.clone()),
            naming_patterns: child.naming_patterns.clone(),
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
            allowed_patterns: child.allowed_patterns.clone(),
            forbidden_patterns: child.forbidden_patterns.clone(),
            allow_extra: child.allow_extra,
            exists: child.exists.clone(),
        }),
    }
}

pub(super) fn merge_directory_bundle(
    parent: Option<&DirectoryBundle>,
    child: Option<&DirectoryBundle>,
) -> Option<DirectoryBundle> {
    match (parent, child) {
        (None, None) => None,
        (Some(parent), None) => Some(DirectoryBundle {
            required: None,
            allowed_names: None,
            allowed_patterns: None,
            forbidden_patterns: None,
            allow_extra: None,
            exists: None,
            ..parent.clone()
        }),
        (None, Some(child)) => Some(child.clone()),
        (Some(parent), Some(child)) => Some(DirectoryBundle {
            naming: child.naming.clone().or_else(|| parent.naming.clone()),
            required: child.required.clone(),
            allowed_names: child.allowed_names.clone(),
            allowed_patterns: child.allowed_patterns.clone(),
            forbidden_patterns: child.forbidden_patterns.clone(),
            allow_extra: child.allow_extra,
            severity: child.severity.clone().or_else(|| parent.severity.clone()),
            exists: child.exists.clone(),
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
    let alternatives = split_naming_conventions(convention);
    if alternatives.len() > 1 {
        return alternatives
            .into_iter()
            .any(|part| validate_name(name, part, regexes));
    }

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

pub(super) fn file_matches_extension_rule(filename: &str, extension: &str) -> bool {
    let extension = extension.trim();
    if extension.is_empty() {
        return false;
    }

    if let Some(pattern) = extension.strip_prefix('.') {
        if extension == ".*" {
            return true;
        }

        if pattern.contains('*') {
            let glob = format!("*{}", extension);
            return Pattern::new(&glob)
                .map(|compiled| compiled.matches(filename))
                .unwrap_or(false);
        }

        return filename.ends_with(extension);
    }

    let suffix = format!(".{}", extension);
    filename.ends_with(&suffix)
}

pub(super) fn file_matches_any_extension(filename: &str, extensions: Option<&[String]>) -> bool {
    extensions
        .map(|extensions| {
            extensions
                .iter()
                .any(|extension| file_matches_extension_rule(filename, extension))
        })
        .unwrap_or(false)
}

pub(super) fn count_satisfies(count: usize, expected: &str) -> bool {
    let expected = expected.trim();
    if expected == "exists" {
        return count > 0;
    }

    if let Some((min, max)) = expected.split_once('-') {
        let min = min.trim().parse::<usize>().ok();
        let max = max.trim().parse::<usize>().ok();
        return min
            .zip(max)
            .is_some_and(|(min, max)| count >= min && count <= max);
    }

    if let Some((min, max)) = expected.split_once("..") {
        let min = min.trim().parse::<usize>().ok();
        let max = max.trim().parse::<usize>().ok();
        return min
            .zip(max)
            .is_some_and(|(min, max)| count >= min && count <= max);
    }

    expected
        .parse::<usize>()
        .map(|required| count == required)
        .unwrap_or(false)
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

pub(super) fn severity_for_directory_bundle(directories: &DirectoryBundle) -> String {
    directories
        .severity
        .clone()
        .unwrap_or_else(|| "medium".to_string())
}

pub(super) fn strip_direct_content_policy(mut rules: EffectiveRules) -> EffectiveRules {
    if let Some(files) = rules.files.as_mut() {
        files.required = None;
        files.allowed_names = None;
        files.allowed_patterns = None;
        files.forbidden_patterns = None;
        files.allow_extra = None;
        files.exists = None;
    }

    if let Some(directories) = rules.directories.as_mut() {
        directories.required = None;
        directories.allowed_names = None;
        directories.allowed_patterns = None;
        directories.forbidden_patterns = None;
        directories.allow_extra = None;
        directories.exists = None;
    }

    rules
}
