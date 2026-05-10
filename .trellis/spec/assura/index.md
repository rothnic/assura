# Assura Project Spec

Assura is a pre-1.0 Rust CLI and library for dependency-aware filesystem
validation. The current product priority is a reliable structure-first CLI that
can dogfood this repository through `.assura/config.yml` and git hooks before
expanding into richer dependency and documentation constraints.

## Canonical Sources

- Workflow and task execution: `.trellis/workflow.md` and `.trellis/tasks/`
- Spec routing: `.trellis/spec/index.md`
- Project structure enforcement: `.assura/config.yml`
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
4. Add richer constraints only after the ls-lint-like baseline is stable.

## Constraints

- Prefer the structure-first config in `.assura/config.yml` for current CLI
  behavior.
- Keep hooks advisory on ordinary branches until the repo passes its own
  structure checks consistently.
- Do not add a second active spec/task system next to Trellis.
- Any new validation constraint should include a failing fixture, a passing
  fixture, and CLI integration coverage where practical.
