//! Catalog of opt-in pinned external Git fixtures and their LS-Lint policies.

use super::fixtures::FixtureKind;

#[derive(Clone, Copy)]
pub(super) struct ExternalFixtureSpec {
    pub(super) fixture_id: &'static str,
    pub(super) repository: &'static str,
    pub(super) revision: &'static str,
    pub(super) ls_lint_config: &'static str,
}

#[cfg(test)]
pub(super) fn external_fixture_specs() -> &'static [ExternalFixtureSpec] {
    &EXTERNAL_FIXTURE_SPECS
}

pub(super) fn external_fixture_spec(kind: FixtureKind) -> Option<ExternalFixtureSpec> {
    match kind {
        FixtureKind::PinnedNextJs => spec_by_id("pinned_nextjs"),
        FixtureKind::PinnedMdBook => spec_by_id("pinned_mdbook"),
        FixtureKind::PinnedVite => spec_by_id("pinned_vite"),
        FixtureKind::PinnedTailwindCss => spec_by_id("pinned_tailwindcss"),
        FixtureKind::PinnedPrettier => spec_by_id("pinned_prettier"),
        FixtureKind::PinnedPnpm => spec_by_id("pinned_pnpm"),
        FixtureKind::PinnedRustlings => spec_by_id("pinned_rustlings"),
        FixtureKind::PinnedClap => spec_by_id("pinned_clap"),
        FixtureKind::PinnedRipgrep => spec_by_id("pinned_ripgrep"),
        FixtureKind::PinnedTokio => spec_by_id("pinned_tokio"),
        _ => None,
    }
}

fn spec_by_id(fixture_id: &str) -> Option<ExternalFixtureSpec> {
    EXTERNAL_FIXTURE_SPECS
        .iter()
        .copied()
        .find(|spec| spec.fixture_id == fixture_id)
}

const EXTERNAL_FIXTURE_SPECS: [ExternalFixtureSpec; 10] = [
    ExternalFixtureSpec {
        fixture_id: "pinned_nextjs",
        repository: "https://github.com/vercel/next.js",
        revision: "51bfe3c1863b191f4b039bc230e8ed5c57b0baf3",
        ls_lint_config: NEXTJS_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_mdbook",
        repository: "https://github.com/rust-lang/mdBook",
        revision: "b7a27d2759e80d804a33a4bc9c31b2b6863a5cb2",
        ls_lint_config: MDBOOK_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_vite",
        repository: "https://github.com/vitejs/vite",
        revision: "cf50e548d4ae7f6ed9ade507d70acbd39b4e0b93",
        ls_lint_config: VITE_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_tailwindcss",
        repository: "https://github.com/tailwindlabs/tailwindcss",
        revision: "7361468f77500105b0559e879e121f34306e8da2",
        ls_lint_config: TAILWIND_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_prettier",
        repository: "https://github.com/prettier/prettier",
        revision: "c4ab460357478d2b847c60a1efb40098b1181931",
        ls_lint_config: PRETTIER_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_pnpm",
        repository: "https://github.com/pnpm/pnpm",
        revision: "dd6b0b62d80340655ac1cf4c4365fcfffe1e3f2c",
        ls_lint_config: PNPM_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_rustlings",
        repository: "https://github.com/rust-lang/rustlings",
        revision: "28d2bb04326d7036514245d73f10fb72b9ed108c",
        ls_lint_config: RUSTLINGS_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_clap",
        repository: "https://github.com/clap-rs/clap",
        revision: "a751c5fe65cd33cb09e85ff3039b4fd0182cdb6e",
        ls_lint_config: CLAP_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_ripgrep",
        repository: "https://github.com/BurntSushi/ripgrep",
        revision: "4649aa9700619f94cf9c66876e9549d83420e16c",
        ls_lint_config: RIPGREP_LS_LINT_CONFIG,
    },
    ExternalFixtureSpec {
        fixture_id: "pinned_tokio",
        repository: "https://github.com/tokio-rs/tokio",
        revision: "14c17fc09656a30230177b600bacceb9db33e942",
        ls_lint_config: TOKIO_LS_LINT_CONFIG,
    },
];

const NEXTJS_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - .gitattributes
  - .gitignore
  - '**/.gitignore'
  - node_modules/**
  - packages/*/node_modules/**
  - examples/*/node_modules/**
  - test/**/node_modules/**
  - .next/**
  - packages/*/.next/**
  - examples/*/.next/**
  - dist/**
  - packages/*/dist/**
  - coverage/**
  - .turbo/**
  - .vercel/**
  - test/**
  - turbopack/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .log: exists:0
  packages:
    .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    "*":
      .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .tsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .rs: exists:0
      .go: exists:0
      .py: exists:0
      .rb: exists:0
  examples:
    .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    "*":
      .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .jsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .tsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .rs: exists:0
      .go: exists:0
      .py: exists:0
      .rb: exists:0
"#;

const MDBOOK_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - .gitattributes
  - .gitignore
  - '**/.gitignore'
  - target/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .md: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .toml: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .js: exists:0
  .ts: exists:0
  .tsx: exists:0
  src:
    .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
    .js: exists:0
    .ts: exists:0
  book:
    .md: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .rs: exists:0
    .js: exists:0
"#;

const VITE_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - node_modules/**
  - packages/*/node_modules/**
  - playground/**/node_modules/**
  - packages/*/dist/**
  - docs/.vitepress/cache/**
  - docs/.vitepress/dist/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  packages:
    .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    "*":
      .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .jsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .tsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .vue: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .svelte: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .rs: exists:0
      .go: exists:0
      .py: exists:0
  playground:
    .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    "*":
      .jsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .vue: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .tsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .svelte: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .rs: exists:0
      .go: exists:0
      .py: exists:0
"#;

const TAILWIND_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - node_modules/**
  - dist/**
  - coverage/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  src:
    .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .rs: exists:0
    .go: exists:0
    .py: exists:0
  tests:
    .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .html: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .rs: exists:0
    .go: exists:0
    .py: exists:0
"#;

const PRETTIER_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - node_modules/**
  - dist/**
  - coverage/**
  - tests_config/run_spec/cache/**
  - tests/integration/cli/ignore-emoji/**
  - tests/integration/cli/patterns-glob/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  src:
    .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .rs: exists:0
    .go: exists:0
    .py: exists:0
  tests:
    .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .jsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .tsx: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .json: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .rs: exists:0
    .go: exists:0
  website:
    .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .md: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .rs: exists:0
    .go: exists:0
    .py: exists:0
"#;

const PNPM_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - node_modules/**
  - packages/*/node_modules/**
  - dist/**
  - packages/*/dist/**
  - coverage/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  packages:
    .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    "*":
      .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .js: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .d.ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .jsx: exists:0
      .tsx: exists:0
      .rs: exists:0
      .go: exists:0
      .py: exists:0
  __typings__:
    .d.ts: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    .js: exists:0
"#;

const RUSTLINGS_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - target/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  exercises:
    .dir: regex:^$|^exercises$|^[0-9]{2}_[a-z_]+$|^quizzes$
    "*":
      .dir: regex:^[0-9]{2}_[a-z_]+$|^quizzes$
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
      .js: exists:0
      .ts: exists:0
  solutions:
    .dir: regex:^$|^solutions$|^[0-9]{2}_[a-z_]+$|^quizzes$
    "*":
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
      .js: exists:0
      .ts: exists:0
"#;

const CLAP_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - target/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  clap_builder:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  clap_complete:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
    examples:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  clap_derive:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  clap_lex:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  clap_complete_fig:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  examples:
    .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  tests:
    .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  .js: exists:0
  .ts: exists:0
  .tsx: exists:0
"#;

const RIPGREP_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - target/**
  - crates/*/target/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  crates:
    .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
    "*":
      .dir: regex:^[a-z0-9][a-z0-9_-]*$
      src:
        .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
      examples:
        .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      tests:
        .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
      .js: exists:0
      .ts: exists:0
  tests:
    .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .js: exists:0
  .ts: exists:0
  .tsx: exists:0
"#;

const TOKIO_LS_LINT_CONFIG: &str = r#"
ignore:
  - .assura/**
  - .git/**
  - target/**
  - tokio/target/**
  - tokio-util/target/**
ls:
  .dir: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .*: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  tokio:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  tokio-util:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  tokio-stream:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  tokio-test:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  tokio-macros:
    src:
      .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$ | exists
  examples:
    .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  tests:
    .rs: regex:^$|^[A-Za-z0-9._@%+\[\](){}=-]+$
  .js: exists:0
  .ts: exists:0
  .tsx: exists:0
"#;
