---
id: analysis-2026-05-09-documentation-cleanup-register
type: analysis
title: Documentation cleanup register
status: active
created: 2026-05-09
updated: 2026-05-09
owners:
  - assura-maintainers
related:
  - docs/analysis/2026-05-09-trellis-governance-adr.md
  - docs/analysis/2026-05-09-project-assessment-and-alignment.md
  - .assura/config.yml
---

# Documentation cleanup register

## Purpose

This register prevents old planning systems and historical claims from becoming
competing sources of truth for agents. Trellis is now the canonical workflow and
spec system; this file tracks what should be migrated, archived, or deleted.

## Register

| Path | Status | Treatment | Rationale |
| --- | --- | --- | --- |
| `.trellis/` | Canonical | Keep and enforce | Trellis is the active workflow, task, spec, and workspace system. |
| `.codex/` | Canonical | Keep and enforce | Generated Codex integration for Trellis workflow state. |
| `.agents/skills/trellis-*` | Canonical generated support | Keep and update through Trellis | Shared agent skills created by Trellis init. |
| `AGENTS.md` | Canonical entrypoint | Keep current | Agents should find Trellis and Assura validation from here. |
| `.assura/config.yml` | Canonical enforcement config | Keep and evolve | Owns project structure enforcement. |
| `docs/analysis/` | Canonical assessment archive | Keep | Senior-engineer assessment reports live here. |
| `docs/analysis/2026-05-09-project-assessment-and-alignment.md` | Historical assessment | Keep in analysis archive | Existing assessment was promoted out of root docs and now documents the pre-remediation state. |
| `docs/archive/` | Historical archive | Keep with archive-specific Assura policy | Holds superseded docs that are useful evidence but not current source of truth. |
| `specs-bak/` | Historical | Deleted | Replaced by `.trellis/spec/`; no longer exposed as a competing source of truth. |
| `openspec/` | Historical/proposal reference | Deleted | Not canonical after Trellis adoption. |
| `.github/skills/openspec-*` | Historical support | Deleted | Keeps agents from seeing OpenSpec as an active competing workflow. |
| `.github/prompts/opsx-*` | Historical support | Deleted | Same as OpenSpec skills. |
| `docs/archive/phase*-review.md` | Historical | Archived | Useful history, not current state. |
| `docs/archive/actual-state-audit.md` | Superseded | Archived | Current assessment lives in `docs/analysis/`. |
| `docs/archive/implementation-gaps.md` | Superseded backlog | Archived | Avoids a second active backlog; actionable follow-up should move to Trellis tasks. |
| `docs/archive/configuration-spec.md`, `docs/archive/config-v2.md`, `docs/archive/final-config-design.md` | Conflicting config docs | Archived | Multiple config narratives are retained only as historical evidence. |
| `docs/release-notes.md` | Potentially stale | Audit against code before publishing claims | Release claims must not exceed current CLI behavior. |

## Enforcement follow-up

Assura should eventually enforce this register through project constraints:

- no active `specs/` or `specs-bak/` source of truth while Trellis is canonical;
- no active OpenSpec prompt/skill docs unless the ADR is revised;
- all current analysis docs must live under `docs/analysis/`;
- all archived historical docs must move under a single archive location;
- every active backlog item must live in Trellis tasks, not standalone gap docs.
