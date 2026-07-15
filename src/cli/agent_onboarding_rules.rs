//! Project-owned rule recommendations for agent onboarding.

use super::agent_onboarding::DetectedSection;
use super::agent_onboarding_report::RuleRecommendation;
use super::agent_onboarding_templates::{
    AGENTIC_PROJECT_PRESET, PROJECT_AGENTIC_BASELINE_REFERENCE, PROJECT_AGENTIC_BASELINE_RULE,
};
use serde_yaml::Value;
use std::fs;
use std::path::Path;

pub(super) fn recommended_rules(
    detected: &DetectedSection,
    config_path: &Path,
) -> Result<Vec<RuleRecommendation>, String> {
    let contents = fs::read_to_string(config_path).map_err(|error| error.to_string())?;
    let config: Value = serde_yaml::from_str(&contents).map_err(|error| error.to_string())?;
    let local_rule = &config["rules"][PROJECT_AGENTIC_BASELINE_RULE];
    let local_uses_preset = uses_rule(&local_rule["use"], AGENTIC_PROJECT_PRESET);
    let root_uses_local = uses_rule(
        &config["structure"]["./"]["use"],
        PROJECT_AGENTIC_BASELINE_REFERENCE,
    );
    let status = if local_uses_preset && root_uses_local {
        "applied"
    } else if local_uses_preset {
        "available"
    } else if local_rule.is_null() {
        "not-applied"
    } else {
        "conflict"
    };
    let reason = match status {
        "applied" => format!(
            "{} project detected; broad agent-ready preset applied through a project-owned wrapper",
            detected.project_type
        ),
        "available" => format!(
            "{} project detected; project-owned wrapper added without replacing the existing root rule",
            detected.project_type
        ),
        "conflict" => format!(
            "{} project detected; existing project-owned wrapper preserved for manual review",
            detected.project_type
        ),
        _ => format!(
            "{} project detected; selected config does not define the recommended project-owned wrapper",
            detected.project_type
        ),
    };

    Ok(vec![RuleRecommendation {
        preset: AGENTIC_PROJECT_PRESET,
        local_rule: PROJECT_AGENTIC_BASELINE_REFERENCE,
        status,
        reason,
        includes: vec![
            "$agents-dir",
            "$agent-skill-dir",
            "$agent-skill-file",
            "$agent-skill-resource-dir",
        ],
    }])
}

fn uses_rule(value: &Value, expected: &str) -> bool {
    match value {
        Value::String(reference) => reference == expected,
        Value::Sequence(references) => references
            .iter()
            .any(|reference| reference.as_str() == Some(expected)),
        _ => false,
    }
}

pub(super) fn preserve_existing_rule(existing: &Value, defaults: &mut Value) {
    let rule_key = Value::String(PROJECT_AGENTIC_BASELINE_RULE.to_string());
    let Some(existing_rules) = existing["rules"].as_mapping() else {
        return;
    };
    if !existing_rules.contains_key(&rule_key) {
        return;
    }
    if let Some(default_rules) = defaults["rules"].as_mapping_mut() {
        default_rules.remove(&rule_key);
    }
}
