//! Runtime compiled structure-check configuration.

use super::ls_fast_plan::{collect_fast_regex_patterns, compile_lslint_fast_scopes, FastScope};
use super::patterns;
use super::rule_plan::{compile_rule_scopes, RuleScope};
use super::rules::{collect_configured_dirs, collect_naming_regexes, CompiledExclusion};
use crate::config::config::Config;
use glob::Pattern;
use regex_lite::Regex;
use std::collections::HashMap;
use std::path::PathBuf;

const TOOL_STATE_EXCLUSIONS: &[&str] = &[".assura/**"];

#[derive(Clone)]
pub(in crate::cli::check) struct CompiledStructureConfig {
    pub(super) config: Config,
    pub(super) configured_dirs: Vec<PathBuf>,
    pub(super) exclude_patterns: Vec<CompiledExclusion>,
    pub(super) naming_regexes: HashMap<String, Regex>,
    pub(super) glob_patterns: HashMap<String, Pattern>,
    pub(super) rule_scopes: Vec<RuleScope>,
    pub(super) lslint_fast_scopes: Option<Vec<FastScope>>,
    pub(super) has_direct_count_constraints: bool,
}

pub(super) struct PrecompiledStructurePlan {
    pub(super) configured_dirs: Vec<PathBuf>,
    pub(super) exclusion_patterns: Vec<String>,
    pub(super) naming_regex_patterns: Vec<String>,
    pub(super) glob_pattern_sources: Vec<String>,
    pub(super) rule_scopes: Vec<RuleScope>,
    pub(super) lslint_fast_scopes: Option<Vec<FastScope>>,
    pub(super) has_direct_count_constraints: bool,
}

impl CompiledStructureConfig {
    #[cfg(feature = "yaml-config")]
    pub(in crate::cli::check) fn new(config: Config, fail_fast: bool) -> Self {
        Self::new_with_options(config, fail_fast, true)
    }

    pub(super) fn new_for_check(config: Config, fail_fast: bool) -> Self {
        Self::new_with_options(config, fail_fast, false)
    }

    fn new_with_options(config: Config, fail_fast: bool, keep_full_rule_plan: bool) -> Self {
        let mut configured_dirs = Vec::new();
        let mut naming_regexes = HashMap::new();
        let mut glob_patterns = HashMap::new();
        let mut has_direct_count_constraints = false;
        for (path, node) in &config.structure {
            let base = super::normalize_config_dir(path);
            collect_configured_dirs(base.clone(), node, &mut configured_dirs);
            collect_naming_regexes(node, &mut naming_regexes);
            patterns::collect_glob_patterns(node, &mut glob_patterns);
            has_direct_count_constraints |= node_has_direct_count_constraints(node);
        }
        configured_dirs.sort();
        configured_dirs.dedup();

        let lslint_fast_scopes = if fail_fast {
            None
        } else {
            compile_lslint_fast_scopes(&config)
        };
        if !keep_full_rule_plan {
            if let Some(scopes) = lslint_fast_scopes.as_deref() {
                let required_patterns = collect_fast_regex_patterns(scopes);
                naming_regexes
                    .retain(|pattern, _| required_patterns.binary_search(pattern).is_ok());
            }
        }
        let exclude_patterns = compile_exclusions(config.exclude.iter().map(String::as_str));
        let rule_scopes = if keep_full_rule_plan || lslint_fast_scopes.is_none() {
            compile_rule_scopes(&config)
        } else {
            Vec::new()
        };

        Self {
            config,
            configured_dirs,
            exclude_patterns,
            naming_regexes,
            glob_patterns,
            rule_scopes,
            lslint_fast_scopes,
            has_direct_count_constraints,
        }
    }

    pub(super) fn from_precompiled_plan(
        config: Config,
        plan: PrecompiledStructurePlan,
        fail_fast: bool,
    ) -> Self {
        let exclude_patterns =
            compile_exclusions(plan.exclusion_patterns.iter().map(String::as_str));
        let naming_regexes = plan
            .naming_regex_patterns
            .into_iter()
            .filter_map(|pattern| Regex::new(&pattern).ok().map(|regex| (pattern, regex)))
            .collect();
        let glob_patterns = plan
            .glob_pattern_sources
            .into_iter()
            .filter_map(|pattern| {
                Pattern::new(&pattern)
                    .ok()
                    .map(|compiled| (pattern, compiled))
            })
            .collect();
        let lslint_fast_scopes = (!fail_fast).then_some(plan.lslint_fast_scopes).flatten();

        Self {
            config,
            configured_dirs: plan.configured_dirs,
            exclude_patterns,
            naming_regexes,
            glob_patterns,
            rule_scopes: plan.rule_scopes,
            lslint_fast_scopes,
            has_direct_count_constraints: plan.has_direct_count_constraints,
        }
    }
}

fn compile_exclusions<'a>(configured: impl Iterator<Item = &'a str>) -> Vec<CompiledExclusion> {
    TOOL_STATE_EXCLUSIONS
        .iter()
        .copied()
        .chain(configured)
        .map(CompiledExclusion::new)
        .collect()
}

fn node_has_direct_count_constraints(node: &crate::config::config::DirectoryNode) -> bool {
    node.files
        .as_ref()
        .and_then(|files| files.exists.as_ref())
        .is_some()
        || node
            .directories
            .as_ref()
            .and_then(|directories| directories.exists.as_ref())
            .is_some()
        || node
            .self_directory
            .as_ref()
            .and_then(|directory| directory.exists.as_ref())
            .is_some()
        || node
            .children
            .as_ref()
            .is_some_and(|children| children.values().any(node_has_direct_count_constraints))
}
