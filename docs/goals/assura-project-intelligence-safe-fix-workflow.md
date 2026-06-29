---
id: goal-assura-project-intelligence-safe-fix-workflow
type: goal
title: Assura project intelligence safe fix workflow
status: completed
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-context-pack.md
  - docs/goals/assura-rust-markdown-validation-and-fixing.md
  - docs/goals/assura-project-intelligence-agent-cli-surface.md
  - docs/goals/assura-project-intelligence-lsp-editor-transport.md
---

# Assura Project Intelligence Safe Fix Workflow

## Objective

Turn safe-fix dry-run support into a complete bounded repair workflow that
humans, agents, and editor integrations can preview, apply, audit, and recover
from.

## Current Gap

`assura fix markdown --dry-run --format json` returns a versioned dry-run
contract, and `assura content context-pack` exposes preview metadata without
writes. That is useful but not enough for everyday use: agents need a stable
plan, users need confidence before writes, and integrations need audit output
after applying changes.

## Scope

- Define a common safe-fix plan schema shared by CLI, agent envelopes, and
  future agent CLI, editor, or optional protocol wrappers.
- Support bounded apply for accepted fix classes with explicit opt-in.
- Include before/after counts, changed paths, applied fix IDs, skipped fixes,
  and failure reasons in machine-readable output.
- Preserve deterministic behavior and idempotency.
- Document recovery expectations, including VCS-first rollback guidance.
- Decide whether additional Markdown fixes are safe enough to include.

## Non-Goals

- No automatic repair without explicit approval.
- No broad formatter replacement.
- No semantic rewrite or content generation.
- No cross-file relation repair until a separate goal proves safety.

## Definition Of Done

- Dry-run and apply outputs share a stable schema family.
- Applying a safe fix is idempotent and bounded to documented fix classes.
- Tests cover no-op, dry-run, apply, partial skip, invalid path, and dirty
  non-target file behavior.
- Agent/editor surfaces can request previews without writes.
- Context-pack and transport surfaces can correlate previewed fixes with
  applied/audited fixes.
- Docs and support policy clearly classify safe-fix apply behavior.

## Validation Commands

```bash
cargo fmt --check
cargo test --test markdown_lint_fix_tests --quiet
cargo test --test project_intelligence_real_repo_proof beacon_crm_materialized_markdown_drift_previews_safe_fix_without_writing --quiet
cargo test --test content_query_cli --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo test --test content_runtime_dx_docs project_intelligence_demo_is_discoverable_and_covers_adoption_commands --quiet
cargo run --quiet -- fix markdown --dry-run --format json .
tmp=$(mktemp -d)
cp -R tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid/. "$tmp"
python3 - "$tmp/docs/epics/epic_checkout.md" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace("# Checkout Onboarding\n\n", "# Checkout Onboarding\n   \n"))
PY
cargo run --quiet -- fix markdown --apply --format json "$tmp"
cargo run --quiet -- check --format json .
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm every write path has an explicit user or integration opt-in.
- R2: Confirm dry-run and apply reports can be correlated by fix ID or path.
- R3: Confirm no unsafe rewrite is labeled as a safe fix.
- R4: Confirm failure modes leave the repository in a predictable state.

## Reviewer Blocking Criteria

Block if fixes can apply implicitly, if output hides which files changed, if
partially applied repairs are not reported, or if the implementation expands
into semantic rewriting without a separate safety proof.

## Progress Log

- 2026-06-29: Implemented locally on task
  `.trellis/tasks/06-29-06-29-project-intelligence-safe-fix-workflow`. The
  Markdown safe-fix command now previews by default, keeps `--dry-run` as an
  explicit no-write mode, and requires `--apply` before writing. JSON reports
  keep `assura.safe-fix.markdown.v1` and add `mode`, `rule`, before/after
  counts, per-file records, per-fix IDs, changed paths, applied fix IDs,
  skipped fixes, failures, and VCS-first rollback guidance. The only supported
  fix class remains configured blank-line trailing spaces.
- 2026-06-29: Added cross-surface correlation evidence: content-query
  safe-fix previews ingest Markdown safe-fix structure findings and expose
  `audit_id` values matching CLI dry-run `fixes[].id`, so context-pack,
  session, and future transport wrappers can correlate previews with apply
  audits without writing implicitly.
- 2026-06-29: Independent review found that partial apply failures could hide
  already-written changes because write errors returned only stderr. Fixed by
  recording failed files/fixes in the same JSON audit report, preserving
  already changed paths and applied IDs, and exiting nonzero when failures are
  present. Review also requested positive context-pack and session `audit_id`
  coverage; added both regression tests.
