//! Fast naming matchers for LS-Lint-compatible checks.

use super::case::{convention_to_case_validator, validate_single_name_with_path};
use super::patterns::{best_lslint_suffix_pair, matches_single_compiled_pattern};
use crate::config::config::split_naming_conventions;
use glob::Pattern;
use regex_lite::Regex;
use std::collections::HashMap;

#[derive(Clone)]
pub(super) struct FastFileNaming {
    suffix_patterns: Vec<(String, FastNaming)>,
    glob_patterns: Vec<FastPatternNaming>,
    default: Option<FastNaming>,
}

#[derive(Clone)]
pub(super) struct FastPatternNaming {
    pattern: String,
    matcher: FastPatternMatcher,
    naming: FastNaming,
}

pub(super) struct FastFileNamingMatch<'a> {
    pub(super) naming: &'a FastNaming,
    pub(super) lslint_extension_pattern: bool,
}

#[derive(Clone)]
enum FastPatternMatcher {
    Glob(String),
}

#[derive(Clone)]
pub(super) enum FastNaming {
    Static {
        label: String,
        validate: fn(&str) -> bool,
    },
    EmptyRegex {
        label: String,
    },
    ExactAny {
        label: String,
        literals: Vec<String>,
    },
    ContainsAny {
        label: String,
        literals: Vec<String>,
    },
    Regex {
        label: String,
        pattern: String,
    },
    Alternatives {
        label: String,
        alternatives: Vec<FastNaming>,
    },
    Dynamic(String),
}

pub(super) fn validate_fast_file_stem(
    stem: &str,
    path: &str,
    naming: &FastNaming,
    regexes: &HashMap<String, Regex>,
) -> bool {
    validate_fast_name(stem, path, naming, regexes)
        || stem
            .split_once('.')
            .map(|(base, _)| validate_fast_name(base, path, naming, regexes))
            .unwrap_or(false)
}

pub(super) fn validate_fast_name(
    name: &str,
    path: &str,
    naming: &FastNaming,
    regexes: &HashMap<String, Regex>,
) -> bool {
    match naming {
        FastNaming::Static { validate, .. } => validate(name),
        FastNaming::EmptyRegex { .. } => name.is_empty(),
        FastNaming::ExactAny { literals, .. } => literals.iter().any(|literal| literal == name),
        FastNaming::ContainsAny { literals, .. } => {
            literals.iter().any(|literal| name.contains(literal))
        }
        FastNaming::Regex { pattern, .. } => {
            validate_single_name_with_path(name, path, &format!("regex:{pattern}"), regexes)
        }
        FastNaming::Alternatives { alternatives, .. } => alternatives
            .iter()
            .any(|alternative| validate_fast_name(name, path, alternative, regexes)),
        FastNaming::Dynamic(convention) => {
            validate_single_name_with_path(name, path, convention, regexes)
        }
    }
}

impl FastFileNaming {
    pub(super) fn from_parts(
        mut suffix_patterns: Vec<(String, FastNaming)>,
        mut glob_patterns: Vec<(String, FastNaming)>,
        default: Option<FastNaming>,
    ) -> Self {
        suffix_patterns.sort_by(|left, right| {
            right
                .0
                .len()
                .cmp(&left.0.len())
                .then_with(|| left.0.cmp(&right.0))
        });
        glob_patterns.sort_by(|left, right| {
            right
                .0
                .len()
                .cmp(&left.0.len())
                .then_with(|| left.0.cmp(&right.0))
        });
        Self {
            suffix_patterns,
            glob_patterns: glob_patterns
                .into_iter()
                .map(|(pattern, naming)| FastPatternNaming::new(pattern, naming))
                .collect(),
            default,
        }
    }

    pub(super) fn parts(
        &self,
    ) -> (
        &[(String, FastNaming)],
        &[FastPatternNaming],
        Option<&FastNaming>,
    ) {
        (
            &self.suffix_patterns,
            &self.glob_patterns,
            self.default.as_ref(),
        )
    }

    pub(super) fn naming_for<'a>(
        &'a self,
        filename: &str,
        glob_patterns: &HashMap<String, Pattern>,
    ) -> Option<FastFileNamingMatch<'a>> {
        let suffix_match = best_lslint_suffix_pair(&self.suffix_patterns, filename);
        let glob_match = self
            .glob_patterns
            .iter()
            .find(|pattern| pattern.matches(filename, glob_patterns));

        match (suffix_match, glob_match) {
            (Some((suffix, suffix_naming)), Some(glob)) => {
                if glob.pattern.len() > suffix.len() {
                    Some(FastFileNamingMatch {
                        naming: &glob.naming,
                        lslint_extension_pattern: false,
                    })
                } else {
                    Some(FastFileNamingMatch {
                        naming: suffix_naming,
                        lslint_extension_pattern: true,
                    })
                }
            }
            (Some((_, suffix_naming)), None) => Some(FastFileNamingMatch {
                naming: suffix_naming,
                lslint_extension_pattern: true,
            }),
            (None, Some(glob)) => Some(FastFileNamingMatch {
                naming: &glob.naming,
                lslint_extension_pattern: false,
            }),
            (None, None) => self.default.as_ref().map(|naming| FastFileNamingMatch {
                naming,
                lslint_extension_pattern: false,
            }),
        }
    }
}

impl FastPatternNaming {
    fn new(pattern: String, naming: FastNaming) -> Self {
        Self {
            matcher: FastPatternMatcher::Glob(pattern.clone()),
            pattern,
            naming,
        }
    }

    pub(super) fn parts(&self) -> (&str, &FastNaming) {
        (&self.pattern, &self.naming)
    }

    fn matches(&self, filename: &str, glob_patterns: &HashMap<String, Pattern>) -> bool {
        match &self.matcher {
            FastPatternMatcher::Glob(pattern) => {
                matches_single_compiled_pattern(pattern, filename, glob_patterns)
            }
        }
    }
}

impl FastNaming {
    pub(super) fn label(&self) -> &str {
        match self {
            FastNaming::Static { label, .. }
            | FastNaming::EmptyRegex { label }
            | FastNaming::ExactAny { label, .. }
            | FastNaming::ContainsAny { label, .. }
            | FastNaming::Regex { label, .. }
            | FastNaming::Alternatives { label, .. }
            | FastNaming::Dynamic(label) => label,
        }
    }
}

pub(super) fn compile_fast_naming(convention: &str) -> FastNaming {
    let convention = convention.trim();
    if convention.contains('|') && (!convention.starts_with("regex:") || convention.contains(" | "))
    {
        let alternatives = split_naming_conventions(convention)
            .into_iter()
            .map(compile_fast_naming)
            .collect::<Vec<_>>();
        if alternatives.len() > 1 {
            return FastNaming::Alternatives {
                label: convention.to_string(),
                alternatives,
            };
        }
    }

    if let Some(pattern) = convention.strip_prefix("regex:") {
        if pattern.starts_with('!') || pattern.contains("${") {
            return FastNaming::Dynamic(convention.to_string());
        }
        if pattern == "^$" {
            return FastNaming::EmptyRegex {
                label: convention.to_string(),
            };
        }
        if let Some(literals) = simple_regex_literals(pattern) {
            return if literals.exact {
                FastNaming::ExactAny {
                    label: convention.to_string(),
                    literals: literals.values,
                }
            } else {
                FastNaming::ContainsAny {
                    label: convention.to_string(),
                    literals: literals.values,
                }
            };
        }
        return FastNaming::Regex {
            label: convention.to_string(),
            pattern: pattern.to_string(),
        };
    }

    if let Some(validate) = convention_to_case_validator(convention) {
        return FastNaming::Static {
            label: convention.to_string(),
            validate,
        };
    }

    FastNaming::Dynamic(convention.to_string())
}

pub(super) fn collect_fast_naming_regex_patterns(naming: &FastNaming, patterns: &mut Vec<String>) {
    match naming {
        FastNaming::Regex { pattern, .. } => {
            let pattern = pattern.strip_prefix('!').unwrap_or(pattern);
            if !pattern.contains("${") {
                patterns.push(pattern.to_string());
            }
        }
        FastNaming::Alternatives { alternatives, .. } => {
            for alternative in alternatives {
                collect_fast_naming_regex_patterns(alternative, patterns);
            }
        }
        FastNaming::Static { .. }
        | FastNaming::EmptyRegex { .. }
        | FastNaming::ExactAny { .. }
        | FastNaming::ContainsAny { .. }
        | FastNaming::Dynamic(_) => {}
    }
}

struct SimpleRegexLiterals {
    exact: bool,
    values: Vec<String>,
}

fn simple_regex_literals(pattern: &str) -> Option<SimpleRegexLiterals> {
    let (exact, body) = pattern
        .strip_prefix('^')
        .and_then(|body| body.strip_suffix('$'))
        .map(|body| (true, body))
        .unwrap_or((false, pattern));
    if exact && body.contains('|') && strip_wrapping_group(body) == body {
        return None;
    }
    let body = strip_wrapping_group(body);
    let mut values = Vec::new();
    for part in body.split('|') {
        if part.is_empty() {
            return None;
        }
        values.push(simple_regex_literal(part)?);
    }
    Some(SimpleRegexLiterals { exact, values })
}

fn strip_wrapping_group(pattern: &str) -> &str {
    pattern
        .strip_prefix('(')
        .and_then(|body| body.strip_suffix(')'))
        .filter(|body| !body.contains(['(', ')']))
        .unwrap_or(pattern)
}

fn simple_regex_literal(pattern: &str) -> Option<String> {
    let mut literal = String::with_capacity(pattern.len());
    let mut escaped = false;
    for ch in pattern.chars() {
        if escaped {
            if matches!(ch, '.' | '_' | '-' | ' ') || ch.is_ascii_alphanumeric() {
                literal.push(ch);
                escaped = false;
                continue;
            }
            return None;
        }
        if ch == '\\' {
            escaped = true;
        } else if matches!(
            ch,
            '^' | '$' | '(' | ')' | '[' | ']' | '{' | '}' | '*' | '+' | '?' | '|'
        ) {
            return None;
        } else if matches!(ch, '.' | '_' | '-' | ' ') || ch.is_ascii_alphanumeric() {
            literal.push(ch);
        } else {
            return None;
        }
    }
    (!escaped).then_some(literal)
}
