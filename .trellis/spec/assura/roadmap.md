# Assura Roadmap

This is the high-level roadmap agents should use to orient work. Keep each epic
name short enough to scan quickly, then track concrete work in Trellis tasks.

## Epic Roadmap

| Order | Epic | Status | Active/Open Work |
| --- | --- | --- | --- |
| 1 | Trellis Workflow Foundation | Active | `00-bootstrap-guidelines` |
| 2 | Tooling Baseline Cleanup | Next | `05-10-01-rustfmt-baseline-cleanup`, clippy cleanup, CI signal cleanup |
| 3 | Assura Self-Check Clean | Next | reduce `assura check .` baseline violations to zero |
| 4 | Documentation Source Truth | Next | migrate, archive, or delete stale docs and workflow artifacts |
| 5 | Windows CI Restore | Later | fix `libgit2-sys` MSVC linker failure and restore matrix entry |
| 6 | Beyond Ls-Lint Rules | Later | frontmatter schemas, cross references, traceability, graph checks |

## Active Epic

**Trellis Workflow Foundation** is active.

Current owning task: `.trellis/tasks/00-bootstrap-guidelines`.

Current objective: make project workflow, roadmap, status, tooling debt, and
source-of-truth conventions obvious to agents and the developer before starting
new product feature work.

## Recommended Next Epic

After the current self-enforcement PR is reviewed, move to
**Tooling Baseline Cleanup**.

The first recommended work item is a dedicated rustfmt cleanup PR because it is
mechanical, easy to review separately, and turns one failing quality gate into a
clean blocking signal.

## Roadmap Rules

- Use this roadmap in every non-trivial workflow status snapshot.
- Keep epic names at 3-5 words where practical.
- Put detailed implementation work in Trellis tasks, not in this roadmap.
- If a new epic is needed, add it here and identify the first Trellis task that
  owns it.
- If an epic is active, say which task owns it and what the next recommended
  action is.
