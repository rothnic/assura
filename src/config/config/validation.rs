//! Validators shared by structure-first config bundles.

/// Regex for size string validation.
pub(super) static SIZE_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"^\d+\s*(B|KB|MB|GB|TB)$").unwrap());

/// Validates that a naming convention string is valid.
pub(crate) fn validate_naming_convention(conv: &str) -> Result<(), validator::ValidationError> {
    let alternatives = split_naming_conventions(conv);
    if alternatives.len() > 1 {
        for part in alternatives {
            validate_naming_convention(part)?;
        }
        return Ok(());
    }

    let valid_conventions = [
        "snake_case",
        "camelCase",
        "PascalCase",
        "kebab-case",
        "SCREAMING_SNAKE_CASE",
        "dot.case",
        "flatcase",
        "FLATCASE",
        "COBOL-CASE",
        "Train-Case",
        "lowercase",
        "UPPERCASE",
        "regex:",
    ];

    if valid_conventions
        .iter()
        .any(|&c| conv == c || conv.starts_with(c))
        || conv.starts_with("regex:")
    {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_naming_convention");
        err.message = Some(
            format!(
                "'{}' is not a valid naming convention. Valid options: {:?}",
                conv, valid_conventions
            )
            .into(),
        );
        Err(err)
    }
}

/// Split OR-composed naming conventions without splitting pipes inside regexes.
pub(crate) fn split_naming_conventions(conv: &str) -> Vec<&str> {
    let trimmed = conv.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    if !trimmed.contains('|') {
        return vec![trimmed];
    }

    if trimmed.contains(" | ") {
        return trimmed
            .split(" | ")
            .map(str::trim)
            .filter(|part| !part.is_empty())
            .collect();
    }

    if trimmed.starts_with("regex:") {
        return vec![trimmed];
    }

    let segments: Vec<&str> = trimmed.split('|').map(str::trim).collect();
    let regex_start = segments.iter().position(|part| part.starts_with("regex:"));
    match regex_start {
        Some(index) => {
            let mut parts: Vec<&str> = segments[..index]
                .iter()
                .copied()
                .filter(|part| !part.is_empty())
                .collect();
            let regex_start = trimmed.find(segments[index]).unwrap_or(0);
            parts.push(trimmed[regex_start..].trim());
            parts
        }
        None => segments
            .into_iter()
            .filter(|part| !part.is_empty())
            .collect(),
    }
}

/// Validates that a size string is valid, such as `100KB`, `1MB`, or `10 MB`.
pub(crate) fn validate_size_string(size: &str) -> Result<(), validator::ValidationError> {
    if SIZE_REGEX.is_match(size) {
        Ok(())
    } else {
        let mut err = validator::ValidationError::new("invalid_size_string");
        err.message = Some(
            format!(
                "'{}' is not a valid size string. Expected format: '<number><unit>' where unit is B, KB, MB, GB, or TB",
                size
            )
            .into(),
        );
        Err(err)
    }
}
