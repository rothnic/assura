# Define next end-to-end work goal

## Goal

Create a durable `docs/goals/` planning document that describes the next
1-2 week Assura work chunk end to end, with a clear objective, scope,
definition of done, and proof gates.

## What I Already Know

- Recent `master` history completed the v0.1 onboarding release, agent nudge
  MVP, and LS-Lint performance evidence work.
- The prior planning request asked for useful 1-2 week options based on recent
  git activity.
- The current branch does not contain the earlier `docs/guides/near-term-*`
  memo referenced in the prior conversation summary.
- Existing goal docs already cover CLI-to-CLI performance verification,
  native LS-Lint performance rearchitecture, and pinned fixture benchmark
  suite work.
- The next useful chunk should prove product usefulness end to end, not repeat
  a completed performance claim.

## Assumptions

- The document should live under `docs/goals/` with the existing goal-doc
  style.
- The target chunk should be sized for roughly 1-2 weeks of engineering work.
- The goal should be independently reviewable through commands, docs, and
  user-perspective evidence.

## Requirements

- Define the end-to-end objective in terms of user/project outcome.
- Include starting repo truth and explicit non-goals.
- Define a concrete definition of done that covers implementation, tests,
  docs, website/user workflow, and review evidence.
- Include validation commands and reviewer blocking criteria.
- Keep the document actionable enough that a future engineer can pick it up
  without needing this conversation.

## Acceptance Criteria

- [x] A new `docs/goals/` document exists for the next chunk of work.
- [x] The document has frontmatter metadata, objective, scope, non-goals,
      definition of done, validation commands, and review criteria.
- [x] The document references current source-of-truth docs and avoids stale
      claims about unsupported behavior.
- [x] Repo formatting/structure checks relevant to a docs-only change pass, or
      exact blockers are recorded.

## Out of Scope

- Implementing the goal itself.
- Changing current CLI behavior.
- Rewriting existing completed performance goal docs.

## Technical Notes

- Read `.trellis/spec/assura/index.md` and `.trellis/spec/assura/roadmap.md`.
- Read existing goal docs in `docs/goals/`.
- `docs/goals/README.md` does not currently exist on `master`.
