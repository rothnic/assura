//! Semantic validation for configured file bundles.

use super::{validate_naming_convention_text, validate_range, validate_size_string_text};
use crate::config::config::FileBundle;

pub(super) fn validate_file_bundle(bundle: &FileBundle, context: &str) -> Result<(), String> {
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
    if let Some(patterns) = &bundle.max_lines_patterns {
        for (pattern, max_lines) in patterns {
            validate_range(
                *max_lines,
                1,
                100_000,
                &format!("{context}.max_lines_patterns.{pattern}"),
            )?;
        }
    }
    if let Some(max_size) = &bundle.max_size {
        validate_size_string_text(max_size)
            .map_err(|error| format!("{context}.max_size: {error}"))?;
    }
    if let Some(patterns) = &bundle.max_size_patterns {
        for (pattern, max_size) in patterns {
            validate_size_string_text(max_size)
                .map_err(|error| format!("{context}.max_size_patterns.{pattern}: {error}"))?;
        }
    }

    Ok(())
}
