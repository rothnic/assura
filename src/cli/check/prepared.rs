//! Prepared structure-check plans for hot validation sessions.

use super::{
    compiled_fingerprint::SourceConfigFingerprint, discover_project, CheckError, CheckTargetMode,
    CompiledStructureConfig, StructureCheckReport, StructureCheckTimings, StructureChecker,
};
use crate::config::config::Config;
use crate::config::loader::ConfigLoader;
use crate::stable_hash::stable_hash;
use std::fs;
use std::path::{Path, PathBuf};

/// A parsed, validated, and compiled structure-check configuration.
///
/// This is intended for long-lived editor or daemon sessions where the
/// configuration usually stays unchanged while checked files change. It keeps
/// rule planning separate from per-run traversal and reloads only when the
/// configuration bytes change.
pub struct PreparedStructureCheck {
    project_root: PathBuf,
    config_path: PathBuf,
    config_hash: u64,
    config_fingerprint: Option<SourceConfigFingerprint>,
    fail_fast: bool,
    check_compiled: CompiledStructureConfig,
    compiled: CompiledStructureConfig,
}

impl PreparedStructureCheck {
    /// Load and compile the configuration discovered for a checked path.
    pub fn load_for_path(
        path: Option<PathBuf>,
        config_path: Option<PathBuf>,
        fail_fast: bool,
    ) -> Result<Self, CheckError> {
        let requested_path = match path {
            Some(path) => path,
            None => std::env::current_dir()?,
        };

        if !requested_path.exists() {
            return Err(CheckError::MissingPath(requested_path));
        }

        let checked_path = requested_path.canonicalize()?;
        let (project_root, discovered_config_path) = discover_project(&checked_path, config_path)?;
        if !checked_path.starts_with(&project_root) {
            return Err(CheckError::OutsideProject {
                checked_path,
                project_root,
            });
        }

        Self::load_project(project_root, discovered_config_path, fail_fast)
    }

    /// Validate one path using the prepared configuration.
    pub fn check_path(&self, path: PathBuf) -> Result<StructureCheckReport, CheckError> {
        if !path.exists() {
            return Err(CheckError::MissingPath(path));
        }

        let checked_path = path.canonicalize()?;
        if !checked_path.starts_with(&self.project_root) {
            return Err(CheckError::OutsideProject {
                checked_path,
                project_root: self.project_root.clone(),
            });
        }

        let mut checker = StructureChecker::from_compiled(
            self.project_root.clone(),
            &self.check_compiled,
            self.fail_fast,
        );
        let mut timings = StructureCheckTimings::default();
        checker.check(
            checked_path,
            self.config_path.clone(),
            CheckTargetMode::Recursive,
            &mut timings,
        )
    }

    /// Validate one changed path and its direct aggregate scopes without
    /// traversing the whole project.
    ///
    /// This is intended for editor and daemon integrations that already keep a
    /// project-level result warm and need a low-latency answer for the file or
    /// directory currently being edited. It does not prove whole-project
    /// success; callers that need a complete report should use `check_path`.
    pub fn check_changed_path(&self, path: PathBuf) -> Result<StructureCheckReport, CheckError> {
        let checked_path = self.resolve_changed_path(path)?;
        if !checked_path.starts_with(&self.project_root) {
            return Err(CheckError::OutsideProject {
                checked_path,
                project_root: self.project_root.clone(),
            });
        }

        let mut report = StructureCheckReport {
            success: true,
            project_root: self.project_root.clone(),
            config_path: self.config_path.clone(),
            checked_path: checked_path.clone(),
            files_checked: 0,
            dirs_checked: 0,
            violations: Vec::new(),
        };

        let mut checker = StructureChecker::from_compiled(
            self.project_root.clone(),
            &self.compiled,
            self.fail_fast,
        );
        checker.validate_configured_structure(&mut report);
        checker.validate_one_changed_path(&checked_path, &mut report);
        report.sort_violations_staged();
        report.refresh_success();
        Ok(report)
    }

    fn resolve_changed_path(&self, path: PathBuf) -> Result<PathBuf, CheckError> {
        if path.exists() {
            return Ok(path.canonicalize()?);
        }

        let path = if path.is_absolute() {
            path
        } else {
            self.project_root.join(path)
        };
        let mut cursor = path.as_path();
        let mut missing_components = Vec::new();
        while !cursor.exists() {
            let Some(name) = cursor.file_name() else {
                return Ok(path);
            };
            missing_components.push(name.to_os_string());
            let Some(parent) = cursor.parent() else {
                return Ok(path);
            };
            cursor = parent;
        }

        let mut resolved = cursor.canonicalize()?;
        for component in missing_components.iter().rev() {
            resolved.push(component);
        }
        Ok(resolved)
    }

    /// Return the project root this prepared checker was built for.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Return the configuration path this prepared checker watches.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Build a prepared checker from an already parsed config.
    pub fn from_config(
        project_root: PathBuf,
        config_path: PathBuf,
        config: Config,
        fail_fast: bool,
    ) -> Self {
        let check_compiled = CompiledStructureConfig::new_for_check(config.clone(), fail_fast);
        let compiled = CompiledStructureConfig::new(config, fail_fast);
        Self {
            project_root,
            config_path,
            config_hash: 0,
            config_fingerprint: None,
            fail_fast,
            check_compiled,
            compiled,
        }
    }

    /// Reload the compiled plan only when the configuration content changed.
    pub fn reload_if_config_changed(&mut self) -> Result<bool, CheckError> {
        let content = fs::read_to_string(&self.config_path).map_err(CheckError::Io)?;
        let config_hash = stable_hash(content.as_bytes());
        if config_hash == self.config_hash {
            self.config_fingerprint = SourceConfigFingerprint::from_path(&self.config_path).ok();
            return Ok(false);
        }

        let config = ConfigLoader::parse_validated(&content)?;
        self.check_compiled =
            CompiledStructureConfig::new_for_check(config.clone(), self.fail_fast);
        self.compiled = CompiledStructureConfig::new(config, self.fail_fast);
        self.config_hash = config_hash;
        self.config_fingerprint = SourceConfigFingerprint::from_path(&self.config_path).ok();
        Ok(true)
    }

    fn load_project(
        project_root: PathBuf,
        config_path: PathBuf,
        fail_fast: bool,
    ) -> Result<Self, CheckError> {
        let content = fs::read_to_string(&config_path).map_err(CheckError::Io)?;
        let config_hash = stable_hash(content.as_bytes());
        let config = ConfigLoader::parse_validated(&content)?;
        let check_compiled = CompiledStructureConfig::new_for_check(config.clone(), fail_fast);
        let compiled = CompiledStructureConfig::new(config, fail_fast);
        let config_fingerprint = SourceConfigFingerprint::from_path(&config_path).ok();

        Ok(Self {
            project_root,
            config_path,
            config_hash,
            config_fingerprint,
            fail_fast,
            check_compiled,
            compiled,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_project(root: &std::path::Path, naming: &str, file_name: &str) {
        fs::create_dir_all(root.join(".assura")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join(".assura/config.yml"),
            format!(
                r#"
structure:
  src/:
    files:
      naming_patterns:
        "*.ts": {naming}
"#
            ),
        )
        .unwrap();
        fs::write(root.join("src").join(file_name), "").unwrap();
    }

    fn write_count_project(root: &std::path::Path, file_names: &[&str]) {
        fs::create_dir_all(root.join(".assura")).unwrap();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(
            root.join(".assura/config.yml"),
            r#"
structure:
  src/:
    files:
      exists:
        "*.ts": "2"
"#,
        )
        .unwrap();
        for file_name in file_names {
            fs::write(root.join("src").join(file_name), "").unwrap();
        }
    }

    #[test]
    fn prepared_check_validates_changed_file_without_tree_walk() {
        let temp = tempfile::tempdir().unwrap();
        write_project(temp.path(), "kebab-case", "bad_name.ts");
        fs::write(temp.path().join("src").join("good-file.ts"), "").unwrap();

        let prepared =
            PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
                .unwrap();

        let report = prepared
            .check_changed_path(temp.path().join("src").join("bad_name.ts"))
            .unwrap();
        assert!(!report.success);
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.dirs_checked, 0);
        assert_eq!(report.violation_count(), 1);

        let report = prepared
            .check_changed_path(temp.path().join("src").join("good-file.ts"))
            .unwrap();
        assert!(report.success);
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.dirs_checked, 0);
    }

    #[test]
    fn prepared_check_rechecks_parent_counts_for_changed_file() {
        let temp = tempfile::tempdir().unwrap();
        write_count_project(temp.path(), &["one.ts"]);

        let prepared =
            PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
                .unwrap();

        let report = prepared
            .check_changed_path(temp.path().join("src").join("one.ts"))
            .unwrap();
        assert!(!report.success);
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.dirs_checked, 0);
        assert!(report
            .violations
            .iter()
            .any(|violation| violation.rule == "exists_count"));

        fs::write(temp.path().join("src").join("two.ts"), "").unwrap();
        let report = prepared
            .check_changed_path(temp.path().join("src").join("two.ts"))
            .unwrap();
        assert!(report.success);
        assert_eq!(report.files_checked, 1);
        assert_eq!(report.dirs_checked, 0);
    }

    #[test]
    fn prepared_check_rechecks_parent_counts_for_deleted_file() {
        let temp = tempfile::tempdir().unwrap();
        write_count_project(temp.path(), &["one.ts", "two.ts"]);

        let prepared =
            PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
                .unwrap();

        let deleted = temp.path().join("src").join("two.ts");
        fs::remove_file(&deleted).unwrap();
        let report = prepared.check_changed_path(deleted).unwrap();
        assert!(!report.success);
        assert_eq!(report.files_checked, 0);
        assert_eq!(report.dirs_checked, 0);
        assert!(report
            .violations
            .iter()
            .any(|violation| violation.rule == "exists_count"));
    }

    #[test]
    fn prepared_check_reloads_when_config_changes() {
        let temp = tempfile::tempdir().unwrap();
        write_project(temp.path(), "kebab-case", "bad_name.ts");

        let mut prepared =
            PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
                .unwrap();
        let report = prepared.check_path(temp.path().to_path_buf()).unwrap();
        assert!(!report.success);

        write_project(temp.path(), "snake_case", "bad_name.ts");
        assert!(prepared.reload_if_config_changed().unwrap());
        let report = prepared.check_path(temp.path().to_path_buf()).unwrap();
        assert!(report.success);
    }

    #[test]
    fn prepared_check_keeps_plan_when_config_is_unchanged() {
        let temp = tempfile::tempdir().unwrap();
        write_project(temp.path(), "kebab-case", "good-file.ts");

        let mut prepared =
            PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
                .unwrap();

        assert!(!prepared.reload_if_config_changed().unwrap());
        let report = prepared.check_path(temp.path().to_path_buf()).unwrap();
        assert!(report.success);
    }

    #[test]
    fn prepared_check_keeps_plan_for_same_content_rewrite() {
        let temp = tempfile::tempdir().unwrap();
        write_project(temp.path(), "kebab-case", "good-file.ts");

        let mut prepared =
            PreparedStructureCheck::load_for_path(Some(temp.path().to_path_buf()), None, false)
                .unwrap();
        let config_path = temp.path().join(".assura/config.yml");
        let content = fs::read_to_string(&config_path).unwrap();
        fs::write(&config_path, content).unwrap();

        assert!(!prepared.reload_if_config_changed().unwrap());
        let report = prepared.check_path(temp.path().to_path_buf()).unwrap();
        assert!(report.success);
    }
}
