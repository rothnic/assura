# Compact Project Review Common Issues

## Goal

Execute `docs/goals/assura-compact-project-review-common-issues.md` as the next
core structure-validation stability slice after performance-polish closure.
Give humans and local agents one compact project review path for: "Is this repo
healthy, and what should I fix or configure next?"

## Live Revalidation

- Performance micro-optimization is paused by
  `docs/analysis/2026-07-05-performance-decision-matrix.md`; the next product
  value is core structure validation quality and first diagnostic clarity.
- `cargo run --quiet -- check --format json .` is clean, so the compact review
  should not duplicate existing structure validation.
- `cargo run --quiet -- doctor . --format json` shows the current gap:
  structure passes, but recommended paths and inactive capabilities still need
  clearer prioritization.
- `cargo run --quiet -- content agent-query gaps . --format json` reports
  1,086 unresolved repository references, which must be filtered/classified
  before surfacing as high-priority guidance.
- Revalidation artifact:
  `docs/analysis/2026-07-05-compact-project-review-revalidation.md`.

## Requirements

- Provide one compact review command, recipe, or documented flow over existing
  Assura truth surfaces before adding new core behavior.
- Report structure check status, doctor/onboarding status, next actions,
  structure-fit guidance, content/reference gap summary, and finding severity.
- Keep JSON stable enough for agent wrappers and text concise enough for agent
  turns.
- Classify low-value generated/archive/log/benchmark reference noise as omitted
  or informational rather than blocking.
- Distinguish "fix this file" from "decide whether this path belongs in the
  project contract."
- Link to lower-level commands for detailed evidence.

## Non-Goals

- No automatic `.assura/config.yml` rewrites.
- No broad new search engine, persistent store, hosted service, MCP server, or
  editor-specific requirement.
- No replacement for `assura check`, `assura doctor`, or content-query
  commands.
- No hard blocking policy based only on raw unresolved-reference counts.

## Acceptance Criteria

- [ ] One compact review path exists for humans and local agents.
- [ ] JSON output is bounded, stable, and points to detailed evidence when
      needed.
- [ ] Text/agent output prioritizes fix-now, configure-intentionally,
      inspect-before-changing, and informational items.
- [ ] Structure-fit guidance tells agents to inspect the nearby project shape
      before adding directories or editing config.
- [ ] Generated/archive/log/benchmark reference noise is filtered, classified,
      or explicitly omitted.
- [ ] Tests cover clean repo, structure mismatch, duplicated/unmodeled path
      pressure, inactive guidance, noisy gap, and actionable gap scenarios.
- [ ] Docs show before-new-path, before-PR, and onboarding usage.

## Validation Commands

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo fmt --check
cargo test --test agent_surface_cli --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo test --test content_query_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
git diff --check
```

If the first implementation slice is docs-only, keep validation scoped to the
workflow gate, Assura self-check, docs, evidence, target-state, and diff
hygiene.

## Review Notes

- Block if the implementation invents a parallel validator instead of reusing
  existing Assura truth.
- Block if raw generated/archive/log reference noise becomes a hard failure.
- Block if agent output is unbounded or requires scraping text.
- Block if users cannot tell whether to fix a file or decide whether a path
  belongs in the repository contract.
