//! Agent guidance and project-local skill contract validation.

mod markdown;
mod paths;
mod routing;
mod skill;

use super::rules::{display_rel, is_excluded_rel_with};
use super::{CheckError, StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::AgentGuidanceConfig;
use markdown::{
    heading_anchor, markdown_heading_sequence, markdown_heading_texts, markdown_links,
    markdown_section, path_to_slash,
};
use paths::{
    checked_path_relevant_to, checked_path_relevant_to_any, compile_agent_guidance_patterns,
    pattern_matches_any, safe_agent_guidance_path,
};
use routing::{
    compile_skill_name_patterns, skill_name_from_path, skill_reference_allowed, SkillRoutingTable,
};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

struct AgentGuidanceLineLimit<'a> {
    limit: Option<usize>,
    label: &'a str,
    hint: &'a str,
}

impl StructureChecker {
    pub(super) fn validate_agent_guidance(
        &self,
        policies: &[AgentGuidanceConfig],
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for policy in policies {
            self.validate_agent_guidance_policy(policy, checked_path, report)?;
        }
        Ok(())
    }

    fn validate_agent_guidance_policy(
        &self,
        policy: &AgentGuidanceConfig,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let agents_rel = safe_agent_guidance_path(&policy.agents_path)?;
        let agents_abs = self.project_root.join(&agents_rel);
        let skill_files = self.matching_agent_guidance_skill_files(policy, &self.project_root)?;
        let skill_check_relevant =
            checked_path_relevant_to_any(checked_path, &self.project_root, &skill_files);
        let agents_check_relevant = checked_path_relevant_to(checked_path, &agents_abs);
        let project_check = checked_path == self.project_root;

        if project_check || agents_check_relevant || skill_check_relevant {
            self.validate_agents_guidance_file(policy, &agents_rel, &skill_files, report)?;
        }

        if project_check || skill_check_relevant {
            for skill_rel in skill_files {
                self.validate_skill_contract_file(policy, &skill_rel, report)?;
            }
        }

        Ok(())
    }

    fn matching_agent_guidance_skill_files(
        &self,
        policy: &AgentGuidanceConfig,
        checked_path: &Path,
    ) -> Result<Vec<PathBuf>, CheckError> {
        if policy.skill_paths.is_empty() {
            return Ok(Vec::new());
        }
        let compiled = compile_agent_guidance_patterns(&policy.skill_paths)?;
        let project_root = self.project_root.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let walker = walkdir::WalkDir::new(checked_path)
            .into_iter()
            .filter_entry(move |entry| {
                let path = entry.path();
                if path == checked_path {
                    return true;
                }
                let rel = path.strip_prefix(&project_root).unwrap_or(path);
                !is_excluded_rel_with(&exclude_patterns, rel)
            });

        let mut matches = Vec::new();
        for entry in walker {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = self.relative_path(entry.path());
            if self.is_excluded_rel(&rel) {
                continue;
            }
            if pattern_matches_any(&compiled, &rel) {
                matches.push(rel);
            }
        }
        matches.sort();
        matches.dedup();
        Ok(matches)
    }

    fn validate_agents_guidance_file(
        &self,
        policy: &AgentGuidanceConfig,
        agents_rel: &Path,
        skill_files: &[PathBuf],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let agents_path = self.project_root.join(agents_rel);
        if !agents_path.exists() {
            self.push_agent_guidance_violation(
                report,
                policy,
                agents_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` requires `{}` to exist",
                    policy.id,
                    display_rel(agents_rel)
                ),
            );
            return Ok(());
        }

        let content = fs::read_to_string(agents_path)?;
        self.validate_required_markdown_sections(
            policy,
            agents_rel,
            &content,
            &policy.required_agents_sections,
            "AGENTS.md",
            report,
        );
        self.validate_duplicate_heading_anchors(policy, agents_rel, &content, report);
        self.validate_best_practices_reference(policy, agents_rel, &content, report);
        self.validate_line_limit(
            policy,
            agents_rel,
            &content,
            AgentGuidanceLineLimit {
                limit: policy.max_agents_lines,
                label: "AGENTS.md",
                hint: "keep AGENTS.md as a use-case router with stable sections and the skill-loading table; move operational detail into docs/process/ or project-local SKILL.md entrypoints",
            },
            report,
        );
        self.validate_skill_index(policy, agents_rel, &content, skill_files, report)?;
        self.validate_skill_routing_section(policy, agents_rel, &content, skill_files, report)?;
        Ok(())
    }

    fn validate_skill_contract_file(
        &self,
        policy: &AgentGuidanceConfig,
        skill_rel: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let content = fs::read_to_string(self.project_root.join(skill_rel))?;
        self.validate_skill_frontmatter(policy, skill_rel, &content, report);
        self.validate_required_markdown_sections(
            policy,
            skill_rel,
            &content,
            &policy.required_skill_sections,
            "SKILL.md",
            report,
        );
        self.validate_line_limit(
            policy,
            skill_rel,
            &content,
            AgentGuidanceLineLimit {
                limit: policy.max_skill_lines,
                label: "SKILL.md",
                hint: "keep SKILL.md as frontmatter plus a concise workflow index; move long examples and runbooks into references/, scripts/, assets/, or docs/process/ and link them from the configured reference section",
            },
            report,
        );
        self.validate_skill_reference_sections(policy, skill_rel, &content, report);
        self.validate_skill_doc_routing_section(policy, skill_rel, &content, report);
        Ok(())
    }

    fn validate_best_practices_reference(
        &self,
        policy: &AgentGuidanceConfig,
        agents_rel: &Path,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        let Some(reference) = policy.best_practices_reference.as_deref() else {
            return;
        };
        if contains_normalized_text(content, reference) {
            return;
        }
        self.push_agent_guidance_violation(
            report,
            policy,
            agents_rel.to_path_buf(),
            format!(
                "Agent guidance `{}` AGENTS.md `{}` must reference `{reference}` so agents can find the progressive-disclosure guidance",
                policy.id,
                display_rel(agents_rel)
            ),
        );
    }

    fn validate_required_markdown_sections(
        &self,
        policy: &AgentGuidanceConfig,
        rel: &Path,
        content: &str,
        required_sections: &[String],
        label: &str,
        report: &mut StructureCheckReport,
    ) {
        if required_sections.is_empty() {
            return;
        }
        let headings = markdown_heading_texts(content);
        for section in required_sections {
            if !headings.contains(section.as_str()) {
                self.push_agent_guidance_violation(
                    report,
                    policy,
                    rel.to_path_buf(),
                    format!(
                        "Agent guidance `{}` {label} `{}` is missing required section `{section}`",
                        policy.id,
                        display_rel(rel)
                    ),
                );
            }
        }
    }

    fn validate_duplicate_heading_anchors(
        &self,
        policy: &AgentGuidanceConfig,
        rel: &Path,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        let mut anchors = HashSet::new();
        for heading in markdown_heading_sequence(content) {
            let anchor = heading_anchor(heading);
            if anchor.is_empty() || anchors.insert(anchor.clone()) {
                continue;
            }
            self.push_agent_guidance_violation(
                report,
                policy,
                rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` `{}` has duplicate heading anchor `{anchor}`; rename headings so agent links stay stable",
                    policy.id,
                    display_rel(rel)
                ),
            );
        }
    }

    fn validate_line_limit(
        &self,
        policy: &AgentGuidanceConfig,
        rel: &Path,
        content: &str,
        line_limit: AgentGuidanceLineLimit<'_>,
        report: &mut StructureCheckReport,
    ) {
        let AgentGuidanceLineLimit { limit, label, hint } = line_limit;
        let Some(limit) = limit else {
            return;
        };
        let lines = content.lines().count();
        if lines <= limit {
            return;
        }
        self.push_agent_guidance_violation(
            report,
            policy,
            rel.to_path_buf(),
            format!(
                "Agent guidance `{}` {label} `{}` has {lines} lines, exceeding limit {limit}; {hint}",
                policy.id,
                display_rel(rel)
            ),
        );
    }

    fn validate_skill_index(
        &self,
        policy: &AgentGuidanceConfig,
        agents_rel: &Path,
        content: &str,
        skill_files: &[PathBuf],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let Some(section) = policy.skill_index_section.as_deref() else {
            return Ok(());
        };
        let Some(section_content) = markdown_section(content, section) else {
            return Ok(());
        };

        let expected = skill_files
            .iter()
            .map(|path| path_to_slash(path))
            .collect::<HashSet<_>>();
        let links = markdown_links(section_content)
            .into_iter()
            .filter(|target| target.contains(".agents/skills/"))
            .collect::<Vec<_>>();
        if !expected.is_empty() && links.is_empty() {
            self.push_agent_guidance_violation(
                report,
                policy,
                agents_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` AGENTS.md Skills section must link to project-local skill entrypoints",
                    policy.id
                ),
            );
        }

        let linked = links
            .iter()
            .filter_map(|target| normalize_project_link(target))
            .collect::<HashSet<_>>();
        for target in &linked {
            let target_rel = safe_agent_guidance_path(target)?;
            if !self.project_root.join(&target_rel).exists() {
                self.push_agent_guidance_violation(
                    report,
                    policy,
                    agents_rel.to_path_buf(),
                    format!(
                        "Agent guidance `{}` AGENTS.md Skills section links to missing project-local skill `{}`",
                        policy.id, target
                    ),
                );
            }
        }
        for skill in expected {
            if !linked.contains(&skill) {
                self.push_agent_guidance_violation(
                    report,
                    policy,
                    agents_rel.to_path_buf(),
                    format!(
                        "Agent guidance `{}` AGENTS.md Skills section does not link to project-local skill `{skill}`",
                        policy.id
                    ),
                );
            }
        }
        Ok(())
    }

    fn validate_skill_routing_section(
        &self,
        policy: &AgentGuidanceConfig,
        agents_rel: &Path,
        content: &str,
        skill_files: &[PathBuf],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let Some(section) = policy.skill_routing_section.as_deref() else {
            return Ok(());
        };
        let Some(section_content) = markdown_section(content, section) else {
            return Ok(());
        };
        if section_content.trim().is_empty() {
            return Ok(());
        }
        let Some(table) = SkillRoutingTable::parse(section_content) else {
            self.push_agent_guidance_violation(
                report,
                policy,
                agents_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` AGENTS.md `{}` section `{section}` must use a Markdown table with use-case and skill-loading columns; leave it empty if no routing rules are known yet",
                    policy.id,
                    display_rel(agents_rel)
                ),
            );
            return Ok(());
        };
        let skill_names = skill_files
            .iter()
            .filter_map(|path| skill_name_from_path(path))
            .collect::<HashSet<_>>();
        let allowed_patterns = compile_skill_name_patterns(&policy.allowed_skill_name_patterns)?;
        for skill_ref in table.skill_references() {
            if skill_reference_allowed(&skill_ref, &skill_names, &allowed_patterns) {
                continue;
            }
            self.push_agent_guidance_violation(
                report,
                policy,
                agents_rel.to_path_buf(),
                format!(
                    "Agent guidance `{}` AGENTS.md `{}` section `{section}` routes to unknown skill or pattern `{skill_ref}`; use an existing skill name or a configured allowed skill-name pattern",
                    policy.id,
                    display_rel(agents_rel)
                ),
            );
        }
        Ok(())
    }

    pub(super) fn push_agent_guidance_violation(
        &self,
        report: &mut StructureCheckReport,
        policy: &AgentGuidanceConfig,
        path: PathBuf,
        message: String,
    ) {
        report.violations.push(StructureViolation::new(
            path,
            format!("agent_guidance:{}", policy.id),
            message,
            policy.severity.as_deref().unwrap_or("low"),
        ));
    }
}

fn normalize_project_link(target: &str) -> Option<String> {
    let without_fragment = target.split('#').next()?.trim();
    let normalized = without_fragment
        .strip_prefix("./")
        .unwrap_or(without_fragment);
    normalized
        .starts_with(".agents/skills/")
        .then(|| normalized.to_string())
}

fn contains_normalized_text(content: &str, needle: &str) -> bool {
    let normalized_content = content.split_whitespace().collect::<Vec<_>>().join(" ");
    let normalized_needle = needle.split_whitespace().collect::<Vec<_>>().join(" ");
    normalized_content.contains(&normalized_needle)
}
