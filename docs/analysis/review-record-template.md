---
title: Review record template
date: 2026-06-02
status: template
---

# Review Record Template

Use this template for goal PR evidence records under `docs/analysis/`.

## Scope Review

- Goal file:
- Active Trellis task:
- Branch and PR:
- Public surfaces changed:
- Non-goals preserved:

## Evidence Inventory

| Evidence | Location | Checked In | Reproduction Command |
| --- | --- | --- | --- |
| Review record | `docs/analysis/<date>-<goal>-review.md` | Yes | N/A |
| Generated report | `target/<path>` | No | `<command>` |

## Validation Commands

| Command | Status | Notes |
| --- | --- | --- |
| `cargo xtask fast` | Not run | Replace before PR. |
| `cargo xtask evidence` | Not run | Replace before PR. |

## Review Tasks

| Task | Status | Evidence |
| --- | --- | --- |
| R0. Scope and source-of-truth review | Pending |  |
| R1. Design and contract review | Pending |  |
| R2. Implementation review | Pending |  |
| R3. Evidence reproduction review | Pending |  |
| R4. User journey review | Pending |  |
| R5. PR and completion review | Pending |  |
| R6. Alpha stability and stale-docs review | Pending | Confirm no internal stability shim was preserved unless tied to an explicit current support claim, and docs were updated or removed with the behavior change. |

## Review Feedback Closure

| Source | Finding | Decision | Evidence |
| --- | --- | --- | --- |
| Review agent |  | Pending |  |
| Gemini or PR review |  | Pending |  |

## Handoff

- PR:
- Next goal:
- Known risks:
