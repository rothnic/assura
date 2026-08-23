//! Lightweight naming convention validation for check-only builds.

use crate::config::config::split_naming_conventions;
use regex_lite::Regex;
use std::collections::HashMap;

pub(super) fn validate_name_with_path(
    name: &str,
    path: &str,
    convention: &str,
    regexes: &HashMap<String, Regex>,
) -> bool {
    let convention = convention.trim();
    if convention.is_empty() {
        return false;
    }

    if !convention.contains('|')
        || (convention.starts_with("regex:") && !convention.contains(" | "))
    {
        return validate_single_name_with_path(name, path, convention, regexes);
    }

    let alternatives = split_naming_conventions(convention);
    if alternatives.len() > 1 {
        return alternatives
            .into_iter()
            .any(|part| validate_name_with_path(name, path, part, regexes));
    }

    alternatives
        .first()
        .map(|part| validate_single_name_with_path(name, path, part, regexes))
        .unwrap_or(false)
}

pub(super) fn validate_single_name_with_path(
    name: &str,
    path: &str,
    convention: &str,
    regexes: &HashMap<String, Regex>,
) -> bool {
    if let Some(expected) = convention.strip_prefix("exact:") {
        return !expected.is_empty() && name == expected;
    }
    if let Some(pattern) = convention.strip_prefix("regex:") {
        return validate_regex_name(name, path, pattern, regexes);
    }

    match convention_to_case_validator(convention) {
        Some(validate) => validate(name),
        None => false,
    }
}

pub(super) fn validate_file_stem_with_path(
    stem: &str,
    path: &str,
    convention: &str,
    regexes: &HashMap<String, Regex>,
) -> bool {
    split_naming_conventions(convention)
        .into_iter()
        .any(|part| {
            if part.starts_with("exact:") || part.starts_with("regex:") {
                return validate_single_name_with_path(stem, path, part, regexes);
            }
            stem.split('.').all(|segment| {
                !segment.is_empty() && validate_single_name_with_path(segment, path, part, regexes)
            })
        })
}

fn validate_regex_name(
    name: &str,
    path: &str,
    pattern: &str,
    regexes: &HashMap<String, Regex>,
) -> bool {
    let (negated, pattern) = pattern
        .strip_prefix('!')
        .map(|pattern| (true, pattern))
        .unwrap_or((false, pattern));
    let substituted = substitute_lslint_regex_path(pattern, path);
    let pattern = substituted.as_deref().unwrap_or(pattern);
    let matched = regexes
        .get(pattern)
        .map(|regex| regex.is_match(name))
        .or_else(|| Regex::new(pattern).ok().map(|regex| regex.is_match(name)))
        .unwrap_or(false);

    matched != negated
}

fn substitute_lslint_regex_path(pattern: &str, path: &str) -> Option<String> {
    if path.is_empty() || !pattern.contains("${") {
        return None;
    }

    let segments = path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();
    if segments.is_empty() {
        return None;
    }

    let mut substituted = pattern.to_string();
    for (index, segment) in segments.iter().enumerate() {
        let ancestor_index = segments.len() - 1 - index;
        substituted = substituted.replace(&format!("${{{ancestor_index}}}"), segment);
    }
    Some(substituted)
}

pub(super) fn convention_to_case_validator(convention: &str) -> Option<fn(&str) -> bool> {
    match convention {
        "snake_case" | "snakecase" => Some(validate_snake_case),
        "camelCase" | "camelcase" => Some(validate_camel_case),
        "PascalCase" | "pascalcase" => Some(validate_pascal_case),
        "kebab-case" | "kebabcase" => Some(validate_kebab_case),
        "SCREAMING_SNAKE_CASE" | "screamingsnakecase" => Some(validate_screaming_snake_case),
        "dot.case" => Some(validate_dot_case),
        "flatcase" => Some(validate_flatcase),
        "FLATCASE" => Some(validate_screaming_flatcase),
        "COBOL-CASE" => Some(validate_cobol_case),
        "Train-Case" => Some(validate_train_case),
        "lowercase" => Some(validate_lowercase),
        "UPPERCASE" => Some(validate_uppercase),
        _ => None,
    }
}

fn validate_lowercase(name: &str) -> bool {
    if name.is_ascii() {
        return name
            .bytes()
            .all(|byte| !byte.is_ascii_alphabetic() || byte.is_ascii_lowercase());
    }

    name.chars()
        .all(|ch| !ch.is_alphabetic() || ch.is_lowercase())
}

fn validate_uppercase(name: &str) -> bool {
    if name.is_ascii() {
        return !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_');
    }

    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_uppercase() || ch.is_numeric() || ch == '_')
}

fn validate_snake_case(name: &str) -> bool {
    if name.is_ascii() {
        return validate_delimited_ascii(name, b'_', |byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit()
        });
    }

    !name.is_empty()
        && !name.starts_with('_')
        && !name.ends_with('_')
        && !name.contains("__")
        && name
            .chars()
            .all(|ch| ch.is_lowercase() || ch.is_numeric() || ch == '_')
}

fn validate_camel_case(name: &str) -> bool {
    if name.is_ascii() {
        return name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
            && validate_mixed_case_body_ascii(name);
    }

    name.chars().next().is_some_and(char::is_lowercase) && validate_mixed_case_body(name)
}

fn validate_pascal_case(name: &str) -> bool {
    if name.is_ascii() {
        return name
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
            && validate_mixed_case_body_ascii(name);
    }

    name.chars().next().is_some_and(char::is_uppercase) && validate_mixed_case_body(name)
}

fn validate_mixed_case_body_ascii(name: &str) -> bool {
    let bytes = name.as_bytes();
    for (index, byte) in bytes.iter().copied().enumerate() {
        if byte == b'_' || byte == b'-' {
            return false;
        }
        if byte.is_ascii_uppercase() {
            if index == 0 {
                continue;
            }
            if bytes[index - 1].is_ascii_digit() {
                continue;
            }
            if index >= 2
                && bytes[index - 1].is_ascii_uppercase()
                && bytes[index - 2].is_ascii_lowercase()
            {
                continue;
            }
            if !bytes[index - 1].is_ascii_lowercase() {
                return false;
            }
        }
    }
    true
}

fn validate_mixed_case_body(name: &str) -> bool {
    let chars = name.chars().collect::<Vec<_>>();
    for (index, ch) in chars.iter().copied().enumerate() {
        if ch == '_' || ch == '-' {
            return false;
        }
        if ch.is_uppercase() {
            if index == 0 {
                continue;
            }
            if chars[index - 1].is_numeric() {
                continue;
            }
            if index >= 2 && chars[index - 1].is_uppercase() && chars[index - 2].is_lowercase() {
                continue;
            }
            if !chars[index - 1].is_lowercase() {
                return false;
            }
        }
    }
    true
}

fn validate_kebab_case(name: &str) -> bool {
    if name.is_ascii() {
        return validate_delimited_ascii(name, b'-', |byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit()
        });
    }

    !name.is_empty()
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .chars()
            .all(|ch| ch.is_lowercase() || ch.is_numeric() || ch == '-')
}

fn validate_screaming_snake_case(name: &str) -> bool {
    if name.is_ascii() {
        return validate_delimited_ascii(name, b'_', |byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit()
        });
    }

    !name.is_empty()
        && !name.starts_with('_')
        && !name.ends_with('_')
        && !name.contains("__")
        && name
            .chars()
            .all(|ch| ch.is_uppercase() || ch.is_numeric() || ch == '_')
}

fn validate_dot_case(name: &str) -> bool {
    if name.is_ascii() {
        return validate_delimited_ascii(name, b'.', |byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit()
        });
    }

    !name.is_empty()
        && !name.starts_with('.')
        && !name.ends_with('.')
        && !name.contains("..")
        && name
            .chars()
            .all(|ch| ch.is_lowercase() || ch.is_numeric() || ch == '.')
}

fn validate_flatcase(name: &str) -> bool {
    if name.is_ascii() {
        return !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
    }

    !name.is_empty() && name.chars().all(|ch| ch.is_lowercase() || ch.is_numeric())
}

fn validate_screaming_flatcase(name: &str) -> bool {
    if name.is_ascii() {
        return !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit());
    }

    !name.is_empty() && name.chars().all(|ch| ch.is_uppercase() || ch.is_numeric())
}

fn validate_cobol_case(name: &str) -> bool {
    if name.is_ascii() {
        return validate_delimited_ascii(name, b'-', |byte| {
            byte.is_ascii_uppercase() || byte.is_ascii_digit()
        });
    }

    !name.is_empty()
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name
            .chars()
            .all(|ch| ch.is_uppercase() || ch.is_numeric() || ch == '-')
}

fn validate_train_case(name: &str) -> bool {
    if name.is_ascii() {
        return !name.is_empty()
            && !name.starts_with('-')
            && !name.ends_with('-')
            && !name.contains("--")
            && name.split('-').all(validate_train_case_ascii_part);
    }

    !name.is_empty()
        && !name.starts_with('-')
        && !name.ends_with('-')
        && !name.contains("--")
        && name.split('-').all(|part| {
            !part.is_empty()
                && (part.chars().all(char::is_numeric)
                    || (part.chars().next().is_some_and(char::is_uppercase)
                        && part
                            .chars()
                            .skip(1)
                            .all(|ch| ch.is_lowercase() || ch.is_numeric())))
        })
}

fn validate_train_case_ascii_part(part: &str) -> bool {
    !part.is_empty()
        && (part.bytes().all(|byte| byte.is_ascii_digit())
            || (part
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_uppercase())
                && part
                    .bytes()
                    .skip(1)
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())))
}

fn validate_delimited_ascii(name: &str, delimiter: u8, valid_segment_byte: fn(u8) -> bool) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut previous_was_delimiter = true;
    for byte in name.bytes() {
        if byte == delimiter {
            if previous_was_delimiter {
                return false;
            }
            previous_was_delimiter = true;
        } else if valid_segment_byte(byte) {
            previous_was_delimiter = false;
        } else {
            return false;
        }
    }

    !previous_was_delimiter
}
