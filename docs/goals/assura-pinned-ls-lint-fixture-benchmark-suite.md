---
id: goal-assura-pinned-ls-lint-fixture-benchmark-suite
type: goal
title: Assura pinned LS-Lint fixture benchmark suite
status: completed
created: 2026-05-17
owners:
  - assura-maintainers
related:
  - docs/goals/assura-cli-to-cli-ls-lint-performance-verification.md
  - docs/goals/assura-ls-lint-realistic-parity-core-performance.md
  - docs/analysis/2026-05-17-cli-to-cli-performance-decision-record.md
  - docs/analysis/2026-05-17-filesystem-validation-throughput-research.md
  - website/src/content/docs/reference/performance.mdx
  - website/src/content/docs/reference/performance-test-cases.mdx
  - website/src/content/docs/reference/performance-implementation.mdx
  - tests/ls_lint_realistic_fixture_manifest.yml
  - src/cli/performance_report/fixtures.rs
  - src/cli/performance_report/fixture_metadata.rs
  - benches/history/current.json
  - website/public/data/performance/current.json
---

# Assura Pinned LS-Lint Fixture Benchmark Suite

## Objective

Upgrade Assura performance evidence from controlled generated fixtures to a
durable benchmark suite that includes meaningful LS-Lint policies and pinned
real-project fixture sources.

The next public performance claim should be based on test cases that users can
understand:

- a rich monorepo policy with directory whitelists, source-tree restrictions,
  config-file exceptions, ignored generated outputs, and extension bans,
- pinned open-source repositories that are stable across time,
- measured Assura CLI versus native LS-Lint binary rows for every headline case,
- a single test-case definition page that result rows link to instead of
  embedding long rulesets in the performance summary.

## Why This Goal Exists

PR #11 established the fair CLI-to-CLI measurement path, but the public docs
still showed the limitation clearly: the measured generated cases are stable,
yet several are too simple or too synthetic to be persuasive.

Specific gaps to close:

- `rule_heavy_repo` currently proves extension matching cost, not realistic
  project governance.
- The performance summary must stay concise; LS-Lint configs and fixture
  rationale belong on the dedicated test-case page.
- The benchmark suite needs recognizable, pinned project shapes before the
  website makes a broader end-user performance claim.
- Future agents need one source of truth for test-case definition, fixture
  manifest metadata, generated report metadata, and website rendering.

## Current Repo Truth

- `assura performance-report` emits separate row families for `assura-cli`,
  `ls-lint-cli`, `assura-in-process`, `assura:phase:*`, `traversal:*`, and
  `strategy:*`.
- The public comparison should use `assura-cli` and `ls-lint-cli` rows only.
- Traversal, phase, and strategy rows are technical diagnostics and should stay
  on the implementation page unless they directly change the product claim.
- `website/src/content/docs/reference/performance.mdx` should remain a concise
  summary page.
- `website/src/content/docs/reference/performance-test-cases.mdx` is the
  intended source of truth for benchmark case definitions.
- `tests/ls_lint_realistic_fixture_manifest.yml` already includes a pinned
  external `mdBook` fixture entry for materialization tests, but the performance
  report does not yet measure pinned external repository fixtures.
- A useful local LS-Lint policy example exists at
  `/Users/nroth/workspace/job-tracker-agent-app/.ls-lint.yml`. Use it as a
  reference shape only; do not make Assura depend on that local checkout.

## Required Research Inputs

Before implementation, inspect:

- `website/src/content/docs/reference/performance-test-cases.mdx`
- `src/cli/performance_report/fixtures.rs`
- `src/cli/performance_report/fixture_metadata.rs`
- `src/cli/performance_report/fixture_io.rs`
- `tests/ls_lint_realistic_fixture_manifest.yml`
- `tests/realistic_lslint_fixtures.rs`
- `docs/analysis/2026-05-17-cli-to-cli-performance-decision-record.md`
- `docs/analysis/2026-05-17-filesystem-validation-throughput-research.md`
- `.agents/skills/assura-performance-reporting/SKILL.md`

Also review LS-Lint 2.3 docs for advanced rule behavior:

- <https://ls-lint.org/2.3/configuration/the-basics.html>
- <https://ls-lint.org/2.3/configuration/the-rules.html>

## Scope

This goal covers:

- adding a richer generated monorepo-policy performance fixture,
- adding pinned external repository performance fixtures,
- updating fixture metadata and report rows so every headline case links to a
  stable test-case definition,
- refreshing tracked benchmark data and website performance data,
- updating website docs so the summary page stays concise and details live on
  the test-case and implementation pages,
- adding regression coverage for fixture metadata, materialization, and
  report-row classification.

This goal does not cover:

- changing the default traversal architecture unless a fixture exposes a real
  correctness or performance blocker,
- claiming performance for every LS-Lint configuration,
- measuring cold package install or dependency resolution,
- adding Assura-only notation features to headline native LS-Lint parity rows.

## Acceptance Criteria

### 1. Rich Monorepo Policy Fixture

Add a generated fixture that models a strict real project policy rather than a
toy extension list.

It should include representative LS-Lint behavior such as:

- root directory whitelist,
- special well-known root docs such as `README`, `AGENTS`, `CONTRIBUTING`, and
  `LICENSE`,
- `apps/*` and `packages/*` style project areas where supported or an explicit
  documented equivalent if LS-Lint compatibility requires generated explicit
  scopes,
- app-root config exceptions for files such as `next.config`, `eslint.config`,
  `postcss.config`, and `tailwind.config`,
- JavaScript/JSX/MJS/CJS disallowed in source subtrees through `regex:^$`
  rules,
- docs restricted to documentation file types,
- scripts and infra subtrees with their own extension policy,
- ignored generated paths such as `node_modules`, `dist`, `coverage`, `.next`,
  `.turbo`, and package/app-local variants.

The generated tree must contain enough files and ignored directories to make
the fixture meaningful for traversal and rule matching.

### 2. Pinned External Repository Fixtures

Add at least two pinned external fixture sources to the performance report:

- a pinned Next.js checkout or equivalent large frontend/monorepo repository,
- a pinned mdBook checkout or equivalent Rust documentation/library repository.

For each external fixture:

- pin repository URL and immutable tag or commit SHA in the manifest,
- resolve and record the commit SHA used for the run,
- define the exact LS-Lint config used for the benchmark,
- generate or write the equivalent Assura config,
- materialize the fixture without vendoring third-party source into this repo,
- define ignored generated/cache directories,
- record checked file count, ignored file count, directory count, rule count,
  source type, source revision, and config references in performance rows.

If a pinned repository is too large or flaky for ordinary CI, document the
blocker and keep it behind an explicit opt-in flag. The goal is stable public
evidence, not a slow or unreliable default CI path.

### 3. Report Contract

The performance report must classify new rows correctly:

- native LS-Lint parity rows that should drive the website claim are
  `headline-candidate` and `diagnostic=false`,
- synthetic stress rows remain diagnostic,
- traversal, phase, and strategy rows remain diagnostic,
- Assura-only extension rows are excluded from the native LS-Lint parity
  headline unless explicitly labeled as a separate claim.

The report schema and checked-in current data must accept and preserve the new
fixture metadata.

### 4. Website Contract

The website should keep this structure:

- `/reference/performance/`: concise summary and result table only,
- `/reference/performance-test-cases/`: single source of truth for case
  definitions, rule/policy rationale, pinned source references, and caveats,
- `/reference/performance-implementation/`: traversal, phase, and strategy
  implementation details.

Result rows on the summary page must link to the corresponding test-case
definition. Do not embed full LS-Lint configs inside the summary table.

The website must clearly distinguish:

- measured generated fixtures,
- measured pinned repository fixtures,
- planned but not-yet-measured cases,
- diagnostic stress fixtures.

### 5. Tests And Validation

Add focused tests for:

- fixture manifest entries for each new case,
- external fixture materialization and pinned revision handling,
- metadata presence on new performance rows,
- headline versus diagnostic classification,
- report schema acceptance for new fixture metadata,
- website build stability.

Run at minimum:

```bash
cargo fmt --all -- --check
git diff --check
cargo test --all-targets --quiet
cargo run --quiet -- check --format json .
cargo run --release --quiet -- performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 15
pnpm --dir website build
```

If external fixture measurement requires network access or is too slow for
every run, add a smaller deterministic default path and document the explicit
command for the full evidence refresh.

## Done Definition

The goal is complete when:

- the new goal-owned fixtures are implemented and measured or blockers are
  documented with exact next actions,
- tracked benchmark data is refreshed,
- website docs render the new case structure without turning the summary page
  into a dense rules page,
- local validation passes,
- GitHub checks pass on the PR,
- the PR body links to the new benchmark data and test-case definitions.

## Recommended Next Prompt

Use this prompt to start the next work chunk:

```text
Continue Assura performance work using docs/goals/assura-pinned-ls-lint-fixture-benchmark-suite.md.

Start by reading AGENTS.md, .agents/skills/assura-performance-reporting/SKILL.md, the goal file, website/src/content/docs/reference/performance-test-cases.mdx, src/cli/performance_report/fixtures.rs, fixture_metadata.rs, fixture_io.rs, tests/ls_lint_realistic_fixture_manifest.yml, and tests/realistic_lslint_fixtures.rs.

Keep scope narrow: add meaningful LS-Lint benchmark fixtures and evidence, not traversal rewrites. First implement the rich monorepo-policy generated fixture, then add pinned external fixture support for Next.js/mdBook or document blockers. Keep the performance summary page concise and put fixture/ruleset detail on the test-case page. Run the goal validation commands and update PR #11 with results.
```

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-05-17 | Started Trellis task for the pinned LS-Lint fixture benchmark suite and captured implementation/check context. | `.trellis/tasks/05-17-pinned-ls-lint-fixture-benchmark-suite/` |
| 2026-05-17 | Implemented the rich generated `monorepo_policy` fixture, opt-in pinned Next.js/mdBook fixture support, report metadata/schema updates, rolling JSONL history cap, and refreshed checked-in performance/website data. | `src/cli/performance_report/`; `tests/ls_lint_realistic_fixture_manifest.yml`; `benches/history/current.json`; `website/public/data/performance/current.json`; `website/src/content/docs/reference/performance-test-cases.mdx` |
| 2026-05-17 | Final local gates passed after review fixes. External pinned repository measurement remains opt-in and documented because it clones large third-party repositories. | `cargo fmt --all -- --check`; `git diff --check`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo test --all-targets --quiet`; `cargo run --quiet -- check --format json .`; `cargo run --release --quiet -- performance-report --output benches/history/current.json --history benches/history/ls-lint-comparison-history.jsonl --website-dir website/public/data/performance --iterations 15`; `pnpm --dir website build` |
| 2026-05-17 | Goal created from PR #11 performance-doc review and follow-up recommendations. | `docs/goals/assura-pinned-ls-lint-fixture-benchmark-suite.md`; `website/src/content/docs/reference/performance-test-cases.mdx` |
| 2026-05-18 | Revalidated the native LS-Lint correction, lightweight `assura-check` evidence path, generated `monorepo_policy` row metadata, and website build. Context review: the current public claim remains narrower than a universal 2x statement; checked-in data shows `assura-check` wins the current realistic generated set overall, while only the generated-heavy case exceeds 2x. | `cargo fmt --all -- --check`; `git diff --check`; `cargo test --all-targets --quiet`; `cargo clippy --all-targets --quiet -- -D warnings`; `target/release/assura performance-report --output target/performance/pinned-fixtures-smoke.json --history target/performance/pinned-fixtures-smoke.jsonl --website-dir target/performance/pinned-fixtures-website --iterations 1`; `cargo run --quiet -- check --format json .`; `pnpm build` |
