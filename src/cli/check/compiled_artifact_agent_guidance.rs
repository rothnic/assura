use crate::config::config::AgentGuidanceConfig;

/// Binary-safe agent guidance policy stored in compiled artifacts.
#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableAgentGuidanceConfig {
    id: String,
    agents_path: String,
    skill_paths: Vec<String>,
    required_agents_sections: Vec<String>,
    required_skill_frontmatter: Vec<String>,
    required_skill_sections: Vec<String>,
    skill_index_section: Option<String>,
    best_practices_reference: Option<String>,
    skill_routing_section: Option<String>,
    #[serde(default)]
    allowed_skill_name_patterns: Vec<String>,
    #[serde(default)]
    skill_reference_sections: Vec<String>,
    #[serde(default)]
    skill_reference_prefixes: Vec<String>,
    max_agents_lines: Option<usize>,
    max_skill_lines: Option<usize>,
    severity: Option<String>,
}

impl From<AgentGuidanceConfig> for PortableAgentGuidanceConfig {
    fn from(config: AgentGuidanceConfig) -> Self {
        Self {
            id: config.id,
            agents_path: config.agents_path,
            skill_paths: config.skill_paths,
            required_agents_sections: config.required_agents_sections,
            required_skill_frontmatter: config.required_skill_frontmatter,
            required_skill_sections: config.required_skill_sections,
            skill_index_section: config.skill_index_section,
            best_practices_reference: config.best_practices_reference,
            skill_routing_section: config.skill_routing_section,
            allowed_skill_name_patterns: config.allowed_skill_name_patterns,
            skill_reference_sections: config.skill_reference_sections,
            skill_reference_prefixes: config.skill_reference_prefixes,
            max_agents_lines: config.max_agents_lines,
            max_skill_lines: config.max_skill_lines,
            severity: config.severity,
        }
    }
}

impl From<PortableAgentGuidanceConfig> for AgentGuidanceConfig {
    fn from(config: PortableAgentGuidanceConfig) -> Self {
        Self {
            id: config.id,
            agents_path: config.agents_path,
            skill_paths: config.skill_paths,
            required_agents_sections: config.required_agents_sections,
            required_skill_frontmatter: config.required_skill_frontmatter,
            required_skill_sections: config.required_skill_sections,
            skill_index_section: config.skill_index_section,
            best_practices_reference: config.best_practices_reference,
            skill_routing_section: config.skill_routing_section,
            allowed_skill_name_patterns: config.allowed_skill_name_patterns,
            skill_reference_sections: config.skill_reference_sections,
            skill_reference_prefixes: config.skill_reference_prefixes,
            max_agents_lines: config.max_agents_lines,
            max_skill_lines: config.max_skill_lines,
            severity: config.severity,
        }
    }
}
