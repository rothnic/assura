//! Compiled structure-rule scopes for the full validation engine.

use super::rules::{
    dir_contains, join_config_child, merge_directory_bundle, merge_file_bundle,
    merge_markdown_bundle, normalize_config_dir, strip_direct_content_policy, EffectiveRules,
};
use super::scope_patterns::{
    path_has_matching_scope_ancestor, path_has_scope_magic, path_matches_scope_pattern,
};
use crate::config::config::{Config, DirectoryNode};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(super) struct RuleScope {
    path: PathBuf,
    exact: EffectiveRules,
    descendant: EffectiveRules,
}

impl RuleScope {
    pub(super) fn new(path: PathBuf, exact: EffectiveRules, descendant: EffectiveRules) -> Self {
        Self {
            path,
            exact,
            descendant,
        }
    }

    pub(super) fn parts(&self) -> (&Path, &EffectiveRules, &EffectiveRules) {
        (&self.path, &self.exact, &self.descendant)
    }
}

pub(super) fn compile_rule_scopes(config: &Config) -> Vec<RuleScope> {
    let mut scopes = Vec::new();
    for (path, node) in &config.structure {
        let base = normalize_config_dir(path);
        compile_scope_node(base, node, &EffectiveRules::default(), &mut scopes);
    }
    scopes
}

pub(super) fn rules_for_dir(dir_rel: &Path, scopes: &[RuleScope]) -> EffectiveRules {
    scopes
        .iter()
        .rev()
        .filter_map(|scope| scope_match(scope, dir_rel).map(|exact| (scope, exact)))
        .map(|scope| {
            if scope.1 {
                scope.0.exact.clone()
            } else {
                scope.0.descendant.clone()
            }
        })
        .next()
        .unwrap_or_default()
}

fn scope_match(scope: &RuleScope, dir_rel: &Path) -> Option<bool> {
    if path_has_scope_magic(&scope.path) {
        if path_matches_scope_pattern(&scope.path, dir_rel) {
            return Some(true);
        }
        return path_has_matching_scope_ancestor(&scope.path, dir_rel).then_some(false);
    }

    dir_contains(&scope.path, dir_rel).then_some(dir_rel == scope.path)
}

fn compile_scope_node(
    node_rel: PathBuf,
    node: &DirectoryNode,
    inherited: &EffectiveRules,
    scopes: &mut Vec<RuleScope>,
) {
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

    scopes.push(RuleScope::new(
        node_rel.clone(),
        effective.clone(),
        strip_direct_content_policy(effective.clone()),
    ));

    if let Some(children) = &node.children {
        for (child_name, child) in children {
            let child_rel = join_config_child(&node_rel, child_name);
            compile_scope_node(child_rel, child, &effective, scopes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::config::{DirectoryBundle, DirectoryNode, FileBundle};
    use std::collections::HashMap;

    #[test]
    fn compiled_scopes_inherit_parent_rules() {
        let mut children = HashMap::new();
        children.insert(
            "src".to_string(),
            DirectoryNode::new().with_files(FileBundle::new().with_naming("snake_case")),
        );
        let mut root =
            DirectoryNode::new().with_directories(DirectoryBundle::new().with_naming("kebab-case"));
        root.children = Some(children);
        let config = Config::new().with_node("./", root);

        let scopes = compile_rule_scopes(&config);
        let rules = rules_for_dir(Path::new("src"), &scopes);

        assert_eq!(
            rules
                .directories
                .as_ref()
                .and_then(|directories| directories.naming.as_deref()),
            Some("kebab-case")
        );
        assert_eq!(
            rules
                .files
                .as_ref()
                .and_then(|files| files.naming.as_deref()),
            Some("snake_case")
        );
    }

    #[test]
    fn compiled_scopes_strip_direct_content_policy_for_descendants() {
        let config = Config::new().with_node(
            "src",
            DirectoryNode::new().with_files(
                FileBundle::new()
                    .with_naming("snake_case")
                    .with_allowed_names(vec!["lib.rs".to_string()])
                    .with_allow_extra(false),
            ),
        );

        let scopes = compile_rule_scopes(&config);
        let exact = rules_for_dir(Path::new("src"), &scopes);
        let descendant = rules_for_dir(Path::new("src/nested"), &scopes);

        assert_eq!(
            exact
                .files
                .as_ref()
                .and_then(|files| files.allowed_names.as_ref())
                .map(Vec::len),
            Some(1)
        );
        assert_eq!(
            descendant
                .files
                .as_ref()
                .and_then(|files| files.naming.as_deref()),
            Some("snake_case")
        );
        assert!(descendant
            .files
            .as_ref()
            .and_then(|files| files.allowed_names.as_ref())
            .is_none());
    }
}
