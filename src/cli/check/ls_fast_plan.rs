//! Compiled plan types for the LS-Lint-compatible check fast path.

use super::ls_fast_naming::{
    collect_fast_naming_regex_patterns, compile_fast_naming, FastFileNaming, FastNaming,
};
use super::patterns::{is_lslint_extension_pattern, simple_suffix_pattern};
use super::rules::{
    dir_contains, join_config_child, merge_directory_bundle, merge_file_bundle,
    merge_markdown_bundle, normalize_config_dir, strip_direct_content_policy, EffectiveRules,
};
use super::scope_patterns::{path_has_scope_magic, path_scope_specificity, CompiledScopePattern};
use crate::config::config::{Config, DirectoryBundle, DirectoryNode, FileBundle};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub(super) struct FastScope {
    path: PathBuf,
    inherit: bool,
    has_scope_magic: bool,
    scope_pattern: Option<CompiledScopePattern>,
    pub(super) exact: FastRules,
    pub(super) descendant: FastRules,
}

#[derive(Clone)]
pub(super) struct FastRules {
    pub(super) effective: EffectiveRules,
    pub(super) file_naming: Option<FastFileNaming>,
    pub(super) directory_naming: Option<FastNaming>,
    pub(super) self_directory_naming: Option<FastNaming>,
    pub(super) has_direct_file_policy: bool,
    pub(super) has_direct_directory_policy: bool,
}

pub(super) fn compile_lslint_fast_scopes(config: &Config) -> Option<Vec<FastScope>> {
    if !config.patterns.is_empty() || children_have_potential_overlap(&config.structure) {
        return None;
    }
    if config.extensions.as_ref().is_some_and(|extensions| {
        !extensions.custom_constraints.is_empty()
            || !extensions.release_contracts.is_empty()
            || !extensions.support_matrices.is_empty()
            || !extensions.manifest_semantics.is_empty()
            || !extensions.test_relationships.is_empty()
            || !extensions.module_topologies.is_empty()
            || !extensions.docs_lifecycles.is_empty()
            || !extensions.repository_references.is_empty()
            || !extensions.agent_guidance.is_empty()
            || !extensions.requirements_traceability.is_empty()
            || !extensions.computed_checks.is_empty()
            || !extensions.relationships.is_empty()
    }) {
        return None;
    }

    let mut scopes = Vec::new();
    let mut naming_cache = FastNamingCache::default();
    for (path, node) in &config.structure {
        if !is_fast_node(node) {
            return None;
        }

        let base = normalize_config_dir(path);
        compile_scope_node(
            base,
            node,
            &EffectiveRules::default(),
            &mut scopes,
            &mut naming_cache,
        )?;
    }

    compose_static_ancestor_scopes(&mut scopes, &mut naming_cache);
    scopes.sort_by_key(|scope| Reverse(scope.specificity()));
    Some(scopes)
}

pub(super) fn fast_rules_for_dir<'a>(
    dir_rel: &Path,
    scopes: &'a [FastScope],
) -> Option<&'a FastRules> {
    let (scope, exact) = scopes
        .iter()
        .find_map(|scope| scope_match(scope, dir_rel).map(|exact| (scope, exact)))?;
    if exact {
        Some(&scope.exact)
    } else {
        Some(&scope.descendant)
    }
}

pub(super) fn fast_rules_for_dir_indexed<'a>(
    dir_rel: &Path,
    scopes: &'a [FastScope],
    index: &HashMap<PathBuf, usize>,
) -> Option<&'a FastRules> {
    fast_scope_for_dir_indexed(dir_rel, scopes, index).map(|scope| scope.rules)
}

pub(super) struct FastScopeMatch<'a> {
    pub(super) rules: &'a FastRules,
}

pub(super) struct FastTargetScopeMatch<'a> {
    pub(super) index_dir: PathBuf,
    pub(super) rules: &'a FastRules,
    pub(super) exact_rules: &'a FastRules,
}

pub(super) fn fast_target_scope_for_dir<'a>(
    dir_rel: &Path,
    scopes: &'a [FastScope],
) -> Option<FastTargetScopeMatch<'a>> {
    scopes.iter().find_map(|scope| {
        if scope.has_scope_magic {
            let pattern = scope.scope_pattern.as_ref()?;
            if pattern.matches_path(dir_rel) {
                return Some(FastTargetScopeMatch {
                    index_dir: dir_rel.to_path_buf(),
                    rules: &scope.exact,
                    exact_rules: &scope.exact,
                });
            }
            return pattern
                .matching_ancestor(dir_rel)
                .map(|index_dir| FastTargetScopeMatch {
                    index_dir,
                    rules: &scope.descendant,
                    exact_rules: &scope.exact,
                });
        }

        dir_contains(&scope.path, dir_rel).then(|| FastTargetScopeMatch {
            index_dir: scope.path.clone(),
            rules: if dir_rel == scope.path {
                &scope.exact
            } else {
                &scope.descendant
            },
            exact_rules: &scope.exact,
        })
    })
}

pub(super) fn fast_scope_for_dir_indexed<'a>(
    dir_rel: &Path,
    scopes: &'a [FastScope],
    index: &HashMap<PathBuf, usize>,
) -> Option<FastScopeMatch<'a>> {
    let mut cursor = Some(dir_rel);
    while let Some(candidate) = cursor {
        if let Some(scope_index) = index.get(candidate) {
            let scope = scopes.get(*scope_index)?;
            let rules = if dir_rel == scope.path {
                &scope.exact
            } else {
                &scope.descendant
            };
            return Some(FastScopeMatch { rules });
        }
        cursor = candidate.parent();
    }
    None
}

impl FastScope {
    pub(super) fn new(path: PathBuf, exact: FastRules, descendant: FastRules) -> Self {
        let has_scope_magic = path_has_scope_magic(&path);
        let scope_pattern = has_scope_magic.then(|| CompiledScopePattern::new(&path));
        Self {
            path,
            inherit: true,
            has_scope_magic,
            scope_pattern,
            exact,
            descendant,
        }
    }

    fn with_inherit(mut self, inherit: bool) -> Self {
        self.inherit = inherit;
        self
    }

    pub(super) fn parts(&self) -> (&Path, &FastRules, &FastRules) {
        (&self.path, &self.exact, &self.descendant)
    }

    fn specificity(&self) -> (usize, usize, usize, usize) {
        path_scope_specificity(&self.path)
    }

    pub(super) fn has_scope_magic(&self) -> bool {
        self.has_scope_magic
    }
}

impl FastRules {
    pub(super) fn from_parts(
        effective: EffectiveRules,
        file_naming: Option<FastFileNaming>,
        directory_naming: Option<FastNaming>,
        self_directory_naming: Option<FastNaming>,
    ) -> Self {
        let has_direct_file_policy = has_direct_file_policy(&effective);
        let has_direct_directory_policy = has_direct_directory_policy(&effective);
        Self {
            effective,
            file_naming,
            directory_naming,
            self_directory_naming,
            has_direct_file_policy,
            has_direct_directory_policy,
        }
    }

    pub(super) fn parts(
        &self,
    ) -> (
        &EffectiveRules,
        Option<&FastFileNaming>,
        Option<&FastNaming>,
        Option<&FastNaming>,
    ) {
        (
            &self.effective,
            self.file_naming.as_ref(),
            self.directory_naming.as_ref(),
            self.self_directory_naming.as_ref(),
        )
    }

    #[cfg(test)]
    pub(super) fn new(effective: EffectiveRules) -> Self {
        let mut naming_cache = FastNamingCache::default();
        Self::new_with_cache(effective, &mut naming_cache)
    }

    fn new_with_cache(effective: EffectiveRules, naming_cache: &mut FastNamingCache) -> Self {
        let file_naming = effective.files.as_ref().and_then(|files| {
            let default = files
                .naming
                .as_ref()
                .map(|naming| naming_cache.compile(naming));
            let mut suffix_patterns = Vec::new();
            let glob_patterns = files
                .naming_patterns
                .as_ref()
                .map(|patterns| {
                    patterns
                        .iter()
                        .filter_map(|(pattern, naming)| {
                            let naming = naming_cache.compile(naming);
                            if simple_suffix_pattern(pattern).is_some()
                                && is_lslint_extension_pattern(pattern)
                            {
                                suffix_patterns.push((pattern.clone(), naming));
                                return None;
                            }
                            Some((pattern.clone(), naming))
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            (default.is_some() || !suffix_patterns.is_empty() || !glob_patterns.is_empty())
                .then(|| FastFileNaming::from_parts(suffix_patterns, glob_patterns, default))
        });

        let directory_naming = effective
            .directories
            .as_ref()
            .and_then(|directories| directories.naming.as_ref())
            .map(|naming| naming_cache.compile(naming));
        let self_directory_naming = effective
            .self_directory
            .as_ref()
            .and_then(|directory| directory.naming.as_ref())
            .map(|naming| naming_cache.compile(naming));

        Self {
            has_direct_file_policy: has_direct_file_policy(&effective),
            has_direct_directory_policy: has_direct_directory_policy(&effective),
            effective,
            file_naming,
            directory_naming,
            self_directory_naming,
        }
    }
}

#[derive(Default)]
struct FastNamingCache {
    by_convention: HashMap<String, FastNaming>,
}

impl FastNamingCache {
    fn compile(&mut self, convention: &str) -> FastNaming {
        if let Some(naming) = self.by_convention.get(convention) {
            return naming.clone();
        }

        let naming = compile_fast_naming(convention);
        self.by_convention
            .insert(convention.to_string(), naming.clone());
        naming
    }
}

fn compile_scope_node(
    node_rel: PathBuf,
    node: &DirectoryNode,
    inherited: &EffectiveRules,
    scopes: &mut Vec<FastScope>,
    naming_cache: &mut FastNamingCache,
) -> Option<()> {
    let effective = if node.inherit {
        EffectiveRules {
            files: merge_file_bundle(inherited.files.as_ref(), node.files.as_ref()),
            directories: merge_directory_bundle(
                inherited.directories.as_ref(),
                node.directories.as_ref(),
            ),
            self_directory: merge_directory_bundle(
                inherited.self_directory.as_ref(),
                node.self_directory.as_ref(),
            ),
            markdown: merge_markdown_bundle(inherited.markdown.as_ref(), node.markdown.as_ref()),
        }
    } else {
        EffectiveRules {
            files: node.files.clone().map(Arc::new),
            directories: node.directories.clone().map(Arc::new),
            self_directory: node.self_directory.clone().map(Arc::new),
            markdown: node.markdown.clone().map(Arc::new),
        }
    };

    scopes.push(
        FastScope::new(
            node_rel.clone(),
            FastRules::new_with_cache(effective.clone(), naming_cache),
            FastRules::new_with_cache(strip_direct_content_policy(effective.clone()), naming_cache),
        )
        .with_inherit(node.inherit),
    );

    if let Some(children) = &node.children {
        for (child_name, child) in children {
            let child_rel = join_config_child(&node_rel, child_name);
            compile_scope_node(child_rel, child, &effective, scopes, naming_cache)?;
        }
    }

    Some(())
}

fn scope_match(scope: &FastScope, dir_rel: &Path) -> Option<bool> {
    if scope.has_scope_magic {
        let Some(pattern) = &scope.scope_pattern else {
            return None;
        };
        if pattern.matches_path(dir_rel) {
            return Some(true);
        }
        return pattern.has_matching_ancestor(dir_rel).then_some(false);
    }

    dir_contains(&scope.path, dir_rel).then_some(dir_rel == scope.path)
}

include!("ls_fast_scope_composition.rs");

fn has_direct_file_policy(effective: &EffectiveRules) -> bool {
    effective.files.as_ref().is_some_and(|files| {
        files.allowed_names.is_some()
            || files.allowed_patterns.is_some()
            || files.forbidden_patterns.is_some()
            || files.allow_extra.is_some()
    })
}

fn has_direct_directory_policy(effective: &EffectiveRules) -> bool {
    effective.directories.as_ref().is_some_and(|directories| {
        directories.allowed_names.is_some()
            || directories.allowed_patterns.is_some()
            || directories.forbidden_patterns.is_some()
            || directories.allow_extra.is_some()
    })
}

pub(super) fn collect_fast_regex_patterns(scopes: &[FastScope]) -> Vec<String> {
    let mut patterns = Vec::new();
    for scope in scopes {
        collect_fast_rules_regex_patterns(&scope.exact, &mut patterns);
        collect_fast_rules_regex_patterns(&scope.descendant, &mut patterns);
    }
    patterns.sort();
    patterns.dedup();
    patterns
}

fn collect_fast_rules_regex_patterns(rules: &FastRules, patterns: &mut Vec<String>) {
    if let Some(file_naming) = &rules.file_naming {
        let (suffix_patterns, glob_patterns, default) = file_naming.parts();
        for (_, naming) in suffix_patterns {
            collect_fast_naming_regex_patterns(naming, patterns);
        }
        for pattern in glob_patterns {
            let (_, naming) = pattern.parts();
            collect_fast_naming_regex_patterns(naming, patterns);
        }
        if let Some(naming) = default {
            collect_fast_naming_regex_patterns(naming, patterns);
        }
    }
    if let Some(naming) = &rules.directory_naming {
        collect_fast_naming_regex_patterns(naming, patterns);
    }
    if let Some(naming) = &rules.self_directory_naming {
        collect_fast_naming_regex_patterns(naming, patterns);
    }
}

fn is_fast_node(node: &DirectoryNode) -> bool {
    node.markdown.is_none()
        && node.exists.is_none()
        && node
            .self_directory
            .as_ref()
            .map_or(true, is_fast_self_directory_bundle)
        && node.files.as_ref().map_or(true, is_fast_file_bundle)
        && node
            .directories
            .as_ref()
            .map_or(true, is_fast_directory_bundle)
        && node.children.as_ref().map_or(true, |children| {
            !children_have_potential_overlap(children) && children.values().all(is_fast_node)
        })
}

fn children_have_potential_overlap(children: &HashMap<String, DirectoryNode>) -> bool {
    children.len() > 1
        && children
            .keys()
            .any(|name| path_has_scope_magic(Path::new(name)))
}

fn is_fast_file_bundle(files: &FileBundle) -> bool {
    files.max_lines.is_none()
        && files.max_lines_patterns.is_none()
        && files.max_size.is_none()
        && files.max_size_patterns.is_none()
        && files.naming_patterns.as_ref().map_or(true, |patterns| {
            patterns
                .keys()
                .all(|pattern| is_lslint_extension_pattern(pattern))
        })
        && files.require_docs.is_none()
        && files.extensions.is_none()
        && files.required.is_none()
}

fn is_fast_directory_bundle(directories: &DirectoryBundle) -> bool {
    directories.required.is_none()
}

fn is_fast_self_directory_bundle(directory: &DirectoryBundle) -> bool {
    directory.required.is_none()
        && directory.allowed_names.is_none()
        && directory.allowed_patterns.is_none()
        && directory.forbidden_patterns.is_none()
        && directory.allow_extra.is_none()
}
