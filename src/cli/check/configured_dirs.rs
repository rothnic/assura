//! Configured directory scope recognition.

use super::scope_patterns::{path_has_scope_magic, CompiledScopePattern};
use std::path::{Path, PathBuf};

pub(super) struct ConfiguredDirSet {
    exact: Vec<PathBuf>,
    patterns: Vec<CompiledScopePattern>,
}

impl ConfiguredDirSet {
    pub(super) fn new(exact: Vec<PathBuf>) -> Self {
        let patterns = exact
            .iter()
            .filter(|path| path_has_scope_magic(path))
            .map(|path| CompiledScopePattern::new(path))
            .collect();
        Self { exact, patterns }
    }

    pub(super) fn contains(&self, rel: &Path) -> bool {
        self.exact
            .binary_search_by(|configured| configured.as_path().cmp(rel))
            .is_ok()
            || self
                .patterns
                .iter()
                .any(|pattern| pattern.matches_path(rel))
    }
}
