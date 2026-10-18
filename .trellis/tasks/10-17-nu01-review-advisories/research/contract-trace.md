# NU01 Review advisory contract trace

Source read at `49d85358bae85de04f1ce7faa746b377687dcaf3` in the owned checkout. This is source analysis, not a fresh runtime reproduction.

## Existing data path

- `src/cli/doctor.rs::build_project_doctor_with_structure_report` runs the recursive structure check once and builds `ProjectDoctorReport` from that report/config.
- `src/cli/doctor_report.rs::blocking_violations` currently filters `StructureCheckReport.violations` to `blocking == true` and `.take(5)`. The underlying report already carries all violations with path, rule, severity, blocking, and message.
- `src/cli/project_review/report.rs::ProjectReviewReport::from_parts` maps Doctor blocking violations and Doctor gaps, then inactive suggestions, content gaps, and structure-fit guidance. No configured nonblocking structure violations are promoted.
- `history.rs` fingerprints all constructed findings before persisted history comparison; only blocking findings currently use detail-derived path/message identity. History persists all non-resolved findings before rendering bounds.
- Review is assembled by the existing command and its exit behavior is advisory (success after successful assembly); Check's gate remains derived from `StructureCheckReport` blocking state.

## Required design boundary

Reuse the existing Doctor check result and Review fingerprint/history machinery. Add a separately named configured nonblocking violation collection with honest total/overflow information, then map it to advisory Review findings before optional setup/inactive suggestions. Ensure unchanged advisory findings remain in the full report/history and are still filtered by the existing bounded text/agent action output. Do not alter blocking counts, Check success, or Review's advisory exit.

## Existing field behavior to account for

The current Review schema is `assura.project-review.v2`; the Doctor schema is `assura.project-doctor.v1`. Before changing serialized fields, inspect their test assertions and consumers. Review JSON and agent schema already contain full finding history; agent output bounds findings at 12, while action buckets take four entries and text finding rows render three plus an overflow count. Stable identity must include the violating path and rule while avoiding volatile measured values in detail.

## Prior evidence lead

The 2026-10-17 TypeScript trial notes in the primary checkout report a 501-line source-file addition under a configured max-lines policy: Check's nonblocking violation count increased, while Review still showed advisory zero and no named finding. Those notes are historical and do not replace reproducing current source behavior with the pinned released binary and a disposable repository.

## Research status

Hindsight search for this Review/Doctor contract returned only an unrelated execution page and contributed no technical conclusion. No build, test, or runtime command was run for this checkpoint. Implementation and test work awaits the coordinator's resource-lane clearance.
