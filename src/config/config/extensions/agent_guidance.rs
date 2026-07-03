use serde::{Deserialize, Serialize};

/// A reusable contract for project-local agent guidance and skill routing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AgentGuidanceConfig {
    /// Stable local identifier used in diagnostics.
    pub id: String,
    /// Project-local AGENTS.md path.
    pub agents_path: String,
    /// Project-local SKILL.md glob patterns.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skill_paths: Vec<String>,
    /// Required AGENTS.md heading texts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_agents_sections: Vec<String>,
    /// Required SKILL.md frontmatter fields.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_skill_frontmatter: Vec<String>,
    /// Required SKILL.md heading texts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_skill_sections: Vec<String>,
    /// AGENTS.md section that should contain project-local skill links.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_index_section: Option<String>,
    /// Advisory maximum line count for AGENTS.md.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_agents_lines: Option<usize>,
    /// Advisory maximum line count for SKILL.md entrypoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_skill_lines: Option<usize>,
    /// Optional diagnostic severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub severity: Option<String>,
}
