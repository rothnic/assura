---
title: Content Runtime Index Performance
status: active
---

# Content Runtime Index Performance

## Decision

Use an internal single-walk file index for the current repo-native content
runtime. Do not add Deeb, SQLite, or another persistent cache/index dependency
for this release slice.

The runtime files remain the canonical source of truth. The index is an
ephemeral in-memory grouping of matched repository files by configured
collection. It is rebuilt for each validation and is not exposed as a public
model source.

## Evidence

Local command:

```bash
cargo bench --bench content_runtime -- --noplot
```

Local environment: macOS x86_64, branch
`codex/content-runtime-index-performance`, run after adding the single-walk
content file index.

Scenario: generated repository with 240 Markdown goal records, 240 JSON spec
records, and 240 unrelated Markdown notes. The comparable no-content-runtime
baseline uses the same file tree and a structure-only Assura config.

| Path | Median |
| --- | ---: |
| `content_runtime/repository_validate_warm/240_goals_240_specs` | 21.370 ms |
| `content_runtime/repository_cold_in_process/240_goals_240_specs` | 21.756 ms |
| `content_runtime_check/assura_check_cold_in_process/no_content_runtime_baseline` | 1.232 ms |
| `content_runtime_check/assura_check_cold_in_process/with_content_runtime` | 21.826 ms |

The measured `assura check` content-runtime overhead is about 20.594 ms for
480 validated objects plus 240 unrelated files on this machine. The direct
repository benchmark and the full `assura check` benchmark are close, which
indicates the validation work, not structure-check dispatch, dominates this
fixture.

`repository_validate_warm` reuses the compiled runtime schema validators.
`repository_cold_in_process` reloads config, rebuilds the repository model,
recompiles schema validators, and validates on each measured iteration.
`assura_check_cold_in_process` runs the public check path in-process, including
config discovery/load and content repository construction, but excludes process
startup. Process cold-start evidence remains covered by the existing
`assura performance-report` history.

The tracked release performance report was refreshed with:

```bash
target/release/assura performance-report \
  --output benches/history/current.json \
  --history benches/history/ls-lint-comparison-history.jsonl \
  --website-dir website/public/data/performance \
  --iterations 5
```

That report is broader structure-first release evidence rather than the
content-runtime-specific benchmark above. Its current cold headline verdict
remains `not-complete`; the warm persistent-session verdict remains
`complete`.

## Cache And Deeb Assessment

The implemented single-walk index removes the avoidable
`collections * project_files` traversal shape from the first runtime slices.
Each repository file is walked once, normalized once, and matched against the
configured collection patterns. Matches are then processed in collection order
and sorted by normalized path, preserving deterministic diagnostics.

Deeb is rejected for this increment because the measured workload is a
single-pass validation workload, not a repeated-query workload. Adding a
database/cache dependency would introduce a second state surface without
proving enough value for normal `assura check`. If later goals add persistent
editor sessions, daemonized validation, or repeated graph queries across many
content operations, Deeb can be reconsidered against those workloads.

## Release Notes

- Accepted: internal ephemeral file index for collection matching.
- Rejected for now: persistent Deeb/SQLite cache as a runtime dependency.
- Still required before release readiness: broader performance evidence on
  larger real repositories and hosted CI artifact review.
