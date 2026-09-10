# Maturity execution train progress — preserved history

The entries below were moved from `progress.md` on 2026-09-10 UTC solely to
keep the live progress index below its configured 1,000-line limit. Their
wording and evidence are unchanged; `progress.md` links this file through the
history chain.

## Iteration 39 — 2026-09-06 — A03 closure bookkeeping correction

- Post-merge verification of PR #179 (`929fbff`) found its evidence text correctly closed A03, but its backlog edit had matched Q02's repeated status fields instead of A03. Q02 also has no referenced evidence file, so its accidental `done` status was not supportable.
- Dedicated documentation-only PR #180 changes exactly those two records: A03 is `done` with its merged evidence, and Q02 is returned to unclaimed `pending`. JSON validation, the installed `assura check --format agent .`, and whitespace validation pass. No product behavior or CI scope changed.
- Context health: not exposed. This is the first exact-ID bookkeeping mismatch in the train; the immediate correction and card-ID-anchored patch are sufficient, so no reusable skill is justified. Next ready cards after PR #180 merges: A04 and A05, each dependent on A03.

## Iteration 40 — 2026-09-06 — A04 hook ownership and real-event proof

- A04 is active in isolated current-master worktree `goal/a04-verified-hook-lifecycle`. Focused RED/green tests establish exact generated-content ownership for Git hook install, force refresh, status, direct removal, and bulk removal; custom hooks, legacy-marker collisions, and custom sidecars are preserved.
- A disposable real Git repository at a path with spaces proved actual candidate
  hook events against a seeded violation: advisory `git push` exited 0 and opt-in
  `ASSURA_BLOCKING_PUSH=1 git push` exited 1. The feature-branch pre-commit
  warning was also observed without claiming merge protection. Ten public hook
  lifecycle tests, four private resolver tests, and thirteen host lifecycle
  tests passed; independent final review is clean.
- Context health: not exposed. Multiple reviewer findings all reduced to one reusable ownership rule and are now captured in the harness hook spec. The local `cargo xtask pr` runner did not yield a terminal result within the observation window, so it remains inconclusive; hosted performance/PR gates are mandatory before merge.

## Iteration 43 — 2026-09-06 — A03 target-state contract repair

- Post-merge verification exposed one verifier inconsistency rather than a
  product regression: the accepted A03 public guide used the evidence-first
  procedure, while `cargo xtask target-state` still required its three retired
  questionnaire markers. The isolated baseline failed only for those markers;
  no guide content was restored and no gate was disabled.
- Implementation `932306e` extracts the production guide contract for focused
  tests, requires the current procedure/action/path, and rejects the retired
  heading/action/path. Focused RED failed three tests for the stale behavior;
  GREEN and post-commit runs passed all three. The exact implementation passed
  `cargo xtask target-state`, `cargo xtask fast`, and the complete local
  `cargo xtask pr` gate, including Clippy, deterministic docs evidence, and the
  48-page website build.
- Independent review of `6d613f8..932306e` found no issues and marked spec and
  quality PASS. Context health: the omitted target-state update is now captured
  by positive, missing-current, retired-guidance, and contextual-exception
  controls; no new reusable skill is warranted. Next: commit this evidence,
  run the final exact-HEAD PR gate, then let the controller open the PR.
