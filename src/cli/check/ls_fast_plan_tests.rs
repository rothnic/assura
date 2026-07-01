//! Tests for the LS-Lint-compatible fast plan compiler.

use super::ls_fast_naming::{compile_fast_naming, validate_fast_name};
use super::ls_fast_plan::{
    collect_fast_regex_patterns, compile_lslint_fast_scopes, fast_rules_for_dir, FastRules,
};
use super::rules::EffectiveRules;
use crate::config::config::{
    Config, DirectoryBundle, DirectoryNode, ExtensionConfig, FileBundle, RepositoryReferenceConfig,
};
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn fast_file_naming_prefers_longest_suffix_match() {
    let effective = EffectiveRules {
        files: Some(Arc::new(FileBundle {
            naming_patterns: Some(HashMap::from([
                ("*.ts".to_string(), "snake_case".to_string()),
                ("*.kind-01.ts".to_string(), "kebab-case".to_string()),
            ])),
            ..FileBundle::default()
        })),
        ..EffectiveRules::default()
    };

    let rules = FastRules::new(effective);
    let naming = rules
        .file_naming
        .as_ref()
        .and_then(|file_naming| file_naming.naming_for("feature-01.kind-01.ts", &HashMap::new()))
        .unwrap();

    assert_eq!(naming.naming.label(), "kebab-case");
}

#[test]
fn fast_file_naming_uses_exact_extension_segments() {
    let effective = EffectiveRules {
        files: Some(Arc::new(FileBundle {
            naming_patterns: Some(HashMap::from([
                ("*.ts".to_string(), "snake_case".to_string()),
                ("*.kind-01.ts".to_string(), "kebab-case".to_string()),
            ])),
            ..FileBundle::default()
        })),
        ..EffectiveRules::default()
    };

    let rules = FastRules::new(effective);
    let file_naming = rules.file_naming.as_ref().unwrap();

    let kind_match = file_naming
        .naming_for("feature-01.kind-01.ts", &HashMap::new())
        .unwrap();
    assert_eq!(kind_match.naming.label(), "kebab-case");

    let plain_match = file_naming
        .naming_for("feature_01.ts", &HashMap::new())
        .unwrap();
    assert_eq!(plain_match.naming.label(), "snake_case");
}

#[test]
fn fast_plan_allows_direct_file_and_directory_policies() {
    let mut config = Config::new();
    config.structure.insert(
        "./".to_string(),
        DirectoryNode {
            files: Some(FileBundle {
                allowed_names: Some(vec!["README.md".to_string()]),
                allowed_patterns: Some(vec!["*.toml".to_string()]),
                forbidden_patterns: Some(vec!["*.tmp".to_string()]),
                allow_extra: Some(false),
                ..FileBundle::default()
            }),
            directories: Some(DirectoryBundle {
                allowed_names: Some(vec!["src".to_string()]),
                allowed_patterns: Some(vec!["packages-*".to_string()]),
                forbidden_patterns: Some(vec!["dist".to_string()]),
                allow_extra: Some(false),
                ..DirectoryBundle::default()
            }),
            ..DirectoryNode::default()
        },
    );

    let scopes = compile_lslint_fast_scopes(&config).unwrap();
    assert_eq!(scopes.len(), 1);
}

#[test]
fn fast_plan_rejects_repository_reference_diagnostics() {
    let mut config = Config::new();
    config.structure.insert(
        "./".to_string(),
        DirectoryNode {
            files: Some(FileBundle {
                allowed_names: Some(vec!["README.md".to_string()]),
                ..FileBundle::default()
            }),
            ..DirectoryNode::default()
        },
    );
    config.extensions = Some(ExtensionConfig {
        repository_references: vec![RepositoryReferenceConfig {
            id: "source_refs".to_string(),
            paths: vec!["src/**".to_string()],
            severity: Some("high".to_string()),
        }],
        ..ExtensionConfig::default()
    });

    assert!(compile_lslint_fast_scopes(&config).is_none());
}

#[test]
fn fast_plan_supports_wildcard_directory_scopes() {
    let mut packages_children = HashMap::new();
    packages_children.insert(
        "*".to_string(),
        DirectoryNode {
            files: Some(FileBundle {
                naming_patterns: Some(HashMap::from([(
                    "*.ts".to_string(),
                    "kebab-case".to_string(),
                )])),
                ..FileBundle::default()
            }),
            ..DirectoryNode::default()
        },
    );

    let mut config = Config::new();
    config.structure.insert(
        "./".to_string(),
        DirectoryNode {
            children: Some(HashMap::from([(
                "packages".to_string(),
                DirectoryNode {
                    children: Some(packages_children),
                    ..DirectoryNode::default()
                },
            )])),
            ..DirectoryNode::default()
        },
    );

    let scopes = compile_lslint_fast_scopes(&config).unwrap();
    let package_rules = fast_rules_for_dir(std::path::Path::new("packages/core"), &scopes)
        .expect("wildcard package scope should match");
    assert!(package_rules.file_naming.is_some());

    let descendant_rules = fast_rules_for_dir(std::path::Path::new("packages/core/src"), &scopes)
        .expect("wildcard package descendant should inherit");
    assert!(descendant_rules.file_naming.is_some());
}

#[test]
fn fast_naming_simplifies_common_regex_literals() {
    let empty = compile_fast_naming("regex:^$");
    assert!(validate_fast_name("", "", &empty, &HashMap::new()));
    assert!(!validate_fast_name("index", "", &empty, &HashMap::new()));

    let contains = compile_fast_naming("regex:(next\\.config|postcss\\.config)");
    assert!(validate_fast_name(
        "next.config",
        "",
        &contains,
        &HashMap::new()
    ));
    assert!(validate_fast_name(
        "postcss.config",
        "",
        &contains,
        &HashMap::new()
    ));
    assert!(!validate_fast_name(
        "tailwind.config",
        "",
        &contains,
        &HashMap::new()
    ));

    let exact = compile_fast_naming("regex:^(README|AGENTS)$");
    assert!(validate_fast_name("README", "", &exact, &HashMap::new()));
    assert!(!validate_fast_name("MYREADME", "", &exact, &HashMap::new()));
}

#[test]
fn fast_regex_collection_skips_simplified_regexes() {
    let mut config = Config::new();
    config.structure.insert(
        "./".to_string(),
        DirectoryNode {
            files: Some(FileBundle {
                naming_patterns: Some(HashMap::from([
                    ("*.js".to_string(), "regex:^$".to_string()),
                    ("*.md".to_string(), "regex:(README|AGENTS)".to_string()),
                    ("*.custom".to_string(), "regex:^[a-z]+-[0-9]+$".to_string()),
                ])),
                ..FileBundle::default()
            }),
            ..DirectoryNode::default()
        },
    );

    let scopes = compile_lslint_fast_scopes(&config).unwrap();
    assert_eq!(
        collect_fast_regex_patterns(&scopes),
        vec!["^[a-z]+-[0-9]+$".to_string()]
    );
}
