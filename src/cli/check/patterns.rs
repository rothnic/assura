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
        if let Some(suffix) = simple_suffix_pattern(pattern) {
            return name.ends_with(suffix) || rel.ends_with(suffix);
        }

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
    if let Some(suffix) = simple_suffix_pattern(pattern) {
        return name.ends_with(suffix);
    }

    compiled_patterns
        .get(pattern)
        .map(|compiled| compiled.matches(name))
        .unwrap_or(false)
}

pub(super) fn best_lslint_suffix_match<'a>(
    patterns: &'a HashMap<String, String>,
    filename: &str,
) -> Option<(&'a str, &'a str)> {
    patterns
        .iter()
        .filter_map(|(pattern, naming)| {
            let suffix = simple_suffix_pattern(pattern)?;
            filename
                .ends_with(suffix)
                .then_some((pattern.as_str(), naming.as_str()))
        })
        .max_by(|(left, _), (right, _)| left.len().cmp(&right.len()).then_with(|| right.cmp(left)))
}

pub(super) fn simple_suffix_pattern(pattern: &str) -> Option<&str> {
    let suffix = pattern.strip_prefix('*')?;
    if suffix.is_empty() || suffix.contains(['*', '?', '[', ']']) {
        return None;
    }
    Some(suffix)
}

fn collect_patterns<'a>(
    patterns: impl Iterator<Item = &'a String>,
    compiled: &mut HashMap<String, Pattern>,
) {
    for pattern in patterns {
        if compiled.contains_key(pattern) {
            continue;
        }
        if simple_suffix_pattern(pattern).is_some() {
            continue;
        }
        if let Ok(glob) = Pattern::new(pattern) {
            compiled.insert(pattern.clone(), glob);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::best_lslint_suffix_match;
    use std::collections::HashMap;

    #[test]
    fn best_lslint_suffix_match_prefers_most_specific_suffix() {
        let patterns = HashMap::from([
            ("*.ts".to_string(), "kebab-case".to_string()),
            ("*.test.ts".to_string(), "camelCase".to_string()),
        ]);

        assert_eq!(
            best_lslint_suffix_match(&patterns, "button.test.ts"),
            Some(("*.test.ts", "camelCase"))
        );
    }

    #[test]
    fn best_lslint_suffix_match_defers_when_patterns_are_not_simple_dot_suffixes() {
        let patterns = HashMap::from([
            ("*.ts".to_string(), "kebab-case".to_string()),
            ("component-*.ts".to_string(), "camelCase".to_string()),
        ]);

        assert_eq!(
            best_lslint_suffix_match(&patterns, "component-a.ts"),
            Some(("*.ts", "kebab-case"))
        );
    }

    #[test]
    fn best_lslint_suffix_match_handles_simple_non_dot_suffixes_without_allocating_patterns() {
        let patterns = HashMap::from([
            ("*rc".to_string(), "flatcase".to_string()),
            ("component-*.ts".to_string(), "camelCase".to_string()),
        ]);

        assert_eq!(
            best_lslint_suffix_match(&patterns, "bashrc"),
            Some(("*rc", "flatcase"))
        );
    }
}
