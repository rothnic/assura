//! Path explanation reports backed by the structure-check rule plan.

use super::compiled_config::CompiledStructureConfig;
use super::report::CheckError;
use super::rule_plan::{self, rules_for_dir};
use super::rules::{is_excluded_rel_with, EffectiveRules};
use crate::config::loader::ConfigLoader;
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Read-only explanation of the structure rules affecting one path.
#[derive(Debug, Clone, Serialize)]
pub struct PathExplainReport {
    /// Stable output schema identifier.
    pub schema: &'static str,
    /// Project root used to resolve relative config paths.
    pub project_root: String,
    /// Configuration file used for explanation.
    pub config_path: String,
    /// Requested path.
    pub path: String,
    /// Repository-relative path.
    pub relative_path: String,
    /// Whether the requested path exists.
    pub exists: bool,
    /// Path kind: `file`, `directory`, or `missing`.
    pub kind: &'static str,
    /// Whether the path is excluded by Assura policy.
    pub excluded: bool,
    /// Directory scope used for effective rule lookup.
    pub effective_directory: String,
    /// Matching configured scopes, in evaluation order.
    pub applied_scopes: Vec<PathExplainScope>,
    /// Effective rules that apply to the path's containing directory.
    pub effective_rules: PathExplainRules,
    /// Checks skipped for this path and why.
    pub skipped_checks: Vec<PathExplainSkip>,
    /// Suppression state for this path.
    pub suppressions: Vec<PathExplainSkip>,
    /// Ranked follow-up actions for agents.
    pub next_actions: Vec<PathExplainNextAction>,
}

impl PathExplainReport {
    pub(crate) fn render_text(&self) -> String {
        let mut lines = vec![
            "Assura path explanation".to_string(),
            format!(
                "path={} kind={} excluded={}",
                self.relative_path, self.kind, self.excluded
            ),
            format!("effective_directory={}", self.effective_directory),
            format!(
                "applied_scopes={}",
                self.applied_scopes
                    .iter()
                    .map(|scope| format!("{}:{}", scope.scope, scope.match_kind))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ];
        if !self.skipped_checks.is_empty() {
            lines.push(format!(
                "skipped={}",
                self.skipped_checks
                    .iter()
                    .map(|skip| format!("{}={}", skip.name, skip.status))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        if let Some(action) = self.next_actions.first() {
            lines.push(format!("next: {}", action.action));
        }
        lines.join("\n")
    }
}

/// One configured structure scope contributing to a path explanation.
#[derive(Debug, Clone, Serialize)]
pub struct PathExplainScope {
    /// Configured scope path.
    pub scope: String,
    /// `exact` for the matching scope, `inherited` for ancestor-scope effects.
    pub match_kind: &'static str,
    /// Whether this configured scope inherits parent rules before its local overrides.
    pub inherits_parent: bool,
    /// Whether this configured scope explicitly resets inherited parent rules.
    pub inheritance_reset: bool,
    /// Compact rule summary for this scope.
    pub rules: PathExplainRules,
}

/// Compact summary of effective rules.
#[derive(Debug, Clone, Default, Serialize)]
pub struct PathExplainRules {
    /// File naming rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_naming: Option<String>,
    /// File naming patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_naming_patterns: Vec<String>,
    /// Direct file count constraints.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_exists: Vec<String>,
    /// Allowed direct file names.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_allowed_names: Vec<String>,
    /// Allowed direct file patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_allowed_patterns: Vec<String>,
    /// Forbidden direct file patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub file_forbidden_patterns: Vec<String>,
    /// Whether extra direct files are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_allow_extra: Option<bool>,
    /// Effective file-rule severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_severity: Option<String>,
    /// Direct directory naming rule.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_naming: Option<String>,
    /// Direct directory count constraints.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directory_exists: Vec<String>,
    /// Allowed direct directory names.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directory_allowed_names: Vec<String>,
    /// Allowed direct directory patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directory_allowed_patterns: Vec<String>,
    /// Forbidden direct directory patterns.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub directory_forbidden_patterns: Vec<String>,
    /// Whether extra direct directories are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_allow_extra: Option<bool>,
    /// Effective direct-child directory severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub directory_severity: Option<String>,
    /// Effective self-directory severity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub self_directory_severity: Option<String>,
    /// Whether Markdown frontmatter is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_require_frontmatter: Option<bool>,
    /// Markdown required sections.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub markdown_required_sections: Vec<String>,
    /// Whether trailing-space lint is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub markdown_lint_trailing_spaces: Option<bool>,
    /// Effective Markdown rule severities as `rule:severity` entries.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub markdown_rule_severities: Vec<String>,
}

/// Explanation of one skipped check class.
#[derive(Debug, Clone, Serialize)]
pub struct PathExplainSkip {
    /// Stable skip identifier.
    pub name: &'static str,
    /// Skip status.
    pub status: &'static str,
    /// Human-readable detail.
    pub detail: String,
}

/// Ranked next action for a path explanation.
#[derive(Debug, Clone, Serialize)]
pub struct PathExplainNextAction {
    /// Stable priority order.
    pub priority: u32,
    /// Action to take.
    pub action: String,
    /// Follow-up command or surface.
    pub follow_up: String,
}

/// Explain the effective structure rules for one existing path.
pub fn explain_structure_path(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
) -> Result<PathExplainReport, CheckError> {
    let requested_path = match path {
        Some(path) => path,
        None => std::env::current_dir()?,
    };
    if !requested_path.exists() {
        return Err(CheckError::MissingPath(requested_path));
    }

    let checked_path = requested_path.canonicalize()?;
    let (project_root, config_path) = super::discover_project(&checked_path, config_path)?;
    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let config = ConfigLoader::load(&config_path)?;
    let compiled = CompiledStructureConfig::new(config, false);
    let rel = checked_path
        .strip_prefix(&project_root)
        .unwrap_or(&checked_path)
        .to_path_buf();
    let kind = if checked_path.is_dir() {
        "directory"
    } else if checked_path.is_file() {
        "file"
    } else {
        "missing"
    };
    let effective_directory = if checked_path.is_dir() {
        rel.clone()
    } else {
        rel.parent().map(Path::to_path_buf).unwrap_or_default()
    };
    let excluded = is_excluded_rel_with(&compiled.exclude_patterns, &rel);
    let applied_scopes = compiled
        .rule_scopes
        .iter()
        .filter_map(|scope| {
            rule_plan::scope_match(scope, &effective_directory).map(|exact| {
                let (path, inherit, exact_rules, descendant_rules) = scope.parts();
                PathExplainScope {
                    scope: path_display(path),
                    match_kind: if exact { "exact" } else { "inherited" },
                    inherits_parent: inherit,
                    inheritance_reset: !inherit,
                    rules: summarize_effective_rules(if exact {
                        exact_rules
                    } else {
                        descendant_rules
                    }),
                }
            })
        })
        .collect::<Vec<_>>();
    let effective_rules =
        summarize_effective_rules(&rules_for_dir(&effective_directory, &compiled.rule_scopes));
    let skipped_checks = explain_skipped_checks(kind, excluded, rel.as_path(), &effective_rules);
    let mut next_actions = Vec::new();
    if excluded {
        next_actions.push(PathExplainNextAction {
            priority: 1,
            action: format!(
                "Review exclude policy if {} should be checked.",
                path_display(&rel)
            ),
            follow_up: "Inspect .assura/config.yml exclude patterns".to_string(),
        });
    }
    if applied_scopes.is_empty() {
        next_actions.push(PathExplainNextAction {
            priority: 1,
            action: format!(
                "Add a structure scope if {} needs policy.",
                path_display(&rel)
            ),
            follow_up: "Edit .assura/config.yml structure".to_string(),
        });
    }
    next_actions.push(PathExplainNextAction {
        priority: (next_actions.len() + 1) as u32,
        action: "Run a full structure check before treating this explanation as project health."
            .to_string(),
        follow_up: "assura check --format json .".to_string(),
    });

    Ok(PathExplainReport {
        schema: "assura.path-explain.v1",
        project_root: path_display(&project_root),
        config_path: path_display(&config_path),
        path: path_display(&checked_path),
        relative_path: path_display(&rel),
        exists: true,
        kind,
        excluded,
        effective_directory: path_display(&effective_directory),
        applied_scopes,
        effective_rules,
        skipped_checks,
        suppressions: vec![PathExplainSkip {
            name: "inline_suppressions",
            status: "not_configured",
            detail: "Assura structure checks do not currently support inline suppressions for this path.".to_string(),
        }],
        next_actions,
    })
}

fn summarize_effective_rules(rules: &EffectiveRules) -> PathExplainRules {
    let mut summary = PathExplainRules::default();
    if let Some(files) = rules.files.as_ref() {
        summary.file_naming = files.naming.clone();
        summary.file_naming_patterns = files
            .naming_patterns
            .as_ref()
            .map(sorted_keys)
            .unwrap_or_default();
        summary.file_exists = files
            .exists
            .as_ref()
            .map(sorted_count_entries)
            .unwrap_or_default();
        summary.file_allowed_names = files.allowed_names.clone().unwrap_or_default();
        summary.file_allowed_patterns = files.allowed_patterns.clone().unwrap_or_default();
        summary.file_forbidden_patterns = files.forbidden_patterns.clone().unwrap_or_default();
        summary.file_allow_extra = files.allow_extra;
        summary.file_severity = Some(
            files
                .severity
                .clone()
                .unwrap_or_else(|| "medium".to_string()),
        );
    }
    if let Some(directories) = rules.directories.as_ref() {
        summary.directory_naming = directories.naming.clone();
        summary.directory_exists = directories
            .exists
            .as_ref()
            .map(sorted_count_entries)
            .unwrap_or_default();
        summary.directory_allowed_names = directories.allowed_names.clone().unwrap_or_default();
        summary.directory_allowed_patterns =
            directories.allowed_patterns.clone().unwrap_or_default();
        summary.directory_forbidden_patterns =
            directories.forbidden_patterns.clone().unwrap_or_default();
        summary.directory_allow_extra = directories.allow_extra;
        summary.directory_severity = Some(
            directories
                .severity
                .clone()
                .unwrap_or_else(|| "medium".to_string()),
        );
    }
    if let Some(directory) = rules.self_directory.as_ref() {
        summary.self_directory_severity = Some(
            directory
                .severity
                .clone()
                .unwrap_or_else(|| "medium".to_string()),
        );
    }
    if let Some(markdown) = rules.markdown.as_ref() {
        summary.markdown_require_frontmatter = markdown.require_frontmatter;
        summary.markdown_required_sections = markdown.required_sections.clone().unwrap_or_default();
        summary.markdown_lint_trailing_spaces = markdown.lint_trailing_spaces;
        summary.markdown_rule_severities = markdown
            .rules
            .as_ref()
            .map(markdown_rule_severities)
            .unwrap_or_default();
    }
    summary
}

fn explain_skipped_checks(
    kind: &str,
    excluded: bool,
    rel: &Path,
    rules: &PathExplainRules,
) -> Vec<PathExplainSkip> {
    let mut skipped = Vec::new();
    if excluded {
        skipped.push(PathExplainSkip {
            name: "all_structure_checks",
            status: "skipped",
            detail: "Path is excluded by project policy.".to_string(),
        });
        return skipped;
    }

    if kind != "file" {
        skipped.push(PathExplainSkip {
            name: "file_content_checks",
            status: "skipped",
            detail: "Path is not a file, so file content and Markdown checks do not apply."
                .to_string(),
        });
    }

    let is_markdown = rel.extension().and_then(|ext| ext.to_str()) == Some("md");
    if kind == "file" && !is_markdown {
        skipped.push(PathExplainSkip {
            name: "markdown_checks",
            status: "not_applicable",
            detail: "Path is not a Markdown file.".to_string(),
        });
    } else if kind == "file"
        && is_markdown
        && rules.markdown_require_frontmatter.is_none()
        && rules.markdown_required_sections.is_empty()
        && rules.markdown_lint_trailing_spaces.is_none()
    {
        skipped.push(PathExplainSkip {
            name: "markdown_checks",
            status: "not_configured",
            detail: "No Markdown policy is active for this path.".to_string(),
        });
    }

    if kind == "file"
        && rules.file_naming.is_none()
        && rules.file_naming_patterns.is_empty()
        && rules.file_exists.is_empty()
        && rules.file_allowed_names.is_empty()
        && rules.file_allowed_patterns.is_empty()
        && rules.file_forbidden_patterns.is_empty()
        && rules.file_allow_extra.is_none()
        && !is_markdown
    {
        skipped.push(PathExplainSkip {
            name: "file_structure_rules",
            status: "not_configured",
            detail: "No file structure rules are active for this path.".to_string(),
        });
    }

    skipped.push(PathExplainSkip {
        name: "binary_read",
        status: if is_markdown {
            "read_as_text_when_markdown_policy_requires"
        } else {
            "not_required"
        },
        detail: if is_markdown {
            "Markdown files may be read as UTF-8 when Markdown checks are configured.".to_string()
        } else {
            "This explanation does not read file contents; ordinary structure checks only read non-Markdown files when configured rules require text content.".to_string()
        },
    });

    skipped
}

fn sorted_keys(map: &HashMap<String, String>) -> Vec<String> {
    let mut keys = map.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    keys
}

fn markdown_rule_severities(
    rules: &HashMap<String, crate::config::config::MarkdownRuleConfig>,
) -> Vec<String> {
    let mut severities = rules
        .iter()
        .filter_map(|(rule, config)| {
            config
                .severity
                .as_ref()
                .map(|severity| format!("{rule}:{severity}"))
        })
        .collect::<Vec<_>>();
    severities.sort();
    severities
}

fn sorted_count_entries(map: &HashMap<String, String>) -> Vec<String> {
    let mut entries = map
        .iter()
        .map(|(pattern, count)| format!("{pattern}:{count}"))
        .collect::<Vec<_>>();
    entries.sort();
    entries
}

fn path_display(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path.display().to_string()
    }
}
