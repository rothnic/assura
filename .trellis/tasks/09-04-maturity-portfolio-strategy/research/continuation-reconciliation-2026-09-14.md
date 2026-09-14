# Continuation reconciliation — 2026-09-14

This is a branch/PR hygiene record, not product or release completion. The
refresh source is `origin/master` at
`fff5c9c4730e1d3fef9cc294f42083a019dccce5` with tree
`c890900dffcf058c41bf73c17a44b166dd1489cf`.

## Disposition

| Candidate | Current finding | Owner / next action | Restore evidence |
| --- | --- | --- | --- |
| PR #194, `goal/r03-calibration-cohort`, `4c3c4981` | Superseded. R03 was reviewed and merged through PR #197 (`369507cd`) and the evidence reconciliation through PR #198 (`c3caa0bf`). | Archive/close the stale PR and live ref; no code merge. | Remote `archive/2026-09-12/r03-calibration-cohort` and verified bundle `r03-calibration-cohort.bundle` (SHA-256 `d74f0aa0fc654da178061d82a5c9c24e22171614bfb252fb5341bdcc2cfcc09d`). |
| PR #195, `goal/r03-calibration-collector`, `0d0f941f` | Superseded by the same R03 merge path; its collector-specific evidence is retained as historical input. | Archive/close the stale stacked PR and live ref; no code merge. | Remote `archive/2026-09-12/r03-calibration-collector` and verified bundle `r03-calibration-collector.bundle` (SHA-256 `3a02687cb7e123cc38b3322772e81aa394e2843247dbb8f72b5af51579649027`). |
| PR #142, `codex/release-install-paths`, `dc990c93` | Superseded. R05 installer hardening was current-master integrated by PR #153 (`2ee15e42`) and is `done` in the revision-pinned ledger. | Archive/close the stale PR and live ref; do not rebase the old installer patch. | Verified bundle `release-install-paths.bundle` (SHA-256 `d3eb2031308b83b20a5cdd5c4a20a8e1d8f09846d5b1c82617ab6b559503ddf`). |
| PR #187, `goal/w02-performance-claim-containment`, `cba6628f` | Held, not done. Its exact hosted path is not current and its successful Workers Build is coupled to the production-builds surface. Current-master CF04 evidence is separate and does not grant W02 acceptance or Cloudflare authority. | Owner `goal/w02-release-aware-installation`; keep W02 held. Smallest next action is Nick's explicit Cloudflare approval or a verified no-deploy separation, followed by a fresh current-master candidate and exact-head hosted gates. | Remote `archive/2026-09-12/w02-performance-claim-containment` and verified bundle `w02-performance-claim-containment.bundle` (SHA-256 `5c6b9f00fcdad9ae0ad9b9976f6f855c8487c4664990198a03fab0da718d75e3`). |
| `codex/assura-landing-experience`, `5c8cc8fa` | Stale mixed branch. Core landing/project-review behavior landed in PR #140 (`96dca6d6`); the remaining 115 commits mix later historical task, product, and generated-site work without a current PR. | Owner `nroth`; deliberately archive, do not merge wholesale. Reopen only as a named current slice with a fresh review path. | Verified bundle `landing-experience.bundle` (SHA-256 `76de0980318b13c4128f71fff563768a3526ec1616aad0c3651b4d723b5e7715`). |
| `codex/assura-0.3.1-release-audit`, `58813d51` | Stale local release-metadata branch; current source is already 0.4.0, while R06 remains pending and publication/tag authority is not granted. | Owner `nroth`; deliberately archive. R06 may restore only as a narrowed current candidate after readiness evidence and publication authority. | Verified bundle `release-audit.bundle` (SHA-256 `90d2ebcff4f87436426c2d543349c50a68441b2a82f4620a12d5ca681b72b7d2`). |

Bundle directory: `/Users/nroth/.codex/archives/assura/2026-09-14/`. Each bundle
requires the recorded current-master ancestor. The exact restore commands are:

```text
git fetch /Users/nroth/.codex/archives/assura/2026-09-14/r03-calibration-cohort.bundle refs/remotes/origin/goal/r03-calibration-cohort:refs/heads/goal/r03-calibration-cohort
git fetch /Users/nroth/.codex/archives/assura/2026-09-14/r03-calibration-collector.bundle refs/remotes/origin/goal/r03-calibration-collector:refs/heads/goal/r03-calibration-collector
git fetch /Users/nroth/.codex/archives/assura/2026-09-14/release-install-paths.bundle refs/heads/codex/release-install-paths:refs/heads/codex/release-install-paths
git fetch /Users/nroth/.codex/archives/assura/2026-09-14/w02-performance-claim-containment.bundle refs/remotes/origin/goal/w02-performance-claim-containment:refs/heads/goal/w02-performance-claim-containment
git fetch /Users/nroth/.codex/archives/assura/2026-09-14/landing-experience.bundle refs/heads/codex/assura-landing-experience:refs/heads/codex/assura-landing-experience
git fetch /Users/nroth/.codex/archives/assura/2026-09-14/release-audit.bundle refs/heads/codex/assura-0.3.1-release-audit:refs/heads/codex/assura-0.3.1-release-audit
```

The clean local branches `codex/markdown-linter-spec`,
`codex/project-intelligence-backend-spike`, `master`, and
`docs/p01-scope-current-master` have no current open PR or resolved ownership
handle in this audit. They remain untouched rather than being deleted.

The root checkout's untracked
`.trellis/tasks/09-04-maturity-portfolio-strategy/research/a04-host-status-doctor-permission-gap.md`,
the dirty CF03 benchmark worktree, the dirty GitHub-master evaluation worktree,
and two prunable registrations are foreign or historical topology. They remain
outside this change; the prunable registrations are removable only after the
exact missing paths are rechecked.

No item above is promoted from held, verified, prepared, or process-only to
product completion. R06, W02, W03, A07, and F01 retain their existing ledger
authority boundaries.

Independent review completed local source and bundle inspection but could not
query GitHub without an unavailable network approval. The coordinator ran the
bounded local substitute: all six bundles verified, each exact restore ref
fetched to a temporary namespace and matched its recorded tip, then was
compare-and-deleted; `git diff --check`, the current-source self-check,
`cargo xtask evidence`, and `cargo xtask docs` passed. No live GitHub result is
inferred from that local review.
