//! Unit tests for shared structure-rule helpers.

use super::rules::{is_excluded_rel_with, CompiledExclusion};
use std::path::Path;

#[test]
fn prefix_exclusions_match_without_glob_pattern() {
    let patterns = vec![CompiledExclusion::new("dist/**")];
    assert!(is_excluded_rel_with(&patterns, Path::new("dist")));
    assert!(is_excluded_rel_with(&patterns, Path::new("dist/app.js")));
    assert!(!is_excluded_rel_with(
        &patterns,
        Path::new("src/dist/app.js")
    ));
    assert!(!is_excluded_rel_with(&patterns, Path::new("dist-file.js")));
}

#[test]
fn exact_literal_exclusions_match_without_glob_pattern() {
    let patterns = vec![CompiledExclusion::new(".assura")];
    assert!(is_excluded_rel_with(&patterns, Path::new(".assura")));
    assert!(!is_excluded_rel_with(
        &patterns,
        Path::new(".assura/config.yml")
    ));
    assert!(!is_excluded_rel_with(&patterns, Path::new("docs/.assura")));
}

#[test]
fn non_prefix_exclusions_still_use_glob_matching() {
    let patterns = vec![CompiledExclusion::new("**/*.tmp")];
    assert!(is_excluded_rel_with(&patterns, Path::new("src/cache.tmp")));
    assert!(!is_excluded_rel_with(&patterns, Path::new("src/cache.ts")));
}

#[test]
fn brace_exclusions_still_use_scope_pattern_matching() {
    let patterns = vec![CompiledExclusion::new("src/{a,b}")];
    assert!(is_excluded_rel_with(&patterns, Path::new("src/a")));
    assert!(is_excluded_rel_with(&patterns, Path::new("src/b")));
    assert!(!is_excluded_rel_with(&patterns, Path::new("src/c")));
    assert!(!is_excluded_rel_with(&patterns, Path::new("docs/a")));
}

#[test]
fn wildcard_prefix_exclusions_fall_back_to_glob_matching() {
    let patterns = vec![CompiledExclusion::new("build-*/**")];
    assert!(is_excluded_rel_with(
        &patterns,
        Path::new("build-app/cache.bin")
    ));
    assert!(is_excluded_rel_with(
        &patterns,
        Path::new("build-web/nested/cache.bin")
    ));
    assert!(!is_excluded_rel_with(
        &patterns,
        Path::new("src/build-app/cache.bin")
    ));
}
