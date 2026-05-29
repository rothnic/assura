//! Regression tests for performance-report fixture enumeration and metadata.

use super::external_fixture_catalog::external_fixture_specs;
use super::fixtures::{materialize_fixture, scenarios};
use std::collections::HashSet;
use std::fs;

#[test]
fn default_scenarios_include_rich_monorepo_policy_without_external_repos() {
    let scenarios = scenarios(false);
    assert!(scenarios
        .iter()
        .any(|scenario| scenario.id == "monorepo_policy"));
    assert!(!scenarios.iter().any(|scenario| matches!(
        scenario.kind,
        kind if kind.is_external_pinned()
    )));

    let fixture = materialize_fixture(
        scenarios
            .into_iter()
            .find(|scenario| scenario.id == "monorepo_policy")
            .unwrap(),
    )
    .unwrap();

    assert_eq!(fixture.metadata.source_type, "generated");
    assert_eq!(fixture.metadata.source_revision, "generated-fixtures-v2");
    assert_eq!(fixture.metadata.cohort, "realistic-equivalent");
    assert_eq!(fixture.metadata.rule_count, 38);
    assert!(fixture.metadata.checked_file_count > 30);
    assert!(fixture.metadata.ignored_file_count > 0);
    assert_eq!(
        fixture.metadata.config_generation_method,
        "hand-authored-equivalent-pair"
    );
    let ls_lint_config = fs::read_to_string(fixture.root.join(".ls-lint.yml")).unwrap();
    assert!(ls_lint_config.contains("regex:^$"));
    assert!(ls_lint_config
        .contains(r".dir: regex:^(\.github|apps|packages|docs|scripts|infra|config|tests|node_modules|dist|coverage)$"));
    assert!(ls_lint_config.contains(
        r".*: regex:^(README|AGENTS|CONTRIBUTING|LICENSE|package|pnpm-lock|tsconfig|turbo)$"
    ));
    let _ = fs::remove_dir_all(fixture.root);
}

#[test]
fn external_scenarios_are_opt_in_and_pinned() {
    let scenarios = scenarios(true);
    let external = scenarios
        .iter()
        .filter(|scenario| scenario.kind.is_external_pinned())
        .collect::<Vec<_>>();
    let specs = external_fixture_specs();
    let spec_ids = specs
        .iter()
        .map(|spec| spec.fixture_id)
        .collect::<HashSet<_>>();
    let spec_revisions = specs
        .iter()
        .map(|spec| (spec.fixture_id, spec.revision))
        .collect::<std::collections::HashMap<_, _>>();
    let config_bodies = specs
        .iter()
        .map(|spec| spec.ls_lint_config)
        .collect::<HashSet<_>>();

    assert_eq!(external.len(), 10);
    assert_eq!(specs.len(), 10);
    assert_eq!(
        config_bodies.len(),
        10,
        "each real repo needs a unique policy"
    );

    for scenario in external {
        assert!(
            spec_ids.contains(scenario.id),
            "missing external spec for {}",
            scenario.id
        );
        assert_eq!(
            Some(&scenario.source_revision),
            spec_revisions.get(scenario.id),
            "scenario revision must match external spec for {}",
            scenario.id
        );
    }
}
