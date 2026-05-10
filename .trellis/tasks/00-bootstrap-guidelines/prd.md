# Bootstrap Task: Assura Project Guidelines

**You (the AI) are running this task. The developer does not read this file.**

This task exists because Trellis was initialized in Assura for the first time.
The default Trellis scaffold is intentionally generic; Assura needs a focused
workflow for a Rust CLI/library, project-structure validation, documentation
governance, and self-enforcement through `.assura/config.yml`.

## Objective

Make future AI sessions discover Assura's real conventions quickly:

- Trellis is the canonical workflow, task, and spec system.
- `.trellis/spec/assura/index.md` is the Assura-specific project spec.
- `.assura/config.yml` is the active self-enforcement config.
- `docs/analysis/` contains dated assessment and governance reports.
- Generated backend/frontend Trellis templates are not authoritative until
  they are filled with project-specific content and linked from a task.

## Status

- [x] Add Assura-specific project spec.
- [x] Adopt Trellis-first governance ADR.
- [x] Add documentation cleanup register.
- [x] Wire Assura baseline `check` behavior to real structure validation.
- [ ] Convert remaining durable conventions from `AGENTS.md` and docs into
      `.trellis/spec/assura/` files.
- [ ] Archive or rewrite stale OpenSpec, phase-review, and config docs after
      their useful content is migrated.
- [ ] Decide whether the generated backend/frontend spec templates should be
      removed, archived, or populated for the website and Rust implementation.

## Spec Work To Do

Create or update Assura-specific specs under `.trellis/spec/assura/` as the
project stabilizes:

| Spec | Purpose |
| --- | --- |
| `index.md` | Canonical product direction and source-of-truth map. |
| `cli.md` | Public CLI command behavior, exit codes, and output formats. |
| `config.md` | Supported structure-first config semantics and inheritance rules. |
| `self-enforcement.md` | How `.assura/config.yml` dogfoods the repo. |
| `docs-governance.md` | Canonical, historical, archive, and delete treatments. |
| `testing.md` | Required checks for CLI, config, hooks, docs, and fixtures. |

Only add these files when they contain real, current project guidance. Do not
create empty placeholders.

## Process Notes

- Start from real code and current command output, not old phase-complete
  summaries.
- Prefer fixing overclaims by either implementing the capability or downgrading
  the documentation.
- Keep active work in Trellis tasks. Do not create a second backlog in root docs.
- When adding new Assura constraints, add passing and failing fixtures where
  practical, then dogfood the rule in `.assura/config.yml`.

## Completion

Finish this bootstrap task when the active Assura specs describe the current
workflow well enough that a new agent can start from `AGENTS.md`, follow
Trellis, run `assura check .`, and understand which docs are canonical without
asking the user.
