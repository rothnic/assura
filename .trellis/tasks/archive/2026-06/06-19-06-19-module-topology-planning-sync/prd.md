# Module Topology Planning Sync

## Objective

Synchronize Assura planning state after the module topology rule merged in PR
#72 and its Trellis task was archived in PR #73.

## Requirements

- Mark `docs/goals/assura-rule-module-topology.md` as completed with merged PR
  evidence and a short completion result.
- Update `.trellis/spec/assura/roadmap.md` so module topology is no longer the
  recommended next epic.
- Update
  `docs/analysis/2026-06-09-assura-best-practice-target-state.md` so completed
  first-slice detectors are not described as unimplemented.
- Route the next candidate to docs lifecycle and stale-claim detection unless
  live validation finds a higher-priority open detector.

## Validation

- `python3 ./.trellis/scripts/workflow_gate.py --platform codex`
- `cargo run --quiet -- check --format json .`
- `cargo xtask evidence`
- `cargo xtask docs`
- `cargo xtask target-state`
- `git diff --check`
