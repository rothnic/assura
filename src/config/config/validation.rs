//! Validators shared by structure-first config bundles.

#[cfg(feature = "yaml-config")]
use super::{
    Config, CustomConstraintConfig, DirectoryBundle, DirectoryNode, ExtensionConfig, FileBundle,
    MarkdownBundle, RelationshipConstraintConfig,
};
#[cfg(feature = "yaml-config")]
use glob::Pattern;
#[cfg(feature = "yaml-config")]
use std::collections::HashSet;
#[cfg(feature = "yaml-config")]
use std::path::{Component, Path};

#[cfg(feature = "yaml-config")]
mod docs_lifecycles;
#[cfg(feature = "yaml-config")]
mod manifest_semantics;
#[cfg(feature = "yaml-config")]
mod module_topologies;
#[cfg(feature = "yaml-config")]
mod quality;
#[cfg(feature = "yaml-config")]
mod release_contracts;
#[cfg(feature = "yaml-config")]
mod support_matrices;
#[cfg(feature = "yaml-config")]
mod test_relationships;

/// Validate structure-first config semantics without the full validator stack.
#[cfg(feature = "yaml-config")]
pub(crate) fn validate_config_semantics(config: &Config) -> Result<(), String> {
    for (pattern, bundle) in &config.patterns {
        validate_file_bundle(bundle, &format!("patterns.{pattern}"))?;
    }

    for (path, node) in &config.structure {
        validate_directory_node(node, &format!("structure.{path}"))?;
    }
    if let Some(extensions) = &config.extensions {
        validate_extension_config(extensions)?;
    }
    if let Some(quality) = &config.quality {
        quality::validate_quality_config(quality)?;
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
    if let Some(directory) = &node.self_directory {
        validate_directory_bundle(directory, &format!("{context}.self_directory"))?;
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
    if bundle.required_fields.is_some() {
        return Err(format!(
            "{context}.required_fields: unsupported for typed frontmatter fields; define required fields with top-level models and collections instead. Keep markdown.require_frontmatter only for generic Markdown frontmatter presence."
        ));
    }

    if let Some(depth) = bundle.max_heading_depth {
        validate_range(
            usize::from(depth),
            1,
            6,
            &format!("{context}.max_heading_depth"),
        )?;
    }

    bundle.validate_outline_semantics(context)?;

    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_extension_config(config: &ExtensionConfig) -> Result<(), String> {
    let mut ids = HashSet::new();
    for constraint in &config.custom_constraints {
        validate_custom_constraint(constraint)?;
        if !ids.insert(&constraint.id) {
            return Err(format!(
                "extensions.custom_constraints.{}: duplicate custom constraint id",
                constraint.id
            ));
        }
    }
    let mut release_contract_ids = HashSet::new();
    for contract in &config.release_contracts {
        release_contracts::validate_release_contract_config(contract)?;
        if !release_contract_ids.insert(&contract.id) {
            return Err(format!(
                "extensions.release_contracts.{}: duplicate release contract id",
                contract.id
            ));
        }
    }
    let mut manifest_semantics_ids = HashSet::new();
    for policy in &config.manifest_semantics {
        manifest_semantics::validate_manifest_semantics_config(policy)?;
        if !manifest_semantics_ids.insert(policy.id.as_str()) {
            return Err(format!(
                "extensions.manifest_semantics.{}: duplicate manifest semantics id",
                policy.id
            ));
        }
    }
    let mut support_matrix_ids = HashSet::new();
    for matrix in &config.support_matrices {
        support_matrices::validate_support_matrix_config(matrix, &manifest_semantics_ids)?;
        if !support_matrix_ids.insert(&matrix.id) {
            return Err(format!(
                "extensions.support_matrices.{}: duplicate support matrix id",
                matrix.id
            ));
        }
    }
    let mut test_relationship_ids = HashSet::new();
    for policy in &config.test_relationships {
        test_relationships::validate_test_relationship_config(policy)?;
        if !test_relationship_ids.insert(&policy.id) {
            return Err(format!(
                "extensions.test_relationships.{}: duplicate test relationship id",
                policy.id
            ));
        }
    }
    let mut module_topology_ids = HashSet::new();
    for policy in &config.module_topologies {
        module_topologies::validate_module_topology_config(policy)?;
        if !module_topology_ids.insert(&policy.id) {
            return Err(format!(
                "extensions.module_topologies.{}: duplicate module topology id",
                policy.id
            ));
        }
    }
    let mut docs_lifecycle_ids = HashSet::new();
    for policy in &config.docs_lifecycles {
        docs_lifecycles::validate_docs_lifecycle_config(policy)?;
        if !docs_lifecycle_ids.insert(&policy.id) {
            return Err(format!(
                "extensions.docs_lifecycles.{}: duplicate docs lifecycle id",
                policy.id
            ));
        }
    }
    let mut relationship_ids = HashSet::new();
    for relationship in &config.relationships {
        validate_relationship_constraint(relationship)?;
        if !relationship_ids.insert(&relationship.id) {
            return Err(format!(
                "extensions.relationships.{}: duplicate relationship id",
                relationship.id
            ));
        }
    }
    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_identifier(value: &str, context: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{context}: id must not be empty"));
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
    {
        return Err(format!(
            "{context}: expected lowercase ASCII letters, digits, '-' or '_'"
        ));
    }
    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_custom_constraint(constraint: &CustomConstraintConfig) -> Result<(), String> {
    let context = format!("extensions.custom_constraints.{}", constraint.id);
    if constraint.id.trim().is_empty() {
        return Err("extensions.custom_constraints: id must not be empty".to_string());
    }
    validate_identifier(&constraint.id, &format!("{context}.id"))?;
    match constraint.kind.as_str() {
        "paired_file_exists" => {
            validate_relative_pattern(&constraint.source, &format!("{context}.source"))?;
            validate_relative_template(&constraint.target, &format!("{context}.target"))?;
        }
        "command_surface_docs" => {
            validate_relative_pattern(&constraint.source, &format!("{context}.source"))?;
            validate_relative_path_text(&constraint.target, &format!("{context}.target"))?;
        }
        _ => {
            return Err(format!(
                "{context}.type: unsupported custom constraint {:?}",
                constraint.kind
            ));
        }
    }
    if let Some(severity) = &constraint.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_relationship_constraint(
    relationship: &RelationshipConstraintConfig,
) -> Result<(), String> {
    let context = format!("extensions.relationships.{}", relationship.id);
    validate_identifier(&relationship.id, &format!("{context}.id"))?;
    validate_identifier(&relationship.need, &format!("{context}.need"))?;
    validate_relative_path_text(&relationship.source, &format!("{context}.source"))?;
    if relationship
        .source_declaration
        .as_ref()
        .is_some_and(|source_declaration| source_declaration.trim().is_empty())
    {
        return Err(format!(
            "{context}.source_declaration: value must not be empty"
        ));
    }
    if relationship.providers.is_empty() {
        return Err(format!(
            "{context}.providers: at least one provider is required"
        ));
    }
    for provider in &relationship.providers {
        validate_relative_template(&provider.path, &format!("{context}.providers.path"))?;
        if provider
            .section
            .as_ref()
            .is_some_and(|section| section.trim().is_empty())
        {
            return Err(format!(
                "{context}.providers.section: value must not be empty"
            ));
        }
        if provider
            .kind
            .as_ref()
            .is_some_and(|kind| kind.trim().is_empty())
        {
            return Err(format!("{context}.providers.kind: value must not be empty"));
        }
        if provider
            .declaration
            .as_ref()
            .is_some_and(|declaration| declaration.trim().is_empty())
        {
            return Err(format!(
                "{context}.providers.declaration: value must not be empty"
            ));
        }
    }
    if let Some(severity) = &relationship.severity {
        validate_severity(severity).map_err(|error| format!("{context}.severity: {error}"))?;
    }
    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_relative_pattern(value: &str, context: &str) -> Result<(), String> {
    validate_relative_path_text(value, context)?;
    Pattern::new(value).map_err(|error| format!("{context}: invalid glob pattern: {error}"))?;
    Ok(())
}

#[cfg(feature = "yaml-config")]
fn validate_relative_template(value: &str, context: &str) -> Result<(), String> {
    validate_relative_path_text(value, context)
}

#[cfg(feature = "yaml-config")]
fn validate_relative_path_text(value: &str, context: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{context}: value must not be empty"));
    }
    let path = Path::new(value);
    if path.is_absolute()
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(format!(
            "{context}: value must be relative and must not contain '..'"
        ));
    }
    Ok(())
}

fn validate_severity(value: &str) -> Result<(), String> {
    match value {
        "critical" | "high" | "medium" | "low" => Ok(()),
        _ => Err("expected one of critical, high, medium, or low".to_string()),
    }
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
        "snakecase",
        "camelCase",
        "camelcase",
        "PascalCase",
        "pascalcase",
        "kebab-case",
        "kebabcase",
        "SCREAMING_SNAKE_CASE",
        "screamingsnakecase",
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
