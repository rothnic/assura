//! Docs lifecycle and stale-claim policy validation.

use super::rules::{display_rel, is_excluded_rel_with};
use super::{CheckError, StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::DocsLifecycleConfig;
use crate::markdown_links::{markdown_links, parse_markdown_link_target};
use glob::Pattern;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

impl StructureChecker {
    pub(super) fn validate_docs_lifecycles(
        &self,
        policies: &[DocsLifecycleConfig],
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for policy in policies {
            self.validate_docs_lifecycle_policy(policy, checked_path, report)?;
        }
        Ok(())
    }

    fn validate_docs_lifecycle_policy(
        &self,
        policy: &DocsLifecycleConfig,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let active_docs = self.matching_docs_lifecycle_files(&policy.active, checked_path)?;
        let status_docs =
            self.matching_docs_lifecycle_files(&policy.require_frontmatter_status, checked_path)?;
        let historical = compile_patterns(&policy.historical)?;
        let exceptions = compile_patterns(&policy.historical_exceptions)?;
        let allowed_statuses = policy
            .allowed_statuses
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();

        for rel in status_docs {
            let content = fs::read_to_string(self.project_root.join(&rel))?;
            match frontmatter_status(&content) {
                Some(status) if allowed_statuses.contains(status.as_str()) => {}
                Some(status) => self.push_docs_lifecycle_violation(
                    report,
                    policy,
                    rel,
                    format!(
                        "Docs lifecycle `{}` found unsupported status `{status}`; expected one of {}",
                        policy.id,
                        policy.allowed_statuses.join(", ")
                    ),
                ),
                None => self.push_docs_lifecycle_violation(
                    report,
                    policy,
                    rel.clone(),
                    format!(
                        "Docs lifecycle `{}` requires frontmatter status for `{}`",
                        policy.id,
                        display_rel(&rel)
                    ),
                ),
            }
        }

        for rel in active_docs {
            let content = fs::read_to_string(self.project_root.join(&rel))?;
            let source_has_historical_exception = pattern_matches_any(&exceptions, &rel);
            self.validate_historical_links(
                policy,
                &historical,
                &exceptions,
                &rel,
                &content,
                report,
            );
            if !source_has_historical_exception {
                self.validate_claim_patterns(policy, &rel, &content, report)?;
            }
        }
        Ok(())
    }

    fn validate_historical_links(
        &self,
        policy: &DocsLifecycleConfig,
        historical: &[CompiledPattern],
        exceptions: &[CompiledPattern],
        source_rel: &Path,
        content: &str,
        report: &mut StructureCheckReport,
    ) {
        if historical.is_empty() {
            return;
        }
        for target_rel in markdown_link_targets(source_rel, content) {
            if pattern_matches_any(historical, &target_rel)
                && !pattern_matches_any(exceptions, &target_rel)
            {
                self.push_docs_lifecycle_violation(
                    report,
                    policy,
                    source_rel.to_path_buf(),
                    format!(
                        "Docs lifecycle `{}` active doc `{}` links to historical doc `{}` without a configured exception",
                        policy.id,
                        display_rel(source_rel),
                        display_rel(&target_rel)
                    ),
                );
            }
        }
    }

    fn validate_claim_patterns(
        &self,
        policy: &DocsLifecycleConfig,
        source_rel: &Path,
        content: &str,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for claim in &policy.claim_patterns {
            if !text_matches_claim_pattern(content, &claim.pattern) {
                continue;
            }
            let mut evidence_found = false;
            let mut missing_evidence = Vec::new();
            for evidence_file in &claim.evidence_files {
                let evidence_rel = safe_docs_lifecycle_path(evidence_file)?;
                let evidence_path = self.project_root.join(&evidence_rel);
                if !evidence_path.exists() {
                    missing_evidence.push(evidence_file.as_str());
                    continue;
                }
                let evidence_content = fs::read_to_string(evidence_path)?;
                if text_matches_claim_pattern(&evidence_content, &claim.pattern) {
                    evidence_found = true;
                }
            }
            if evidence_found {
                continue;
            }
            let expected = format!(
                " Expected evidence files: {}.",
                claim.evidence_files.join(", ")
            );
            let missing = if missing_evidence.is_empty() {
                String::new()
            } else {
                format!(" Missing evidence files: {}.", missing_evidence.join(", "))
            };
            self.push_docs_lifecycle_violation(
                report,
                policy,
                source_rel.to_path_buf(),
                format!(
                    "Docs lifecycle `{}` claim `{}` pattern `{}` appears in `{}` but no declared evidence file contains current evidence.{}{}",
                    policy.id,
                    claim.id,
                    claim.pattern,
                    display_rel(source_rel),
                    expected,
                    missing
                ),
            );
        }
        Ok(())
    }

    fn matching_docs_lifecycle_files(
        &self,
        patterns: &[String],
        checked_path: &Path,
    ) -> Result<Vec<PathBuf>, CheckError> {
        if patterns.is_empty() {
            return Ok(Vec::new());
        }
        let compiled = compile_patterns(patterns)?;
        let mut matches = Vec::new();
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

    fn push_docs_lifecycle_violation(
        &self,
        report: &mut StructureCheckReport,
        policy: &DocsLifecycleConfig,
        path: PathBuf,
        message: String,
    ) {
        report.violations.push(StructureViolation::new(
            path,
            format!("docs_lifecycle:{}", policy.id),
            message,
            policy.severity.as_deref().unwrap_or("medium"),
        ));
    }
}

struct CompiledPattern {
    raw: String,
    pattern: Pattern,
}

fn compile_patterns(patterns: &[String]) -> Result<Vec<CompiledPattern>, CheckError> {
    patterns
        .iter()
        .map(|raw| {
            Pattern::new(raw)
                .map(|pattern| CompiledPattern {
                    raw: raw.clone(),
                    pattern,
                })
                .map_err(|error| {
                    CheckError::Config(crate::cli::config::ConfigError::Invalid(format!(
                        "docs lifecycle pattern `{raw}` is invalid: {error}"
                    )))
                })
        })
        .collect()
}

fn pattern_matches_any(patterns: &[CompiledPattern], rel: &Path) -> bool {
    patterns.iter().any(|compiled| {
        compiled.pattern.matches_path(rel) && pattern_depth_allows(&compiled.raw, rel)
    })
}

fn pattern_depth_allows(pattern: &str, path: &Path) -> bool {
    pattern.contains("**")
        || pattern.split('/').filter(|part| !part.is_empty()).count() == path.components().count()
}

fn frontmatter_status(content: &str) -> Option<String> {
    let rest = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    let frontmatter = rest.split_once("\n---")?.0;
    frontmatter.lines().find_map(|line| {
        line.trim()
            .strip_prefix("status:")
            .map(|value| {
                value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string()
            })
            .filter(|value| !value.is_empty())
    })
}

fn markdown_link_targets(source_rel: &Path, content: &str) -> Vec<PathBuf> {
    let mut targets = Vec::new();
    for link in markdown_links(content) {
        if let Some(target) = parse_markdown_link_target(source_rel, &link.target) {
            targets.push(target.path);
        }
    }
    targets.sort();
    targets.dedup();
    targets
}

fn text_matches_claim_pattern(content: &str, pattern: &str) -> bool {
    if !pattern.contains('*') && !pattern.contains('?') && !pattern.contains('[') {
        return claim_tokens(content).any(|token| token == pattern);
    }
    let Ok(glob) = Pattern::new(pattern) else {
        return false;
    };
    claim_tokens(content).any(|token| glob.matches(token))
}

fn claim_tokens(content: &str) -> impl Iterator<Item = &str> {
    content
        .split(|ch: char| ch.is_whitespace() || matches!(ch, '`' | '"' | '\'' | '(' | ')' | ','))
        .map(|token| token.trim_matches(|ch: char| matches!(ch, '.' | ':' | ';' | '!' | '?')))
        .filter(|token| !token.is_empty())
}

fn safe_docs_lifecycle_path(configured_path: &str) -> Result<PathBuf, CheckError> {
    let rel = PathBuf::from(configured_path);
    if rel.as_os_str().is_empty()
        || rel.is_absolute()
        || !rel
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(CheckError::Config(
            crate::cli::config::ConfigError::Invalid(format!(
                "docs lifecycle path `{configured_path}` must be project-relative and must not use parent traversal"
            )),
        ));
    }
    Ok(rel)
}

#[cfg(test)]
mod tests {
    use super::{frontmatter_status, markdown_link_targets, text_matches_claim_pattern};
    use std::path::{Path, PathBuf};

    #[test]
    fn frontmatter_status_extracts_status() {
        assert_eq!(
            frontmatter_status("---\nstatus: active\ntitle: Example\n---\n# Doc"),
            Some("active".to_string())
        );
        assert_eq!(
            frontmatter_status("---\r\nstatus: planned\r\n---\r\n# Doc"),
            Some("planned".to_string())
        );
    }

    #[test]
    fn markdown_link_targets_normalize_relative_links() {
        assert_eq!(
            markdown_link_targets(
                Path::new("docs/analysis/current.md"),
                "[old](../archive/old.md) [external](https://example.com)"
            ),
            vec![PathBuf::from("docs/archive/old.md")]
        );
        assert!(
            markdown_link_targets(Path::new("docs/current.md"), "[escape](../../old.md)")
                .is_empty()
        );
    }

    #[test]
    fn claim_patterns_support_literal_and_glob_tokens() {
        assert!(text_matches_claim_pattern(
            "Download `assura-linux-amd64.tar.gz`",
            "assura-*.tar.gz"
        ));
        assert!(text_matches_claim_pattern("The cold 2x claim", "2x"));
        assert!(!text_matches_claim_pattern("The cold 12x claim", "2x"));
        assert!(!text_matches_claim_pattern(
            "The docs-lifecycles policy",
            "docs-lifecycle"
        ));
    }
}
