# Assura Roadmap Goal Sequence

## Goal

Create a new PR that adds a long-form, full-featured Assura roadmap expressed
as durable `docs/goals/` artifacts.

## Requirements

- Add a master goal that sequences the roadmap goals and references every goal
  file.
- Add individual goal files for major measurable chunks of work.
- Each goal should be scoped as about two weeks of work for a team of
  engineers, not a single-agent microtask.
- Each goal needs objective, scope, non-goals, measurable definition of done,
  validation commands, review tasks, and reviewer blocking criteria.
- The master goal must show the goal order, dependencies, readiness gates,
  review tasks required for each goal, and how completion evidence should be
  linked.
- Keep the roadmap consistent with current Assura public surfaces:
  `assura check`, structure-first validation, stable
  `assura check --format agent`, and Codex delivery only through
  `--agent codex`.
- Do not introduce package feedback CLIs, per-agent CLI entrypoints, or
  per-agent `--format` values.
- Keep files under `docs/goals/` root, kebab-case, with frontmatter, matching
  `.assura/config.yml`.

## Acceptance Criteria

- [ ] Master goal exists and references all roadmap goal files.
- [ ] Individual goals are specific, measurable, sequenced, and reviewer-ready.
- [ ] Review tasks are explicit enough for future agents to verify completion.
- [ ] `.trellis/spec/assura/roadmap.md` points to the master roadmap goal as
  the next durable planning source.
- [ ] `cargo run --quiet -- check --format json .` passes.
- [ ] `git diff --check` passes.
- [ ] A review agent reviews the docs before PR creation.
- [ ] Branch is committed, pushed, and opened as a PR.
