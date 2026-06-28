//! Safe Markdown fix operations backed by structure rule scopes.

use super::markdown::fix_blank_line_trailing_spaces;
use super::{discover_project, CheckError, CompiledStructureConfig, StructureChecker};
use crate::config::loader::ConfigLoader;
use std::path::{Path, PathBuf};

/// Markdown fix rule supported by the first safe-fix slice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkdownFixRule {
    /// Remove spaces and tabs from otherwise blank Markdown lines.
    TrailingSpaces,
}

/// Summary of a Markdown fix run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownFixReport {
    /// Project root used for config discovery.
    pub project_root: PathBuf,
    /// Path checked by the user.
    pub checked_path: PathBuf,
    /// Configured Markdown files considered for this fix rule.
    pub files_checked: usize,
    /// Files written with at least one fix.
    pub files_changed: usize,
    /// Individual line fixes applied.
    pub fixes_applied: usize,
}

/// Apply a safe Markdown fix for configured Markdown scopes.
pub fn run_markdown_fix(
    path: Option<PathBuf>,
    config_path: Option<PathBuf>,
    rule: MarkdownFixRule,
) -> Result<MarkdownFixReport, CheckError> {
    let checked_path = match path {
        Some(path) => {
            if !path.exists() {
                return Err(CheckError::MissingPath(path));
            }
            path.canonicalize()?
        }
        None => std::env::current_dir()?,
    };
    let (project_root, config_path) = discover_project(&checked_path, config_path)?;
    if !checked_path.starts_with(&project_root) {
        return Err(CheckError::OutsideProject {
            checked_path,
            project_root,
        });
    }

    let config = ConfigLoader::load(&config_path)?;
    let compiled = CompiledStructureConfig::new_for_check(config, false);
    let mut checker = StructureChecker::from_compiled_owned(project_root.clone(), compiled, false);
    let mut report = MarkdownFixReport {
        project_root,
        checked_path: checked_path.clone(),
        files_checked: 0,
        files_changed: 0,
        fixes_applied: 0,
    };

    checker.fix_markdown_path(&checked_path, rule, &mut report)?;
    Ok(report)
}

impl StructureChecker {
    fn fix_markdown_path(
        &mut self,
        checked_path: &Path,
        rule: MarkdownFixRule,
        report: &mut MarkdownFixReport,
    ) -> Result<(), CheckError> {
        if checked_path.is_file() {
            self.fix_markdown_file(checked_path, rule, report)?;
            return Ok(());
        }

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
                !super::rules::is_excluded_rel_with(&exclude_patterns, rel)
            });

        for entry in walker {
            let entry = entry?;
            if entry.file_type().is_file() {
                self.fix_markdown_file(entry.path(), rule, report)?;
            }
        }

        Ok(())
    }

    fn fix_markdown_file(
        &mut self,
        path: &Path,
        rule: MarkdownFixRule,
        report: &mut MarkdownFixReport,
    ) -> Result<(), CheckError> {
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            return Ok(());
        }

        let rel = self.relative_path(path);
        if self.is_excluded_rel(&rel) {
            return Ok(());
        }

        let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
        let rules = self.resolve_rules(parent_rel);
        let Some(markdown) = rules.markdown else {
            return Ok(());
        };

        match rule {
            MarkdownFixRule::TrailingSpaces => {
                if markdown.lint_trailing_spaces != Some(true) {
                    return Ok(());
                }

                report.files_checked += 1;
                let content = std::fs::read_to_string(path)?;
                let (fixed, fixes) = fix_blank_line_trailing_spaces(&content);
                if fixes > 0 {
                    std::fs::write(path, fixed)?;
                    report.files_changed += 1;
                    report.fixes_applied += fixes;
                }
            }
        }

        Ok(())
    }
}
