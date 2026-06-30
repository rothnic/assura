//! Traversal strategy for structure-first checks.

use super::rules::is_excluded_rel_with;
use super::{CheckError, StructureCheckReport, StructureChecker};
use std::fs::FileType;
use std::path::Path;
#[cfg(feature = "full-cli")]
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraversalStrategy {
    #[cfg(feature = "full-cli")]
    Serial,
    #[cfg(feature = "full-cli")]
    SerialSorted,
    Walkdir,
    #[cfg(feature = "full-cli")]
    ParallelJwalk,
}

impl TraversalStrategy {
    #[cfg(feature = "full-cli")]
    fn parallelism(self) -> jwalk::Parallelism {
        match self {
            Self::Serial | Self::SerialSorted | Self::Walkdir => jwalk::Parallelism::Serial,
            Self::ParallelJwalk => {
                let threads = thread::available_parallelism()
                    .map(usize::from)
                    .unwrap_or(1);
                if threads > 1 {
                    jwalk::Parallelism::RayonNewPool(threads)
                } else {
                    jwalk::Parallelism::Serial
                }
            }
        }
    }
}

#[cfg(feature = "full-cli")]
fn traversal_strategy_for_check(fail_fast: bool) -> TraversalStrategy {
    if fail_fast {
        return TraversalStrategy::SerialSorted;
    }

    match std::env::var("ASSURA_CHECK_TRAVERSAL").as_deref() {
        Ok("jwalk-serial") => TraversalStrategy::Serial,
        Ok("walkdir") => TraversalStrategy::Walkdir,
        Ok("parallel-jwalk") => TraversalStrategy::ParallelJwalk,
        _ => TraversalStrategy::Walkdir,
    }
}

#[cfg(not(feature = "full-cli"))]
fn traversal_strategy_for_check(_fail_fast: bool) -> TraversalStrategy {
    TraversalStrategy::Walkdir
}

impl StructureChecker {
    pub(in crate::cli::check) fn validate_one_changed_path(
        &mut self,
        path: &Path,
        report: &mut StructureCheckReport,
    ) {
        if path.exists() {
            self.validate_one_existing_path(path, report);
        }

        if let Some(parent) = path
            .parent()
            .filter(|parent| parent.starts_with(&self.project_root))
        {
            self.validate_directory_contents(parent, report);
        }
    }

    pub(in crate::cli::check) fn validate_one_existing_path(
        &mut self,
        path: &Path,
        report: &mut StructureCheckReport,
    ) {
        let Ok(metadata) = path.metadata() else {
            return;
        };
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            self.validate_walk_entry(path, path, file_type, report);
            return;
        }

        if file_type.is_dir() {
            report.dirs_checked += 1;
            self.validate_directory(path, report);
        } else if file_type.is_file() {
            report.files_checked += 1;
            self.validate_file(path, report);
        }
    }

    pub(super) fn walk_and_validate(
        &mut self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let strategy = traversal_strategy_for_check(self.fail_fast);
        match strategy {
            #[cfg(feature = "full-cli")]
            TraversalStrategy::Serial | TraversalStrategy::SerialSorted => {
                self.walk_and_validate_serial(checked_path, report, strategy)
            }
            TraversalStrategy::Walkdir => self.walk_and_validate_walkdir(checked_path, report),
            #[cfg(feature = "full-cli")]
            TraversalStrategy::ParallelJwalk => {
                self.walk_and_validate_parallel(checked_path, report)
            }
        }
    }

    #[cfg(feature = "full-cli")]
    fn walk_and_validate_serial(
        &mut self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
        strategy: TraversalStrategy,
    ) -> Result<(), CheckError> {
        let walker = self.walker(checked_path, strategy);
        for entry in walker {
            let entry = entry?;
            self.validate_walk_path(&entry.path(), checked_path, report);

            if self.fail_fast && report.has_blocking_violations() {
                break;
            }
        }
        Ok(())
    }

    fn walk_and_validate_walkdir(
        &mut self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let checked_path_buf = checked_path.to_path_buf();
        let project_root = self.project_root.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let walker = walkdir::WalkDir::new(checked_path)
            .into_iter()
            .filter_entry(move |entry| {
                let path = entry.path();
                if path == checked_path_buf {
                    return true;
                }
                let rel = path.strip_prefix(&project_root).unwrap_or(path);
                !is_excluded_rel_with(&exclude_patterns, rel)
            });

        for entry in walker {
            let entry = entry?;
            self.validate_walk_entry(entry.path(), checked_path, entry.file_type(), report);
        }
        Ok(())
    }

    #[cfg(feature = "full-cli")]
    fn walk_and_validate_parallel(
        &mut self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let mut paths = Vec::new();
        for entry in self.walker(checked_path, TraversalStrategy::ParallelJwalk) {
            paths.push(entry?.path());
        }
        paths.sort();
        paths.dedup();

        for path in paths {
            self.validate_walk_path(&path, checked_path, report);
        }
        Ok(())
    }

    #[cfg(feature = "full-cli")]
    fn walker(
        &self,
        checked_path: &Path,
        strategy: TraversalStrategy,
    ) -> jwalk::WalkDirGeneric<((), ())> {
        let project_root = self.project_root.clone();
        let exclude_patterns = self.exclude_patterns.clone();
        let mut walker = jwalk::WalkDir::new(checked_path)
            .skip_hidden(false)
            .parallelism(strategy.parallelism());
        if strategy == TraversalStrategy::SerialSorted {
            walker = walker.sort(true);
        }
        walker.process_read_dir(move |_depth, _path, _state, children| {
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
        })
    }

    #[cfg(feature = "full-cli")]
    fn validate_walk_path(
        &mut self,
        path: &Path,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) {
        let Ok(metadata) = path.metadata() else {
            return;
        };
        self.validate_walk_entry(path, checked_path, metadata.file_type(), report);
    }

    fn validate_walk_entry(
        &mut self,
        path: &Path,
        checked_path: &Path,
        file_type: FileType,
        report: &mut StructureCheckReport,
    ) {
        if file_type.is_symlink() {
            if let Ok(metadata) = path.metadata() {
                self.validate_walk_entry(path, checked_path, metadata.file_type(), report);
            }
            return;
        }

        if path == checked_path && file_type.is_dir() {
            self.validate_directory_contents(path, report);
            return;
        }

        if file_type.is_dir() {
            report.dirs_checked += 1;
            self.validate_directory(path, report);
        } else if file_type.is_file() {
            report.files_checked += 1;
            self.validate_file(path, report);
        }
    }
}
