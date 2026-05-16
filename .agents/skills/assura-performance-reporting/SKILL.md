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
cargo run --quiet -- performance-report \
  --output target/performance/<name>.json \
  --history target/performance/<name>.jsonl \
  --website-dir target/performance/website-data \
  --iterations 3
```

Check the report shape:

```bash
jq -r '.ls_lint_status' target/performance/<name>.json
jq -r '.results[] | [.fixture_id,.tool_name,.median_runtime_ms,.status] | @tsv' target/performance/<name>.json
```

Expected row families:

- `assura`
- `ls-lint`
- `walkdir`
- `jwalk-serial`
- `jwalk-parallel`
- `assura:config-discovery`
- `assura:config-load`
- `assura:checker-init`
- `assura:configured-structure`
- `assura:walk-and-validate`
- `assura:report-sort`

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
cargo run --quiet -- performance-report \
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

- Warm LS-Lint rows should execute a prepared cached binary in measured loops,
  not repeat package resolution per iteration.
- Website history should copy the intended full history when a history file is
  provided.
- Keep production traversal choices separate from traversal-only evidence rows:
  raw parallel `jwalk` can win traversal while full validation may still prefer
  deterministic serial validation.
