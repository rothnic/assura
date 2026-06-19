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
    }
    Ok(())
}
