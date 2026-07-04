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
        native_ls_lint_parity: native_ls_lint_parity(scenario.kind),
        assura_config_path: ".assura/config.yml",
        ls_lint_config_path: ls_lint_config_path(scenario.kind),
        config_generation_method: config_generation_method(scenario.kind),
        shared_config_id: format!("{}:{}", source_revision, scenario.id),
        expected_assura_exit_status: 0,
        expected_ls_lint_exit_status: expected_ls_lint_exit_status(scenario.kind),
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
        FixtureKind::RealProjectAgenticFeedback => &[
            ".assura",
            "node_modules",
            "dist",
            "coverage",
            ".next",
            ".turbo",
        ],
        FixtureKind::RuleHeavyRepo => &[".assura", ".ls-lint.yml"],
        FixtureKind::IgnoredGeneratedHeavyRepo => &[".assura", "generated", "coverage"],
        FixtureKind::MultipartExtensionRegression | FixtureKind::ManyConfiguredScopesRegression => {
            &[".assura"]
        }
        FixtureKind::NativeSmall
        | FixtureKind::NativeMedium
        | FixtureKind::NativeLarge
        | FixtureKind::NativeReferenceHeavy
        | FixtureKind::NativeAdapterMix
        | FixtureKind::NativeRealProject => &[".assura"],
        FixtureKind::PinnedNextJs
        | FixtureKind::PinnedMdBook
        | FixtureKind::PinnedVite
        | FixtureKind::PinnedTailwindCss
        | FixtureKind::PinnedPrettier
        | FixtureKind::PinnedPnpm
        | FixtureKind::PinnedRustlings
        | FixtureKind::PinnedClap
        | FixtureKind::PinnedRipgrep
        | FixtureKind::PinnedTokio => external_ignored_paths(kind),
    }
}

fn external_ignored_paths(kind: FixtureKind) -> &'static [&'static str] {
    match kind {
        FixtureKind::PinnedNextJs => &[
            ".assura",
            ".git",
            ".gitattributes",
            ".gitignore",
            "**/.gitignore",
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
            "test",
            "turbopack",
        ],
        FixtureKind::PinnedMdBook => &[
            ".assura",
            ".git",
            ".gitattributes",
            ".gitignore",
            "**/.gitignore",
            "target",
        ],
        FixtureKind::PinnedVite => &[
            ".assura",
            ".git",
            "node_modules",
            "packages/*/node_modules",
            "playground/**/node_modules",
            "packages/*/dist",
            "docs/.vitepress/cache",
            "docs/.vitepress/dist",
        ],
        FixtureKind::PinnedTailwindCss => &[".assura", ".git", "node_modules", "dist", "coverage"],
        FixtureKind::PinnedPrettier => &[
            ".assura",
            ".git",
            "node_modules",
            "dist",
            "coverage",
            "tests/integration/cli/ignore-emoji",
            "tests/integration/cli/patterns-glob",
        ],
        FixtureKind::PinnedPnpm => &[
            ".assura",
            ".git",
            "node_modules",
            "packages/*/node_modules",
            "dist",
            "packages/*/dist",
            "coverage",
        ],
        FixtureKind::PinnedRustlings
        | FixtureKind::PinnedClap
        | FixtureKind::PinnedRipgrep
        | FixtureKind::PinnedTokio => &[".assura", ".git", "target", "crates/*/target"],
        _ => &[".assura", ".git"],
    }
}

fn source_type(kind: FixtureKind) -> &'static str {
    if matches!(kind, FixtureKind::RealProjectAgenticFeedback) {
        "checked-repo-fixture"
    } else if matches!(
        kind,
        FixtureKind::NativeSmall
            | FixtureKind::NativeMedium
            | FixtureKind::NativeLarge
            | FixtureKind::NativeReferenceHeavy
            | FixtureKind::NativeAdapterMix
            | FixtureKind::NativeRealProject
    ) {
        "generated-native"
    } else if kind.is_external_pinned() {
        "external-pinned-repo"
    } else {
        "generated"
    }
}

fn source_revision(scenario: FixtureScenario, root: &Path) -> String {
    if scenario.kind.is_external_pinned() {
        fs::read_to_string(root.join(".assura/source-revision.txt"))
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|_| scenario.source_revision.to_string())
    } else {
        scenario.source_revision.to_string()
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
        | FixtureKind::RealProjectAgenticFeedback
        | FixtureKind::RuleHeavyRepo
        | FixtureKind::IgnoredGeneratedHeavyRepo
        | FixtureKind::MultipartExtensionRegression
        | FixtureKind::ManyConfiguredScopesRegression => {
            if matches!(kind, FixtureKind::RealProjectAgenticFeedback) {
                "agent-feedback-loop"
            } else {
                "realistic-equivalent"
            }
        }
        FixtureKind::NativeSmall
        | FixtureKind::NativeMedium
        | FixtureKind::NativeLarge
        | FixtureKind::NativeReferenceHeavy
        | FixtureKind::NativeAdapterMix
        | FixtureKind::NativeRealProject => "assura-native",
        FixtureKind::PinnedNextJs
        | FixtureKind::PinnedMdBook
        | FixtureKind::PinnedVite
        | FixtureKind::PinnedTailwindCss
        | FixtureKind::PinnedPrettier
        | FixtureKind::PinnedPnpm
        | FixtureKind::PinnedRustlings
        | FixtureKind::PinnedClap
        | FixtureKind::PinnedRipgrep
        | FixtureKind::PinnedTokio => "real-repo-headline",
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
        FixtureKind::RealProjectAgenticFeedback => 19,
        FixtureKind::RuleHeavyRepo => 38,
        FixtureKind::MultipartExtensionRegression => 1,
        FixtureKind::ManyConfiguredScopesRegression => 801,
        FixtureKind::NativeSmall => 80,
        FixtureKind::NativeMedium => 800,
        FixtureKind::NativeLarge => 8000,
        FixtureKind::NativeReferenceHeavy => 1000,
        FixtureKind::NativeAdapterMix => 1000,
        FixtureKind::NativeRealProject => 480,
        FixtureKind::PinnedNextJs => 22,
        FixtureKind::PinnedMdBook => 14,
        FixtureKind::PinnedVite => 22,
        FixtureKind::PinnedTailwindCss => 12,
        FixtureKind::PinnedPrettier => 19,
        FixtureKind::PinnedPnpm => 14,
        FixtureKind::PinnedRustlings => 11,
        FixtureKind::PinnedClap => 13,
        FixtureKind::PinnedRipgrep => 13,
        FixtureKind::PinnedTokio => 12,
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
        FixtureKind::RealProjectAgenticFeedback => {
            "Goal 03 real-project agent feedback fixture with root whitelisting, app/package scopes, direct-count rules, source bans, and generated-output pruning"
        }
        FixtureKind::RuleHeavyRepo => {
            "repo-shaped multi-extension naming with wildcard file naming parity"
        }
        FixtureKind::IgnoredGeneratedHeavyRepo => {
            "repo-shaped TypeScript naming with generated and coverage pruning"
        }
        FixtureKind::MultipartExtensionRegression => {
            "long multipart extension naming regression fixture"
        }
        FixtureKind::ManyConfiguredScopesRegression => {
            "large LS-Lint config with many explicit package scopes and root directory naming"
        }
        FixtureKind::NativeSmall => {
            "native content authoring matrix with 25 goals, 25 specs, 5 ADRs, Markdown links, relations, context packs, and search"
        }
        FixtureKind::NativeMedium => {
            "native content authoring matrix with 250 goals, 250 specs, 50 ADRs, Markdown links, relations, context packs, and search"
        }
        FixtureKind::NativeLarge => {
            "native content authoring matrix with 2500 goals, 2500 specs, 500 ADRs, Markdown links, relations, context packs, and search"
        }
        FixtureKind::NativeReferenceHeavy => {
            "native reference-heavy matrix with many-to-many relations, missing targets, Markdown links, and diagnostic classification"
        }
        FixtureKind::NativeAdapterMix => {
            "native adapter-mix matrix with Markdown frontmatter, JSON records, code-symbol references, relations, and context packs"
        }
        FixtureKind::NativeRealProject => {
            "native real-project shaped content authoring matrix with goals, specs, ADRs, notes, broken references, and agent-query rows"
        }
        FixtureKind::PinnedNextJs => {
            "pinned Next.js checkout with package/example route naming, source-family rules, foreign-language bans, and generated-output pruning"
        }
        FixtureKind::PinnedMdBook => {
            "pinned mdBook checkout with required Rust/book content, TOML/Markdown naming, and JS/TS source bans"
        }
        FixtureKind::PinnedVite => {
            "pinned Vite checkout with package/playground source families, template languages, foreign-language bans, and docs/build-output pruning"
        }
        FixtureKind::PinnedTailwindCss => {
            "pinned Tailwind CSS checkout with source/test language policy, HTML fixtures, foreign-language bans, and generated-output pruning"
        }
        FixtureKind::PinnedPrettier => {
            "pinned Prettier checkout with source/test/website language policy, fixture ignores, and generated-output pruning"
        }
        FixtureKind::PinnedPnpm => {
            "pinned pnpm checkout with package scopes, TypeScript declaration policy, frontend-source bans, and generated-output pruning"
        }
        FixtureKind::PinnedRustlings => {
            "pinned Rustlings checkout with exercise/solution group naming, required Rust files, JS/TS bans, and target pruning"
        }
        FixtureKind::PinnedClap => {
            "pinned clap checkout with explicit crate source scopes, examples/tests Rust policy, JS/TS bans, and target-output pruning"
        }
        FixtureKind::PinnedRipgrep => {
            "pinned ripgrep checkout with crate source scopes, examples/tests Rust policy, JS/TS bans, and target pruning"
        }
        FixtureKind::PinnedTokio => {
            "pinned Tokio checkout with explicit crate source scopes, examples/tests Rust policy, JS/TS bans, and target pruning"
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
        | FixtureKind::IgnoredGeneratedHeavyRepo
        | FixtureKind::MultipartExtensionRegression
        | FixtureKind::ManyConfiguredScopesRegression => "ls-lint-conversion",
        FixtureKind::RealProjectAgenticFeedback => "assura-native-policy-fixture",
        FixtureKind::NativeSmall
        | FixtureKind::NativeMedium
        | FixtureKind::NativeLarge
        | FixtureKind::NativeReferenceHeavy
        | FixtureKind::NativeAdapterMix
        | FixtureKind::NativeRealProject => "assura-native-generated-matrix",
        FixtureKind::MonorepoPolicy => "hand-authored-equivalent-pair",
        FixtureKind::PinnedNextJs
        | FixtureKind::PinnedMdBook
        | FixtureKind::PinnedVite
        | FixtureKind::PinnedTailwindCss
        | FixtureKind::PinnedPrettier
        | FixtureKind::PinnedPnpm
        | FixtureKind::PinnedRustlings
        | FixtureKind::PinnedClap
        | FixtureKind::PinnedRipgrep
        | FixtureKind::PinnedTokio => "external-ls-lint-conversion",
    }
}

fn native_ls_lint_parity(kind: FixtureKind) -> bool {
    !matches!(
        kind,
        FixtureKind::RealProjectAgenticFeedback
            | FixtureKind::NativeSmall
            | FixtureKind::NativeMedium
            | FixtureKind::NativeLarge
            | FixtureKind::NativeReferenceHeavy
            | FixtureKind::NativeAdapterMix
            | FixtureKind::NativeRealProject
    )
}

fn ls_lint_config_path(kind: FixtureKind) -> &'static str {
    if matches!(
        kind,
        FixtureKind::RealProjectAgenticFeedback
            | FixtureKind::NativeSmall
            | FixtureKind::NativeMedium
            | FixtureKind::NativeLarge
            | FixtureKind::NativeReferenceHeavy
            | FixtureKind::NativeAdapterMix
            | FixtureKind::NativeRealProject
    ) {
        "not-applicable"
    } else {
        ".ls-lint.yml"
    }
}

fn expected_ls_lint_exit_status(kind: FixtureKind) -> i32 {
    if matches!(
        kind,
        FixtureKind::RealProjectAgenticFeedback
            | FixtureKind::NativeSmall
            | FixtureKind::NativeMedium
            | FixtureKind::NativeLarge
            | FixtureKind::NativeReferenceHeavy
            | FixtureKind::NativeAdapterMix
            | FixtureKind::NativeRealProject
    ) {
        1
    } else {
        0
    }
}
