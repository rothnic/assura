---
id: analysis-2026-05-10-structure-enforcement-review
type: analysis
title: Structure enforcement review
status: active
created: 2026-05-10
updated: 2026-05-10
owners:
  - assura-maintainers
related:
  - .assura/config.yml
  - docs/analysis/2026-05-09-trellis-governance-adr.md
  - docs/analysis/2026-05-09-documentation-cleanup-register.md
---

# Structure enforcement review

## Summary

Assura now supports a closed-world structure policy for direct directory
contents. The repository can reject unexpected root files, unexpected child
directories, and forbidden file or directory patterns instead of only reporting
naming, frontmatter, rustdoc, and size violations for files that happen to be
present.

This matters because a clean `assura check .` should mean the repository shape
matches the expected source of truth, not only that existing files are named
well.

## Directory roles

| Path | Role | Enforcement treatment |
| --- | --- | --- |
| `.assura/` | Assura self-validation config and managed git hooks | Closed direct contents |
| `.trellis/` | Canonical workflow, task, spec, and workspace system | Closed direct contents except runtime exclusions |
| `.codex/` | Project-local Codex/Trellis support | Closed direct contents; not an installable integration package |
| `.agents/` | Project-local shared skills and agent instructions | Closed entrypoint contents with inherited skill naming exceptions |
| `docs/analysis/` | Current assessment and decision records | Active docs, frontmatter required |
| `docs/archive/` | Historical evidence and superseded planning docs | Historical docs, frontmatter required |
| `src/` | Rust library and CLI implementation | Closed direct module directories |
| `integrations/agents/` | Installable downstream agent integration packages | Closed direct package directories |
| `website/` | Documentation website source | Closed top-level website entries |

Generated outputs such as `target/`, nested `node_modules/`, nested `dist/`,
and website framework output are excluded from validation. They are not modeled
as source directories.

## Disallowed shape

The root policy rejects new top-level directories unless they are explicitly
added to `.assura/config.yml`. Agent integration package source should not
return to a top-level `opencode-plugin/` directory or branch into separate
per-agent roots. New installable agent packages should be added under
`integrations/agents/<agent>/`.

Historical planning systems remain non-canonical. OpenSpec, `specs-bak/`, and
old platform prompt/skill surfaces should not return unless the Trellis
governance ADR is updated first.

## LS-Lint parity matrix

Assura's structure-first config now covers the basic filesystem-control surface
used by LS-Lint 2.3. The relevant upstream behavior is documented in the
LS-Lint configuration basics and the LS-Lint 2.3 `exists` announcement:

- <https://ls-lint.org/2.3/configuration/the-basics.html>
- <https://ls-lint.org/blog/announcements/v2.3.0.html>

| LS-Lint feature | Assura status | Notes |
| --- | --- | --- |
| Extension naming rules such as `.rs: snake_case` | Supported | Converted into `files.naming_patterns`. |
| Wildcard extension rules such as `.*` and `.*.js` | Supported | Used for direct file matching and extension checks. |
| Directory rules through `.dir` | Supported | Converted into the `directories` bundle. |
| OR syntax with `|` | Supported | Naming rules are alternatives; `exists` tokens become count checks. |
| `exists` and `exists:1` | Supported | Count checks on direct files or directories. |
| `exists:0` | Supported | Fails when matching direct files or directories are present. |
| `exists:N-M` and `exists:N..M` | Supported | Inclusive count ranges. |
| Ignore patterns | Supported | Converted to Assura `exclude` entries. |
| Recursive closed-world policy | Partially supported | Direct-content rules are local by design; deeper directories must be modeled or inherit naming only. |
| Full LS-Lint CLI compatibility | Out of scope | Assura keeps its own structure-first config and compatibility conversion path. |

## Current gaps closed

Before this task, a well-named file such as `notes.md` could appear in a closed
root without failing. A new directory could also pass if it matched inherited
naming. The new policy fields make those cases fail as `unexpected_file` or
`unexpected_directory`.

## Follow-up

The Codex package under `integrations/agents/codex/` is only a skeleton. The
next integration task should design the install command, hook registration
shape, local Assura binary discovery, and structured feedback format before
claiming runtime behavior.
