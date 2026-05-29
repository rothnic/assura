//! LS-Lint-compatible directory scope pattern matching.

use glob::Pattern;
use std::path::Path;

#[derive(Debug, Clone)]
pub(super) struct CompiledScopePattern {
    alternatives: Vec<Vec<CompiledPathSegment>>,
}

#[derive(Debug, Clone)]
enum CompiledPathSegment {
    Recursive,
    Literal(String),
    Pattern(Pattern),
    Invalid,
}

impl CompiledScopePattern {
    pub(super) fn new(pattern: &Path) -> Self {
        Self::from_str(&path_to_string(pattern))
    }

    pub(super) fn from_str(pattern: &str) -> Self {
        let alternatives = expand_braces(pattern)
            .into_iter()
            .map(|pattern| {
                split_path_segments(&pattern)
                    .into_iter()
                    .map(compile_segment)
                    .collect()
            })
            .collect();
        Self { alternatives }
    }

    pub(super) fn matches_path(&self, target: &Path) -> bool {
        self.matches_str(&path_to_string(target))
    }

    pub(super) fn matches_str(&self, target: &str) -> bool {
        let target = split_path_segments(target);
        self.alternatives
            .iter()
            .any(|pattern| matches_compiled_segments(pattern, &target))
    }

    pub(super) fn has_matching_ancestor(&self, target: &Path) -> bool {
        let mut ancestor = target.parent();
        while let Some(path) = ancestor {
            if self.matches_path(path) {
                return true;
            }
            ancestor = path.parent();
        }
        false
    }
}

pub(super) fn path_has_scope_magic(path: &Path) -> bool {
    path_to_string(path).contains(['*', '?', '{', '}'])
}

pub(super) fn path_matches_scope_pattern(pattern: &Path, target: &Path) -> bool {
    CompiledScopePattern::new(pattern).matches_path(target)
}

pub(super) fn path_has_matching_scope_ancestor(pattern: &Path, target: &Path) -> bool {
    CompiledScopePattern::new(pattern).has_matching_ancestor(target)
}

fn expand_braces(pattern: &str) -> Vec<String> {
    let Some(open) = pattern.find('{') else {
        return vec![pattern.to_string()];
    };
    let Some(close_offset) = pattern[open + 1..].find('}') else {
        return vec![pattern.to_string()];
    };
    let close = open + 1 + close_offset;
    let prefix = &pattern[..open];
    let suffix = &pattern[close + 1..];

    let mut expanded = Vec::new();
    for option in pattern[open + 1..close].split(',') {
        for nested in expand_braces(&format!("{prefix}{option}{suffix}")) {
            expanded.push(nested);
        }
    }
    expanded
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn split_path_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn compile_segment(segment: &str) -> CompiledPathSegment {
    if segment == "**" {
        return CompiledPathSegment::Recursive;
    }
    if !segment.contains(['*', '?', '[', ']']) {
        return CompiledPathSegment::Literal(segment.to_string());
    }
    Pattern::new(segment)
        .map(CompiledPathSegment::Pattern)
        .unwrap_or(CompiledPathSegment::Invalid)
}

fn matches_compiled_segments(pattern: &[CompiledPathSegment], target: &[&str]) -> bool {
    match (pattern.split_first(), target.split_first()) {
        (None, None) => true,
        (None, Some(_)) => false,
        (Some((CompiledPathSegment::Recursive, rest)), _) => {
            matches_compiled_segments(rest, target)
                || target
                    .split_first()
                    .is_some_and(|(_, target_rest)| matches_compiled_segments(pattern, target_rest))
        }
        (Some((pattern_segment, rest)), Some((target_segment, target_rest))) => {
            compiled_segment_matches(pattern_segment, target_segment)
                && matches_compiled_segments(rest, target_rest)
        }
        (Some(_), None) => false,
    }
}

fn compiled_segment_matches(pattern: &CompiledPathSegment, target: &str) -> bool {
    match pattern {
        CompiledPathSegment::Recursive => false,
        CompiledPathSegment::Literal(pattern) => pattern == target,
        CompiledPathSegment::Pattern(pattern) => pattern.matches(target),
        CompiledPathSegment::Invalid => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{path_has_matching_scope_ancestor, path_matches_scope_pattern};
    use std::path::Path;

    #[test]
    fn scope_patterns_support_doublestar_and_braces() {
        assert!(path_matches_scope_pattern(
            Path::new("src/**/c"),
            Path::new("src/a/b/c")
        ));
        assert!(path_matches_scope_pattern(
            Path::new("src/**/c"),
            Path::new("src/c")
        ));
        assert!(path_matches_scope_pattern(
            Path::new("src/{a,b}/*"),
            Path::new("src/a/components")
        ));
        assert!(!path_matches_scope_pattern(
            Path::new("src/{a,b}/*"),
            Path::new("src/c/components")
        ));
        assert!(path_has_matching_scope_ancestor(
            Path::new("src/**/c"),
            Path::new("src/a/b/c/packages")
        ));
    }
}
