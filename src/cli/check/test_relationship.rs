//! Source/test relationship policy validation.

use super::rules::is_excluded_rel_with;
use super::{CheckError, StructureCheckReport, StructureChecker, StructureViolation};
use crate::config::config::{TestRelationshipConfig, TestRelationshipIgnoredTestConfig};
use glob::Pattern;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

impl StructureChecker {
    pub(super) fn validate_test_relationships(
        &self,
        policies: &[TestRelationshipConfig],
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let source_files = self.test_relationship_files(checked_path)?;
        let project_files = self.test_relationship_files(&self.project_root)?;
        for policy in policies {
            self.validate_test_relationship_policy(policy, &source_files, &project_files, report)?;
        }
        Ok(())
    }

    fn validate_test_relationship_policy(
        &self,
        policy: &TestRelationshipConfig,
        source_files: &[PathBuf],
        project_files: &[PathBuf],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        self.validate_required_test_evidence(policy, source_files, project_files, report);
        self.validate_ignored_tests(policy, project_files, report)?;
        self.validate_fixture_families(policy, report)?;
        Ok(())
    }

    fn validate_required_test_evidence(
        &self,
        policy: &TestRelationshipConfig,
        source_files: &[PathBuf],
        project_files: &[PathBuf],
        report: &mut StructureCheckReport,
    ) {
        for relationship in &policy.relationships {
            let source_pattern = compile_test_relationship_pattern(&relationship.source);
            let matching_sources = source_files
                .iter()
                .filter(|path| source_pattern.matches_path(path))
                .collect::<Vec<_>>();
            if matching_sources.is_empty() {
                continue;
            }

            for required_test in &relationship.required_tests {
                let test_pattern = compile_test_relationship_pattern(required_test);
                if project_files
                    .iter()
                    .any(|path| test_pattern.matches_path(path))
                {
                    continue;
                }
                self.push_test_relationship_violation(
                    report,
                    policy,
                    matching_sources[0].to_path_buf(),
                    format!(
                        "Test relationship `{}` source glob `{}` matched source files but required test glob `{}` matched no files",
                        policy.id, relationship.source, required_test
                    ),
                );
            }
        }
    }

    fn validate_ignored_tests(
        &self,
        policy: &TestRelationshipConfig,
        files: &[PathBuf],
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        for file in files {
            if file.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                continue;
            }
            let content = fs::read_to_string(self.project_root.join(file))?;
            for ignored_test in ignored_rust_tests(file, &content) {
                if accepted_ignored_test(policy, &ignored_test) {
                    continue;
                }
                self.push_test_relationship_violation(
                    report,
                    policy,
                    file.clone(),
                    format!(
                        "Test relationship `{}` found ignored/manual Rust test `{}::{}` without an accepted reason category",
                        policy.id,
                        display_rel_path(file),
                        ignored_test.test
                    ),
                );
            }
        }
        Ok(())
    }

    fn validate_fixture_families(
        &self,
        policy: &TestRelationshipConfig,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let declared = policy
            .fixture_families
            .iter()
            .map(|family| safe_test_relationship_path(&family.path))
            .collect::<Result<HashSet<_>, _>>()?;

        for family in &policy.fixture_families {
            let rel = safe_test_relationship_path(&family.path)?;
            if !self.project_root.join(&rel).is_dir() {
                self.push_test_relationship_violation(
                    report,
                    policy,
                    rel,
                    format!(
                        "Test relationship `{}` declares fixture family `{}` but the directory does not exist",
                        policy.id, family.path
                    ),
                );
            }
        }

        for root in &policy.fixture_roots {
            let root_rel = safe_test_relationship_path(root)?;
            let root_path = self.project_root.join(&root_rel);
            if !root_path.is_dir() {
                self.push_test_relationship_violation(
                    report,
                    policy,
                    root_rel,
                    format!(
                        "Test relationship `{}` configured fixture root `{root}` does not exist",
                        policy.id
                    ),
                );
                continue;
            }
            for entry in fs::read_dir(root_path)? {
                let entry = entry?;
                if !entry.file_type()?.is_dir() {
                    continue;
                }
                let rel = self.relative_path(&entry.path());
                if self.is_excluded_rel(&rel) || declared.contains(&rel) {
                    continue;
                }
                self.push_test_relationship_violation(
                    report,
                    policy,
                    rel.clone(),
                    format!(
                        "Test relationship `{}` fixture family `{}` under root `{root}` is not declared with owner and purpose",
                        policy.id,
                        display_rel_path(&rel)
                    ),
                );
            }
        }
        Ok(())
    }

    fn test_relationship_files(&self, checked_path: &Path) -> Result<Vec<PathBuf>, CheckError> {
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
        let mut files = Vec::new();
        for entry in walker {
            let entry = entry?;
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = self.relative_path(entry.path());
            if !self.is_excluded_rel(&rel) {
                files.push(rel);
            }
        }
        files.sort();
        files.dedup();
        Ok(files)
    }

    fn push_test_relationship_violation(
        &self,
        report: &mut StructureCheckReport,
        policy: &TestRelationshipConfig,
        path: PathBuf,
        message: String,
    ) {
        report.violations.push(StructureViolation::new(
            path,
            format!("test_relationship:{}", policy.id),
            message,
            policy.severity.as_deref().unwrap_or("medium"),
        ));
    }
}

struct IgnoredRustTest {
    path: PathBuf,
    test: String,
}

fn ignored_rust_tests(path: &Path, content: &str) -> Vec<IgnoredRustTest> {
    let lines = content.lines().map(str::trim).collect::<Vec<_>>();
    let mut ignored = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if !(line.starts_with("#[") && line[2..].starts_with("ignore")) {
            continue;
        }
        let test = lines
            .iter()
            .skip(index + 1)
            .find_map(|candidate| rust_test_name(candidate))
            .unwrap_or("<unknown>")
            .to_string();
        ignored.push(IgnoredRustTest {
            path: path.to_path_buf(),
            test,
        });
    }
    ignored
}

fn rust_test_name(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("fn ")?;
    let name = rest
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .next()?;
    (!name.is_empty()).then_some(name)
}

fn accepted_ignored_test(policy: &TestRelationshipConfig, ignored: &IgnoredRustTest) -> bool {
    policy.ignored_tests.iter().any(|accepted| {
        ignored_test_path_matches(accepted, &ignored.path) && accepted.test == ignored.test
    })
}

fn ignored_test_path_matches(accepted: &TestRelationshipIgnoredTestConfig, path: &Path) -> bool {
    compile_test_relationship_pattern(&accepted.path).matches_path(path)
}

fn compile_test_relationship_pattern(pattern: &str) -> Pattern {
    Pattern::new(pattern).expect("test relationship patterns are semantically validated")
}

fn safe_test_relationship_path(configured_path: &str) -> Result<PathBuf, CheckError> {
    let rel = PathBuf::from(configured_path);
    if rel.as_os_str().is_empty()
        || rel.is_absolute()
        || !rel
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return Err(CheckError::Config(
            crate::cli::config::ConfigError::Invalid(format!(
                "test relationship path `{configured_path}` must be project-relative and must not use parent traversal"
            )),
        ));
    }
    Ok(rel)
}

fn display_rel_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
