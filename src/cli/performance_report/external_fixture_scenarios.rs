//! Opt-in performance scenarios backed by pinned external Git repositories.

use super::fixtures::{FixtureKind, FixtureScenario};

pub(super) fn external_fixture_scenarios() -> [FixtureScenario; 10] {
    [
        FixtureScenario {
            id: "pinned_nextjs",
            source_revision: "51bfe3c1863b191f4b039bc230e8ed5c57b0baf3",
            rule_cohort: "pinned-frontend-monorepo",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedNextJs,
        },
        FixtureScenario {
            id: "pinned_mdbook",
            source_revision: "b7a27d2759e80d804a33a4bc9c31b2b6863a5cb2",
            rule_cohort: "pinned-rust-docs",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedMdBook,
        },
        FixtureScenario {
            id: "pinned_vite",
            source_revision: "cf50e548d4ae7f6ed9ade507d70acbd39b4e0b93",
            rule_cohort: "pinned-frontend-tooling-monorepo",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedVite,
        },
        FixtureScenario {
            id: "pinned_tailwindcss",
            source_revision: "7361468f77500105b0559e879e121f34306e8da2",
            rule_cohort: "pinned-css-tooling-package",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedTailwindCss,
        },
        FixtureScenario {
            id: "pinned_prettier",
            source_revision: "c4ab460357478d2b847c60a1efb40098b1181931",
            rule_cohort: "pinned-js-formatting-tooling",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedPrettier,
        },
        FixtureScenario {
            id: "pinned_pnpm",
            source_revision: "dd6b0b62d80340655ac1cf4c4365fcfffe1e3f2c",
            rule_cohort: "pinned-package-manager-monorepo",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedPnpm,
        },
        FixtureScenario {
            id: "pinned_rustlings",
            source_revision: "28d2bb04326d7036514245d73f10fb72b9ed108c",
            rule_cohort: "pinned-rust-education",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedRustlings,
        },
        FixtureScenario {
            id: "pinned_clap",
            source_revision: "a751c5fe65cd33cb09e85ff3039b4fd0182cdb6e",
            rule_cohort: "pinned-rust-workspace",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedClap,
        },
        FixtureScenario {
            id: "pinned_ripgrep",
            source_revision: "4649aa9700619f94cf9c66876e9549d83420e16c",
            rule_cohort: "pinned-rust-cli",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedRipgrep,
        },
        FixtureScenario {
            id: "pinned_tokio",
            source_revision: "14c17fc09656a30230177b600bacceb9db33e942",
            rule_cohort: "pinned-rust-async-workspace",
            dirs: 0,
            files_per_dir: 0,
            kind: FixtureKind::PinnedTokio,
        },
    ]
}
