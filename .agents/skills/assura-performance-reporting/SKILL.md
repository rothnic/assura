---
name: assura-performance-reporting
description: Use when changing Assura performance reporting, LS-Lint comparison rows, checked-in benchmark history, or website performance data.
---

# Assura Performance Reporting

Use this skill for `assura performance-report`, PR performance evidence, and
LS-Lint comparison changes.

## Quick Gate

Run a target artifact first before touching tracked history:

```bash
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --output target/performance/<name>.json \
  --history target/performance/<name>.jsonl \
  --website-dir target/performance/website-data \
  --iterations 3
```

Check the report shape:

```bash
jq -r '.claim_summary' target/performance/<name>.json
jq -r '.ls_lint_status' target/performance/<name>.json
jq -r '.results[] | [.fixture_id,.tool_name,.median_runtime_ms,.status] | @tsv' target/performance/<name>.json
jq -r '.results[] | select(.tool_name=="assura-check-cli") | [.fixture_id,.median_runtime_ms,.two_x_target_runtime_ms,.runtime_to_two_x_target_ratio,.process_floor_runtime_ms,.process_floor_blocks_two_x] | @tsv' target/performance/<name>.json
```

Expected row families:

- `assura-cli`
- `assura-check-cli`
- `assura-check-cached-cli`
- `assura-check-compiled-cli`
- `assura-check-hot-cli`
- `assura-check-changed-path-cli`
- `assura-check-dirty-project-cli`
- `assura-check-dirty-project-session-cli`
- `assura-check-dirty-project-socket`
- `assura-prepared-full-check`
- `assura-prepared-five-changed-paths`
- `assura-check-status-cli`
- `assura-rust-cli-floor`
- `process-floor`
- `ls-lint-cli`
- `assura-in-process`
- `assura:phase:config-discovery`
- `assura:phase:config-load`
- `assura:phase:checker-init`
- `assura:phase:configured-structure`
- `assura:phase:walk-and-validate`
- `assura:phase:report-sort`
- `traversal:walkdir`
- `traversal:jwalk-serial`
- `traversal:jwalk-parallel`
- `strategy:walkdir-cli`
- `strategy:jwalk-serial-cli`
- `strategy:jwalk-parallel-cli`

## Before/After Evidence

When a goal requires comparison to the current PR baseline, create a temporary
worktree for the baseline commit instead of checking out over current work:

```bash
git worktree add /private/tmp/assura-pr-baseline <baseline-sha>
cargo run --quiet -- performance-report \
  --output target/performance/baseline.json \
  --iterations 3
git worktree remove /private/tmp/assura-pr-baseline
```

Compare only runs from the same machine and similar build profile. Do not treat
checked-in history from another host as proof of a local regression or
improvement.

## Tracked Data

After the target artifact is valid, update tracked report data:

```bash
cargo build --release --bin assura --no-default-features --features json-output,yaml-config
cargo build --release --bin assura-full
cargo build --release -p assura-check-cli
target/release/assura performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
```

If website docs or public data changed, run:

```bash
cd website && pnpm build
```

If `pnpm build` fails because dependencies are missing, run `pnpm install` in
`website/` and retry the build.

## Review Notes

- Warm LS-Lint rows should execute the prepared native binary from the pinned
  package in measured loops, not repeat package resolution or time the package
  Node wrapper per iteration.
- Before proposing another cold-start micro-optimization for the native LS-Lint
  2x goal, check `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md`
  and `docs/archive/assura-native-ls-lint-performance-rearchitecture.md`. Already
  rejected experiments include `opt-level=z`, minimal quiet binaries, raw Unix
  entrypoints, quiet-only parser fast paths, default builds without JSON/cache,
  automatic compiled-plan caches, count integration, lazy relative path
  stringification, lazy file-stem computation, broad dot-suffix lookup maps,
  versioned status marker files, and single-read status-file parsing.
- Do not treat batch, compiled-config, hot daemon, dirty-project, status-file,
  or in-process rows as completion evidence for the cold `assura-cli`
  headline claim unless the goal explicitly changes execution model. These rows
  are useful diagnostics and scoped product modes; the public one-shot gate is
  currently driven by `claim_summary.assura_row_family="assura-cli"`.
- The current compiled-config runtime is already separated from YAML in the
  release artifact. Verify with `strings target/release/assura-check-compiled
  | rg "serde_yaml|unsafe-libyaml|serde_json|notify"` before proposing another
  crate split or parser swap.
- `rule_heavy_repo` was the main non-floor cold miss in earlier smoke runs.
  A narrow exact LS-Lint extension-segment lookup for non-wildcard patterns
  landed with no-slower evidence; reopen adjacent matching changes only with a
  clearly different design and before/after evidence.
- Build the primary `assura` launcher with `--no-default-features --features
  json-output,yaml-config`, then build `assura-full` and `assura-check-cli` in
  separate release invocations before producing performance evidence. A single
  `cargo build --release -p assura --bins` can link full CLI dependencies into
  the primary launcher and make `assura check` evidence slower than the release
  bundle.
- When changing `src/cli/performance_report/**`, build the full Assura release
  binary set before running `target/release/assura performance-report`.
  `performance-report` dispatches through the full companion binary, so
  rebuilding only the lightweight launcher can leave stale report/catalog code
  in the measured artifact.
- Do not infer check-binary payload from package-level `cargo tree -p
  assura-check-cli` alone. The package contains compiler, daemon, status, and
  client binaries, so the package tree includes YAML/JSON/notify dependencies
  even when per-binary release linking eliminates them. Verify suspected payload
  with `otool -L`, `strings target/release/<binary> | rg <crate>`, `size`, and
  a release performance smoke before adding another crate split.
- `assura-check-cli` intentionally depends on `assura` with
  `default-features = false`; keep full CLI, markdown, intelligence, graph,
  watch, config-validation derive, and git surfaces behind optional features so
  check-only evidence does not pay for unrelated startup work.
- Keep diagnostic traversal experiments behind full CLI features. The default
  check-only path should not link `jwalk`, `rayon`, or `crossbeam` unless a
  measured production path actually uses them.
- LS-Lint-compatible fast paths must be conservative: activate only for
  naming/count/ignore rules, fall back for richer Assura validation, and keep
  `--fail-fast` on the deterministic full engine unless sorted fast traversal is
  explicitly implemented and tested.
- Release evidence assumes the workspace release profile uses LTO, one codegen
  unit, stripping, and `panic = "abort"` unless a benchmark explicitly says
  otherwise.
- Linux cold-start evidence may use the repo alias
  `cargo build-assura-check-linux-static`, which builds the check-only package
  for `x86_64-unknown-linux-gnu` with static CRT flags. Label that evidence as
  Linux static-CRT evidence; this is the current checked-in cold 2x completion
  scope. Do not mix it with default dynamic local macOS rows.
- Keep `claim_summary` and `warm_claim_summary` separate in PR and website
  language. `claim_summary` is the cold release-artifact `assura check` gate;
  `warm_claim_summary` is the persistent editor-session gate.
- Website history should copy the intended full history when a history file is
  provided.
- Keep production traversal choices separate from traversal-only evidence rows:
  raw parallel `jwalk` can win traversal while full validation may still prefer
  deterministic serial validation.
- Keep user-facing docs focused on release-style `assura check` versus
  `ls-lint-cli` rows and the top-level `claim_summary` verdict. Put
  `assura-check-cli`, traversal, phase, hot daemon, status-file, and strategy
  tradeoffs on technical implementation pages unless they directly change the
  public product comparison.
