//! Project-local SKILL.md contract checks.

use super::markdown::{frontmatter_mapping, markdown_section};
use super::routing::{local_reference_targets, ReferenceRoutingTable};
use crate::cli::check::rules::display_rel;
use crate::cli::check::{StructureCheckReport, StructureChecker};
use crate::config::config::AgentGuidanceConfig;
use std::path::Path;

impl StructureChecker {
    pub(super) fn validate_skill_frontmatter(
        &self,
        policy: &AgentGuidanceConfig,
        skill_rel: &Path,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        if policy.required_skill_frontmatter.is_empty() {
            return;
        }
        let Some(frontmatter) = frontmatter_mapping(content) else {
            self.push_agent_guidance_violation(
                report,
                policy,
                skill_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` SKILL.md `{}` is missing YAML frontmatter",
                    policy.id,
                    display_rel(skill_rel)
                ),
            );
            return;
        };
        for field in &policy.required_skill_frontmatter {
            if !frontmatter.contains_key(field.as_str()) {
                self.push_agent_guidance_violation(
                    report,
                    policy,
                    skill_rel.to_path_buf(),
                    format!(
                        "Agent guidance `{}` SKILL.md `{}` is missing frontmatter field `{field}`",
                        policy.id,
                        display_rel(skill_rel)
                    ),
                );
            }
        }
    }

    pub(super) fn validate_skill_reference_sections(
        &self,
        policy: &AgentGuidanceConfig,
        skill_rel: &Path,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        for section in &policy.skill_reference_sections {
            let Some(section_content) = markdown_section(content, section) else {
                continue;
            };
            if section_content.trim().is_empty()
                && policy.skill_doc_routing_section.as_deref() == Some(section.as_str())
            {
                continue;
            }
            let has_reference = local_reference_targets(section_content)
                .iter()
                .any(|target| {
                    policy
                        .skill_reference_prefixes
                        .iter()
                        .any(|prefix| target.starts_with(prefix))
                });
            if has_reference {
                continue;
            }
            self.push_agent_guidance_violation(
                report,
                policy,
                skill_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` SKILL.md `{}` section `{section}` must reference supporting docs or assets using one of these prefixes: {}; keep SKILL.md as a concise index and move detail into references/, scripts/, assets/, or process docs",
                    policy.id,
                    display_rel(skill_rel),
                    policy.skill_reference_prefixes.join(", ")
                ),
            );
        }
    }

    pub(super) fn validate_skill_doc_routing_section(
        &self,
        policy: &AgentGuidanceConfig,
        skill_rel: &Path,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        let Some(section) = policy.skill_doc_routing_section.as_deref() else {
            return;
        };
        let Some(section_content) = markdown_section(content, section) else {
            return;
        };
        if section_content.trim().is_empty() {
            return;
        }
        let Some(table) = ReferenceRoutingTable::parse(section_content) else {
            self.push_agent_guidance_violation(
                report,
                policy,
                skill_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` SKILL.md `{}` section `{section}` must use a Markdown table with use-case and local reference columns; leave it empty if no doc routing rules are known yet",
                    policy.id,
                    display_rel(skill_rel)
                ),
            );
            return;
        };

        let references = table.reference_targets();
        let has_allowed_reference = references.iter().any(|target| {
            policy
                .skill_reference_prefixes
                .iter()
                .any(|prefix| target.starts_with(prefix))
        });
        if !has_allowed_reference {
            self.push_agent_guidance_violation(
                report,
                policy,
                skill_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` SKILL.md `{}` section `{section}` must route at least one use case to supporting docs or assets using one of these prefixes: {}",
                    policy.id,
                    display_rel(skill_rel),
                    policy.skill_reference_prefixes.join(", ")
                ),
            );
        }
        for target in references {
            if policy
                .skill_reference_prefixes
                .iter()
                .any(|prefix| target.starts_with(prefix))
            {
                continue;
            }
            self.push_agent_guidance_violation(
                report,
                policy,
                skill_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` SKILL.md `{}` section `{section}` routes to unsupported local reference `{target}`; use configured prefixes: {}",
                    policy.id,
                    display_rel(skill_rel),
                    policy.skill_reference_prefixes.join(", ")
                ),
            );
        }
    }
}
