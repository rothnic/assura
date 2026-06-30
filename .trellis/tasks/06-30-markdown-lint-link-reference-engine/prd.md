# Markdown Lint Repository Reference Engine

## Goal

Turn the Markdown linter request into an executable Assura goal with clear
proof gates, then use that goal as the owning task for the next implementation
slice.

## What I Already Know

- The Project Intelligence Simple Usability PR was merged and deployed, but
  the Trellis task remained active until this branch archived it.
- Assura has partial Markdown support: frontmatter presence, content-model
  frontmatter validation, heading/outline validation, and a narrow
  trailing-space lint/fix rule.
- The user now wants a broader Markdown linter and repository-reference graph
  that also validates required headings, can add missing headings when desired,
  catches broken internal code/file references, enforces GitHub-renderable
  relative links, supports configurable warning levels, and provides an escape
  hatch.
- Source comments, docstrings, and string-like references to docs can go stale
  too. Whole-repo checks should catch that, and warm session/watch/future
  daemon paths should use discovered source-to-target edges for quick,
  context-efficient affected-reference feedback.
- The existing Rust Markdown tooling evaluation says broad linter integration
  needs evidence and must respect Assura's local/offline/MSRV constraints.

## Assumptions

- This task starts as planning and goal definition. Implementation should
  follow the new goal unless explicitly pulled into this same branch.
- The default product path should be CLI-first and local, with no MCP, hosted
  service, JavaScript runtime, or network dependency.
- Internal references in Markdown should be Markdown links, not inline-code
  path strings, whenever the intent is navigation.
- Code/comment references to repository docs should be represented as reference
  facts even when they cannot render as Markdown links from the source file.

## Requirements

- Archive the completed Project Intelligence Simple Usability task.
- Add a new goal for Markdown lint and repository reference validation.
- Update the roadmap so the active iteration points at the new markdown goal.
- Make the goal explicit about:
  - broad lint coverage beyond trailing spaces;
  - configured required headings and safe missing-heading insertion;
  - GitHub-renderable relative links to files, headings, lines, and line
    ranges;
  - broken target detection;
  - malformed bare or code-spanned references;
  - stale code/comment/docstring references to docs and other repository files;
  - inbound and outbound reference edges for changed-source and changed-target
    invalidation in warm session, watch, or future daemon workflows;
  - configurable per-rule severity;
  - reasoned suppressions.

## Acceptance Criteria

- [x] Previous Project Intelligence Simple Usability task is archived.
- [x] New Trellis task owns the markdown/repository-reference follow-up.
- [x] Goal doc includes objective, current gap, scope, non-goals, definition
      of done, validation commands, review tasks, and blocker criteria.
- [x] Roadmap identifies the previous iteration as completed and the markdown
      lint/repository-reference iteration as active.
- [x] Docs-only validation passes.

## Out Of Scope

- No full Markdown linter implementation in this planning slice unless the user
  explicitly asks to continue directly into implementation.
- No Cloudflare deployment changes.
- No MCP or hosted integration design.

## Technical Notes

- Previous completed goal:
  [assura-rust-markdown-validation-and-fixing.md](../../../docs/goals/assura-rust-markdown-validation-and-fixing.md)
- Markdown tooling analysis:
  [2026-06-18-markdown-tooling-evaluation.md](../../../docs/analysis/2026-06-18-markdown-tooling-evaluation.md)
- Config notation:
  [config-notation.md](../../spec/assura/config-notation.md)
- Roadmap:
  [roadmap.md](../../spec/assura/roadmap.md)

## Definition Of Done

- `cargo run --quiet -- check --format json .` passes.
- `cargo xtask docs` passes.
- `cargo xtask evidence` passes.
- `git diff --check` passes.
