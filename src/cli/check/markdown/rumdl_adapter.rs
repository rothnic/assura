//! Optional subprocess adapter for the `rumdl` Markdown candidate engine.

use super::{markdown_severity, suppression::MarkdownSuppressions};
use crate::cli::check::rules::display_rel;
use crate::cli::check::{StructureCheckReport, StructureChecker};
use crate::config::config::{MarkdownBundle, MarkdownlintCandidateConfig};
use crate::stable_hash::stable_hash;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn validate_rumdl_markdownlint_candidate(
    checker: &StructureChecker,
    rel: &Path,
    markdown: &MarkdownBundle,
    content: &str,
    suppressions: &mut MarkdownSuppressions,
    report: &mut StructureCheckReport,
) {
    let Some(candidate) = &markdown.markdownlint_candidate else {
        return;
    };
    if candidate.enabled != Some(true) {
        return;
    }
    let engine = candidate.engine.as_deref().unwrap_or("rumdl");
    if engine != "rumdl" {
        checker.push_violation(
            report,
            rel.to_path_buf(),
            "markdown_engine",
            format!(
                "Markdown file '{}' enables unsupported markdownlint candidate engine '{}'",
                display_rel(rel),
                engine
            ),
            markdown_severity(markdown, "markdown_engine", "medium"),
        );
        return;
    }

    match run_rumdl(candidate, rel, content) {
        Ok(diagnostics) => {
            push_mapped_rumdl_diagnostics(
                checker,
                rel,
                markdown,
                suppressions,
                report,
                &diagnostics,
            );
        }
        Err(error) => checker.push_violation(
            report,
            rel.to_path_buf(),
            "markdown_engine",
            format!(
                "Markdown file '{}' could not run rumdl markdownlint candidate: {}",
                display_rel(rel),
                error
            ),
            markdown_severity(markdown, "markdown_engine", "medium"),
        ),
    }
}

fn run_rumdl(
    candidate: &MarkdownlintCandidateConfig,
    rel: &Path,
    content: &str,
) -> Result<Vec<RumdlDiagnostic>, String> {
    let binary = candidate.binary.as_deref().unwrap_or("rumdl");
    let fixture = IsolatedMarkdownFile::new(rel, content)?;
    let output = Command::new(binary)
        .args(["check", "--output-format", "json", "--no-cache"])
        .arg(fixture.path())
        .output()
        .map_err(|error| format!("failed to spawn '{}': {}", binary, error))?;

    let exit_code = output.status.code();
    match exit_code {
        Some(0) | Some(1) => {}
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(format!(
                "unexpected exit code {:?}{}",
                exit_code,
                if stderr.is_empty() {
                    String::new()
                } else {
                    format!(": {stderr}")
                }
            ));
        }
    }

    serde_json::from_slice::<Vec<RumdlDiagnostic>>(&output.stdout)
        .map_err(|error| format!("invalid rumdl JSON output: {error}"))
}

fn push_mapped_rumdl_diagnostics(
    checker: &StructureChecker,
    rel: &Path,
    markdown: &MarkdownBundle,
    suppressions: &mut MarkdownSuppressions,
    report: &mut StructureCheckReport,
    diagnostics: &[RumdlDiagnostic],
) {
    for diagnostic in diagnostics {
        let Some(rule) = map_rumdl_rule(&diagnostic.rule) else {
            continue;
        };
        if native_check_owns_rule(markdown, rule) {
            continue;
        }
        let line = diagnostic.line.unwrap_or(1).max(1);
        if suppressions.suppresses(rule, line) {
            continue;
        }
        checker.push_violation(
            report,
            rel.to_path_buf(),
            rule,
            format!(
                "Markdown file '{}' has rumdl finding {} on line {}: {}",
                display_rel(rel),
                diagnostic.rule,
                line,
                diagnostic
                    .message
                    .as_deref()
                    .unwrap_or("markdownlint finding")
            ),
            markdown_severity(markdown, rule, default_severity_for_mapped_rule(rule)),
        );
    }
}

fn map_rumdl_rule(rule: &str) -> Option<&'static str> {
    match rule {
        "MD001" => Some("markdown_heading_increment"),
        "MD009" => Some("markdown_trailing_spaces"),
        "MD012" => Some("markdown_multiple_blank_lines"),
        "MD018" | "MD019" => Some("markdown_heading_marker_spacing"),
        "MD024" => Some("markdown_duplicate_heading"),
        _ => None,
    }
}

fn native_check_owns_rule(markdown: &MarkdownBundle, rule: &str) -> bool {
    matches!(rule, "markdown_trailing_spaces") && markdown.lint_trailing_spaces == Some(true)
        || matches!(
            rule,
            "markdown_heading_increment"
                | "markdown_multiple_blank_lines"
                | "markdown_heading_marker_spacing"
                | "markdown_duplicate_heading"
        ) && markdown.lint_common == Some(true)
}

fn default_severity_for_mapped_rule(rule: &str) -> &'static str {
    match rule {
        "markdown_trailing_spaces" => "low",
        _ => "medium",
    }
}

#[derive(Debug, Deserialize)]
struct RumdlDiagnostic {
    line: Option<usize>,
    rule: String,
    message: Option<String>,
}

struct IsolatedMarkdownFile {
    dir: PathBuf,
    path: PathBuf,
}

impl IsolatedMarkdownFile {
    fn new(rel: &Path, content: &str) -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("system clock before unix epoch: {error}"))?
            .as_nanos();
        let hash = stable_hash(rel.to_string_lossy().as_bytes());
        let dir = std::env::temp_dir().join(format!(
            "assura-rumdl-{}-{:016x}-{nonce}",
            std::process::id(),
            hash
        ));
        fs::create_dir_all(&dir).map_err(|error| format!("create temp dir: {error}"))?;
        let path = dir.join("candidate.md");
        fs::write(&path, content).map_err(|error| format!("write temp markdown: {error}"))?;
        Ok(Self { dir, path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for IsolatedMarkdownFile {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::check::report::StructureCheckReport;
    use crate::config::config::{MarkdownRuleConfig, MarkdownlintCandidateConfig};
    use std::collections::HashMap;

    #[test]
    fn maps_supported_rumdl_rules_to_stable_assura_rule_ids() {
        assert_eq!(map_rumdl_rule("MD001"), Some("markdown_heading_increment"));
        assert_eq!(map_rumdl_rule("MD009"), Some("markdown_trailing_spaces"));
        assert_eq!(
            map_rumdl_rule("MD012"),
            Some("markdown_multiple_blank_lines")
        );
        assert_eq!(
            map_rumdl_rule("MD018"),
            Some("markdown_heading_marker_spacing")
        );
        assert_eq!(
            map_rumdl_rule("MD019"),
            Some("markdown_heading_marker_spacing")
        );
        assert_eq!(map_rumdl_rule("MD024"), Some("markdown_duplicate_heading"));
        assert_eq!(map_rumdl_rule("MD051"), None);
    }

    #[test]
    fn enabled_native_checks_own_overlapping_stable_rules() {
        let mut markdown = MarkdownBundle::new();
        assert!(!native_check_owns_rule(
            &markdown,
            "markdown_trailing_spaces"
        ));
        assert!(!native_check_owns_rule(
            &markdown,
            "markdown_heading_increment"
        ));

        markdown.lint_trailing_spaces = Some(true);
        assert!(native_check_owns_rule(
            &markdown,
            "markdown_trailing_spaces"
        ));
        assert!(!native_check_owns_rule(
            &markdown,
            "markdown_heading_increment"
        ));

        markdown.lint_common = Some(true);
        assert!(native_check_owns_rule(
            &markdown,
            "markdown_heading_increment"
        ));
        assert!(native_check_owns_rule(
            &markdown,
            "markdown_multiple_blank_lines"
        ));
        assert!(native_check_owns_rule(
            &markdown,
            "markdown_heading_marker_spacing"
        ));
        assert!(native_check_owns_rule(
            &markdown,
            "markdown_duplicate_heading"
        ));
        assert!(!native_check_owns_rule(
            &markdown,
            "markdown_link_heading_anchor"
        ));
    }

    #[test]
    fn mapped_rumdl_diagnostics_respect_suppressions_and_rule_severity() {
        let checker = StructureChecker::new(
            PathBuf::from("/tmp/assura-rumdl-test"),
            crate::config::config::Config::default(),
            false,
        );
        let mut markdown = MarkdownBundle::new();
        markdown.markdownlint_candidate = Some(MarkdownlintCandidateConfig {
            enabled: Some(true),
            engine: Some("rumdl".to_string()),
            binary: None,
        });
        markdown.rules = Some(HashMap::from([(
            "markdown_trailing_spaces".to_string(),
            MarkdownRuleConfig {
                severity: Some("low".to_string()),
            },
        )]));
        let mut suppressions = MarkdownSuppressions::parse(
            "# Note\n\n<!-- assura-ignore markdown_heading_increment: fixture skip -->\n### Deep\n",
        );
        let mut report = StructureCheckReport {
            success: true,
            project_root: PathBuf::from("/tmp/assura-rumdl-test"),
            config_path: PathBuf::from("/tmp/assura-rumdl-test/.assura/config.yml"),
            checked_path: PathBuf::from("/tmp/assura-rumdl-test"),
            files_checked: 0,
            dirs_checked: 0,
            violations: Vec::new(),
        };
        let diagnostics = vec![
            RumdlDiagnostic {
                line: Some(4),
                rule: "MD001".to_string(),
                message: Some("heading increment".to_string()),
            },
            RumdlDiagnostic {
                line: Some(2),
                rule: "MD009".to_string(),
                message: Some("trailing spaces".to_string()),
            },
        ];

        push_mapped_rumdl_diagnostics(
            &checker,
            Path::new("docs/note.md"),
            &markdown,
            &mut suppressions,
            &mut report,
            &diagnostics,
        );

        assert_eq!(report.violations.len(), 1);
        assert_eq!(report.violations[0].rule, "markdown_trailing_spaces");
        assert_eq!(report.violations[0].severity, "low");
        assert!(!report.violations[0].blocking);
    }
}
