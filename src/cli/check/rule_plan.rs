//! Compiled structure-rule scopes for the full validation engine.

use super::rules::{
    dir_contains, join_config_child, merge_directory_bundle, merge_file_bundle,
    merge_markdown_bundle, normalize_config_dir, strip_direct_content_policy, EffectiveRules,
};
use super::scope_patterns::{
    path_has_matching_scope_ancestor, path_has_scope_magic, path_matches_scope_pattern,
    path_scope_specificity,
};
use crate::config::config::{Config, DirectoryNode};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub(super) struct RuleScope {
    path: PathBuf,
    inherit: bool,
    exact: EffectiveRules,
    descendant: EffectiveRules,
}

impl RuleScope {
    pub(super) fn new(
        path: PathBuf,
        inherit: bool,
        exact: EffectiveRules,
        descendant: EffectiveRules,
    ) -> Self {
        Self {
            path,
            inherit,
            exact,
            descendant,
        }
    }

    pub(super) fn parts(&self) -> (&Path, bool, &EffectiveRules, &EffectiveRules) {
        (&self.path, self.inherit, &self.exact, &self.descendant)
    }
}

pub(super) fn compile_rule_scopes(config: &Config) -> Vec<RuleScope> {
    let mut scopes = Vec::new();
    for (path, node) in &config.structure {
        let base = normalize_config_dir(path);
        compile_scope_node(base, node, &EffectiveRules::default(), &mut scopes);
    }
    scopes.sort_by_key(|scope| path_scope_specificity(&scope.path));
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

pub(super) fn scope_match(scope: &RuleScope, dir_rel: &Path) -> Option<bool> {
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
        node.inherit,
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
    use crate::config::loader::ConfigLoader;
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

    #[test]
    fn dynamic_directory_scopes_apply_normalized_captured_rule_fragments() {
        let config = ConfigLoader::parse(
            r#"
version: "2.0"
rules:
  "@assura-skill-dir":
    SKILL.md: exists:1
    agents/: exists:0-1
    references/: exists:0-1
    scripts/: exists:0-1
    assets/: exists:0-1
    extra: false
structure:
  .agents/skills/:
    extra: true
    "{skill}/":
      use: "@assura-skill-dir"
"#,
        )
        .unwrap();

        let scopes = compile_rule_scopes(&config);
        let debug_scopes = scopes
            .iter()
            .map(|scope| {
                let (path, _, exact, _) = scope.parts();
                format!(
                    "{} files_exists={:?} dir_allowed={:?}",
                    path.display(),
                    exact.files.as_ref().and_then(|files| files.exists.as_ref()),
                    exact
                        .directories
                        .as_ref()
                        .and_then(|directories| directories.allowed_names.as_ref())
                )
            })
            .collect::<Vec<_>>();
        assert!(
            scopes.iter().any(|scope| {
                scope.parts().0 == Path::new(".agents/skills/{skill}")
                    && scope
                        .parts()
                        .2
                        .files
                        .as_ref()
                        .and_then(|files| files.exists.as_ref())
                        .and_then(|exists| exists.get("SKILL.md"))
                        == Some(&"1".to_string())
            }),
            "compiled scopes should include the captured skill rule: {debug_scopes:#?}"
        );
        let rules = rules_for_dir(
            Path::new(".agents/skills/assura-project-maintenance"),
            &scopes,
        );
        assert_eq!(
            rules
                .files
                .as_ref()
                .and_then(|files| files.exists.as_ref())
                .and_then(|exists| exists.get("SKILL.md")),
            Some(&"1".to_string())
        );
        assert_eq!(
            rules
                .directories
                .as_ref()
                .and_then(|directories| directories.allowed_names.as_ref())
                .map(Vec::as_slice),
            Some(
                &[
                    "agents".to_string(),
                    "references".to_string(),
                    "scripts".to_string(),
                    "assets".to_string()
                ][..]
            )
        );
    }

    #[test]
    fn literal_scopes_override_captured_scopes_at_same_depth() {
        let config = ConfigLoader::parse(
            r#"
version: "2.0"
structure:
  .agents/skills/:
    "{skill}/":
      SKILL.md: exists:1
    special-skill/:
      README.md: exists:1
"#,
        )
        .unwrap();

        let scopes = compile_rule_scopes(&config);
        let rules = rules_for_dir(Path::new(".agents/skills/special-skill"), &scopes);

        assert_eq!(
            rules
                .files
                .as_ref()
                .and_then(|files| files.exists.as_ref())
                .and_then(|exists| exists.get("README.md")),
            Some(&"1".to_string())
        );
        assert!(
            rules
                .files
                .as_ref()
                .and_then(|files| files.exists.as_ref())
                .and_then(|exists| exists.get("SKILL.md"))
                .is_none(),
            "literal same-depth scope should win over the captured default"
        );
    }

    #[test]
    fn constrained_patterns_override_long_capture_names_at_same_depth() {
        let config = ConfigLoader::parse(
            r#"
version: "2.0"
structure:
  .agents/skills/:
    "{very_long_capture_name}/":
      SKILL.md: exists:1
    "release-*/":
      README.md: exists:1
"#,
        )
        .unwrap();

        let scopes = compile_rule_scopes(&config);
        let rules = rules_for_dir(Path::new(".agents/skills/release-maintenance"), &scopes);

        assert_eq!(
            rules
                .files
                .as_ref()
                .and_then(|files| files.exists.as_ref())
                .and_then(|exists| exists.get("README.md")),
            Some(&"1".to_string())
        );
        assert!(
            rules
                .files
                .as_ref()
                .and_then(|files| files.exists.as_ref())
                .and_then(|exists| exists.get("SKILL.md"))
                .is_none(),
            "capture variable names should not add literal specificity"
        );
    }
}
