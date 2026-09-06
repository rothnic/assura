//! Deterministic specialization evidence for agent onboarding.

use super::agent_onboarding::DetectedSection;
use super::agent_onboarding_report::{CheckItem, RuleRecommendation};
use super::AgentContentTemplate;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

/// Write the evidence record describing the selected onboarding specialization.
pub(super) fn write_specialization_profile(
    project_root: &Path,
    detected: &DetectedSection,
    recipe_file: Option<&Path>,
    verified: &[CheckItem],
    rule_recommendations: &[RuleRecommendation],
) -> Result<(), String> {
    let (profile, source, source_path, stack) = match recipe_file {
        Some(recipe_file) => (
            "local-policy",
            recipe_file.display().to_string(),
            recipe_file.to_path_buf(),
            detected.project_type,
        ),
        None => match detected.project_type {
            "rust" => (
                "rust-library",
                "Cargo.toml".to_string(),
                project_root.join("Cargo.toml"),
                "rust",
            ),
            "node" => (
                "typescript-bun-utility",
                "package.json".to_string(),
                project_root.join("package.json"),
                "node",
            ),
            "python" => (
                "python-pytest",
                "pyproject.toml".to_string(),
                project_root.join("pyproject.toml"),
                "python",
            ),
            _ => (
                "repository-default",
                "repository inspection".to_string(),
                project_root.join(".assura/config.yml"),
                detected.project_type,
            ),
        },
    };
    let source_hash = fs::read(&source_path)
        .map(|contents| format!("{:x}", Sha256::digest(contents)))
        .unwrap_or_else(|_| format!("{:x}", Sha256::digest(source.as_bytes())));
    let config_status = verified
        .iter()
        .find(|item| item.name == "structure_config")
        .map(|item| item.status)
        .unwrap_or("fail");
    let mut conflicts = detected
        .manifest_conflicts
        .iter()
        .map(|source| {
            serde_json::json!({
                "kind": "manifest",
                "source": source,
                "detail": "multiple stack manifests require user specialization authority",
            })
        })
        .collect::<Vec<_>>();
    conflicts.extend(
        rule_recommendations
            .iter()
            .filter(|rule| rule.status == "conflict")
            .map(|rule| {
                serde_json::json!({
                    "kind": "recommended_rule",
                    "source": rule.preset,
                    "detail": rule.reason,
                })
            }),
    );
    let profile = serde_json::json!({
        "schema": "assura.profile-selection.v1",
        "profile": profile,
        "source": source,
        "source_hash": source_hash,
        "decisions": [{"key": "stack", "value": stack, "evidence": source}],
        "conflicts": conflicts,
        "verification": {"config": config_status},
    });
    validate_specialization_profile(&profile)?;
    let directory = project_root.join(".assura/onboarding");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    fs::write(
        directory.join("profile-selection.json"),
        serde_json::to_vec_pretty(&profile).expect("specialization profile is serializable"),
    )
    .map_err(|error| error.to_string())
}

fn validate_specialization_profile(profile: &serde_json::Value) -> Result<(), String> {
    if profile.get("schema").and_then(serde_json::Value::as_str)
        != Some("assura.profile-selection.v1")
    {
        return Err(
            "specialization profile schema must be assura.profile-selection.v1".to_string(),
        );
    }
    for field in ["profile", "source", "source_hash"] {
        if profile
            .get(field)
            .and_then(serde_json::Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(format!(
                "specialization profile {field} must be a non-empty string"
            ));
        }
    }
    let decisions = profile
        .get("decisions")
        .and_then(serde_json::Value::as_array)
        .filter(|items| !items.is_empty())
        .ok_or_else(|| "specialization profile decisions must be a non-empty array".to_string())?;
    for decision in decisions {
        for field in ["key", "value", "evidence"] {
            if decision
                .get(field)
                .and_then(serde_json::Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Err(format!(
                    "specialization decision {field} must be a non-empty string"
                ));
            }
        }
    }
    let conflicts = profile
        .get("conflicts")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| "specialization profile conflicts must be an array".to_string())?;
    for conflict in conflicts {
        for field in ["kind", "source", "detail"] {
            if conflict
                .get(field)
                .and_then(serde_json::Value::as_str)
                .is_none_or(str::is_empty)
            {
                return Err(format!(
                    "specialization conflict {field} must be a non-empty string"
                ));
            }
        }
    }
    match profile
        .pointer("/verification/config")
        .and_then(serde_json::Value::as_str)
    {
        Some("pass" | "fail") => Ok(()),
        _ => Err("specialization profile verification.config must be pass or fail".to_string()),
    }
}

/// Report specialization honestly until an agent proves its selected policy.
pub(super) fn inactive_capabilities(
    detected: &DetectedSection,
    template: AgentContentTemplate,
    has_conflict: bool,
) -> Vec<CheckItem> {
    let mut items = vec![CheckItem {
        name: "project_specialization",
        status: if has_conflict {
            "conflict_requires_user"
        } else if matches!(detected.project_type, "rust" | "node" | "python") {
            "configured_unverified"
        } else {
            "needs_agent_specialization"
        },
        detail: if has_conflict {
            "a project-owned policy conflicts with the recommended rule; preserve it and request user authority"
        } else if matches!(detected.project_type, "rust" | "node" | "python") {
            "repository evidence selected a profile and the materialized config passed; prove a negative policy case before verification"
        } else {
            "specialize from repository evidence in .assura/onboarding/agent-next.md"
        },
    }];
    if !template.activates_content() {
        items.push(CheckItem {
            name: "content_models",
            status: "inactive",
            detail: "deferred until --content-template is selected",
        });
    }
    items
}

#[cfg(test)]
mod tests {
    use super::validate_specialization_profile;

    #[test]
    fn rejects_a_profile_without_evidence_for_each_decision() {
        let profile = serde_json::json!({
            "schema": "assura.profile-selection.v1",
            "profile": "rust-library",
            "source": "Cargo.toml",
            "source_hash": "abc",
            "decisions": [{"key": "stack", "value": "rust"}],
            "conflicts": [],
            "verification": {"config": "pass"}
        });

        assert!(validate_specialization_profile(&profile).is_err());
    }

    #[test]
    fn rejects_a_conflict_without_its_source_or_detail() {
        let profile = serde_json::json!({
            "schema": "assura.profile-selection.v1",
            "profile": "ambiguous",
            "source": "repository inspection",
            "source_hash": "abc",
            "decisions": [{"key": "stack", "value": "ambiguous", "evidence": "repository inspection"}],
            "conflicts": [{"kind": "manifest"}],
            "verification": {"config": "pass"}
        });

        assert!(validate_specialization_profile(&profile).is_err());
    }
}
