//! Validators shared by structure-first config bundles.

#[cfg(feature = "yaml-config")]
use super::{Config, DirectoryBundle, DirectoryNode, FileBundle, MarkdownBundle};

/// Validate structure-first config semantics without the full validator stack.
#[cfg(feature = "yaml-config")]
pub(crate) fn validate_config_semantics(config: &Config) -> Result<(), String> {
    for (pattern, bundle) in &config.patterns {
        validate_file_bundle(bundle, &format!("patterns.{pattern}"))?;
    }

    for (path, node) in &config.structure {
        validate_directory_node(node, &format!("structure.{path}"))?;
    }

    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_directory_node(node: &DirectoryNode, context: &str) -> Result<(), String> {
    if let Some(files) = &node.files {
        validate_file_bundle(files, &format!("{context}.files"))?;
    }
    if let Some(directories) = &node.directories {
        validate_directory_bundle(directories, &format!("{context}.directories"))?;
    }
    if let Some(markdown) = &node.markdown {
        validate_markdown_bundle(markdown, &format!("{context}.markdown"))?;
    }
    if let Some(children) = &node.children {
        for (child_name, child) in children {
            validate_directory_node(child, &format!("{context}.children.{child_name}"))?;
        }
    }

    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_file_bundle(bundle: &FileBundle, context: &str) -> Result<(), String> {
    if let Some(naming) = &bundle.naming {
        validate_naming_convention_text(naming)
            .map_err(|error| format!("{context}.naming: {error}"))?;
    }
    if let Some(patterns) = &bundle.naming_patterns {
        for (pattern, naming) in patterns {
            validate_naming_convention_text(naming)
                .map_err(|error| format!("{context}.naming_patterns.{pattern}: {error}"))?;
        }
    }
    if let Some(max_lines) = bundle.max_lines {
        validate_range(max_lines, 1, 100_000, &format!("{context}.max_lines"))?;
    }
    if let Some(max_size) = &bundle.max_size {
        validate_size_string_text(max_size)
            .map_err(|error| format!("{context}.max_size: {error}"))?;
    }

    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_directory_bundle(bundle: &DirectoryBundle, context: &str) -> Result<(), String> {
    if let Some(naming) = &bundle.naming {
        validate_naming_convention_text(naming)
            .map_err(|error| format!("{context}.naming: {error}"))?;
    }

    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_markdown_bundle(bundle: &MarkdownBundle, context: &str) -> Result<(), String> {
    if let Some(depth) = bundle.max_heading_depth {
        validate_range(
            usize::from(depth),
            1,
            6,
            &format!("{context}.max_heading_depth"),
        )?;
    }

    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_range(value: usize, min: usize, max: usize, context: &str) -> Result<(), String> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(format!("{context} must be between {min} and {max}"))
    }
}

/// Validates that a naming convention string is valid.
#[cfg(feature = "full-cli")]
pub(crate) fn validate_naming_convention(conv: &str) -> Result<(), validator::ValidationError> {
    validate_naming_convention_text(conv).map_err(|message| {
        let mut err = validator::ValidationError::new("invalid_naming_convention");
        err.message = Some(message.into());
        err
    })
}

#[cfg(any(feature = "yaml-config", feature = "full-cli"))]
fn validate_naming_convention_text(conv: &str) -> Result<(), String> {
    let alternatives = split_naming_conventions(conv);
    if alternatives.len() > 1 {
        for part in alternatives {
            validate_naming_convention_text(part)?;
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
        Err(format!(
            "'{}' is not a valid naming convention. Valid options: {:?}",
            conv, valid_conventions
        ))
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
#[cfg(feature = "full-cli")]
pub(crate) fn validate_size_string(size: &str) -> Result<(), validator::ValidationError> {
    validate_size_string_text(size).map_err(|message| {
        let mut err = validator::ValidationError::new("invalid_size_string");
        err.message = Some(message.into());
        err
    })
}

#[cfg(any(feature = "yaml-config", feature = "full-cli"))]
fn validate_size_string_text(size: &str) -> Result<(), String> {
    if is_valid_size_string(size) {
        Ok(())
    } else {
        Err(format!(
            "'{}' is not a valid size string. Expected format: '<number><unit>' where unit is B, KB, MB, GB, or TB",
            size
        ))
    }
}

#[cfg(any(feature = "yaml-config", feature = "full-cli"))]
fn is_valid_size_string(size: &str) -> bool {
    let trimmed = size.trim();
    if trimmed.is_empty() {
        return false;
    }

    let digits_len = trimmed
        .as_bytes()
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digits_len == 0 {
        return false;
    }

    let unit = trimmed[digits_len..].trim_start();
    matches!(unit, "B" | "KB" | "MB" | "GB" | "TB")
}
