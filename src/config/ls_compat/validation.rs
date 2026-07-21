//! Validation helpers for the LS-Lint migration converter.

use super::LsLintMigrationReport;
#[cfg(feature = "yaml-config")]
use crate::config::config::{validate_config_semantics, Config};
use regex_lite::Regex;

#[cfg(feature = "yaml-config")]
pub(super) fn validate_lslint_document_shape(config: &serde_yaml::Value) -> Result<(), String> {
    let Some(mapping) = config.as_mapping() else {
        return Err("Unsupported LS-Lint YAML shape: document must be a mapping".to_string());
    };

    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err(
                "Unsupported LS-Lint YAML shape: top-level keys must be strings".to_string(),
            );
        };
        match key {
            "ls" => {
                if !value.is_mapping() {
                    return Err(
                        "Unsupported LS-Lint YAML shape: 'ls' must be a mapping".to_string()
                    );
                }
            }
            "ignore" => {
                let Some(items) = value.as_sequence() else {
                    return Err(
                        "Unsupported LS-Lint YAML shape: 'ignore' must be a sequence of strings"
                            .to_string(),
                    );
                };
                if items.iter().any(|item| !item.is_string()) {
                    return Err(
                        "Unsupported LS-Lint YAML shape: 'ignore' entries must be strings"
                            .to_string(),
                    );
                }
            }
            other => {
                return Err(format!(
                    "Unsupported LS-Lint YAML shape: unknown top-level key '{other}'"
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn split_rule_tokens(rule: &str) -> Result<Vec<&str>, String> {
    let trimmed = rule.trim();
    if trimmed.is_empty() {
        return Err("Invalid LS-Lint rule syntax: rule must not be empty".to_string());
    }

    if rule.contains(" | ") {
        let tokens = rule.split(" | ").map(str::trim).collect::<Vec<_>>();
        if tokens.iter().any(|token| token.is_empty()) {
            return Err(format!(
                "Invalid LS-Lint rule syntax '{rule}': empty rule around ' | '"
            ));
        }
        return Ok(tokens);
    }

    if trimmed.contains('|') && !trimmed.starts_with("regex:") {
        return Err(format!(
            "Invalid LS-Lint rule syntax '{rule}': multiple rules must use ' | '"
        ));
    }

    Ok(vec![trimmed])
}

pub(super) fn parse_exists_token(token: &str) -> Result<Option<String>, String> {
    if token == "exists" {
        Ok(Some("exists".to_string()))
    } else if let Some(raw) = token.strip_prefix("exists:") {
        Ok(Some(parse_exists_count(raw)?))
    } else {
        Ok(None)
    }
}

fn parse_exists_count(raw: &str) -> Result<String, String> {
    let raw = raw.trim();
    if raw.is_empty() {
        return Err("Invalid LS-Lint exists rule: exists value is empty".to_string());
    }

    let parts = raw.split('-').collect::<Vec<_>>();
    if parts.len() > 1 {
        parse_exists_bound(parts[0], raw)?;
        parse_exists_bound(parts[1], raw)?;
        return Ok(format!("{}-{}", parts[0].trim(), parts[1].trim()));
    }

    parse_exists_bound(raw, raw)?;
    Ok(raw.to_string())
}

fn parse_exists_bound(bound: &str, raw: &str) -> Result<(), String> {
    let bound = bound.trim();
    if bound.is_empty() {
        return Err(format!(
            "Invalid LS-Lint exists rule 'exists:{raw}': range bounds must be non-empty"
        ));
    }
    bound
        .parse::<i16>()
        .map(|_| ())
        .map_err(|error| format!("Invalid LS-Lint exists rule 'exists:{raw}': {error}"))
}

pub(super) fn normalize_lslint_naming_token(token: &str) -> Result<String, String> {
    let Some(pattern) = token.strip_prefix("regex:") else {
        validate_lslint_naming_rule(token)?;
        return Ok(token.to_string());
    };

    if pattern.is_empty() {
        return Err("Unsupported LS-Lint regex rule: pattern is empty".to_string());
    }

    if let Some(pattern) = pattern.strip_prefix('!') {
        let normalized = format!("regex:!^{pattern}$");
        validate_lslint_regex_rule(&normalized)?;
        return Ok(normalized);
    }

    let normalized = format!("regex:^{pattern}$");
    validate_lslint_regex_rule(&normalized)?;
    Ok(normalized)
}

fn validate_lslint_naming_rule(token: &str) -> Result<(), String> {
    match token {
        "lowercase"
        | "camelcase"
        | "camelCase"
        | "pascalcase"
        | "PascalCase"
        | "snakecase"
        | "snake_case"
        | "screamingsnakecase"
        | "SCREAMING_SNAKE_CASE"
        | "kebabcase"
        | "kebab-case" => Ok(()),
        _ => Err(format!("Unknown LS-Lint rule name '{token}'")),
    }
}

fn validate_lslint_regex_rule(rule: &str) -> Result<(), String> {
    let pattern = rule
        .strip_prefix("regex:!")
        .or_else(|| rule.strip_prefix("regex:"))
        .unwrap_or(rule);
    Regex::new(pattern)
        .map(|_| ())
        .map_err(|error| format!("Invalid LS-Lint regex rule '{rule}': {error}"))
}

#[cfg(feature = "yaml-config")]
pub(super) fn migration_report_for_mapping(
    mapping: &serde_yaml::Mapping,
    ignored_patterns: usize,
) -> Result<LsLintMigrationReport, String> {
    let mut report = LsLintMigrationReport {
        ignored_patterns,
        ..LsLintMigrationReport::default()
    };
    collect_report_counts(mapping, &mut report)?;
    Ok(report)
}

#[cfg(feature = "yaml-config")]
fn collect_report_counts(
    mapping: &serde_yaml::Mapping,
    report: &mut LsLintMigrationReport,
) -> Result<(), String> {
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            return Err("Unsupported LS-Lint YAML shape: 'ls' keys must be strings".to_string());
        };
        if let Some(child) = value.as_mapping() {
            report.path_rules += 1;
            collect_report_counts(child, report)?;
            continue;
        }

        if key.starts_with('.') {
            if key != ".dir" {
                report.extension_rules += 1;
            }
            let rule = value.as_str().ok_or_else(|| {
                format!("Unsupported LS-Lint YAML shape: rule for '{key}' must be a string")
            })?;
            for token in split_rule_tokens(rule)? {
                if parse_exists_token(token)?.is_some() {
                    report.exists_rules += 1;
                }
            }
            continue;
        }

        if let Some(rule) = value.as_str() {
            for token in split_rule_tokens(rule)? {
                if parse_exists_token(token)?.is_some() {
                    report.exists_rules += 1;
                } else {
                    normalize_lslint_naming_token(token)?;
                }
            }
        } else {
            return Err(format!(
                "Unsupported LS-Lint YAML shape: value for '{key}' must be a rule string or mapping"
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "yaml-config")]
pub(super) fn validate_converted_config(config: &Config) -> Result<(), String> {
    validate_config_semantics(config)
        .map_err(|error| format!("Converted Assura config is invalid: {error}"))?;
    let yaml = serde_yaml::to_string(config)
        .map_err(|error| format!("Failed to serialize converted Assura config: {error}"))?;
    let reparsed: Config = serde_yaml::from_str(&yaml)
        .map_err(|error| format!("Converted Assura config did not round-trip: {error}"))?;
    validate_config_semantics(&reparsed)
        .map_err(|error| format!("Converted Assura config round-trip is invalid: {error}"))?;
    Ok(())
}
