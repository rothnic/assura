# LS-Lint Gate And Beta Completion Audit

## Objective

Close the final beta roadmap epic only if the current repo evidence proves the
LS-Lint no-slower policy and the broader beta program can pass its completion
assessment.

## Scope

- Re-run the machine-readable LS-Lint no-slower gate over the checked
  performance report.
- Inspect `benches/history/current.json` and website performance data for
  cold-versus-warm claim separation.
- Update `docs/goals/assura-ls-lint-no-slower-performance-gate.md` if live
  evidence still proves its definition of done.
- Audit `docs/goals/assura-beta-code-agnostic-capabilities-program.md` against
  all ten child epics, support classifications, and release evidence.
- Use an independent reviewer before marking the beta program complete or
  recording blockers.

## Non-Goals

- Do not optimize LS-Lint performance unless the current gate fails.
- Do not claim the stricter cold 2x LS-Lint target unless every headline
  fixture meets it.
- Do not publish or fabricate release artifacts as part of a docs-only audit.
- Do not treat warm daemon/session evidence as a substitute for the cold CLI
  no-slower gate.

## Acceptance Criteria

- `cargo xtask performance-no-slower` passes on the checked report.
- Current checked data has no headline realistic-equivalent fixture where the
  accepted cold Assura CLI row is slower than native LS-Lint.
- The LS-Lint child goal status matches live evidence.
- The master beta program records a final audit row with reviewer evidence and
  any release blockers.
- If a beta release tag/artifact is still missing, the program is not marked
  complete and the exact blocker is recorded.

## Validation

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo xtask performance-no-slower
jq '.claim_summary,.warm_claim_summary,.ls_lint_status' benches/history/current.json
cargo xtask release-readiness --format json
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
git diff --check
```
