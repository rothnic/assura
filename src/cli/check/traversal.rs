//! Traversal strategy for structure-first checks.

use super::rules::is_excluded_rel_with;
use super::{CheckError, StructureCheckReport, StructureChecker};
use std::path::Path;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraversalStrategy {
    Serial,
    SerialSorted,
    ParallelJwalk,
}

impl TraversalStrategy {
    fn for_check(fail_fast: bool) -> Self {
        if fail_fast {
            return Self::SerialSorted;
        }

        if std::env::var("ASSURA_CHECK_TRAVERSAL")
            .map(|value| value == "parallel-jwalk")
            .unwrap_or(false)
        {
            Self::ParallelJwalk
        } else {
            Self::Serial
        }
    }

    fn parallelism(self) -> jwalk::Parallelism {
        match self {
            Self::Serial | Self::SerialSorted => jwalk::Parallelism::Serial,
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

impl StructureChecker {
    pub(super) fn walk_and_validate(
        &mut self,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) -> Result<(), CheckError> {
        let strategy = TraversalStrategy::for_check(self.fail_fast);
        match strategy {
            TraversalStrategy::Serial | TraversalStrategy::SerialSorted => {
                self.walk_and_validate_serial(checked_path, report, strategy)
            }
            TraversalStrategy::ParallelJwalk => {
                self.walk_and_validate_parallel(checked_path, report)
            }
        }
    }

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

            if self.fail_fast && !report.violations.is_empty() {
                break;
            }
        }
        Ok(())
    }

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

    fn validate_walk_path(
        &mut self,
        path: &Path,
        checked_path: &Path,
        report: &mut StructureCheckReport,
    ) {
        if path == checked_path && path.is_dir() {
            self.validate_directory_contents(path, report);
            return;
        }

        if path.is_dir() {
            report.dirs_checked += 1;
            self.validate_directory(path, report);
        } else if path.is_file() {
            report.files_checked += 1;
            self.validate_file(path, report);
        }
    }
}
