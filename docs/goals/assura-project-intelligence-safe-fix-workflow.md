---
id: goal-assura-project-intelligence-safe-fix-workflow
type: goal
title: Assura project intelligence safe fix workflow
status: planned
created: 2026-06-29
owners:
  - assura-maintainers
related:
  - docs/goals/assura-project-intelligence-usability-program.md
  - docs/goals/assura-project-intelligence-context-pack.md
  - docs/goals/assura-rust-markdown-validation-and-fixing.md
  - docs/goals/assura-project-intelligence-editor-agent-transports.md
---

# Assura Project Intelligence Safe Fix Workflow

## Objective

Turn safe-fix dry-run support into a complete bounded repair workflow that
humans, agents, and editor integrations can preview, apply, audit, and recover
from.

## Current Gap

`assura fix markdown --dry-run --format json` returns a versioned dry-run
contract, and the context-pack goal should expose preview metadata without
writes. That is useful but not enough for everyday use: agents need a stable
plan, users need confidence before writes, and integrations need audit output
after applying changes.

## Scope

- Define a common safe-fix plan schema shared by CLI, agent envelopes, and
  future editor/MCP wrappers.
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
cargo run --quiet -- fix markdown --dry-run --format json .
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
