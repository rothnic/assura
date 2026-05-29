//! Portable compiled structure-check plan artifacts.

use super::compiled_artifact::{
    PortableDirectoryBundle, PortableFileBundle, PortableMarkdownBundle,
};
use super::compiled_config::{CompiledStructureConfig, PrecompiledStructurePlan};
use super::ls_fast_naming::{compile_fast_naming, FastFileNaming};
use super::ls_fast_plan::{
    collect_fast_regex_patterns, compile_lslint_fast_scopes, FastRules, FastScope,
};
use super::patterns;
use super::rule_plan::{compile_rule_scopes, RuleScope};
use super::rules::{collect_configured_dirs, collect_naming_regexes, EffectiveRules};
use crate::config::config::{Config, DirectoryBundle, DirectoryNode, FileBundle, MarkdownBundle};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(super) struct PortableCompiledPlan {
    configured_dirs: Vec<String>,
    exclusion_patterns: Vec<String>,
    naming_regex_patterns: Vec<String>,
    glob_pattern_sources: Vec<String>,
    rule_scopes: Vec<PortableRuleScope>,
    lslint_fast_scopes: Option<Vec<PortableFastScope>>,
    has_direct_count_constraints: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableRuleScope {
    path: String,
    exact: PortableEffectiveRules,
    descendant: PortableEffectiveRules,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableEffectiveRules {
    files: Option<PortableFileBundle>,
    directories: Option<PortableDirectoryBundle>,
    self_directory: Option<PortableDirectoryBundle>,
    markdown: Option<PortableMarkdownBundle>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableFastScope {
    path: String,
    exact: PortableFastRules,
    descendant: PortableFastRules,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableFastRules {
    effective: PortableEffectiveRules,
    file_naming: Option<PortableFastFileNaming>,
    directory_naming: Option<String>,
    self_directory_naming: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PortableFastFileNaming {
    suffix_patterns: Vec<(String, String)>,
    glob_patterns: Vec<(String, String)>,
    default: Option<String>,
}

impl PortableCompiledPlan {
    pub(super) fn from_config(config: &Config) -> Self {
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
        let mut configured_dirs = configured_dirs
            .into_iter()
            .map(path_to_portable)
            .collect::<Vec<_>>();
        configured_dirs.sort();
        let mut glob_pattern_sources = glob_patterns.into_keys().collect::<Vec<_>>();
        glob_pattern_sources.sort();

        let lslint_fast_scopes = compile_lslint_fast_scopes(config);
        let mut naming_regex_patterns = lslint_fast_scopes
            .as_deref()
            .map(collect_fast_regex_patterns)
            .unwrap_or_else(|| naming_regexes.into_keys().collect::<Vec<_>>());
        naming_regex_patterns.sort();
        let rule_scopes = if lslint_fast_scopes.is_some() {
            Vec::new()
        } else {
            compile_rule_scopes(config)
                .into_iter()
                .map(Into::into)
                .collect()
        };

        Self {
            configured_dirs,
            exclusion_patterns: config.exclude.clone(),
            naming_regex_patterns,
            glob_pattern_sources,
            rule_scopes,
            lslint_fast_scopes: lslint_fast_scopes
                .map(|scopes| scopes.into_iter().map(Into::into).collect()),
            has_direct_count_constraints,
        }
    }

    pub(super) fn is_fast_only(&self) -> bool {
        self.lslint_fast_scopes.is_some() && self.rule_scopes.is_empty()
    }

    pub(super) fn can_run_without_config(&self, fail_fast: bool) -> bool {
        !fail_fast && self.lslint_fast_scopes.is_some()
    }

    pub(super) fn into_compiled_config(
        self,
        config: Config,
        fail_fast: bool,
    ) -> CompiledStructureConfig {
        let plan = PrecompiledStructurePlan {
            configured_dirs: self
                .configured_dirs
                .into_iter()
                .map(PathBuf::from)
                .collect(),
            exclusion_patterns: self.exclusion_patterns,
            naming_regex_patterns: self.naming_regex_patterns,
            glob_pattern_sources: self.glob_pattern_sources,
            rule_scopes: if fail_fast || self.lslint_fast_scopes.is_none() {
                if self.rule_scopes.is_empty() {
                    compile_rule_scopes(&config)
                } else {
                    self.rule_scopes.into_iter().map(Into::into).collect()
                }
            } else {
                Vec::new()
            },
            lslint_fast_scopes: self
                .lslint_fast_scopes
                .map(|scopes| scopes.into_iter().map(Into::into).collect()),
            has_direct_count_constraints: self.has_direct_count_constraints,
        };
        CompiledStructureConfig::from_precompiled_plan(config, plan, fail_fast)
    }
}

impl From<RuleScope> for PortableRuleScope {
    fn from(scope: RuleScope) -> Self {
        let (path, exact, descendant) = scope.parts();
        Self {
            path: path_to_portable(path.to_path_buf()),
            exact: exact.into(),
            descendant: descendant.into(),
        }
    }
}

impl From<PortableRuleScope> for RuleScope {
    fn from(scope: PortableRuleScope) -> Self {
        RuleScope::new(
            PathBuf::from(scope.path),
            scope.exact.into(),
            scope.descendant.into(),
        )
    }
}

impl From<&EffectiveRules> for PortableEffectiveRules {
    fn from(rules: &EffectiveRules) -> Self {
        Self {
            files: rules
                .files
                .as_ref()
                .map(|files| files.as_ref().clone().into()),
            directories: rules
                .directories
                .as_ref()
                .map(|directories| directories.as_ref().clone().into()),
            self_directory: rules
                .self_directory
                .as_ref()
                .map(|directory| directory.as_ref().clone().into()),
            markdown: rules
                .markdown
                .as_ref()
                .map(|markdown| markdown.as_ref().clone().into()),
        }
    }
}

impl From<PortableEffectiveRules> for EffectiveRules {
    fn from(rules: PortableEffectiveRules) -> Self {
        Self {
            files: rules.files.map(FileBundle::from).map(Arc::new),
            directories: rules.directories.map(DirectoryBundle::from).map(Arc::new),
            self_directory: rules
                .self_directory
                .map(DirectoryBundle::from)
                .map(Arc::new),
            markdown: rules.markdown.map(MarkdownBundle::from).map(Arc::new),
        }
    }
}

impl From<FastScope> for PortableFastScope {
    fn from(scope: FastScope) -> Self {
        let (path, exact, descendant) = scope.parts();
        Self {
            path: path_to_portable(path.to_path_buf()),
            exact: exact.into(),
            descendant: descendant.into(),
        }
    }
}

impl From<PortableFastScope> for FastScope {
    fn from(scope: PortableFastScope) -> Self {
        FastScope::new(
            PathBuf::from(scope.path),
            scope.exact.into(),
            scope.descendant.into(),
        )
    }
}

impl From<&FastRules> for PortableFastRules {
    fn from(rules: &FastRules) -> Self {
        let (effective, file_naming, directory_naming, self_directory_naming) = rules.parts();
        Self {
            effective: effective.into(),
            file_naming: file_naming.map(Into::into),
            directory_naming: directory_naming.map(|naming| naming.label().to_string()),
            self_directory_naming: self_directory_naming.map(|naming| naming.label().to_string()),
        }
    }
}

impl From<PortableFastRules> for FastRules {
    fn from(rules: PortableFastRules) -> Self {
        FastRules::from_parts(
            rules.effective.into(),
            rules.file_naming.map(Into::into),
            rules.directory_naming.as_deref().map(compile_fast_naming),
            rules
                .self_directory_naming
                .as_deref()
                .map(compile_fast_naming),
        )
    }
}

impl From<&FastFileNaming> for PortableFastFileNaming {
    fn from(file_naming: &FastFileNaming) -> Self {
        let (suffix_patterns, glob_patterns, default) = file_naming.parts();
        let mut suffix_patterns = suffix_patterns
            .iter()
            .map(|(suffix, naming)| (suffix.clone(), naming.label().to_string()))
            .collect::<Vec<_>>();
        suffix_patterns.sort_by(|left, right| {
            right
                .0
                .len()
                .cmp(&left.0.len())
                .then_with(|| left.0.cmp(&right.0))
        });
        let glob_patterns = glob_patterns
            .iter()
            .map(|pattern| {
                let (source, naming) = pattern.parts();
                (source.to_string(), naming.label().to_string())
            })
            .collect();
        Self {
            suffix_patterns,
            glob_patterns,
            default: default.map(|naming| naming.label().to_string()),
        }
    }
}

impl From<PortableFastFileNaming> for FastFileNaming {
    fn from(file_naming: PortableFastFileNaming) -> Self {
        FastFileNaming::from_parts(
            file_naming
                .suffix_patterns
                .into_iter()
                .map(|(suffix, naming)| (suffix, compile_fast_naming(&naming)))
                .collect(),
            file_naming
                .glob_patterns
                .into_iter()
                .map(|(pattern, naming)| (pattern, compile_fast_naming(&naming)))
                .collect(),
            file_naming.default.as_deref().map(compile_fast_naming),
        )
    }
}

fn node_has_direct_count_constraints(node: &DirectoryNode) -> bool {
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
            .children
            .as_ref()
            .is_some_and(|children| children.values().any(node_has_direct_count_constraints))
}

fn path_to_portable(path: PathBuf) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}
