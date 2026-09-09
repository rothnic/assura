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

## Iteration 47 — 2026-09-08 — A05 honest unconfigured-plan diagnosis

- PR #202 merged as `186426d` after independent review, a concrete finding,
  the accepted test-only correction, and scoped rereview. Its new integration
  contract requires a valid project without `quality.scopes` to exit 2 with the
  configuration diagnosis and no successful plan; it cannot quietly imply
  runnable native coverage.
- The exact test, all 12 A05 integration tests, and `cargo xtask pr` passed.
  The full hosted matrix passed, including Performance Report, five adoption
  lanes, Windows Installer Smoke, release smoke, coverage, all stable OS
  suites, documentation, and website verification. Security Scope passed; the
  conditional Security Audit was scope-skipped and is not called passing test
  evidence. The continuing A05 branch is clean and rebased to the merge.

## Iteration 41 — 2026-09-06 — A04 hosted integration and closure

- PR [#181](https://github.com/rothnic/assura/pull/181) merged as
  `41949fd589f37d01222ef6a695a6a4c3f61ec9a7`. Its independently reviewed and
  locally proven head `ec2cf6a57a001a1c266dcb7b99e147a02cc85a94` passed all
  required hosted checks: documentation, Linux/macOS/Windows stable, MSRV,
  release bundle, Windows installer, five adoption lanes, coverage, evidence
  and security checks, and Performance Report.
- `git fetch origin --prune` followed by `git merge-base --is-ancestor ec2cf6a
  origin/master` exited 0. A04 is now done, with local hooks still described as
  local lifecycle evidence rather than hosted merge protection. Next independent
  ready card: A05; R03 remains blocked pending a comparable performance repair.
