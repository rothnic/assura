# Create Iteration 02 goal docs

## Goal

Create the missing major goal documents for Assura Roadmap Iteration 02 without
reopening already-completed canonical relationship notation work.

## Requirements

- Inspect existing `docs/goals/` files before adding new ones.
- Add five roadmap-level goal files for the next Iteration 02 work.
- Make each goal state why it is not already achieved.
- Align each goal with the product vision: new users should get deterministic
  usability and support certainty rather than hoped-for stability.
- Require notation updates to include performance gates or bounded-cost
  justification.
- Require notation updates to migrate public examples, generated examples,
  fixture configs, and test-case Assura configs.
- Require removed/superseded notation to be deleted instead of retained through
  backwards-compatibility shims unless a support-policy exception is explicit.
- Require Markdown outline work to evaluate maintained Markdown linting,
  frontmatter, parser/AST, and link-checking tools before custom generic checks.
- Add durable goal creation/revalidation guidance for old or separate-context
  goals.
- Require Iteration 02 notation work to end with a use-case matrix that starts
  from LS-Lint-equivalent cases, extends into Assura-native notation, receives
  independent review, and meets performance goals.
- Keep completed Iteration 01 goals and completed notation work marked as done.
- Reference existing planned rule-goal docs instead of duplicating them.
- Update the Iteration 02 roadmap document so future agents can find the new
  goal files in order.
- Keep docs/goals files kebab-case, with frontmatter, matching Assura config.

## Acceptance Criteria

- [x] Five new major Iteration 02 goal docs exist under `docs/goals/`.
- [x] `docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md`
      links those goals in order.
- [x] No new goal claims canonical relationship notation itself is unfinished.
- [x] Each goal includes a current gap and user-certainty bar.
- [x] Notation-changing goals include performance, example/config migration,
      and no-backwards-compatibility gates.
- [x] Markdown outline work includes a tooling-evaluation gate before custom
      generic Markdown checks.
- [x] A reusable goal-validation skill exists and is linked from agent
      guidance.
- [x] Notation goals include LS-Lint baseline coverage, Assura-native extension
      coverage, independent review, and performance gates.
- [x] Docs validation passes for the changed surface.
- [x] Assura self-check passes.

## Out Of Scope

- Implementing any Iteration 02 runtime behavior.
- Activating Iteration 02 as in-progress.
- Creating PRs or branches.
