# Project Intelligence Usability Gap Goals

## Objective

Evaluate what remains before Project Intelligence is usable as a product
workflow, then update Assura's roadmap and goal docs with an executable
successor set.

## Context

- Adoption blueprint and real-repo proof are now complete locally.
- The current program still points at four broad remaining buckets:
  persistent session, editor/agent transports, safe-fix workflow, and release
  hardening.
- The latest proof exposed more immediate usability gaps: starter setup and
  one-command context assembly.

## Acceptance Criteria

- [x] Record a gap evaluation in `docs/analysis/`.
- [x] Add or refresh goal docs so remaining work is ordered and executable.
- [x] Update the usability program and roadmap to point to the next goal.
- [x] Run docs/planning validation.
- [x] Archive this Trellis task after commit.

## Non-Goals

- No Rust behavior changes.
- No implementation of the new goals.
- No PR creation in this task.

## Validation Evidence

- 2026-06-29: `cargo fmt --check`
- 2026-06-29: `git diff --check`
- 2026-06-29: `cargo run --quiet -- check --format json .`
- 2026-06-29: `cargo xtask docs`
- 2026-06-29: `cargo xtask evidence`
- 2026-06-29:
  `python3 ./.trellis/scripts/workflow_gate.py --platform codex --task .trellis/tasks/06-29-06-29-project-intelligence-usability-gap-goals`
  confirmed only current task dirty paths remained.
