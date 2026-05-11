//! Compiled glob pattern helpers for structure-first checks.

use super::rules::rel_to_string;
use crate::config::config::DirectoryNode;
use glob::Pattern;
use std::collections::HashMap;
use std::path::Path;

pub(super) fn collect_glob_patterns(node: &DirectoryNode, patterns: &mut HashMap<String, Pattern>) {
    if let Some(files) = &node.files {
        collect_patterns(files.allowed_patterns.iter().flatten(), patterns);
        collect_patterns(files.forbidden_patterns.iter().flatten(), patterns);
        if let Some(naming_patterns) = &files.naming_patterns {
            collect_patterns(naming_patterns.keys(), patterns);
        }
        if let Some(exists) = &files.exists {
            collect_patterns(exists.keys(), patterns);
        }
    }

    if let Some(directories) = &node.directories {
        collect_patterns(directories.allowed_patterns.iter().flatten(), patterns);
        collect_patterns(directories.forbidden_patterns.iter().flatten(), patterns);
        if let Some(exists) = &directories.exists {
            collect_patterns(exists.keys(), patterns);
        }
    }

    if let Some(children) = &node.children {
        for child in children.values() {
            collect_glob_patterns(child, patterns);
        }
    }
}

pub(super) fn matches_any_compiled_pattern(
    patterns: Option<&[String]>,
    name: &str,
    rel: &Path,
    compiled_patterns: &HashMap<String, Pattern>,
) -> bool {
    let Some(patterns) = patterns else {
        return false;
    };

    let rel = rel_to_string(rel);
    patterns.iter().any(|pattern| {
        compiled_patterns
            .get(pattern)
            .map(|compiled| compiled.matches(name) || compiled.matches(&rel))
            .unwrap_or(false)
    })
}

pub(super) fn matches_single_compiled_pattern(
    pattern: &str,
    name: &str,
    compiled_patterns: &HashMap<String, Pattern>,
) -> bool {
    compiled_patterns
        .get(pattern)
        .map(|compiled| compiled.matches(name))
        .unwrap_or(false)
}

fn collect_patterns<'a>(
    patterns: impl Iterator<Item = &'a String>,
    compiled: &mut HashMap<String, Pattern>,
) {
    for pattern in patterns {
        if compiled.contains_key(pattern) {
            continue;
        }
        if let Ok(glob) = Pattern::new(pattern) {
            compiled.insert(pattern.clone(), glob);
        }
    }
}
