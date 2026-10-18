# NU13: targeted captured-relationship coverage

## Goal

Make ordinary scoped Check trustworthy for a selected captured producer. A
missing required counterpart currently fails a full check but silently passes
when the producer itself is the selected file. Repair that shared traversal
omission before extending incremental coverage or claiming a speed improvement.

Authority: Nick's 2026-10-17 instruction, "Orchestrate the first bit of work
that would be most beneficial to implement." Selection is NU13 from the
independently reviewed new-user backlog at `ebeba97d`; runtime source is
unchanged from current integration `f3540d4a11d36afa43f69c96fd7f1c233599822a`.

## Ownership and prerequisites

- Sole implementation owner: live worker `/root/nu13_implement`.
- Coordinator: `/root`, owning task transitions, validation, review and handoff.
- Separate clean current-base worktree; branch `fix/nu13-targeted-relationships`.
- Canonical proposal: `10-17-new-user-review-backlog/research/backlog.json`, NU13,
  on branch `docs/assura-new-user-review-20261017`; no Beads import or new queue.
- P05 remains a separate coordinator-owned recovery action. Its generation5
  receipt is integrated_pending_verification and its registered checkout is
  missing. Current-base verification and legitimate checkout disposition are
  required before closure; no owner, historical evidence or outcome is changed
  to start this independently owned lane. Execution contract permits separate
  ownership and capacity. Read-only topology retains 15 unreadable and46
  prunable registrations; no unknown cleanup is authorized.

## Contract and decision

Ordinary scoped checks evaluate generated relationships for nonexcluded
producer entries comprising the selected path itself and its traversed
descendants. Providers may reside outside the selected subtree. Selecting a
provider alone does not validate all incoming producer relationships or prove
whole-project success.

Keep the project-root container out of producer enumeration. Preserve explicit
LS-Lint target semantics, exclusions, provider alternatives/content predicates,
ordering, error mapping, symlinks, cache invalidation, and prepared changed-path
full fallback. Presence of a test filename proves presence, not execution or
adequacy. No public type or configuration schema change is required.

Choose the shared root-entry correction rather than a file-only workaround
(which misses directory producers) or unconditional full-project checking
(which silently expands ordinary scope and cost). No blocking product choice
remains after source/contract review.

## Acceptance

- [ ] Selected captured file producer with a missing counterpart produces an
  actionable relationship finding and failure; adding the counterpart passes.
- [ ] Selected captured directory producer is evaluated; an allowed provider
  outside its subtree satisfies the relationship.
- [ ] An unrelated broken producer outside the selected scope is absent from
  targeted findings; provider-only selection creates no reverse finding.
- [ ] Excluded producers remain excluded, including explicit selection; root
  findings, deduplication and deterministic ordering remain correct.
- [ ] Explicit LS-Lint target mode remains unchanged. Reuse or extend meaningful
  full/file/directory, missing/provided, producer-deletion/config-change/cache,
  compiled and prepared-fallback proof with exact findings and exits.
- [ ] Record a real failing baseline and green focused tests; run applicable
  Rust, docs, structure and evidence gates and independent exact-source review.

## Affected surface and validation

Worker owns `src/cli/check/custom_constraints.rs`, the existing
`tests/structure_config_notation_tests.rs` fixtures and strictly necessary
adjacent existing checker tests, the scoped contract in
`.trellis/spec/assura/config-notation.md`, and concise disclosure in
`website/src/content/docs/reference/configuration.md` and
`docs/release-notes.md`. Coordinator owns this task and central backlog tracking.

Follow Rust-quality error/effect, traversal and test guidance. Start with the
focused notation/relationship tests. Run `cargo xtask pr` once settled (it
includes fast), docs prerequisites and evidence/structure checks as applicable;
retain current required hosted performance/OS obligations. Never lower a gate
or infer a pass from a historical failure. Resource-admit every build batch;
use disk scratch and one compiler lane, offloading sustained work when useful.

## Out of scope

Architectural performance optimization or speedup claims, reverse-dependency
completeness, new daemon/index/API/schema/dependency, broader repair, global
binary changes, P05 closure/cleanup, Beads mutations, publication or release.
