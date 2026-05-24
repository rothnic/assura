//! Fast naming matchers for LS-Lint-compatible checks.

use super::case::{convention_to_case_validator, validate_single_name};
use super::patterns::matches_single_compiled_pattern;
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
    naming: &FastNaming,
    regexes: &HashMap<String, Regex>,
) -> bool {
    validate_fast_name(stem, naming, regexes)
        || stem
            .split_once('.')
            .map(|(base, _)| validate_fast_name(base, naming, regexes))
            .unwrap_or(false)
}

pub(super) fn validate_fast_name(
    name: &str,
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
        FastNaming::Regex { pattern, .. } => regexes
            .get(pattern)
            .map(|regex| regex.is_match(name))
            .unwrap_or(false),
        FastNaming::Alternatives { alternatives, .. } => alternatives
            .iter()
            .any(|alternative| validate_fast_name(name, alternative, regexes)),
        FastNaming::Dynamic(convention) => validate_single_name(name, convention, regexes),
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
    ) -> Option<&'a FastNaming> {
        self.best_suffix_naming(filename)
            .or_else(|| {
                self.glob_patterns
                    .iter()
                    .find(|pattern| pattern.matches(filename, glob_patterns))
                    .map(|pattern| &pattern.naming)
            })
            .or(self.default.as_ref())
    }

    fn best_suffix_naming<'a>(&'a self, filename: &str) -> Option<&'a FastNaming> {
        self.suffix_patterns
            .iter()
            .find(|(suffix, _)| filename.ends_with(suffix))
            .map(|(_, naming)| naming)
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
        FastNaming::Regex { pattern, .. } => patterns.push(pattern.clone()),
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
