//! Regression tests for performance-report fixture enumeration and metadata.

use super::fixtures::{materialize_fixture, scenarios, FixtureKind};
use std::fs;

#[test]
fn default_scenarios_include_rich_monorepo_policy_without_external_repos() {
    let scenarios = scenarios(false);
    assert!(scenarios
        .iter()
        .any(|scenario| scenario.id == "monorepo_policy"));
    assert!(!scenarios.iter().any(|scenario| matches!(
        scenario.kind,
        FixtureKind::PinnedNextJs | FixtureKind::PinnedMdBook
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
    let next = scenarios
        .iter()
        .find(|scenario| scenario.id == "pinned_nextjs")
        .expect("Next.js fixture should be opt-in");
    let mdbook = scenarios
        .iter()
        .find(|scenario| scenario.id == "pinned_mdbook")
        .expect("mdBook fixture should be opt-in");

    assert_eq!(
        next.source_revision,
        "ea8bc0ec2bbae18dd6861db15d66b92c36feeeb8"
    );
    assert_eq!(
        mdbook.source_revision,
        "b7a27d2759e80d804a33a4bc9c31b2b6863a5cb2"
    );
    assert!(matches!(next.kind, FixtureKind::PinnedNextJs));
    assert!(matches!(mdbook.kind, FixtureKind::PinnedMdBook));
}
