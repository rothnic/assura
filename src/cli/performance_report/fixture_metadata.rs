//! Machine-readable fixture metadata for performance report rows.

use super::fixtures::{FixtureKind, FixtureMetadata, FixtureScenario};
use glob::Pattern;
use std::fs;
use std::path::Path;

pub(super) fn fixture_metadata(
    scenario: FixtureScenario,
    root: &Path,
) -> Result<FixtureMetadata, String> {
    let counts = count_fixture_entries(root, ignored_paths(scenario.kind))?;
    let source_revision = source_revision(scenario, root);
    Ok(FixtureMetadata {
        source_type: source_type(scenario.kind),
        source_revision: source_revision.clone(),
        cohort: fixture_cohort(scenario.kind),
        checked_file_count: counts.checked_file_count,
        ignored_file_count: counts.ignored_file_count,
        directory_count: counts.directory_count,
        rule_count: rule_count(scenario.kind),
        rule_surface_summary: rule_surface_summary(scenario.kind),
        native_ls_lint_parity: true,
        assura_config_path: ".assura/config.yml",
        ls_lint_config_path: ".ls-lint.yml",
        config_generation_method: config_generation_method(scenario.kind),
        shared_config_id: format!("{}:{}", source_revision, scenario.id),
        expected_assura_exit_status: 0,
        expected_ls_lint_exit_status: 0,
    })
}

struct FixtureCounts {
    checked_file_count: usize,
    ignored_file_count: usize,
    directory_count: usize,
}

fn count_fixture_entries(root: &Path, ignored_paths: &[&str]) -> Result<FixtureCounts, String> {
    let mut counts = FixtureCounts {
        checked_file_count: 0,
        ignored_file_count: 0,
        directory_count: 0,
    };

    for entry in walkdir::WalkDir::new(root).into_iter().skip(1) {
        let entry = entry.map_err(|error| error.to_string())?;
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| error.to_string())?;
        if entry.file_type().is_dir() {
            counts.directory_count += 1;
        } else if entry.file_type().is_file() {
            if is_ignored(relative, ignored_paths) {
                counts.ignored_file_count += 1;
            } else {
                counts.checked_file_count += 1;
            }
        }
    }

    Ok(counts)
}

fn is_ignored(relative: &Path, ignored_paths: &[&str]) -> bool {
    let relative = relative.to_string_lossy().replace('\\', "/");
    ignored_paths.iter().any(|ignored| {
        let ignored = ignored.trim_end_matches("/**").trim_end_matches('/');
        relative == ignored
            || relative.starts_with(&format!("{ignored}/"))
            || Pattern::new(ignored)
                .map(|pattern| pattern.matches(&relative))
                .unwrap_or(false)
            || Pattern::new(&format!("{ignored}/**"))
                .map(|pattern| pattern.matches(&relative))
                .unwrap_or(false)
    })
}

fn ignored_paths(kind: FixtureKind) -> &'static [&'static str] {
    match kind {
        FixtureKind::Sized | FixtureKind::RuleHeavy => &[".assura"],
        FixtureKind::IgnoredGenerated => &[".assura", "generated"],
        FixtureKind::SimpleLibrary => &[".assura", "target"],
        FixtureKind::WebApp => &[".assura", "dist"],
        FixtureKind::MonorepoPackages => &[".assura", "packages/core/dist", "packages/ui/dist"],
        FixtureKind::MonorepoPolicy => &[
            ".assura",
            ".ls-lint.yml",
            "node_modules",
            "apps/web-dashboard/node_modules",
            "apps/web-dashboard/.next",
            "apps/admin-console/dist",
            "packages/core/dist",
            "packages/ui-kit/.turbo",
            ".turbo",
            "coverage",
        ],
        FixtureKind::RuleHeavyRepo => &[".assura", ".ls-lint.yml"],
        FixtureKind::IgnoredGeneratedHeavyRepo => &[".assura", "generated", "coverage"],
        FixtureKind::PinnedNextJs => &[
            ".assura",
            ".git",
            "node_modules",
            "packages/*/node_modules",
            "examples/*/node_modules",
            "test/**/node_modules",
            ".next",
            "packages/*/.next",
            "examples/*/.next",
            "dist",
            "packages/*/dist",
            "coverage",
            ".turbo",
            ".vercel",
        ],
        FixtureKind::PinnedMdBook => &[".assura", ".git", "target"],
    }
}

fn source_type(kind: FixtureKind) -> &'static str {
    match kind {
        FixtureKind::PinnedNextJs | FixtureKind::PinnedMdBook => "external-pinned-repo",
        _ => "generated",
    }
}

fn source_revision(scenario: FixtureScenario, root: &Path) -> String {
    match scenario.kind {
        FixtureKind::PinnedNextJs | FixtureKind::PinnedMdBook => {
            fs::read_to_string(root.join(".assura/source-revision.txt"))
                .map(|value| value.trim().to_string())
                .unwrap_or_else(|_| scenario.source_revision.to_string())
        }
        _ => scenario.source_revision.to_string(),
    }
}

fn fixture_cohort(kind: FixtureKind) -> &'static str {
    match kind {
        FixtureKind::Sized | FixtureKind::RuleHeavy | FixtureKind::IgnoredGenerated => {
            "synthetic-stress"
        }
        FixtureKind::SimpleLibrary
        | FixtureKind::WebApp
        | FixtureKind::MonorepoPackages
        | FixtureKind::MonorepoPolicy
        | FixtureKind::RuleHeavyRepo
        | FixtureKind::IgnoredGeneratedHeavyRepo
        | FixtureKind::PinnedNextJs
        | FixtureKind::PinnedMdBook => "realistic-equivalent",
    }
}

fn rule_count(kind: FixtureKind) -> usize {
    match kind {
        FixtureKind::Sized
        | FixtureKind::IgnoredGenerated
        | FixtureKind::IgnoredGeneratedHeavyRepo => 2,
        FixtureKind::RuleHeavy => 31,
        FixtureKind::SimpleLibrary => 6,
        FixtureKind::WebApp => 7,
        FixtureKind::MonorepoPackages => 10,
        FixtureKind::MonorepoPolicy => 38,
        FixtureKind::RuleHeavyRepo => 38,
        FixtureKind::PinnedNextJs => 2,
        FixtureKind::PinnedMdBook => 5,
    }
}

fn rule_surface_summary(kind: FixtureKind) -> &'static str {
    match kind {
        FixtureKind::Sized => "directory naming plus TypeScript file naming",
        FixtureKind::RuleHeavy => {
            "directory naming plus 30 extension-specific TypeScript naming rules"
        }
        FixtureKind::IgnoredGenerated => {
            "directory naming, TypeScript file naming, and generated-path pruning"
        }
        FixtureKind::SimpleLibrary => {
            "Rust library naming, markdown exists counts, test naming, and target pruning"
        }
        FixtureKind::WebApp => {
            "frontend component, test, CSS module, asset naming, and dist pruning"
        }
        FixtureKind::MonorepoPackages => {
            "package-scoped TypeScript, TSX, test naming, markdown exists counts, and dist pruning"
        }
        FixtureKind::MonorepoPolicy => {
            "strict monorepo policy with root whitelisting, app/package scopes, source bans, docs/scripts/infra rules, and generated-output pruning"
        }
        FixtureKind::RuleHeavyRepo => {
            "repo-shaped multi-extension naming with wildcard file naming parity"
        }
        FixtureKind::IgnoredGeneratedHeavyRepo => {
            "repo-shaped TypeScript naming with generated and coverage pruning"
        }
        FixtureKind::PinnedNextJs => {
            "pinned Next.js checkout with broad native filename and directory naming policy"
        }
        FixtureKind::PinnedMdBook => {
            "pinned mdBook checkout with Rust, docs, TOML, and directory naming policy"
        }
    }
}

fn config_generation_method(kind: FixtureKind) -> &'static str {
    match kind {
        FixtureKind::Sized | FixtureKind::RuleHeavy | FixtureKind::IgnoredGenerated => {
            "hand-authored-equivalent-pair"
        }
        FixtureKind::SimpleLibrary
        | FixtureKind::WebApp
        | FixtureKind::MonorepoPackages
        | FixtureKind::RuleHeavyRepo
        | FixtureKind::IgnoredGeneratedHeavyRepo => "ls-lint-conversion",
        FixtureKind::MonorepoPolicy => "hand-authored-equivalent-pair",
        FixtureKind::PinnedNextJs | FixtureKind::PinnedMdBook => "hand-authored-external-policy",
    }
}
