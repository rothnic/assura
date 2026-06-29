# Project Intelligence Runtime Completion Audit

## Goal

Audit `docs/goals/assura-project-intelligence-runtime-program.md` against live
repo evidence after completing the ninth successor goal. The task decides
whether the master Project Intelligence Runtime program can be marked complete,
or records exact remaining blockers.

## Requirements

- Check every successor in the master program execution sequence.
- Verify each successor goal has current completion evidence, status metadata,
  and archived or otherwise resolved Trellis task state.
- Verify every Program Definition Of Done item in the master goal against
  current code, docs, command output, tests, and support matrices.
- Fix documentation or metadata drift discovered during the audit when the fix
  is mechanical and evidence-backed.
- Do not mark the master goal complete unless all requirements are proven by
  current evidence.

## Acceptance Criteria

- [ ] Audit table maps each successor to status, evidence, validation, review,
  and task/archive state.
- [ ] Program Definition Of Done table maps each item to current proof or an
  explicit blocker.
- [ ] Any stale successor metadata found during audit is fixed or recorded as a
  blocker with exact file paths.
- [ ] Required final validation commands pass or exact blockers are recorded.
- [ ] Master goal status is changed only if the audit proves completion.

## Validation Commands

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask evidence
cargo xtask docs
git diff --check
```

## Review Tasks

- R1: Confirm the audit does not redefine the master program scope around the
  work just completed.
- R2: Confirm each successor completion claim cites live repo evidence.
- R3: Confirm every Program Definition Of Done item has direct proof, not only
  indirect test coverage.
- R4: Confirm any unresolved blocker keeps the master program open.

## Progress Evidence

- 2026-06-29: Started after completing and archiving the Project Intelligence
  Agent Surfaces successor. Initial live scan found at least one likely
  metadata drift case: an earlier successor is described as completed in the
  roadmap but still has `status: planned` in its goal frontmatter. The audit
  will verify all successor goal metadata before making any master completion
  claim.
