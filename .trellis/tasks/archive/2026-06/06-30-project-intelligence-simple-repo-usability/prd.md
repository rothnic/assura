# Project Intelligence Simple Repo Usability

## Goal

Make the Project Intelligence surface easier to understand from the docs and
CLI by showing what works today, correcting the unscored keyword-search gap,
and creating the next goal set for simple repo-wide code and content
intelligence.

## What I Already Know

- The published Project Intelligence demo is too meta for a new user.
- `assura content search` currently performs deterministic keyword matching
  over modeled content facts, Markdown sections, and diagnostics, but does not
  expose a relevance or match score.
- `assura content semantic-search --enable-local` exposes scored local
  candidates, but its command shape is heavier and the docs do not make the
  distinction clear.
- `assura content expand <collection> <id>` follows modeled graph edges from
  one content instance and returns related model instances, symbol refs,
  incoming relation sources, and diagnostics.
- Existing code-symbol commands can connect modeled content to referenced code
  symbols, but the current search path does not search the whole repository or
  unify code and content results.
- The current content runtime can validate Markdown frontmatter and structured
  collections against a runtime schema, report missing fields or invalid
  object shapes, validate modeled references, and preview safe Markdown lint
  repairs.

## Assumptions

- The immediate branch should not attempt a full repo-wide ranking engine.
- It is acceptable to add a deterministic keyword score to the existing
  keyword-search output as long as the docs do not call it semantic truth.
- The larger product gaps should become explicit goal docs with proof gates.

## Requirements

- Add deterministic keyword scores to `assura content search` and the agent
  search alias.
- Update the Project Intelligence demo so the anchor sections are useful when
  linked directly, especially `#expand-related-context`.
- Explain the difference between keyword scores and semantic candidate scores.
- Show concrete examples for frontmatter modeling, schema validation,
  markdown linting, missing references, and graph expansion.
- Create a new successor goal set for:
  - one simple repo intelligence command with low-ceremony defaults;
  - repo-wide code plus content indexing and code-to-content/content-to-code
    traversal;
  - frontmatter/content-model validation and markdown lint demos that prove
    incomplete or invalid content is caught.

## Acceptance Criteria

- [x] `assura content search ... --format json` returns a score for each
      match and tests prove ordering/score presence.
- [x] Human text output includes the score without requiring JSON.
- [x] The demo page explains what `expand` does and shows representative
      output.
- [x] The demo page distinguishes currently supported surfaces from the
      follow-up product goals.
- [x] New goal docs define objective, scope, non-goals, validation commands,
      review tasks, and blocker criteria.
- [x] Roadmap points to the new usability follow-up instead of claiming no
      successor remains.

## Out Of Scope

- No remote embedding provider.
- No MCP or hosted service.
- No full repository code parser beyond existing code-symbol facts.
- No one-shot natural-language query planner in this branch.
- No automatic semantic repair for invalid content.

## Technical Notes

- CLI command definitions: `src/cli/args.rs`.
- Keyword search execution: `src/cli/content_query/mod.rs`.
- Search output types and text rendering:
  `src/cli/content_query/output.rs` and
  `src/cli/content_query/output_text.rs`.
- Existing coverage: `tests/content_query_cli.rs`.
- Demo page: `website/src/content/docs/examples/project-intelligence-demo.md`.
- Program/roadmap files:
  `docs/goals/assura-project-intelligence-usability-program.md` and
  `.trellis/spec/assura/roadmap.md`.

## Definition Of Done

- Focused tests pass.
- `cargo fmt --check` passes.
- `cargo run --quiet -- check --format json .` passes.
- `cargo xtask docs` passes because the website docs change.
- `git diff --check` passes.

## Completion Note

Completed in PR #109 and merged to `master` on 2026-06-30. The live docs were
verified on `assura.dev` after merge. The remaining markdown lint and internal
reference work is intentionally tracked by
[assura-markdown-lint-link-reference-engine.md](../../../../../docs/goals/assura-markdown-lint-link-reference-engine.md).
