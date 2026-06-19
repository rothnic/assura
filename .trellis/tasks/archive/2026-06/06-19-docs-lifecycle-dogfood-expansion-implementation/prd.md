# Docs Lifecycle Dogfood Expansion Implementation

## Goal

Implement `docs/goals/assura-rule-docs-lifecycle-dogfood-expansion.md` by
expanding Assura's own `extensions.docs_lifecycles` dogfood policy from the
first narrow proof slice to a bounded set of current release, support,
validation, roadmap, and performance-facing docs.

## What I Already Know

- PR #78 added the reusable docs lifecycle rule and stale-claim checks.
- PR #81 created the follow-up goal and deferred `2x` because current evidence
  uses `two_x_*` JSON keys rather than an exact `2x` token.
- Current `.assura/config.yml` only covers `.trellis/spec/assura/roadmap.md`
  and `docs/goals/assura-rule-docs-lifecycle-stale-claims.md`.
- Candidate active docs already have `status: active` frontmatter except the
  roadmap, which can be given frontmatter in this implementation if validation
  stays green.
- Website docs stay out of this first slice unless a concrete stable evidence
  path is proven during implementation.

## Requirements

- Expand `.assura/config.yml` `active_goal_docs` with explicit active docs,
  not broad `docs/**` globs.
- Require accepted lifecycle status for the selected active docs.
- Add only executable claim patterns whose exact tokens appear in committed
  evidence files.
- Keep `2x` out of this slice unless exact-token evidence is introduced.
- Preserve `docs/archive/**` as historical context without broad cleanup.
- Update goal progress/roadmap/target-state after implementation state is known.

## Acceptance Criteria

- [x] `assura check --format json .` passes with the expanded policy.
- [x] The expanded policy names explicit active docs and evidence files.
- [x] Release archive and `performance-report` claim tokens have current
      committed evidence files.
- [x] Included docs have lifecycle statuses accepted by the policy.
- [x] Independent review confirms the policy is bounded and does not weaken
      existing target-state/evidence gates.

## Out Of Scope

- No natural-language stale-prose detection.
- No website docs coverage in this slice unless exact stable evidence is proven.
- No `2x` claim coverage until evidence semantics support it.
- No Rust rule mechanics unless validation proves the existing rule cannot
  support the bounded policy.

## Technical Notes

- Goal: `docs/goals/assura-rule-docs-lifecycle-dogfood-expansion.md`.
- Config notation: `.trellis/spec/assura/config-notation.md`.
- Current policy: `.assura/config.yml` `extensions.docs_lifecycles`.
- Evidence files for release archive tokens: `.github/workflows/release.yml`,
  `docs/compatibility-and-surface.md`, and `docs/release-notes.md`.
- Evidence files for `performance-report`: `src/cli/performance_report/mod.rs`,
  `docs/support-policy.md`, and `docs/release-notes.md`.
