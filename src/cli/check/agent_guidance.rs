//! Agent guidance and project-local skill contract validation.

mod markdown;
mod paths;

use super::rules::{display_rel, is_excluded_rel_with};
use super::{CheckError, StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::AgentGuidanceConfig;
use markdown::{
    frontmatter_mapping, heading_anchor, markdown_heading_sequence, markdown_heading_texts,
    markdown_links, markdown_section, path_to_slash,
};
use paths::{
    checked_path_relevant_to, checked_path_relevant_to_any, compile_agent_guidance_patterns,
    pattern_matches_any, safe_agent_guidance_path,
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
        self.validate_line_limit(
            policy,
            agents_rel,
            &content,
            AgentGuidanceLineLimit {
                limit: policy.max_agents_lines,
                label: "AGENTS.md",
                hint: "move operational detail into docs/process/ or project-local skills",
            },
            report,
        );
        self.validate_skill_index(policy, agents_rel, &content, skill_files, report)?;
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
                hint: "move long examples and runbooks into references/ or docs/process/",
            },
            report,
        );
        Ok(())
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

    fn validate_skill_frontmatter(
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

    fn push_agent_guidance_violation(
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
