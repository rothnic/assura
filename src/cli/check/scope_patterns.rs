//! LS-Lint-compatible directory scope pattern matching.

use glob::Pattern;
use std::path::{Path, PathBuf};

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
        self.matching_ancestor(target).is_some()
    }

    pub(super) fn matching_ancestor(&self, target: &Path) -> Option<PathBuf> {
        let mut ancestor = target.parent();
        while let Some(path) = ancestor {
            if self.matches_path(path) {
                return Some(path.to_path_buf());
            }
            ancestor = path.parent();
        }
        None
    }
}

pub(super) fn path_has_scope_magic(path: &Path) -> bool {
    path_to_string(path).contains(['*', '?', '{', '}'])
}

pub(super) fn path_scope_specificity(path: &Path) -> (usize, usize, usize, usize) {
    let mut literal_segments = 0;
    let mut literal_chars = 0;
    for segment in path.iter().filter_map(|segment| segment.to_str()) {
        if !segment.contains(['*', '?', '{', '}', '[', ']']) {
            literal_segments += 1;
        }
        literal_chars += literal_constraint_chars(segment);
    }

    (
        path.components().count(),
        literal_segments,
        literal_chars,
        usize::from(!path_has_scope_magic(path)),
    )
}

pub(super) fn path_matches_scope_pattern(pattern: &Path, target: &Path) -> bool {
    CompiledScopePattern::new(pattern).matches_path(target)
}

pub(super) fn path_has_matching_scope_ancestor(pattern: &Path, target: &Path) -> bool {
    CompiledScopePattern::new(pattern).has_matching_ancestor(target)
}

fn expand_braces(pattern: &str) -> Vec<String> {
    let mut search_start = 0;
    while let Some(open_offset) = pattern[search_start..].find('{') {
        let open = search_start + open_offset;
        let Some(close_offset) = pattern[open + 1..].find('}') else {
            return vec![pattern.to_string()];
        };
        let close = open + 1 + close_offset;
        let body = &pattern[open + 1..close];
        if body.contains(',') {
            return expand_comma_brace(pattern, open, close);
        }
        search_start = close + 1;
    }

    vec![pattern.to_string()]
}

fn expand_comma_brace(pattern: &str, open: usize, close: usize) -> Vec<String> {
    let prefix = &pattern[..open];
    let body = &pattern[open + 1..close];
    let suffix = &pattern[close + 1..];

    let mut expanded = Vec::new();
    for option in body.split(',') {
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

pub(super) fn literal_constraint_chars(segment: &str) -> usize {
    let mut count = 0;
    let mut in_capture = false;
    let mut in_character_class = false;
    for character in segment.chars() {
        match character {
            '{' if !in_character_class => in_capture = true,
            '}' if in_capture && !in_character_class => in_capture = false,
            '[' if !in_capture => in_character_class = true,
            ']' if in_character_class && !in_capture => in_character_class = false,
            '*' | '?' if !in_capture && !in_character_class => {}
            _ if !in_capture && !in_character_class => count += 1,
            _ => {}
        }
    }
    count
}

fn compile_segment(segment: &str) -> CompiledPathSegment {
    if segment == "**" {
        return CompiledPathSegment::Recursive;
    }
    if segment.contains('{') || segment.contains('}') {
        return compile_capture_segment(segment);
    }
    if !segment.contains(['*', '?', '[', ']']) {
        return CompiledPathSegment::Literal(segment.to_string());
    }
    Pattern::new(segment)
        .map(CompiledPathSegment::Pattern)
        .unwrap_or(CompiledPathSegment::Invalid)
}

fn compile_capture_segment(segment: &str) -> CompiledPathSegment {
    let mut pattern = String::new();
    let mut rest = segment;
    loop {
        let Some(open) = rest.find('{') else {
            pattern.push_str(rest);
            break;
        };
        pattern.push_str(&rest[..open]);
        let Some(close_offset) = rest[open + 1..].find('}') else {
            return CompiledPathSegment::Invalid;
        };
        let close = open + 1 + close_offset;
        let name = &rest[open + 1..close];
        if name.is_empty() || name.contains(',') {
            return CompiledPathSegment::Invalid;
        }
        pattern.push('*');
        rest = &rest[close + 1..];
    }
    Pattern::new(&pattern)
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

    #[test]
    fn scope_patterns_support_single_segment_captures() {
        assert!(path_matches_scope_pattern(
            Path::new(".agents/skills/{skill}"),
            Path::new(".agents/skills/assura-project-maintenance")
        ));
        assert!(path_matches_scope_pattern(
            Path::new("docs/{workspace}/packages"),
            Path::new("docs/platform/packages")
        ));
        assert!(path_matches_scope_pattern(
            Path::new("docs/{workspace}-notes"),
            Path::new("docs/platform-notes")
        ));
        assert!(!path_matches_scope_pattern(
            Path::new("docs/{workspace}/packages"),
            Path::new("docs/platform/archive/packages")
        ));
    }

    #[test]
    fn scope_patterns_expand_brace_alternatives_after_captures() {
        assert!(path_matches_scope_pattern(
            Path::new(".agents/skills/{skill}/{references,scripts}"),
            Path::new(".agents/skills/release-maintenance/references")
        ));
        assert!(path_matches_scope_pattern(
            Path::new(".agents/skills/{skill}/{references,scripts}"),
            Path::new(".agents/skills/release-maintenance/scripts")
        ));
        assert!(!path_matches_scope_pattern(
            Path::new(".agents/skills/{skill}/{references,scripts}"),
            Path::new(".agents/skills/release-maintenance/assets")
        ));
    }

    #[test]
    fn scope_patterns_expand_brace_alternatives_before_captures() {
        assert!(path_matches_scope_pattern(
            Path::new(".agents/{codex,opencode}/skills/{skill}"),
            Path::new(".agents/codex/skills/release-maintenance")
        ));
        assert!(path_matches_scope_pattern(
            Path::new(".agents/{codex,opencode}/skills/{skill}"),
            Path::new(".agents/opencode/skills/release-maintenance")
        ));
        assert!(!path_matches_scope_pattern(
            Path::new(".agents/{codex,opencode}/skills/{skill}"),
            Path::new(".agents/claude/skills/release-maintenance")
        ));
    }
}
