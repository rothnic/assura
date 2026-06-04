---
id: goal-assura-ls-lint-realistic-parity-core-performance
type: goal
title: Assura LS-Lint realistic parity and core performance
status: completed
created: 2026-05-15
owners:
  - assura-maintainers
related:
  - .trellis/spec/assura/index.md
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/structure-enforcement.md
  - .trellis/spec/assura/tooling-stabilization.md
  - docs/analysis/2026-05-11-ls-lint-parity-performance-regression-audit.md
  - docs/analysis/2026-05-15-ls-lint-good-enough-comparison-contract.md
  - docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md
  - docs/analysis/2026-05-15-performance-architecture-statement.md
  - docs/analysis/2026-05-15-performance-hotspot-optimization-progress.md
  - docs/analysis/2026-05-15-notation-source-truth.md
  - docs/unified-tree-design.md
  - docs/ls-lint-capability-comparison.md
  - docs/archive/final-config-design.md
  - docs/archive/ls-lint-notation-guide.md
---

# Assura LS-Lint realistic parity and core performance

## Objective

Build the next product-quality foundation for Assura by making LS-Lint
compatibility tests realistic, switching the production `assura check`
traversal path to `jwalk`, and profiling the current core system deeply enough
to identify and address obvious performance optimizations before expanding the
notation beyond LS-Lint.

This goal intentionally focuses less on agent nudges. The next notation work
should build on a reliable compatibility and performance baseline, not on
assumptions from small synthetic fixtures.

## Execution Boundaries

This goal starts by creating a new branch from a clean, up-to-date `master`.
Do not implement this goal directly on `master`.

Recommended branch name:

```bash
git switch master
git pull --ff-only
git switch -c codex/ls-lint-realistic-parity-core-performance
```

This goal ends only after:

- all acceptance criteria are met or blockers are documented with exact
  command output and next action,
- validation commands have passed or blockers are documented,
- benchmark and performance results are saved in the required machine-readable
  and human-readable locations,
- the website or generated preview includes a link to the performance
  comparison chart/table,
- the PR body includes benchmark/performance details and links to artifacts,
  stored data, and website or preview output,
- the branch is pushed, and
- a draft PR is created for review.

The final handoff must include the draft PR URL and a short summary of:

- LS-Lint fixture coverage added or changed,
- `jwalk` migration status,
- hotspot optimizations implemented or deferred,
- incremental/cache research or implementation status,
- Assura versus LS-Lint benchmark results,
- performance regression status versus the stable baseline,
- link to chart-ready historical performance data,
- link to website or preview performance visualization.

## Before Changing Code

Read these files before implementation work starts:

- `AGENTS.md`
- `.agents/skills/assura-goal-execution/SKILL.md`
- `.agents/skills/assura-local-build/SKILL.md`
- `.trellis/workflow.md`
- `.trellis/spec/assura/index.md`
- `.trellis/spec/assura/roadmap.md`
- `.trellis/spec/assura/structure-enforcement.md`
- `.trellis/spec/assura/tooling-stabilization.md`
- `docs/analysis/2026-05-11-ls-lint-parity-performance-regression-audit.md`
- `docs/unified-tree-design.md`
- `docs/ls-lint-capability-comparison.md`
- `docs/archive/final-config-design.md`
- `docs/archive/ls-lint-notation-guide.md`
- `.assura/config.yml`
- `src/cli/check.rs`
- `src/cli/check/direct_contents.rs`
- `src/cli/check/patterns.rs`
- `src/cli/check/rules.rs`
- `src/cli/check/validators.rs`
- `src/config/ls_compat.rs`
- `tests/ls_lint_parity_regression_tests.rs`
- `benches/README.md`
- `benches/profiling.rs`
- `benches/ls_lint_comparison.rs`
- `Cargo.toml`

## Current Repo Truth

- `assura check` is the current public product path through
  `src/cli/check.rs` and `run_structure_check`.
- The production structure checker currently uses `walkdir::WalkDir`.
- `jwalk` is already a dependency and is used in profiling benchmarks, but it
  is not yet the production traversal implementation for `assura check`.
- Existing parity coverage includes core LS-Lint extension rules, wildcard
  extension rules, `.dir`, OR syntax, ignore/exclude behavior, extension and
  `.dir` `exists` counts, direct-child count semantics, scalar exact `exists`
  keys as direct counts for default validation, and scalar naming no-op keys
  matching upstream LS-Lint 2.3.
- Existing parity coverage is still mostly generated in test code rather than
  organized as realistic reusable repository fixtures.
- The compatibility layer now supports directory pattern scopes such as
  `packages/*`, `**`, and `{src,tests}` with regression coverage.
- Existing performance evidence compares current-product `assura check` to
  `@ls-lint/ls-lint@2.3.0` in `benches/ls_lint_comparison.rs`.
- The May 11 audit found that traversal is visible, but rule-heavy glob
  matching and direct count reads can dominate enough that they must be
  investigated alongside the `jwalk` migration.
- Current notation design is split across active and historical docs. The
  active docs do not yet provide one consolidated, current source of truth for
  the next notation extensions.

## Acceptance Criteria

### 0. Good-Enough Definition and Review Evidence

Before optimizing or adding notation behavior, define what "good enough" means
for LS-Lint compatibility and performance evidence. This definition must be
checked into the repo and used by tests, CI, PR review, and website reporting.

- Define a stable realistic-fixture manifest that pins each external fixture
  repository by immutable source:
  - repository URL,
  - commit SHA or release tag,
  - fixture name,
  - fixture purpose,
  - LS-Lint rule surface covered,
  - equivalent Assura rule surface covered,
  - whether the fixture is part of the stable baseline or a new feature
    cohort.
- The harness must materialize fixtures during CI/CD from pinned sources
  instead of storing full third-party repositories in this source tree.
- The harness must cache downloaded fixture sources where CI supports caching,
  but the pinned commit/tag remains the source of truth.
- The stable baseline fixture set must not change silently. Adding new Assura
  features should add new fixture cohorts; it should not rewrite old baseline
  fixtures unless the PR explicitly explains why.
- Define pass/fail criteria for LS-Lint comparison:
  - equivalent valid fixtures pass both LS-Lint and Assura,
  - equivalent invalid fixtures fail both tools where LS-Lint supports the
    behavior,
  - expected Assura violation rules are asserted,
  - Assura-only extensions are labeled separately and are not counted as native
    LS-Lint parity.
- Define pass/fail criteria for performance:
  - PRs must produce a comparison artifact for stable baseline fixtures,
  - stable baseline fixtures must not regress beyond an agreed threshold unless
    the PR documents a deliberate tradeoff,
  - new feature fixtures establish a new baseline instead of weakening old
    baseline expectations,
  - Assura and LS-Lint are compared with equivalent rules on the same
    materialized fixture tree.
- CI must upload or persist machine-readable comparison results for each run.
- PR review must expose a concise performance summary or link to the generated
  artifact so reviewers can see whether the current branch regressed versus
  the chosen baseline.
- The website must expose a performance comparison page or section with a link
  to current data and a chart-ready history. The goal is not just a one-time
  claim; it is visible performance tracking over time.
- The implementation must document how to update pinned fixtures, refresh
  baselines, and add new feature cohorts without losing longitudinal
  comparability.

### 1. Realistic LS-Lint Fixture Suite

- Create durable fixture repositories or fixture generators that model
  realistic projects, not only minimal inline temp directories.
- Include at least:
  - `simple_library`: one source tree, docs, tests, and generated output.
  - `web_app`: frontend-style source, components, tests, assets, and build
    output.
  - `monorepo_packages`: multiple packages with package-local source, tests,
    docs, and generated output.
  - `rule_heavy_repo`: many extension and subextension rules on a realistic
    tree.
  - `ignored_generated_heavy_repo`: large ignored/generated areas that must be
    pruned early.
- For every fixture family, include:
  - a valid repository shape,
  - an invalid repository shape,
  - a simple LS-Lint configuration,
  - a comprehensive LS-Lint configuration where applicable,
  - the generated or equivalent Assura configuration,
  - expected Assura violation rules,
  - native LS-Lint parity versus Assura extension labels.
- Keep fixtures reusable across unit tests, integration tests, and benchmarks.
  Avoid one-off fixture setup that drifts between tests and benches.
- External-real-repo fixtures should be pinned by manifest and materialized by
  the harness; generated synthetic fixtures may remain in code when they target
  narrow edge cases that external projects do not cover.

### 2. LS-Lint Compatibility Matrix

Expand or reorganize compatibility tests so every supported LS-Lint behavior
has realistic coverage:

- Extension rules.
- Wildcard extension rules such as `.*` and `.*.js`.
- Subextension rules such as `.d.ts`, `.test.ts`, `.spec.ts`, and `.module.css`
  where LS-Lint behavior applies.
- `.dir` directory naming.
- Explicit nested directory scopes.
- OR syntax such as `kebab-case | snake_case`.
- Ignore/exclude behavior, including ignored invalid files and ignored invalid
  directories.
- `exists`, `exists:0`, `exists:1`, and `exists:N-M` for file and directory
  counts.
- Direct-child-only `exists` semantics.
- Validation scope must not imply a required directory unless an explicit
  existence or required-directory rule requires it.
- Exact filename `exists` remains documented and tested as an Assura
  compatibility extension, not native LS-Lint parity.
- Directory pattern scopes such as `packages/*`, `**`, and `{src,tests}` are
  supported with tests. Do not regress them into literal required directories.

### 3. Production `jwalk` Migration

- Replace the production `walkdir::WalkDir` traversal in `assura check` with
  `jwalk`.
- Preserve all current `assura check` semantics:
  - valid JSON output,
  - stable relative paths,
  - excluded directory pruning,
  - fail-fast behavior,
  - deterministic violation sorting,
  - no violations for ignored generated content,
  - same pass/fail behavior on existing tests and new realistic fixtures.
- Keep platform behavior portable across Linux, macOS, Windows, and WSL.
- Do not introduce nondeterministic report output. Parallel traversal may be
  nondeterministic internally, but final reports must stay sorted and stable.
- Add focused regression tests if any traversal behavior changes.
- Remove or clearly isolate `walkdir` from the current product path if it is no
  longer needed there. Keeping `walkdir` for unrelated legacy tests or
  benchmark comparison is acceptable when documented.

### 4. Core Performance Hotspot Investigation

Profile the current product path before and after `jwalk` migration. Do not
stop at traversal if other hotspots dominate.

Investigate and either optimize or document next steps for:

- Traversal overhead: `walkdir` versus `jwalk` on small, medium, large, deep,
  wide, and ignored/generated-heavy fixtures.
- Exclusion pruning: confirm ignored/generated directories are skipped before
  expensive validation work.
- Rule resolution cache behavior: confirm `resolve_rules` does not repeatedly
  recompute effective rules for the same directories.
- Glob and naming pattern matching: avoid scanning every configured pattern for
  every file when extension/suffix indexing can narrow candidates.
- Direct count checks: avoid repeated `read_dir` work when file and directory
  direct-child counts can share an index or be derived from traversal data.
- Metadata and content reads: ensure file size, line count, docs, and markdown
  checks only read files when the active rules require those checks.
- Config parsing and pattern compilation: ensure regex and glob compilation are
  done once per check, not per path.
- Parallelism overhead: confirm `jwalk` parallelism helps realistic larger
  repos without hurting small repo latency enough to matter.

### 4.1 Incremental and Cache-Aware Checking Research

Research and design an incremental checking strategy before implementing any
cache that affects correctness.

The research must evaluate:

- Git-assisted change detection:
  - working tree status,
  - tracked file metadata,
  - staged versus unstaged changes,
  - untracked files,
  - rename/delete handling,
  - behavior outside git repositories.
- Hash-based tracking:
  - file content hashing,
  - metadata shortcuts such as size and modified time,
  - when metadata shortcuts are safe versus when content hashes are required,
  - hash algorithm choice and portability.
- Config invalidation:
  - `.assura/config.yml` content changes must invalidate affected cache state,
  - Assura binary/version changes must invalidate incompatible cache state,
  - rule engine/schema version changes must invalidate incompatible cache
    state,
  - fixture or project root changes must not reuse another project's cache.
- Dependency scope:
  - determine which checks are file-local,
  - determine which checks require directory-level recomputation,
  - determine which checks require whole-project recomputation,
  - document how future notation features such as file pairing or package
    structure requirements affect invalidation.
- Cache placement:
  - prefer a cache location that does not create git noise,
  - evaluate `.git/assura/`, `.git/assura-cache/`, platform cache dirs, and an
    explicit configured cache directory,
  - ensure cache files are not accidentally included in project validation,
  - document behavior for repos without `.git`,
  - document cleanup and disabling options.
- CI behavior:
  - decide whether CI should use a cold cache, restored cache, or both,
  - ensure performance comparisons can distinguish cold full-check time from
    warm incremental-check time,
  - prevent restored cache state from hiding correctness regressions.

Acceptance for this sub-area:

- Add a research artifact or design doc describing the chosen incremental
  strategy and rejected alternatives.
- Add an implementation plan that separates safe first steps from later
  advanced incremental behavior.
- If implemented in this goal, add tests proving:
  - unchanged files can be skipped only when safe,
  - changed files are rechecked,
  - deleted and renamed files invalidate relevant directory checks,
  - config changes invalidate the cache,
  - cache absence or corruption falls back to a full check,
  - cache files do not appear as Assura structure violations.
- Benchmark full-check, cold-cache, and warm-cache behavior separately.

### 5. Benchmark Matrix

Update benchmark coverage so release performance evidence is based on the
current product path.

Required scenarios:

| Scenario | Purpose | Required comparison |
| --- | --- | --- |
| `simple_small` | Small common repo latency | Assura current product vs LS-Lint 2.3 |
| `simple_medium` | Normal source/test tree | Assura current product vs LS-Lint 2.3 |
| `monorepo_large` | Many package directories and files | Assura current product vs LS-Lint 2.3 |
| `deep_tree` | Deep directory traversal overhead | Assura `walkdir` baseline if available vs `jwalk` |
| `wide_tree` | Wide directory traversal overhead | Assura `walkdir` baseline if available vs `jwalk` |
| `rule_heavy` | Many extension/subextension/naming patterns | Assura current product vs LS-Lint 2.3 and hotspot slices |
| `ignored_generated_heavy` | Exclusion pruning | Assura current product vs LS-Lint 2.3 |
| `many_direct_counts` | Direct `exists` cost | Assura current product hotspot slice |
| `comprehensive_config` | Realistic combined stress case | Assura current product vs LS-Lint 2.3 |
| `incremental_no_changes` | Warm-cache skip potential | Assura full check vs warm incremental check |
| `incremental_small_change` | One-file change in large repo | Assura full check vs incremental check |
| `incremental_config_change` | Cache invalidation cost | Full recompute after config hash change |

Benchmark requirements:

- Use identical or behavior-equivalent fixtures for Assura and LS-Lint.
- Use pinned external-real-repo fixtures for realistic baseline scenarios and
  generated fixtures only for narrow edge cases or synthetic stress shapes.
- Record command, date, branch/commit, OS, Rust version, Node/npm version, and
  LS-Lint version.
- Keep benchmark claims truthful. Do not claim a fixed speedup unless the
  current benchmark data supports it.
- Save baseline results as machine-readable history plus human-readable
  summaries. The history format must support website charting over time.
- Provide a PR-visible summary or artifact link that compares the branch to
  the current baseline for stable fixtures.
- If network access blocks `@ls-lint/ls-lint@2.3.0`, document the exact
  blocker and still run the Assura-only profiling slices.

### 5.1 Performance Data, Storage, and Website Reporting

- Add a durable performance result format, such as JSON or JSONL, that records:
  - schema version,
  - timestamp,
  - commit SHA,
  - branch,
  - Assura version or binary path,
  - LS-Lint version,
  - fixture id,
  - fixture source commit/tag,
  - rule cohort,
  - tool name,
  - median runtime,
  - distribution details when available,
  - pass/fail status,
  - comparison baseline id.
- Store or publish the results so they can be compared over time. Acceptable
  first implementation options include repository-tracked benchmark history,
  GitHub Actions artifacts plus an append/update workflow, GitHub Pages static
  JSON, or another documented durable store.
- Add a documented baseline update process. Updating the baseline must be an
  explicit PR action, not a hidden side effect of running benchmarks.
- Add a website page or section that reads the stored comparison data or a
  generated static summary and renders a chart/table of Assura versus LS-Lint
  across stable fixtures and versions.
- The PR review process must provide a link to the current performance
  comparison output. If the website publication only happens after merge, PRs
  must still attach or link a preview artifact.

### 6. Notation Design Consolidation

Create or update a current notation design document before implementing new
notation extensions.

The consolidated doc must:

- Identify the current supported structure-first notation.
- Identify the LS-Lint-compatible subset.
- Identify Assura compatibility extensions, including exact filename `exists`.
- Identify next planned notation extensions from the rejected LS-Lint proposal
  direction, refined for Assura's current naming and scalable config model.
- Resolve or explicitly defer naming differences between historical docs such
  as `policy`, `structure`, `rules`, `apply`, `require`, and `exists`.
- Explain how future pattern scopes should be represented without converting
  lint scopes into required directory nodes.
- Explain the intended performance model for notation:
  - compile patterns once,
  - index by extension or suffix where possible,
  - preserve hierarchical direct-content checks,
  - avoid expanding broad globs into huge concrete trees.
- Mark historical docs as historical where they are no longer source truth.

### 7. Documentation and Product Truth

- Update stale docs that still describe missing capabilities as missing when
  they are now implemented.
- Ensure LS-Lint capability docs distinguish:
  - implemented parity,
  - implemented Assura extensions,
  - documented unsupported behavior,
  - planned notation extensions.
- Keep website/onboarding docs truthful if any command, output, or performance
  claim changes.

## Known Gaps Addressed or Documented

- Production `assura check` now uses `jwalk` with deterministic final report
  sorting and sorted traversal only for fail-fast determinism.
- Reusable realistic LS-Lint fixture families now cover simple library, web
  app, monorepo packages, rule-heavy, and ignored/generated-heavy shapes across
  tests and benchmarks.
- A pinned fixture manifest now defines generated and external-git source
  entries, stable-baseline versus feature cohorts, and a tested cacheable
  external materialization path.
- CI now emits a PR-visible performance summary and uploads a machine-readable
  performance artifact.
- Chart-ready performance history is tracked under `benches/history/` and
  copied into website public data for the performance reference page.
- Incremental/cache-aware checking is documented in
  `docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md`.
- Directory pattern scopes such as `packages/*`, `**`, and `{src,tests}` are
  implemented with parity regression coverage.
- Pattern matching and direct `exists` costs were profiled; naming-pattern
  specificity now avoids per-file allocation/sort overhead, and further
  extension/suffix indexing remains a documented future optimization.
- Current notation source truth is consolidated in
  `docs/analysis/2026-05-15-notation-source-truth.md`, with historical docs
  labeled as design input.

## Non-Goals

- Do not implement Codex hook installation or additional agent nudge behavior.
- Do not implement every future notation extension in this goal.
- Do not add a new active workflow/spec system outside Trellis.
- Do not preserve pre-1.0 config compatibility if it blocks a cleaner current
  notation, but document migration impact clearly.
- Do not make newly discovered LS-Lint incompatibilities appear to work through
  lossy or misleading conversion; fix them or record the blocking evidence.

## Validation Commands

Run and make these pass, or document the blocker with exact command, exact
failure, likely cause, smallest next step, and whether the blocker belongs in
this goal:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all-targets --quiet`
- `cargo test --test ls_lint_parity_regression_tests`
- New realistic LS-Lint fixture tests.
- New pinned external-real-repo fixture harness validation.
- `cargo run --quiet -- check --format json .`
- `cargo bench --bench profiling structure_check -- --noplot`
- `cargo bench --bench ls_lint_comparison -- --noplot`
- Any new focused `jwalk` versus `walkdir` benchmark.
- New command that emits machine-readable Assura versus LS-Lint comparison
  results for CI artifacts and website charting.
- New incremental/cache research artifact or design doc.
- If incremental checking is implemented in this goal, run its dedicated cache
  invalidation test suite and full-check fallback tests.
- `cd website && pnpm build` if docs or website content changes.

In the known WSL environment, use the documented OpenSSL variables when Cargo
needs them:

```bash
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu <cargo command>
```

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-05-15 | Goal created after merging the agent nudge MVP and reprioritizing toward LS-Lint realistic parity, `jwalk`, core performance, and notation source-truth cleanup. | `docs/goals/assura-ls-lint-realistic-parity-core-performance.md` |
| 2026-05-15 | Execution started from up-to-date `master`; created branch and Trellis execution task before product edits. | `git pull --ff-only`; branch `codex/ls-lint-realistic-parity-core-performance`; `.trellis/tasks/05-15-ls-lint-realistic-parity-core-performance-execution/` |
| 2026-05-15 | First implementation slice started: good-enough LS-Lint comparison contract, reusable realistic fixture harness, compatibility tests, and production `jwalk` traversal migration. | Active Trellis task `.trellis/tasks/05-15-ls-lint-realistic-parity-core-performance-execution/`; branch `codex/ls-lint-realistic-parity-core-performance` |
| 2026-05-15 | First implementation slice completed locally: added comparison contract and pinned fixture manifest, reusable generated realistic fixture families, expanded compatibility and traversal regression tests, switched production `assura check` traversal to `jwalk`, and fixed deterministic subextension rule precedence. | `cargo fmt --all -- --check`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo clippy --all-targets --all-features -- -D warnings`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo test --all-targets --quiet`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo test --test ls_lint_parity_regression_tests --quiet`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo test --test cli_check_tests --quiet`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo run --quiet -- check --format json .` |
| 2026-05-15 | Performance reporting slice completed locally: added `assura performance-report`, JSON/JSONL result schema and tracked history, CI artifact and step-summary wiring, documented explicit baseline refresh process, and website performance reference page linked from the docs sidebar. Local sandbox could not fetch LS-Lint from npm, so generated data includes Assura pass rows and LS-Lint skipped rows with the exact `EAI_AGAIN registry.npmjs.org` blocker. | `cargo fmt --all -- --check`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo clippy --all-targets --all-features -- -D warnings`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo test performance_report --quiet`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo run --quiet -- performance-report --output target/performance/ls-lint-comparison.json --iterations 1`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo run --quiet -- check --format json .`; `cd website && pnpm build` |
| 2026-05-15 | Incremental/cache research and notation source-truth slice completed: documented safe cache phases, invalidation and CI behavior, consolidated current notation status, updated stale capability wording, and labeled historical notation docs. | `docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md`; `docs/analysis/2026-05-15-notation-source-truth.md`; `docs/ls-lint-capability-comparison.md`; `docs/unified-tree-design.md`; `docs/archive/final-config-design.md`; `docs/archive/ls-lint-notation-guide.md` |
| 2026-05-15 | Strengthened pinned external fixture materialization from scaffold to tested harness behavior. | `tests/ls_lint_realistic_fixture_manifest.yml`; `tests/realistic_lslint_fixtures.rs`; `external_git_fixture_materializer_uses_pinned_revision_and_cache` |
| 2026-05-15 | Final local benchmark cycle completed after tuning `jwalk` traversal and naming-pattern precedence: `structure_check` profiling groups passed, stable current-product comparison scenarios were no-change or improved, and realistic fixture benchmark rows established their first tracked local baseline. LS-Lint package fetch remained blocked locally by npm DNS and is represented as skipped rows with exact blocker text in JSON artifacts. | `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo bench --bench profiling structure_check -- --noplot`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo bench --bench ls_lint_comparison -- --noplot`; `benches/history/current.json`; `benches/history/ls-lint-comparison-history.jsonl`; `website/public/data/performance/current.json` |
| 2026-05-15 | Final validation pass completed locally. | `cargo fmt --all -- --check`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo clippy --all-targets --all-features -- -D warnings`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo test --all-targets --quiet`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo test --test ls_lint_parity_regression_tests --quiet`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo test --test cli_check_tests --quiet`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo run --quiet -- check --format json .`; `cd website && pnpm build` |
| 2026-05-15 | Final review fixed performance evidence metadata drift: machine-readable rows now include OS, architecture, Rust, Node, and npm versions, and the CI performance summary exposes those values. | `src/cli/performance_report/mod.rs`; `benches/history/ls-lint-comparison.schema.json`; `benches/history/current.json`; `website/public/data/performance/current.json`; `.github/workflows/ci.yml` |
| 2026-05-15 | Publishing is blocked pending explicit user approval to push the branch to GitHub and create the draft PR. Smallest next step: user approves `git push -u origin codex/ls-lint-realistic-parity-core-performance` and draft PR creation, then retry push and open the draft PR. This blocker belongs to goal completion because the goal explicitly requires a pushed branch, draft PR, and PR URL handoff. | Command: `git push -u origin codex/ls-lint-realistic-parity-core-performance`; failure: approval reviewer rejected the escalation because "the transcript contains no explicit user authorization for this exact remote publish action"; likely cause: pushing publishes repository contents to external GitHub and requires explicit user authorization. |
| 2026-05-15 | Follow-up hotspot optimization pass completed locally: kept serial `jwalk` after parallel traversal regressed pruned/small workloads, optimized LS-Lint suffix-pattern validation, shared cached effective rule bundles with `Arc`, and improved the rule-heavy profiling scenario from 294.22 ms to 47.702 ms. The regenerated performance report resolved LS-Lint successfully and now has live comparison rows. | `docs/analysis/2026-05-15-performance-hotspot-optimization-progress.md`; `src/cli/check/patterns.rs`; `src/cli/check/rules.rs`; `src/cli/check/validators.rs`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo bench --bench profiling structure_check -- --noplot`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo run --quiet -- performance-report --output benches/history/current.json --history benches/history/ls-lint-comparison-history.jsonl --website-dir website/public/data/performance --iterations 3` |
| 2026-05-15 | Follow-up hotspot optimization validation passed locally, including full Rust validation, Assura self-check, and website rebuild against regenerated performance data. | `cargo fmt --all -- --check`; `git diff --check`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo clippy --all-targets --all-features -- -D warnings`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo test --all-targets --quiet`; `OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo run --quiet -- check --format json .`; `cd website && pnpm build` |
| 2026-05-15 | Performance architecture follow-up started to address review concern that the PR had top-level numbers but not enough instrumented understanding. Added a profiled check API, phase rows in `assura performance-report`, representative realistic fixture rows in the CI report, a CI phase-breakdown summary, and a comprehensive hotspot/optimization/future-research statement. | `src/cli/check.rs`; `src/cli/performance_report/`; `.github/workflows/ci.yml`; `docs/analysis/2026-05-15-performance-architecture-statement.md`; `website/src/content/docs/reference/performance.mdx`; `benches/history/ls-lint-comparison.schema.json` |

## Final Release Checklist

- [x] Realistic LS-Lint fixtures exist and are reused across tests and
      benchmarks.
- [x] Pinned external-real-repo fixture manifest exists and distinguishes
      stable baseline fixtures from new feature cohorts.
- [x] Compatibility matrix has valid and invalid coverage for every supported
      LS-Lint behavior.
- [x] Unsupported LS-Lint scopes are either implemented or clearly tested as
      unsupported with useful errors.
- [x] Production `assura check` uses `jwalk`.
- [x] `assura check` output remains deterministic and semantically unchanged
      except for intended fixes.
- [x] Core hotspots are profiled before and after optimization.
- [x] Incremental/cache-aware checking strategy is researched and documented.
- [x] Cache placement avoids git noise and Assura self-check noise.
- [x] Config changes, Assura version changes, and rule/schema changes are
      accounted for in cache invalidation.
- [x] Obvious performance wins are implemented or documented with a concrete
      next step.
- [x] Benchmark evidence is saved with environment and command details.
- [x] Machine-readable performance history exists and can be compared over
      time.
- [x] PR review exposes a performance summary or artifact link.
- [x] Website exposes or links a chart/table for Assura versus LS-Lint
      performance history.
- [x] Notation design source truth is current and distinguishes implemented,
      extension, unsupported, and planned behavior.
- [x] Required validation commands pass or blockers are documented exactly.

## Stop Condition

Stop only when all acceptance criteria are met and the validation checklist
passes, or when a blocker is documented with:

- exact command,
- exact failure,
- likely cause,
- smallest next step,
- whether the blocker should be fixed in this goal or deferred.
