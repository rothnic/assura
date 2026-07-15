//! Configured directory and missing-scope validation.

use super::rules::{
    count_satisfies, display_rel, join_config_child, normalize_config_dir, severity_for_bundle,
    severity_for_directory_bundle, severity_for_node,
};
use super::scope_patterns::{path_has_scope_magic, path_matches_scope_pattern};
use super::{StructureCheckReport, StructureChecker};
use crate::config::config::DirectoryNode;
use std::path::Path;

impl StructureChecker {
    pub(in crate::cli::check) fn validate_configured_structure(
        &self,
        report: &mut StructureCheckReport,
    ) {
        for (path, node) in &self.config.structure {
            if !node_has_configured_requirements(node) {
                continue;
            }
            let base = normalize_config_dir(path);
            self.validate_node_requirements(&base, node, report);
        }
    }

    fn validate_node_requirements(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        report: &mut StructureCheckReport,
    ) {
        if self.is_excluded_rel(node_rel) {
            return;
        }

        let node_abs = self.project_root.join(node_rel);
        if !node_abs.is_dir() {
            let pattern_has_matches =
                path_has_scope_magic(node_rel) && self.has_matching_directory_scope(node_rel);
            if !pattern_has_matches {
                self.validate_self_directory_exists(node_rel, node, 0, report);
            }

            if node_allows_absence(node) {
                return;
            }
            if !pattern_has_matches {
                self.validate_missing_direct_counts(node_rel, node, report);
            }
            if node.required {
                self.push_violation(
                    report,
                    node_rel.to_path_buf(),
                    "required_directory",
                    format!(
                        "Configured directory '{}' is missing",
                        display_rel(node_rel)
                    ),
                    severity_for_node(node),
                );
            }
            return;
        }

        self.validate_required_file_names(node_rel, node, report);
        self.validate_required_directory_names(node_rel, node, report);
        self.validate_legacy_exists_lists(node_rel, node, report);
        let self_count = usize::from(!node_rel.as_os_str().is_empty());
        self.validate_self_directory_exists(node_rel, node, self_count, report);
        self.validate_child_node_requirements(node_rel, node, report);
    }

    fn validate_required_file_names(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        report: &mut StructureCheckReport,
    ) {
        let Some(files) = &node.files else {
            return;
        };
        let Some(required) = &files.required else {
            return;
        };
        for file in required {
            let file_rel = node_rel.join(file);
            if !self.project_root.join(&file_rel).is_file() {
                self.push_violation(
                    report,
                    file_rel.clone(),
                    "required_file",
                    format!("Required file '{}' is missing", display_rel(&file_rel)),
                    severity_for_bundle(files),
                );
            }
        }
    }

    fn validate_required_directory_names(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        report: &mut StructureCheckReport,
    ) {
        let Some(directories) = &node.directories else {
            return;
        };
        let Some(required) = &directories.required else {
            return;
        };
        for directory in required {
            let dir_rel = node_rel.join(directory);
            if !self.project_root.join(&dir_rel).is_dir() {
                self.push_violation(
                    report,
                    dir_rel.clone(),
                    "required_directory",
                    format!("Required directory '{}' is missing", display_rel(&dir_rel)),
                    directories
                        .severity
                        .clone()
                        .unwrap_or_else(|| "medium".to_string()),
                );
            }
        }
    }

    fn validate_legacy_exists_lists(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        report: &mut StructureCheckReport,
    ) {
        let Some(exists) = &node.exists else {
            return;
        };

        if let Some(files) = &exists.files {
            for file in files {
                let file_rel = node_rel.join(file);
                if !self.project_root.join(&file_rel).is_file() {
                    self.push_violation(
                        report,
                        file_rel.clone(),
                        "required_file",
                        format!("Required file '{}' is missing", display_rel(&file_rel)),
                        severity_for_node(node),
                    );
                }
            }
        }

        if let Some(directories) = &exists.directories {
            for directory in directories {
                let dir_rel = node_rel.join(directory);
                if !self.project_root.join(&dir_rel).is_dir() {
                    self.push_violation(
                        report,
                        dir_rel.clone(),
                        "required_directory",
                        format!("Required directory '{}' is missing", display_rel(&dir_rel)),
                        severity_for_node(node),
                    );
                }
            }
        }
    }

    fn validate_child_node_requirements(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        report: &mut StructureCheckReport,
    ) {
        if let Some(children) = &node.children {
            for (child_name, child) in children {
                if !node_has_configured_requirements(child) {
                    continue;
                }
                let child_rel = join_config_child(node_rel, child_name);
                self.validate_node_requirements(&child_rel, child, report);
            }
        }
    }

    fn validate_self_directory_exists(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        count: usize,
        report: &mut StructureCheckReport,
    ) {
        let Some(directory) = &node.self_directory else {
            return;
        };
        let Some(exists) = &directory.exists else {
            return;
        };

        for expected in exists.values() {
            if count_satisfies(count, expected) {
                continue;
            }
            self.push_violation(
                report,
                node_rel.to_path_buf(),
                "exists_count",
                format!(
                    "Directory '{}' exists {} times, expected {}",
                    display_rel(node_rel),
                    count,
                    expected
                ),
                severity_for_directory_bundle(directory),
            );
        }
    }

    fn validate_missing_direct_counts(
        &self,
        node_rel: &Path,
        node: &DirectoryNode,
        report: &mut StructureCheckReport,
    ) {
        if let Some(files) = &node.files {
            if let Some(exists) = &files.exists {
                for (pattern, expected) in exists {
                    if count_satisfies(0, expected) {
                        continue;
                    }
                    self.push_violation(
                        report,
                        node_rel.to_path_buf(),
                        "exists_count",
                        format!(
                            "Directory '{}' has 0 files matching '{}', expected {}",
                            display_rel(node_rel),
                            pattern,
                            expected
                        ),
                        severity_for_bundle(files),
                    );
                }
            }
        }

        if let Some(directories) = &node.directories {
            if let Some(exists) = &directories.exists {
                for (pattern, expected) in exists {
                    if count_satisfies(0, expected) {
                        continue;
                    }
                    self.push_violation(
                        report,
                        node_rel.to_path_buf(),
                        "exists_count",
                        format!(
                            "Directory '{}' has 0 directories matching '{}', expected {}",
                            display_rel(node_rel),
                            pattern,
                            expected
                        ),
                        severity_for_directory_bundle(directories),
                    );
                }
            }
        }
    }

    fn has_matching_directory_scope(&self, pattern: &Path) -> bool {
        let mut stack = vec![self.project_root.clone()];
        while let Some(directory) = stack.pop() {
            let rel = directory
                .strip_prefix(&self.project_root)
                .unwrap_or(&directory);
            if !rel.as_os_str().is_empty() && path_matches_scope_pattern(pattern, rel) {
                return true;
            }

            let Ok(entries) = std::fs::read_dir(&directory) else {
                continue;
            };
            for entry in entries.filter_map(Result::ok) {
                let Ok(file_type) = entry.file_type() else {
                    continue;
                };
                if !file_type.is_dir() {
                    continue;
                }
                let child = entry.path();
                let child_rel = child.strip_prefix(&self.project_root).unwrap_or(&child);
                if self.is_excluded_rel(child_rel) {
                    continue;
                }
                stack.push(child);
            }
        }
        false
    }
}

fn node_allows_absence(node: &DirectoryNode) -> bool {
    node.self_directory
        .as_ref()
        .and_then(|directory| directory.exists.as_ref())
        .is_some_and(|exists| exists.values().all(|expected| count_satisfies(0, expected)))
}

fn node_has_configured_requirements(node: &DirectoryNode) -> bool {
    node.required
        || node.exists.as_ref().is_some_and(|exists| {
            exists.files.as_ref().is_some_and(|files| !files.is_empty())
                || exists
                    .directories
                    .as_ref()
                    .is_some_and(|directories| !directories.is_empty())
        })
        || node.files.as_ref().is_some_and(|files| {
            files
                .required
                .as_ref()
                .is_some_and(|required| !required.is_empty())
                || files
                    .exists
                    .as_ref()
                    .is_some_and(|exists| !exists.is_empty())
        })
        || node.directories.as_ref().is_some_and(|directories| {
            directories
                .required
                .as_ref()
                .is_some_and(|required| !required.is_empty())
                || directories
                    .exists
                    .as_ref()
                    .is_some_and(|exists| !exists.is_empty())
        })
        || node.self_directory.as_ref().is_some_and(|directory| {
            directory
                .exists
                .as_ref()
                .is_some_and(|exists| !exists.is_empty())
        })
        || node
            .children
            .as_ref()
            .is_some_and(|children| children.values().any(node_has_configured_requirements))
}
