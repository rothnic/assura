//! Project-owned rule recommendations for agent onboarding.

use super::agent_onboarding::DetectedSection;
use super::agent_onboarding_report::RuleRecommendation;
use super::agent_onboarding_templates::{AGENTIC_RECIPE, AGENT_ENTRYPOINT_REFERENCE};
use serde_yaml::Value;
use std::fs;
use std::path::Path;

pub(super) fn recommended_rules(
    detected: &DetectedSection,
    config_path: &Path,
) -> Result<Vec<RuleRecommendation>, String> {
    let contents = fs::read_to_string(config_path).map_err(|error| error.to_string())?;
    let config: Value = serde_yaml::from_str(&contents).map_err(|error| error.to_string())?;
    let expected_rules = [
        "agent-entrypoint",
        "skill-entrypoint",
        "folder-health",
        "closed-entry",
        "closed",
        "skill",
    ];
    let present = expected_rules
        .iter()
        .filter(|name| !config["rules"][**name].is_null())
        .count();
    let root_applies_entrypoint = config["structure"]["AGENTS.md"]
        .as_str()
        .is_some_and(|value| {
            value
                .split(" | ")
                .any(|token| token.trim() == AGENT_ENTRYPOINT_REFERENCE)
        });
    let entrypoint_conflicts = config["rules"]["agent-entrypoint"]["max_lines"]
        .as_u64()
        .is_some_and(|limit| limit != 160);
    let status = if entrypoint_conflicts {
        "conflict"
    } else if present == expected_rules.len() && root_applies_entrypoint {
        "applied"
    } else if present > 0 {
        "available"
    } else {
        "not-applied"
    };
    let reason = match status {
        "applied" => format!(
            "{} project detected; editable agentic-core and structure-health policy is active",
            detected.project_type
        ),
        "available" => format!(
            "{} project detected; project-owned recipe rules are available without replacing the selected root policy",
            detected.project_type
        ),
        "conflict" => format!(
            "{} project detected; existing project-owned recipe values were preserved for manual review",
            detected.project_type
        ),
        _ => format!(
            "{} project detected; selected config does not define the recommended project-owned recipes",
            detected.project_type
        ),
    };

    Ok(vec![RuleRecommendation {
        preset: AGENTIC_RECIPE,
        local_rule: AGENT_ENTRYPOINT_REFERENCE,
        status,
        reason,
        includes: vec![
            "$agent-entrypoint",
            "$skill-entrypoint",
            "$skill",
            "$folder-health",
            "$closed",
        ],
    }])
}

pub(super) fn normalize_existing_root(existing: &mut Value) {
    let Some(structure) = existing["structure"].as_mapping_mut() else {
        return;
    };
    let root_key = Value::String("./".to_string());
    let Some(Value::Mapping(root)) = structure.remove(&root_key) else {
        return;
    };
    for (key, value) in root {
        structure.entry(key).or_insert(value);
    }
}
