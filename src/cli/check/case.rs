//! Lightweight naming convention validation for check-only builds.

use crate::config::config::split_naming_conventions;
use regex_lite::Regex;
use std::collections::HashMap;

pub(super) fn validate_name(
    name: &str,
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
        return validate_single_name(name, convention, regexes);
    }

    let alternatives = split_naming_conventions(convention);
    if alternatives.len() > 1 {
        return alternatives
            .into_iter()
            .any(|part| validate_name(name, part, regexes));
    }

    alternatives
        .first()
        .map(|part| validate_single_name(name, part, regexes))
        .unwrap_or(false)
}

pub(super) fn validate_single_name(
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

    match convention_to_case_validator(convention) {
        Some(validate) => validate(name),
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

pub(super) fn convention_to_case_validator(convention: &str) -> Option<fn(&str) -> bool> {
    match convention {
        "snake_case" => Some(validate_snake_case),
        "camelCase" => Some(validate_camel_case),
        "PascalCase" => Some(validate_pascal_case),
        "kebab-case" => Some(validate_kebab_case),
        "SCREAMING_SNAKE_CASE" => Some(validate_screaming_snake_case),
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
        return !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_');
    }

    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_lowercase() || ch.is_numeric() || ch == '_')
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
    let mut prev_upper = false;
    for byte in name.bytes() {
        if byte == b'_' || byte == b'-' {
            return false;
        }
        if byte.is_ascii_uppercase() {
            if prev_upper {
                return false;
            }
            prev_upper = true;
        } else {
            prev_upper = false;
        }
    }
    true
}

fn validate_mixed_case_body(name: &str) -> bool {
    let mut prev_upper = false;
    for ch in name.chars() {
        if ch == '_' || ch == '-' {
            return false;
        }
        if ch.is_uppercase() {
            if prev_upper {
                return false;
            }
            prev_upper = true;
        } else {
            prev_upper = false;
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
