# Native Performance Roadmap And Goal Text

## Goal

Update Assura's roadmap with the next performance-focused iteration and add a
repo-native goal document that can be used to kick off the work to polish
Assura performance. After the user's new-project dogfood feedback, also add
the agent-ready project onboarding backlog that should sit beside the
performance backlog in priority order.

## What I Already Know

- The completed post-beta program delivered a stricter LS-Lint no-slower gate
  for accepted LS-Lint-equivalent rows.
- The current gap is broader than LS-Lint parity: native Assura capabilities
  such as content collection validation, document graph queries, context packs,
  daemon/session queries, and Markdown/reference workflows need tracked
  performance evidence.
- Existing local benches already cover parts of the gap:
  `benches/content_runtime.rs` and `benches/project_intelligence.rs`.
- CI currently enforces the LS-Lint comparison report, not Assura-native
  performance rows.
- The goal text should make the large future work executable without losing
  the user's intended outcome.
- A new-project dogfood run showed that Assura can pass the configured policy
  while an agent still lacks scaffolded guidance, active content models,
  search/reference discovery, skill contracts, and a clear checked-versus-
  unchecked report.

## Requirements

- Add a roadmap iteration that routes the next large performance polish effort.
- Add a goal file under `docs/goals/` that defines objective, scope, non-goals,
  implementation slices, performance test plan, validation commands, and review
  blocking criteria.
- Keep LS-Lint no-slower parity separate from Assura-native capability
  performance.
- Include explicit coverage for content collection validation/querying,
  document graph, context packs, daemon/session warm behavior, Markdown and
  reference workflows, and CLI-floor attribution.
- Add a second parent backlog goal for making Assura the default scaffold,
  doctor, and feedback loop for agent-ready repositories.
- Make backlog priority explicit: agent-ready onboarding is the broad adoption
  priority; performance polish remains the performance-specific priority.
- Provide copy/paste-ready goal text/path in the final response.

## Acceptance Criteria

- [x] `.trellis/spec/assura/roadmap.md` names the new iteration and points to
      the performance-polish goal.
- [x] `docs/data/public-roadmap.json` exposes the new next roadmap item.
- [x] A new `docs/goals/` performance goal can be referenced directly by a
      future `/goal` kickoff.
- [x] A new `docs/goals/` agent-ready onboarding goal can be referenced
      directly by a future `/goal` kickoff.
- [x] Existing completed post-beta goals are not incorrectly reopened.
- [x] Repo self-check succeeds for the docs/planning change.

## Out Of Scope

- Implementing the native performance report in this task.
- Re-running expensive performance benchmarks.
- Opening or merging a PR from this task unless separately requested.

## Technical Notes

- Roadmap source: `.trellis/spec/assura/roadmap.md`.
- Public roadmap artifact: `docs/data/public-roadmap.json`.
- Existing LS-Lint performance goals:
  `docs/goals/assura-ls-lint-no-slower-performance-gate.md`,
  `docs/goals/assura-performance-floor-and-fixture-gate.md`, and
  `docs/goals/assura-ls-lint-performance-reassessment.md`.
- Existing native performance evidence:
  `docs/analysis/2026-06-28-content-runtime-index-performance.md` and
  `docs/analysis/2026-06-28-project-intelligence-store-spike.md`.
