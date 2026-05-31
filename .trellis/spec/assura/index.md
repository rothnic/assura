# Assura Project Spec

Assura is a pre-1.0 Rust CLI and library for dependency-aware filesystem
validation. The current product priority is a reliable structure-first CLI that
can dogfood this repository through `.assura/config.yml` and git hooks before
expanding into richer dependency and documentation constraints.

## Canonical Sources

- Workflow and task execution: `.trellis/workflow.md` and `.trellis/tasks/`
- Epic roadmap: `.trellis/spec/assura/roadmap.md`
- Workflow status snapshots: `.trellis/spec/assura/workflow-status.md`
- Spec routing: `.trellis/spec/index.md`
- Project structure enforcement: `.assura/config.yml`
- Closed-world structure config contract:
  `.trellis/spec/assura/structure-enforcement.md`
- Codex agent feedback contract:
  `.trellis/spec/assura/codex-agent-feedback.md`
- Tooling and CI stabilization: `.trellis/spec/assura/tooling-stabilization.md`
- CLI entrypoint: `src/main.rs`
- Public check implementation: `src/cli/check.rs` and `src/cli/commands.rs`
- Structure config model: `src/config/config.rs`
- Historical assessment reports: `docs/analysis/`

## Current Direction

1. Make `assura check` load the structure-first config, walk the repo, and
   return trustworthy exit codes.
2. Use Assura on this repo to surface stale docs, old workflow systems, and
   structure drift.
3. Keep Trellis as the canonical task/spec/workflow layer, with OpenSpec and
   `specs-bak/` treated as historical unless a later ADR changes that.
4. Keep the ls-lint-like baseline compatible while adding explicit
   closed-world structure contracts for this repo.

## Constraints

- Prefer the structure-first config in `.assura/config.yml` for current CLI
  behavior.
- Keep hooks advisory on ordinary branches until the repo passes its own
  structure checks consistently.
- Do not add a second active spec/task system next to Trellis.
- Treat PR feedback about new duplication, compatibility shims, unused
  scaffolding, or confusing parallel paths as in-scope cleanup for the current
  branch. Assura is pre-production and pre-1.0, so prefer a direct replacement
  or removal over preserving compatibility layers that would become debt.
- Treat known failing checks as explicit stabilization work. Do not leave them
  as tribal knowledge in PR comments only.
- Every non-trivial task response should include or preserve a concise workflow
  status snapshot: current task, phase/state, branch/PR when relevant, known
  blockers, next options, and the recommended next action.
- Workflow status snapshots should identify the active epic from the Assura
  roadmap and the open Trellis task that currently owns the work.
- Any new validation constraint should include a failing fixture, a passing
  fixture, and CLI integration coverage where practical.
