---
id: goal-assura-public-roadmap-artifact
type: goal
title: Assura public roadmap artifact
status: completed
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-markdown-reference-intelligence-program.md
  - ../../.trellis/spec/assura/roadmap.md
  - ../project-memories.md
  - ../../website/
---

# Assura Public Roadmap Artifact

## Objective

Publish a clean, concise roadmap on the website from a repository-owned
roadmap artifact, so users can quickly see what is done, what is active, and
what is coming next without reading internal Trellis detail.

## Current Gap

The internal roadmap is useful for agents, but it is too dense for the public
website. It mixes historical proof, active task routing, and implementation
detail. Users need a high-level roadmap with short labels and optional links
to deeper goal docs.

## Public Roadmap Contract

- Each public roadmap item label must be two to four words.
- The roadmap page should show simple columns or groups such as Done, Now, and
  Next.
- Each item may link to a detail page or goal doc for users who want more.
- The website must read from a repository artifact, not a hand-copied website
  list.
- The repository artifact must be validated so labels stay concise and links
  stay live.

## Proposed Artifact

Create `docs/data/public-roadmap.json` as the validated public projection of
`.trellis/spec/assura/roadmap.md`. The website renders from this JSON artifact,
and `cargo xtask target-state` keeps labels concise, detail paths live, website
links mapped to content pages, and active public state aligned with the beta
program.

## Scope

- Define the structured roadmap artifact and validation rules.
- Add a website roadmap page or section that renders the artifact.
- Keep labels at two to four words.
- Link each public item to detail content where available.
- Keep internal roadmap/task detail out of the first viewport.
- Add tests or checks that fail on long labels, missing links, or drift between
  public roadmap status and active Trellis roadmap state.

## Non-Goals

- No marketing-heavy landing page.
- No separate CMS or remote roadmap service.
- No duplicated hand-maintained website roadmap.
- No exposing every archived Trellis task publicly.

## Definition Of Done

- Website has a concise public roadmap generated from a repo artifact.
- Roadmap labels are two to four words.
- Done, active, and upcoming items are easy to scan.
- Each item can link to details without making the main roadmap dense.
- Validation fails if labels are too long or detail links break.
- Internal `.trellis/spec/assura/roadmap.md` points to the public artifact or
  consumes the same source.

## Validation Commands

```bash
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm public labels are two to four words.
- R2: Confirm the website renders from a repo artifact.
- R3: Confirm detail links work and do not require reading internal task files.
- R4: Confirm active public roadmap state matches the internal active roadmap.

## Reviewer Blocking Criteria

Block if the website roadmap is hand-copied, labels exceed four words, the
page exposes dense internal task prose as the main roadmap, or links can drift
without validation.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-30 | Completed the public roadmap artifact slice with `docs/data/public-roadmap.json`, a Starlight roadmap page, sidebar routing, and target-state drift checks tied to the current recommended Trellis goal. | `docs/data/public-roadmap.json`; `website/src/content/docs/roadmap.mdx`; `website/src/components/public-roadmap.astro`; `cargo xtask target-state`; independent review. |
