//! Parallel traversal for large LS-Lint-compatible fast checks.

use super::ls_fast::file_stem;
use super::ls_fast_counts::{fast_rules_have_direct_counts, FastDirEntry};
use super::ls_fast_plan::FastScope;
use super::rules::is_excluded_rel_with;
use super::{CheckError, StructureCheckReport, StructureChecker};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::thread;

impl StructureChecker {
    pub(super) fn should_parallelize_lslint_fast_walk(&self, scopes: &[FastScope]) -> bool {
        self.has_direct_count_constraints
            || scopes.iter().any(FastScope::has_scope_magic)
            || scopes.len() > 128
    }

    pub(super) fn walk_lslint_fast_dir_parallel(
        &self,
        dir: &Path,
        report: &mut StructureCheckReport,
        scopes: &[FastScope],
    ) -> Result<(), CheckError> {
        let project_root = self.project_root.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let mut direct_children: HashMap<PathBuf, Vec<FastDirEntry>> = HashMap::new();
        let mut visited_dirs = vec![PathBuf::new()];
        let walker = jwalk::WalkDir::new(dir)
            .skip_hidden(false)
            .parallelism(parallel_jwalk_strategy())
            .process_read_dir(move |_depth, _path, _state, children| {
                children.retain_mut(|entry| {
                    let Ok(entry) = entry else {
                        return true;
                    };
                    let path = entry.path();
                    let rel = path.strip_prefix(&project_root).unwrap_or(&path);
                    if is_excluded_rel_with(&exclude_patterns, rel) {
                        entry.read_children_path = None;
                        return false;
                    }
                    true
                });
            });

        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            if path == dir {
                continue;
            }
            let rel = path
                .strip_prefix(&self.project_root)
                .unwrap_or(&path)
                .to_path_buf();
            let file_type = entry.file_type();
            if self.has_direct_count_constraints {
                let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
                let parent_rules = self.fast_rules_for_dir(parent_rel, scopes);
                if fast_rules_have_direct_counts(parent_rules) {
                    let name = path.file_name().unwrap_or_default().to_os_string();
                    direct_children
                        .entry(parent_rel.to_path_buf())
                        .or_default()
                        .push(FastDirEntry {
                            name,
                            rel: rel.clone(),
                            file_type,
                        });
                }
            }
            if file_type.is_symlink() {
                continue;
            } else if file_type.is_dir() {
                report.dirs_checked += 1;
                visited_dirs.push(rel.clone());
                let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
                let parent_rules = self.fast_rules_for_dir(parent_rel, scopes);
                let child_rules = self.fast_rules_for_dir(&rel, scopes);
                let name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("");
                self.validate_fast_directory(&rel, name, parent_rules, child_rules, report);
            } else if file_type.is_file() {
                report.files_checked += 1;
                let parent_rel = rel.parent().unwrap_or_else(|| Path::new(""));
                if let Some(rules) = self.fast_rules_for_dir(parent_rel, scopes) {
                    let filename = path
                        .file_name()
                        .and_then(|name| name.to_str())
                        .unwrap_or("");
                    self.validate_fast_file(rules, &rel, filename, file_stem(filename), report);
                }
            }
        }

        if self.has_direct_count_constraints {
            visited_dirs.sort();
            visited_dirs.dedup();
            for dir_rel in visited_dirs {
                let Some(rules) = self.fast_rules_for_dir(&dir_rel, scopes) else {
                    continue;
                };
                if !fast_rules_have_direct_counts(Some(rules)) {
                    continue;
                }
                let entries = direct_children.remove(&dir_rel).unwrap_or_default();
                self.validate_fast_directory_counts(&dir_rel, report, rules, &entries);
            }
        }
        Ok(())
    }
}

fn parallel_jwalk_strategy() -> jwalk::Parallelism {
    let threads = thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    if threads > 1 {
        jwalk::Parallelism::RayonNewPool(threads)
    } else {
        jwalk::Parallelism::Serial
    }
}
