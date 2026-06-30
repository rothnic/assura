# Tooling Stabilization

Assura is still pre-1.0 and the development workflow is not yet clean enough to
treat every intended quality gate as blocking. Agents must keep the difference
between product work and tooling-baseline work explicit.

## Current Gate Policy

- Do not merge feature work that introduces new validation debt, duplicated
  systems, unused scaffolding, compatibility layers, or unclear source-of-truth
  paths.
- Keep `cargo test --all-targets --quiet` passing on the local development
  platform before pushing implementation changes.
- Keep repository-wide Rust formatting clean. `cargo fmt --all -- --check` is
  expected to pass in CI after the dedicated rustfmt baseline cleanup.
- Keep Clippy blocking in CI with
  `cargo clippy --all-targets --all-features -- -D warnings`.
- Keep Assura hooks advisory until the repo passes its own `.assura/config.yml`
  baseline consistently on `master` and CI.
- Document every paused or non-blocking check here before treating it as
  acceptable.

## Change-Scoped Validation

Use the narrowest gate that proves the changed surface. This keeps docs and
workflow cleanup from paying the full Rust test cost while preserving strong
checks for product behavior.

| Change surface | Expected local gate |
| --- | --- |
| AGENTS, Trellis workflow, skills, specs, docs analysis | `python3 ./.trellis/scripts/workflow_gate.py --platform <current-platform>` (Codex: `--platform codex`), `cargo run --quiet -- check --format json .`, `cargo xtask evidence`, `git diff --check` |
| Website docs/content | Above plus `cargo xtask docs` |
| Rust source, Cargo metadata, CLI behavior, tests, benchmarks | `cargo xtask pr` or the task-specific Rust commands from the PRD |
| Release packaging, install scripts, performance evidence | Relevant release/performance skill plus `cargo xtask release-smoke` or `cargo xtask full` as appropriate |

Do not run the full Rust suite just because any file changed. Do run it when a
docs/workflow change alters a command contract, CI behavior, release process, or
validation logic that Rust tests exercise.

Assura owns this policy in `.assura/config.yml` under `quality.scopes`.
`assura quality plan` is the config-backed command surface for planning checks
from changed paths and workflow phase. Phases are cumulative for normal
development: `frequent`, `pre-push`, `pr`, `merge`, then `release`;
`scheduled` is reserved for background audits.

Local development should use `cargo xtask changed` as the default
changed-file gate. It shells out to `assura quality plan`, executes selected
local commands, skips GitHub-only check names, and does not rerun narrower
local checks that are covered by a selected broader gate such as
`cargo xtask pr`. Use `--dry-run`, `--files-from`, `--base`, and `--head`
when the changed-file set must be deterministic for review or timing evidence.

Do not adopt generic command caches blindly. Cargo already owns the warm
`target/` cache for normal incremental builds; generic task caches such as Nx or
Turborepo are only useful once tasks have deterministic inputs and declared
outputs worth restoring. For Assura's current Rust validation loop, prefer
scoped gates first, then Rust-native tooling only after a local timing probe
shows a win.

GitHub Actions uses the native cache service through `Swatinem/rust-cache`.
Rust jobs should prefer explicit shared keys by OS/toolchain/profile/target so
related CI jobs and reruns can reuse compiled artifacts. Cache-hit state must be
written to the job summary with `scripts/summarize-rust-cache.sh` so future
timing claims can distinguish cold runs, warm restores, and exact key hits.

CI uses `scripts/ci-scope.sh` as the lightweight bootstrap classifier before
running expensive jobs. The script should mirror `quality.scopes`, but it must
not call `cargo run -- quality plan` in the first scope job because compiling
Assura there would erase the speed win for docs-only changes. The classifier is
intentionally conservative: Rust/Cargo changes run Rust, release, coverage,
rustdoc, and performance gates; release/install changes run release gates;
performance evidence changes run performance gates; workflow or classifier
changes run everything. Docs, Trellis, skills, Assura config, and
agent-policy-only changes keep the evidence gates and Assura self-check active
without scheduling the expensive Rust, release, rustdoc, coverage, and
performance jobs.

The Security Audit workflow also uses the classifier instead of workflow-level
path filters. It runs for Cargo metadata changes and scheduled audits, while
source-only Rust changes use the Rust compile/test gates without scheduling a
dependency audit.

## Performance Evidence Contract

Performance claims that compare Assura with LS-Lint must state the executable
contract being measured and must make that contract visible in machine-readable
evidence.

### Scope / Trigger

- Trigger: any change to `assura performance-report`, `benches/`, checked-in
  performance history, or website performance copy.

### Signatures

- `assura performance-report --output <path> [--history <path>]`
- `cargo xtask performance-no-slower [report.json] [--cohort <name>]
  [--assura-row <row>] [--ls-lint-row <row>]`
- Criterion benchmark: `cargo bench --bench ls_lint_comparison -- --noplot`

### Contracts

- LS-Lint package: keep the exact package spec in report metadata.
- LS-Lint executable: resolve and execute the packaged native binary under
  `node_modules/@ls-lint/ls-lint/bin/`.
- Primary launcher: build `assura` with `--no-default-features --features
  json-output,yaml-config` so the normal `assura check` path excludes full CLI,
  markdown, intelligence, graph, watch, config-validation derive, and git
  dependency surfaces.
- Full companion: build `assura-full` separately so non-check commands remain
  available next to the lightweight launcher in release bundles.
- Check-only support executable: build `assura-check-cli` separately with
  `assura = { default-features = false }` so diagnostic support rows keep the
  same low-latency dependency boundary.
- Release profile: keep check-only release evidence on the workspace release
  profile with LTO, one codegen unit, stripping, and `panic = "abort"` unless a
  report explicitly documents a different profile.
- Measurement loop: do not time `npm exec`, package resolution, or the package
  Node wrapper in headline LS-Lint rows.
- Evidence rows: include the LS-Lint tool name, execution mode, and binary path
  when the row measures LS-Lint.
- No-slower gate: read an existing performance report and fail without running
  benchmarks if any paired headline Assura row is slower than native LS-Lint or
  if a required paired row is missing. Default inputs are
  `benches/history/current.json`, cohort `realistic-equivalent`, Assura row
  `assura-cli`, and LS-Lint row `ls-lint-cli`.
- Native LS-Lint metadata is part of the no-slower gate: selected LS-Lint rows
  must use `tool_name=ls-lint-native-cli` and
  `ls_lint_execution_mode=native-binary-from-pinned-npm-package`.

### Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Package install fails | Emit skipped LS-Lint rows with the exact blocker. |
| Native binary is missing for the current platform | Emit skipped LS-Lint rows; do not fall back to the Node wrapper silently. |
| Website copy mentions a winner | Derive the winner from current generated data, not from prior run assumptions. |
| Any headline fixture has Assura median runtime greater than native LS-Lint | `cargo xtask performance-no-slower` exits nonzero and prints the fixture ID. |
| A headline fixture is missing either paired row | `cargo xtask performance-no-slower` exits nonzero and identifies the missing row. |

### Good / Base / Bad Cases

- Good: `tool_name=ls-lint-native-cli`,
  `ls_lint_execution_mode=native-binary-from-pinned-npm-package`.
- Base: Assura and LS-Lint rows run on the same fixture tree and equivalent
  native LS-Lint rules.
- Bad: Timing `node_modules/.bin/ls-lint` or `npm exec` while labeling the row
  as LS-Lint binary performance.

### Tests Required

- Unit coverage for native binary path selection and row metadata.
- Unit coverage for the no-slower gate using small synthetic reports.
- Website build after checked-in report data changes.
- A regenerated report proving the checked-in JSON uses the native execution
  mode.

### Wrong vs Correct

Wrong:

```text
npm exec --package @ls-lint/ls-lint@2.3.0 -- ls-lint
```

Correct:

```text
node_modules/@ls-lint/ls-lint/bin/ls-lint-<platform>
```

## Deferred Baseline Issues

| Issue | Current Evidence | Treatment | Re-enable / Close Criteria |
| --- | --- | --- | --- |
| Coverage reporting is local to CI | Code coverage generation succeeds, but hosted Codecov upload added external account, token, and rate-limit failure modes before the core tooling baseline was stable. | Keep `cargo tarpaulin` coverage generation in CI, summarize coverage in the GitHub job summary, and publish the Cobertura XML as a GitHub Actions artifact. Do not require Codecov for the current workflow. | Decide on a coverage threshold and enforce it locally in CI, or adopt a hosted service only when trend dashboards and PR annotations are worth the extra dependency. |
| Assura hooks remain advisory | Local cleanup on `codex/assura-self-check-baseline-cleanup` reduced `cargo run -- check .` to zero violations. | Keep hooks advisory until the clean baseline lands on `master` and is observed through normal CI/developer flow. | After the clean baseline is merged and remains stable, switch protected-path pre-push behavior from advisory to blocking or document the remaining reason not to. |

## Next Iteration Plan

1. Stabilize CI signal quality.
   - Pause only checks that are explicitly recorded above.
   - Convert expected baseline failures into tracked cleanup work instead of
     undocumented red checks.
   - Prefer GitHub-native artifacts and summaries over external reporting
     services until the required credentials and blocking policy are justified.
   - Keep platform tests active for Linux, macOS, and Windows unless a new
     platform-specific blocker is recorded here with re-enable criteria.

2. Promote Assura self-check from clean to enforced.
   - Confirm the clean baseline after this cleanup lands on `master`.
   - Keep archived historical docs under `docs/archive/` and removed OpenSpec
     surfaces out of active workflow paths.
   - Move hooks from advisory to blocking only after the clean baseline is
     stable on the protected branch.

3. Monitor restored Windows CI.
   - Treat a new `windows-latest` failure as a fresh CI regression unless it is
     recorded in Deferred Baseline Issues with explicit close criteria.
   - Preserve release Windows smoke jobs separately from the Rust test matrix.

## Closed Baselines

| Issue | Resolution |
| --- | --- |
| Repository-wide rustfmt drift | Dedicated formatting cleanup landed in PR #2. `cargo fmt --all -- --check` is now expected to pass in CI. |
| Repository-wide clippy warnings | Dedicated Clippy cleanup removed the existing warning baseline. `cargo clippy --all-targets --all-features -- -D warnings` is now expected to pass locally and block in CI. |
| Assura self-check violations | Dedicated self-check cleanup archived historical docs, removed non-canonical OpenSpec surfaces, added missing Rust module docs, and split oversized modules. `cargo run -- check .` now reports zero violations locally on the cleanup branch. |
| Windows CI test job restored | PR #93 updated `git2` from 0.18.3 to 0.21.0, which refreshes `libgit2-sys` from 0.16.2+1.7.2 to 0.18.5+1.9.4, then restored `windows-latest` to the Rust `Test Suite` matrix. Hosted Rust CI run `27839085592` showed `Test Suite (windows-latest, stable)` job `82393863614` passing before merge commit `272f3debc107c6ca29674130d9acbe67e23c7a40`. |

## Agent Rules

- If a CI failure matches this file, report it as known baseline debt and point
  to the owning next iteration.
- If a CI failure is not listed here, treat it as new and investigate before
  merging.
- If a PR pauses a check, update this file in the same PR with the reason,
  owner criteria, and re-enable criteria.
