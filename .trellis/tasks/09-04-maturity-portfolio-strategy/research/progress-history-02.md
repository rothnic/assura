# Maturity execution train progress — preserved history

These entries were moved from `progress.md` when the live journal reached its
configured line limit. They remain historical evidence and do not supersede
the newer entries at the top of `progress.md`.

## Iteration 48 — 2026-09-08 — A05 cumulative overlapping-scope contract

- PR #204 merged as `dd2efaf` after independent review identified one concrete
  observability issue, the accepted test-only correction made scope order
  observable with a Rust-only command, and scoped rereview found no remaining
  defect. The fixture proves `frequent` through `merge` plans are cumulative,
  stable, and deduplicated across overlapping base/Rust scopes.
- The focused contract, 13-test A05 integration target, formatting and diff checks, and full `cargo xtask pr` gate passed. The full hosted matrix passed,
  including Performance Report, five adoption lanes, Windows Installer Smoke, release smoke, coverage, all stable OS suites, documentation, and website
  verification. Security Scope passed; the conditional Security Audit was
  scope-skipped and is not called passing test evidence.
- Context level: not exposed. The reviewer correction was specific to a test oracle, and the existing reviewer plus goal-execution loop captured it; no
  new reusable skill is warranted. Next: reconcile this evidence, then select the smallest remaining current-master A05 contract in the clean owner.

## Iteration 49 — 2026-09-08 — execution-continuity control

- Canonical executor and end-to-end prompts now make a checkpoint intermediate:
  retain a named live action or select the next independent ready card, recording
  owner, SHA/worktree, proof, and next observation.
- Local pass, PR, merge-ready state, empty queue, or held publication cannot end
  the train while another active, verified, integration, or cleanup action exists.
- This preserves review, current-master, quality, authority, and unknown-work
  safeguards. Next: validate, commit, independently review, then resume A07.

## Iteration 50 — 2026-09-09 — candidate-binding correction

- A07's first probe lost its candidate through a login shell and receives no allocation credit; the correction plan records the identity canary and next product-boundary check. Continue with review, commit, and a bound canary.
