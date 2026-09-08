//! Tests for direct-content policy handling in the LS-Lint fast plan.

use super::ls_fast_direct_content::strip_direct_content_policy_is_noop;
use super::ls_fast_plan::{compile_lslint_fast_scopes, fast_rules_for_dir};
use super::rules::EffectiveRules;
use crate::config::config::{Config, DirectoryBundle, DirectoryNode, FileBundle};
use crate::config::types::ChildrenLimitConfig;
use std::sync::Arc;

#[test]
fn fast_plan_reuses_noop_direct_content_rule_bundles() {
    let mut config = Config::new();
    config.structure.insert(
        "naming-only".to_string(),
        DirectoryNode {
            files: Some(FileBundle {
                naming: Some("kebab-case".to_string()),
                ..FileBundle::default()
            }),
            inherit: false,
            ..DirectoryNode::default()
        },
    );

    let scopes = compile_lslint_fast_scopes(&config).expect("naming-only config is fast-plan safe");
    let (_, exact, descendant) = scopes
        .iter()
        .find_map(|scope| {
            let (path, exact, descendant) = scope.parts();
            (path == std::path::Path::new("naming-only")).then_some((path, exact, descendant))
        })
        .expect("naming-only scope exists");

    assert!(Arc::ptr_eq(
        exact.effective.files.as_ref().expect("exact files"),
        descendant
            .effective
            .files
            .as_ref()
            .expect("descendant files"),
    ));
}

#[test]
fn fast_plan_strips_direct_content_policy_from_descendants() {
    let config = Config::new().with_node(
        "pkg",
        DirectoryNode::new().with_files(FileBundle {
            naming: Some("kebab-case".to_string()),
            allowed_names: Some(vec!["README.md".to_string()]),
            allow_extra: Some(false),
            ..FileBundle::default()
        }),
    );

    let scopes =
        compile_lslint_fast_scopes(&config).expect("direct-content config is fast-plan safe");
    let exact = fast_rules_for_dir(std::path::Path::new("pkg"), &scopes).expect("exact rules");
    let descendant =
        fast_rules_for_dir(std::path::Path::new("pkg/nested"), &scopes).expect("descendant rules");

    assert_eq!(
        exact
            .effective
            .files
            .as_ref()
            .and_then(|files| files.allowed_names.as_ref()),
        Some(&vec!["README.md".to_string()])
    );
    assert!(descendant
        .effective
        .files
        .as_ref()
        .and_then(|files| files.allowed_names.as_ref())
        .is_none());
    assert!(descendant
        .effective
        .files
        .as_ref()
        .and_then(|files| files.allow_extra)
        .is_none());
    assert_eq!(
        descendant
            .effective
            .files
            .as_ref()
            .and_then(|files| files.naming.as_deref()),
        Some("kebab-case")
    );
}

#[test]
fn direct_content_noop_predicate_rejects_every_explicit_direct_constraint() {
    assert!(strip_direct_content_policy_is_noop(
        &EffectiveRules::default()
    ));

    let file_cases = [
        FileBundle {
            required: Some(Vec::new()),
            ..FileBundle::default()
        },
        FileBundle {
            allowed_names: Some(Vec::new()),
            ..FileBundle::default()
        },
        FileBundle {
            allowed_patterns: Some(Vec::new()),
            ..FileBundle::default()
        },
        FileBundle {
            forbidden_patterns: Some(Vec::new()),
            ..FileBundle::default()
        },
        FileBundle {
            allow_extra: Some(false),
            ..FileBundle::default()
        },
        FileBundle {
            exists: Some(Default::default()),
            ..FileBundle::default()
        },
    ];
    for files in file_cases {
        assert!(!strip_direct_content_policy_is_noop(&EffectiveRules {
            files: Some(Arc::new(files)),
            ..EffectiveRules::default()
        }));
    }

    let directory_cases = [
        DirectoryBundle {
            required: Some(Vec::new()),
            ..DirectoryBundle::default()
        },
        DirectoryBundle {
            allowed_names: Some(Vec::new()),
            ..DirectoryBundle::default()
        },
        DirectoryBundle {
            allowed_patterns: Some(Vec::new()),
            ..DirectoryBundle::default()
        },
        DirectoryBundle {
            forbidden_patterns: Some(Vec::new()),
            ..DirectoryBundle::default()
        },
        DirectoryBundle {
            allow_extra: Some(false),
            ..DirectoryBundle::default()
        },
        DirectoryBundle {
            exists: Some(Default::default()),
            ..DirectoryBundle::default()
        },
    ];
    for directories in directory_cases {
        assert!(!strip_direct_content_policy_is_noop(&EffectiveRules {
            directories: Some(Arc::new(directories.clone())),
            ..EffectiveRules::default()
        }));
        assert!(!strip_direct_content_policy_is_noop(&EffectiveRules {
            self_directory: Some(Arc::new(directories)),
            ..EffectiveRules::default()
        }));
    }

    assert!(!strip_direct_content_policy_is_noop(&EffectiveRules {
        limit_children: Some(Arc::new(ChildrenLimitConfig::new())),
        ..EffectiveRules::default()
    }));
}
