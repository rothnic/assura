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

## Performance Evidence Contract

Performance claims that compare Assura with LS-Lint must state the executable
contract being measured and must make that contract visible in machine-readable
evidence.

### Scope / Trigger

- Trigger: any change to `assura performance-report`, `benches/`, checked-in
  performance history, or website performance copy.

### Signatures

- `assura performance-report --output <path> [--history <path>]`
- Criterion benchmark: `cargo bench --bench ls_lint_comparison -- --noplot`

### Contracts

- LS-Lint package: keep the exact package spec in report metadata.
- LS-Lint executable: resolve and execute the packaged native binary under
  `node_modules/@ls-lint/ls-lint/bin/`.
- Check-only executable: build `assura-check-cli` separately with
  `assura = { default-features = false }` so benchmark rows exclude full CLI,
  markdown, intelligence, graph, watch, config-validation derive, and git
  dependency surfaces.
- Release profile: keep check-only release evidence on the workspace release
  profile with LTO, one codegen unit, stripping, and `panic = "abort"` unless a
  report explicitly documents a different profile.
- Measurement loop: do not time `npm exec`, package resolution, or the package
  Node wrapper in headline LS-Lint rows.
- Evidence rows: include the LS-Lint tool name, execution mode, and binary path
  when the row measures LS-Lint.

### Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Package install fails | Emit skipped LS-Lint rows with the exact blocker. |
| Native binary is missing for the current platform | Emit skipped LS-Lint rows; do not fall back to the Node wrapper silently. |
| Website copy mentions a winner | Derive the winner from current generated data, not from prior run assumptions. |

### Good / Base / Bad Cases

- Good: `tool_name=ls-lint-native-cli`,
  `ls_lint_execution_mode=native-binary-from-pinned-npm-package`.
- Base: Assura and LS-Lint rows run on the same fixture tree and equivalent
  native LS-Lint rules.
- Bad: Timing `node_modules/.bin/ls-lint` or `npm exec` while labeling the row
  as LS-Lint binary performance.

### Tests Required

- Unit coverage for native binary path selection and row metadata.
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
| Windows CI test job paused | GitHub Actions `windows-latest` test job failed linking `libgit2_sys` with unresolved MSVC symbols including `GetNamedSecurityInfoW`, registry APIs, and CryptoAPI symbols. | Temporarily removed from the PR test matrix while stabilizing core workflow. This is not a product-behavior signal yet. | Add the required Windows linker/system-library fix or dependency configuration, prove `cargo test --all-features` passes on `windows-latest`, then restore Windows to the CI matrix. |
| Coverage reporting is local to CI | Code coverage generation succeeds, but hosted Codecov upload added external account, token, and rate-limit failure modes before the core tooling baseline was stable. | Keep `cargo tarpaulin` coverage generation in CI, summarize coverage in the GitHub job summary, and publish the Cobertura XML as a GitHub Actions artifact. Do not require Codecov for the current workflow. | Decide on a coverage threshold and enforce it locally in CI, or adopt a hosted service only when trend dashboards and PR annotations are worth the extra dependency. |
| Assura hooks remain advisory | Local cleanup on `codex/assura-self-check-baseline-cleanup` reduced `cargo run -- check .` to zero violations. | Keep hooks advisory until the clean baseline lands on `master` and is observed through normal CI/developer flow. | After the clean baseline is merged and remains stable, switch protected-path pre-push behavior from advisory to blocking or document the remaining reason not to. |

## Next Iteration Plan

1. Stabilize CI signal quality.
   - Pause only checks that are explicitly recorded above.
   - Convert expected baseline failures into tracked cleanup work instead of
     undocumented red checks.
   - Prefer GitHub-native artifacts and summaries over external reporting
     services until the required credentials and blocking policy are justified.
   - Keep platform tests active for Linux and macOS while Windows is paused.

2. Promote Assura self-check from clean to enforced.
   - Confirm the clean baseline after this cleanup lands on `master`.
   - Keep archived historical docs under `docs/archive/` and removed OpenSpec
     surfaces out of active workflow paths.
   - Move hooks from advisory to blocking only after the clean baseline is
     stable on the protected branch.

3. Re-enable Windows.
   - Investigate the `libgit2_sys` MSVC linker failure after the main workflow
     gates are reliable.
   - Restore the `windows-latest` matrix entry once the fix is proven in CI.

## Closed Baselines

| Issue | Resolution |
| --- | --- |
| Repository-wide rustfmt drift | Dedicated formatting cleanup landed in PR #2. `cargo fmt --all -- --check` is now expected to pass in CI. |
| Repository-wide clippy warnings | Dedicated Clippy cleanup removed the existing warning baseline. `cargo clippy --all-targets --all-features -- -D warnings` is now expected to pass locally and block in CI. |
| Assura self-check violations | Dedicated self-check cleanup archived historical docs, removed non-canonical OpenSpec surfaces, added missing Rust module docs, and split oversized modules. `cargo run -- check .` now reports zero violations locally on the cleanup branch. |

## Agent Rules

- If a CI failure matches this file, report it as known baseline debt and point
  to the owning next iteration.
- If a CI failure is not listed here, treat it as new and investigate before
  merging.
- If a PR pauses a check, update this file in the same PR with the reason,
  owner criteria, and re-enable criteria.
