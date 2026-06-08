//! Quality gate policy configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[cfg(feature = "full-cli")]
use validator::Validate;

/// High-level quality gate policy grouped by changed-file scope.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full-cli", derive(Validate))]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct QualityConfig {
    /// Named quality scopes keyed by stable scope id.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[cfg_attr(feature = "full-cli", validate(nested))]
    pub scopes: HashMap<String, QualityScopeConfig>,
}

/// A changed-file scope and the checks it requires by workflow phase.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full-cli", derive(Validate))]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub struct QualityScopeConfig {
    /// Relative path patterns that activate this scope.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub paths: Vec<String>,

    /// Checks included for every non-empty plan for this scope.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub always: Vec<String>,

    /// Checks intended for frequent local loops.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub frequent: Vec<String>,

    /// Checks intended before pushing a branch.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_push: Vec<String>,

    /// Checks intended before opening or updating a pull request.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pr: Vec<String>,

    /// Checks intended immediately before merging.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub merge: Vec<String>,

    /// Checks intended for release branches or release tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub release: Vec<String>,

    /// Checks intended for scheduled or background validation.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scheduled: Vec<String>,
}

impl QualityScopeConfig {
    /// Return checks required by a cumulative workflow phase.
    pub fn checks_for_phase(&self, phase: &str) -> Vec<String> {
        let mut checks = Vec::new();
        append_unique(&mut checks, &self.always);

        match phase {
            "frequent" => {
                append_unique(&mut checks, &self.frequent);
            }
            "pre_push" => {
                append_unique(&mut checks, &self.frequent);
                append_unique(&mut checks, &self.pre_push);
            }
            "pr" => {
                append_unique(&mut checks, &self.frequent);
                append_unique(&mut checks, &self.pre_push);
                append_unique(&mut checks, &self.pr);
            }
            "merge" => {
                append_unique(&mut checks, &self.frequent);
                append_unique(&mut checks, &self.pre_push);
                append_unique(&mut checks, &self.pr);
                append_unique(&mut checks, &self.merge);
            }
            "release" => {
                append_unique(&mut checks, &self.frequent);
                append_unique(&mut checks, &self.pre_push);
                append_unique(&mut checks, &self.pr);
                append_unique(&mut checks, &self.merge);
                append_unique(&mut checks, &self.release);
            }
            "scheduled" => {
                append_unique(&mut checks, &self.scheduled);
            }
            _ => {}
        }

        checks
    }
}

fn append_unique(target: &mut Vec<String>, source: &[String]) {
    for value in source {
        if !target.contains(value) {
            target.push(value.clone());
        }
    }
}
