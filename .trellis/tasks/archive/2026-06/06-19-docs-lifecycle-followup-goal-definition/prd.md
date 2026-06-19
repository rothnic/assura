# Docs Lifecycle Follow-Up Goal Definition

## Goal

Create and validate the next Assura goal for broadening
`extensions.docs_lifecycles` dogfood coverage beyond the first merged slice.
The follow-up goal should convert live target-state evidence into a concrete,
reviewable implementation slice before any broad docs cleanup starts.

## What I Already Know

- PR #78 merged the first reusable docs lifecycle/stale-claim rule slice.
- PR #79 archived that implementation task and recorded the session journal.
- PR #80 synced the roadmap and completed goal state.
- `.assura/config.yml` currently dogfoods `extensions.docs_lifecycles` only for
  `.trellis/spec/assura/roadmap.md` and
  `docs/goals/assura-rule-docs-lifecycle-stale-claims.md`.
- `docs/analysis/2026-06-09-assura-best-practice-target-state.md` still marks
  dense active docs/history as partially aligned and calls for concrete
  active/archive and stale-claim coverage.
- Current docs with deterministic release/performance claims include
  `docs/project-memories.md`, `docs/release-notes.md`,
  `docs/support-policy.md`, `docs/compatibility-and-surface.md`,
  `docs/release-candidate-checklist.md`, `docs/validation.md`, and website
  reference docs.

## Requirements

- Add a durable goal under `docs/goals/` that scopes the next docs lifecycle
  expansion.
- Keep the follow-up goal focused on dogfood coverage/configuration unless live
  evidence proves a missing rule capability.
- Name concrete active docs, claim tokens, expected evidence files, and
  historical exception policy candidates.
- Include objective, current gap, user certainty bar, scope, non-goals,
  definition of done, validation commands, review tasks, and blocking criteria.
- Update roadmap/target-state routing so the next agent can implement the
  follow-up goal without redoing discovery.
- Preserve the clean Trellis workflow: PR and archive this planning task before
  implementation starts.

## Acceptance Criteria

- [x] New goal file exists and is not a duplicate of the completed first-slice
      goal.
- [x] Goal is validated against live `.assura/config.yml`, roadmap, and
      target-state evidence.
- [x] Roadmap points to the new follow-up goal as the next executable candidate.
- [x] Target-state doc names the follow-up as the next docs lifecycle coverage
      step.
- [x] `cargo run --quiet -- check --format json .`, `cargo xtask evidence`,
      `cargo xtask docs`, and `git diff --check` pass.

## Out Of Scope

- No implementation changes to `src/**` or `.assura/config.yml` in this planning
  task.
- No broad documentation cleanup, archival, or claim rewriting.
- No new natural-language stale-prose classifier.

## Technical Notes

- Primary roadmap: `.trellis/spec/assura/roadmap.md`.
- Target-state evidence:
  `docs/analysis/2026-06-09-assura-best-practice-target-state.md`.
- Completed first-slice goal:
  `docs/goals/assura-rule-docs-lifecycle-stale-claims.md`.
- Current narrow dogfood policy: `.assura/config.yml`
  `extensions.docs_lifecycles.active_goal_docs`.
- Config notation says docs lifecycle claim patterns are literal tokens or
  glob-style token patterns, so this planning task avoids phrase patterns such
  as `assura performance-report`.
- Independent review found the candidate `2x` claim was not executable against
  current JSON evidence because the evidence uses `two_x_*` keys. The planning
  doc now defers `2x` until alias/phrase evidence semantics or exact-token
  evidence exist.
