//! Compiled glob pattern helpers for structure-first checks.

use super::rules::rel_to_string;
use super::scope_patterns::{literal_constraint_chars, CompiledScopePattern};
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
        if let Some(max_lines_patterns) = &files.max_lines_patterns {
            collect_patterns(max_lines_patterns.keys(), patterns);
        }
        if let Some(max_size_patterns) = &files.max_size_patterns {
            collect_patterns(max_size_patterns.keys(), patterns);
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
    if is_lslint_extension_pattern(pattern) {
        return matches_lslint_extension_pattern(pattern, name);
    }

    if let Some(suffix) = simple_suffix_pattern(pattern) {
        return name.ends_with(suffix);
    }

    compiled_patterns
        .get(pattern)
        .map(|compiled| compiled.matches(name))
        .unwrap_or(false)
}

#[cfg(test)]
pub(super) fn best_lslint_suffix_match<'a>(
    patterns: &'a HashMap<String, String>,
    filename: &str,
) -> Option<(&'a str, &'a str)> {
    if !patterns
        .keys()
        .all(|pattern| is_lslint_extension_pattern(pattern))
    {
        return None;
    }

    let filename_segments = lslint_filename_segments(filename);
    best_lslint_extension_match(patterns.iter(), &filename_segments)
        .map(|(pattern, naming)| (pattern.as_str(), naming.as_str()))
}

pub(super) fn best_lslint_suffix_pair<'a, T>(
    patterns: &'a [(String, T)],
    filename: &str,
) -> Option<(&'a str, &'a T)> {
    let filename_segments = lslint_filename_segments(filename);
    best_lslint_extension_match(
        patterns.iter().map(|(pattern, value)| (pattern, value)),
        &filename_segments,
    )
    .map(|(pattern, value)| (pattern.as_str(), value))
}

pub(super) fn best_file_pattern_match<'a, T>(
    patterns: &'a HashMap<String, T>,
    filename: &str,
    rel: &Path,
    compiled_patterns: &HashMap<String, Pattern>,
) -> Option<(&'a str, &'a T)> {
    if patterns
        .keys()
        .all(|pattern| is_lslint_extension_pattern(pattern))
    {
        let filename_segments = lslint_filename_segments(filename);
        return best_lslint_extension_match(patterns.iter(), &filename_segments)
            .map(|(pattern, value)| (pattern.as_str(), value));
    }

    patterns
        .iter()
        .filter(|(pattern, _)| {
            matches_configured_file_pattern(pattern, filename, rel, compiled_patterns)
        })
        .map(|(pattern, value)| (pattern.as_str(), value))
        .max_by(|(left, _), (right, _)| {
            file_pattern_specificity(left)
                .cmp(&file_pattern_specificity(right))
                .then_with(|| right.cmp(left))
        })
}

pub(super) fn matches_configured_file_pattern(
    pattern: &str,
    filename: &str,
    rel: &Path,
    compiled_patterns: &HashMap<String, Pattern>,
) -> bool {
    if is_path_aware_file_pattern(pattern) {
        return CompiledScopePattern::from_str(pattern.trim_start_matches("./")).matches_path(rel);
    }
    matches_single_compiled_pattern(pattern, filename, compiled_patterns)
}

pub(super) fn is_path_aware_file_pattern(pattern: &str) -> bool {
    pattern.starts_with("./") || pattern.contains('/')
}

pub(super) fn file_pattern_uses_lslint_stem(pattern: &str) -> bool {
    pattern
        .rsplit('/')
        .next()
        .is_some_and(is_lslint_extension_pattern)
}

fn file_pattern_specificity(pattern: &str) -> (usize, usize, usize, usize, usize) {
    let path_aware = is_path_aware_file_pattern(pattern);
    let normalized = pattern.trim_start_matches("./");
    let segments = normalized
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    let literal_segments = segments
        .iter()
        .filter(|segment| !segment.contains(['*', '?', '[', ']', '{', '}']))
        .count();
    let constrained_segments = segments.iter().filter(|segment| **segment != "**").count();
    let literal_chars = segments
        .iter()
        .map(|segment| literal_constraint_chars(segment))
        .sum();
    let recursive_segments = segments.iter().filter(|segment| **segment == "**").count();
    (
        literal_chars,
        literal_segments,
        constrained_segments,
        usize::MAX.saturating_sub(recursive_segments),
        usize::from(path_aware),
    )
}

pub(super) fn simple_suffix_pattern(pattern: &str) -> Option<&str> {
    let suffix = pattern.strip_prefix('*')?;
    if suffix.is_empty() || suffix.contains(['*', '?', '[', ']']) {
        return None;
    }
    Some(suffix)
}

pub(super) fn is_lslint_extension_pattern(pattern: &str) -> bool {
    lslint_extension_segments(pattern).is_some()
}

pub(super) fn lslint_file_stem(filename: &str) -> &str {
    filename
        .split_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(filename)
}

fn lslint_extension_segments(pattern: &str) -> Option<Vec<&str>> {
    let suffix = pattern.strip_prefix("*.")?;
    let segments = suffix.split('.').collect::<Vec<_>>();
    (!segments.is_empty()
        && segments
            .iter()
            .all(|segment| !segment.is_empty() && (*segment == "*" || !segment.contains('*'))))
    .then_some(segments)
}

fn matches_lslint_extension_pattern(pattern: &str, filename: &str) -> bool {
    let filename_segments = lslint_filename_segments(filename);
    lslint_extension_match_rank(pattern, &filename_segments).is_some()
}

fn lslint_filename_segments(filename: &str) -> Vec<&str> {
    filename
        .split('.')
        .skip(1)
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn best_lslint_extension_match<'a, T>(
    patterns: impl Iterator<Item = (&'a String, T)>,
    filename_segments: &[&str],
) -> Option<(&'a String, T)> {
    patterns
        .filter_map(|(pattern, value)| {
            lslint_extension_match_rank(pattern, filename_segments)
                .map(|rank| (rank, pattern, value))
        })
        .min_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)))
        .map(|(_, pattern, value)| (pattern, value))
}

fn lslint_extension_match_rank(pattern: &str, filename_segments: &[&str]) -> Option<Vec<bool>> {
    let pattern_segments = lslint_extension_segments(pattern)?;
    if pattern_segments.len() != filename_segments.len() {
        return None;
    }

    let mut rank = Vec::with_capacity(pattern_segments.len());
    for (pattern, segment) in pattern_segments.iter().zip(filename_segments) {
        if *pattern == "*" {
            rank.push(true);
        } else if *pattern == *segment {
            rank.push(false);
        } else {
            return None;
        }
    }
    Some(rank)
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
        let compiled_pattern = capture_pattern_as_glob(pattern).unwrap_or_else(|| pattern.clone());
        if let Ok(glob) = Pattern::new(&compiled_pattern) {
            compiled.insert(pattern.clone(), glob);
        }
    }
}

fn capture_pattern_as_glob(pattern: &str) -> Option<String> {
    if !pattern.contains('{') {
        return None;
    }

    let mut glob = String::with_capacity(pattern.len());
    let mut rest = pattern;
    loop {
        let Some(open) = rest.find('{') else {
            glob.push_str(rest);
            break;
        };
        glob.push_str(&rest[..open]);
        let close = rest[open + 1..].find('}')? + open + 1;
        let name = &rest[open + 1..close];
        if name.is_empty() || name.contains(',') {
            return None;
        }
        glob.push('*');
        rest = &rest[close + 1..];
    }
    Some(glob)
}

#[cfg(test)]
mod tests {
    use super::{
        best_file_pattern_match, best_lslint_suffix_match, capture_pattern_as_glob,
        matches_single_compiled_pattern,
    };
    use glob::Pattern;
    use std::collections::HashMap;
    use std::path::Path;

    #[test]
    fn direct_capture_patterns_compile_to_globs() {
        assert_eq!(capture_pattern_as_glob("{package}"), Some("*".to_string()));
        assert_eq!(
            capture_pattern_as_glob("prefix-{package}-docs"),
            Some("prefix-*-docs".to_string())
        );
        assert_eq!(capture_pattern_as_glob("{one,two}"), None);
    }

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

        assert_eq!(best_lslint_suffix_match(&patterns, "component-a.ts"), None);
    }

    #[test]
    fn lslint_wildcard_subextension_pattern_matches_only_subextensions() {
        let mut compiled = HashMap::new();
        compiled.insert("*.*.js".to_string(), Pattern::new("*.*.js").unwrap());

        assert!(matches_single_compiled_pattern(
            "*.*.js",
            "Button.test.js",
            &compiled
        ));
        assert!(!matches_single_compiled_pattern(
            "*.*.js",
            "button.js",
            &compiled
        ));
        assert!(!matches_single_compiled_pattern(
            "*.*.js",
            "Button.story.test.js",
            &compiled
        ));
        assert!(!matches_single_compiled_pattern(
            "*.js",
            "Button.test.js",
            &compiled
        ));
    }

    #[test]
    fn best_lslint_suffix_match_handles_long_multipart_extensions_without_candidates() {
        let patterns = HashMap::from([
            (
                "*.a.b.c.d.e.f.g.h.i.j.k.js".to_string(),
                "kebab-case".to_string(),
            ),
            (
                "*.*.b.c.d.e.f.g.h.i.j.k.js".to_string(),
                "snake_case".to_string(),
            ),
        ]);

        assert_eq!(
            best_lslint_suffix_match(&patterns, "file.a.b.c.d.e.f.g.h.i.j.k.js"),
            Some(("*.a.b.c.d.e.f.g.h.i.j.k.js", "kebab-case"))
        );
    }

    #[test]
    fn explicit_path_patterns_distinguish_direct_and_recursive_files() {
        let patterns = HashMap::from([
            ("./**/*.ts".to_string(), 2),
            ("./*.ts".to_string(), 1),
            ("*.test.ts".to_string(), 3),
        ]);
        let compiled = HashMap::new();

        assert_eq!(
            best_file_pattern_match(&patterns, "root.ts", Path::new("root.ts"), &compiled),
            Some(("./*.ts", &1))
        );
        assert_eq!(
            best_file_pattern_match(
                &patterns,
                "nested.ts",
                Path::new("src/nested.ts"),
                &compiled
            ),
            Some(("./**/*.ts", &2))
        );
        assert_eq!(
            best_file_pattern_match(
                &patterns,
                "root.test.ts",
                Path::new("root.test.ts"),
                &compiled
            ),
            Some(("*.test.ts", &3))
        );
    }

    #[test]
    fn capture_names_do_not_add_file_pattern_specificity() {
        let patterns = HashMap::from([
            ("./{very_long_capture_name}.ts".to_string(), 1),
            ("./release-*.ts".to_string(), 2),
        ]);

        assert_eq!(
            best_file_pattern_match(
                &patterns,
                "release-build.ts",
                Path::new("release-build.ts"),
                &HashMap::new()
            ),
            Some(("./release-*.ts", &2))
        );
    }
}
