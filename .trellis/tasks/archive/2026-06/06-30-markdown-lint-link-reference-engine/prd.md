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
- The user wants a parent goal that references the major sub-goals for daemon
  readiness, daemon management CLI, VS Code integration, agent awareness, and
  later Zed/JetBrains integrations.
- The latest public GitHub release is `v0.1.0` from 2026-05-24. The user wants
  meaningful progress to produce incremental pre-1.0 versions and GitHub
  release artifacts instead of accumulating only on `master`.
- The user wants a concise public website roadmap driven by a repo artifact,
  with roadmap item labels limited to two to four words and optional links to
  deeper details.
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
- Add a parent program goal and child-goal set for daemon/editor/agent work.
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
- Make the parent goal explicit that:
  - CLI daemon commands are the shared control plane;
  - VS Code is the first editor integration;
  - Zed and JetBrains are future integrations;
  - agents should detect daemon health, receive bounded context, and know
    start/restart/doctor/fallback commands.
- Add an incremental release-train child goal covering version bumps, release
  notes, tags, GitHub release assets, and live release verification.
- Add a public-roadmap child goal covering a repo-backed website roadmap,
  two-to-four-word labels, detail links, and drift validation.

## Acceptance Criteria

- [x] Previous Project Intelligence Simple Usability task is archived.
- [x] New Trellis task owns the markdown/repository-reference follow-up.
- [x] Parent program goal lists the major daemon, editor, CLI, and agent
      sub-goals.
- [x] Parent program includes incremental release/versioning as a major
      sub-goal.
- [x] Parent program includes a public website roadmap artifact as a major
      sub-goal.
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
- Parent program:
  [assura-markdown-reference-intelligence-program.md](../../../docs/goals/assura-markdown-reference-intelligence-program.md)
- Release train:
  [assura-incremental-release-train.md](../../../docs/goals/assura-incremental-release-train.md)
- Public roadmap:
  [assura-public-roadmap-artifact.md](../../../docs/goals/assura-public-roadmap-artifact.md)
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
