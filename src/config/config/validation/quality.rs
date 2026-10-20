//! Quality-scope semantic validation.

use super::{validate_identifier, validate_relative_pattern};
use crate::config::config::QualityConfig;
use std::collections::HashSet;

pub(super) fn validate_quality_config(config: &QualityConfig) -> Result<(), String> {
    let mut ids = HashSet::new();
    for (id, scope) in &config.scopes {
        let context = format!("quality.scopes.{id}");
        validate_identifier(id, &context)?;
        if !ids.insert(id) {
            return Err(format!("{context}: duplicate quality scope id"));
        }
        if scope.paths.is_empty() {
            return Err(format!(
                "{context}.paths: at least one path pattern is required"
            ));
        }
        for pattern in &scope.paths {
            validate_relative_pattern(pattern, &format!("{context}.paths"))?;
        }
        for pattern in &scope.exclude_paths {
            validate_relative_pattern(pattern, &format!("{context}.exclude_paths"))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config::QualityScopeConfig;

    #[test]
    fn invalid_exclusion_patterns_report_the_scope_field() {
        let mut config = QualityConfig::default();
        config.scopes.insert(
            "bun".to_string(),
            QualityScopeConfig {
                paths: vec!["app/**".to_string()],
                exclude_paths: vec!["[".to_string()],
                ..QualityScopeConfig::default()
            },
        );

        let error = validate_quality_config(&config).unwrap_err();
        assert!(
            error.contains("quality.scopes.bun.exclude_paths"),
            "{error}"
        );
    }
}

#[cfg(test)]
mod traversal_tests {
    use super::*;
    use crate::config::config::QualityScopeConfig;

    #[test]
    fn parent_traversal_exclusions_report_the_scope_field() {
        let mut config = QualityConfig::default();
        config.scopes.insert(
            "bun".to_string(),
            QualityScopeConfig {
                paths: vec!["app/**".to_string()],
                exclude_paths: vec!["../outside/**".to_string()],
                ..QualityScopeConfig::default()
            },
        );

        let error = validate_quality_config(&config).unwrap_err();
        assert!(
            error.contains("quality.scopes.bun.exclude_paths"),
            "{error}"
        );
    }
}
