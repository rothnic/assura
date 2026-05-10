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
| `specs-bak/` | Historical | Archive or delete after migration | Replaced by `.trellis/spec/`; useful templates may be harvested first. |
| `openspec/` | Historical/proposal reference | Archive unless a concrete OpenSpec flow is retained | Not canonical after Trellis adoption. |
| `.github/skills/openspec-*` | Historical support | Archive/delete with `openspec/` | Keeps agents from seeing OpenSpec as an active competing workflow. |
| `.github/prompts/opsx-*` | Historical support | Archive/delete with `openspec/` | Same as OpenSpec skills. |
| `docs/PHASE*_REVIEW.md` | Historical | Move to archive | Useful history, not current state. |
| `docs/ACTUAL_STATE_AUDIT.md` | Superseded | Move to archive after current assessment is promoted | Current assessment should be canonical. |
| `docs/IMPLEMENTATION_GAPS.md` | Superseded backlog | Convert actionable items into Trellis tasks, then archive | Avoids a second backlog. |
| `docs/CONFIGURATION_SPEC.md`, `docs/config-v2.md`, `docs/final-config-design.md` | Conflicting config docs | Consolidate into Trellis spec and archive superseded versions | Multiple config narratives confuse agents and users. |
| `docs/release-notes.md` | Potentially stale | Audit against code before publishing claims | Release claims must not exceed current CLI behavior. |

## Enforcement follow-up

Assura should eventually enforce this register through project constraints:

- no active `specs/` or `specs-bak/` source of truth while Trellis is canonical;
- no active OpenSpec prompt/skill docs unless the ADR is revised;
- all current analysis docs must live under `docs/analysis/`;
- all archived historical docs must move under a single archive location;
- every active backlog item must live in Trellis tasks, not standalone gap docs.
