# Expose configured structural advisories in Review

## Goal

Make Review expose configured nonblocking structural violations with named stable findings and complete history, while retaining advisory Review exits, existing Check severity exits, and compact omission of unchanged nonblocking action rows.

## Requirements

- Reuse the existing Doctor/Check result; do not add validation or rerun Doctor in parallel.
- Surface configured actionable findings ahead of optional baseline/setup suggestions.
- Preserve stable finding fingerprints and `new`, `worsened`, `unchanged`, and `resolved` history, including findings omitted from compact text/agent output.
- Preserve honest overflow totals and existing bounded rendering.
- Do not change Review exit 0 or Check severity/exit behavior.

## Acceptance

- [ ] A configured `max_lines` fixture crossing the limit yields the named configured advisory in Review, with stable identity across runs and correct new/worsened/resolved history.
- [ ] Review exits 0 and Check exit behavior remains unchanged.
- [ ] Unchanged nonblocking findings remain hidden from compact action output while full JSON retains their history.
- [ ] Bounded output totals and ordering are correct, including overflow cases.
- [ ] Focused review tests pass; required final validation/review gates are recorded with any held tier.

## Definition of Done

- Focused integration proof, scoped compact Review spec update, coherent commit, and independent review with accepted corrections and rereview.

## Technical Approach

Trace Doctor's single check report into Review's report/history. Carry the existing nonblocking configured violations through its existing findings model, using its stable fingerprint and state machinery. Avoid a second validation pass or parallel policy representation.

## Out of Scope

- Public API/schema migration, dependencies, new modules, broad refactor, Doctor/Check severity changes, delivery mutations, and VPS validation before its lane is cleared.

## Technical Notes

- Source baseline: `49d85358bae85de04f1ce7faa746b377687dcaf3` on `fix/nu01-review-advisories`, base `origin/master`.
- Contract: `.trellis/spec/assura/compact-project-review.md`.
- Historical trial notes are in the primary checkout at `.trellis/tasks/10-17-new-user-review-backlog/research/` and are evidence leads, not current behavior.
