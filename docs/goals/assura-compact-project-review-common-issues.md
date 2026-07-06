---
id: goal-assura-compact-project-review-common-issues
type: goal
title: Assura compact project review and common issues
status: planned
created: 2026-07-04
owners:
  - assura-maintainers
related:
  - ./assura-performance-polish-program.md
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-doctor-explain-feedback.md
  - ./assura-llm-wiki-personal-knowledge-base-starters.md
  - ../analysis/2026-07-04-backlog-priority-agent-harness-and-okf-assessment.md
---

# Assura Compact Project Review And Common Issues

## Objective

Give humans and agents one compact first diagnostic flow for the common
question: "Is this repo healthy, and what should I fix or configure next?"

The flow should combine current Assura truth from structure checks, doctor
state, onboarding readiness, next actions, and high-signal content gaps without
dumping noisy reference findings or forcing users to know every lower-level
command first.

## Current Gap

Assura has useful primitives: `assura check`, `assura doctor`, self-check
rules, agent nudges, content agent-query gaps, daemon status, and onboarding
packets. The common user path is still fragmented. A new user or coding agent
can pass one command and still miss that optional references, generated docs,
skills, model files, or onboarding artifacts are incomplete, duplicated, or
inconsistent with the repository's existing structure.

## Scope

- Design a compact review command, recipe, or documented flow over existing
  surfaces before adding new core behavior.
- Prioritize issues by user action: fix now, configure intentionally, inspect
  before changing, or ignore/generated/archive noise.
- Include structure-fit guidance for new files/directories: check the nearby
  project shape, avoid duplication, and only change `.assura/config.yml` when
  the new path is intentionally part of the repository contract.
- Include an advisory directory heat map that rolls up validation, content-gap,
  worktree, branch, and churn pressure without turning Git availability into a
  hard requirement.
- Keep output bounded for agent use, with stable JSON fields and concise text.
- Filter or classify low-value reference noise from generated artifacts,
  archives, benchmark history, and logs before surfacing content-gap counts.
- Link from the review output to lower-level commands for detailed evidence.
- Document the first-run workflow and the before-PR review workflow.

## Non-Goals

- No automatic config rewrites.
- No broad new search engine.
- No replacement for `assura check`, `assura doctor`, or content-query
  commands.
- No blocking policy based only on noisy unresolved-reference counts.
- No hosted service, MCP server, or editor-specific requirement.

## Definition Of Done

- One documented compact review path exists for humans and local agents.
- The path reports structure check status, doctor/onboarding status, config
  fit warnings, content/reference gap summary, next recommended commands, and
  whether findings are blocking, advisory, inactive, or informational.
- JSON output is stable enough for agent wrappers and avoids scraping text.
- Text output stays concise enough to paste into an agent turn without
  crowding out the task.
- The flow distinguishes "fix this file" from "decide whether this path should
  exist in the project contract."
- The flow shows where work is heating up by directory so agents can notice
  growing untracked, changed, violating, or branch-heavy areas while working.
- Generated/archive/log/benchmark reference noise is filtered, classified, or
  explicitly reported as omitted.
- Docs show how to use the compact review before creating a new top-level
  directory, before opening a PR, and when onboarding an existing repository.
- Tests cover a clean repo, a structure mismatch, duplicated/unmodeled path
  pressure, inactive optional guidance, noisy reference gaps, a genuinely
  actionable content gap, and directory heat-map rollups from real Git state.

## Validation Commands

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo fmt --check
cargo test --test agent_surface_cli --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo test --test content_query_cli --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
git diff --check
```

If the implementation is docs-only for the first slice, keep validation scoped
to workflow gate, Assura self-check, docs/evidence/target-state, and
`git diff --check`.

## Review Tasks

- R1: Confirm the compact review uses existing Assura truth surfaces instead
  of inventing a parallel validator.
- R2: Confirm agent JSON is bounded, stable, and points to detailed evidence
  only when needed.
- R3: Confirm structure-fit guidance nudges agents to inspect the existing
  project shape before adding directories or changing config.
- R4: Confirm noisy reference findings are not promoted into blocking guidance
  without filtering or classification.
- R5: Confirm docs make the common first diagnostic path obvious without hiding
  lower-level commands from advanced users.

## Reviewer Blocking Criteria

Block if the implementation changes `.assura/config.yml` automatically, treats
generated/archive/log reference noise as a hard failure, duplicates validation
logic outside existing surfaces, emits unbounded context for agents, or leaves
users without a clear distinction between "fix the repo" and "decide whether
this path belongs in the repo contract."

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-05 | Implemented the first compact review slice as the `assura review` command with text, JSON, YAML, and agent formats. The command reuses existing project doctor and content agent-query gap summaries, reports blocking/advisory/inactive/informational findings, includes structure-fit guidance, and classifies raw unresolved reference candidates as informational with generated/archive/log/benchmark noise omitted from blocking policy. The line-limit check forced a natural split into `src/cli/project_review.rs` plus `src/cli/project_review/report.rs`; the durable module family was declared narrowly in `.assura/config.yml`. | `src/cli/project_review.rs`; `src/cli/project_review/report.rs`; `tests/project_review_cli.rs`; `.trellis/spec/assura/compact-project-review.md`; `docs/validation.md`; `cargo test --test project_review_cli --quiet`; `cargo run --quiet --bin assura-full -- check --format json .`; `cargo run --quiet --bin assura-full -- review . --format text`. |
| 2026-07-05 | Added the advisory directory heat-map slice to `assura review`. The report now includes `heatmap` in JSON/agent output and compact `heat:`/`hot:` text lines with validation, naming, content-gap totals, Git branch/worktree state, churn, risk flags, and top hot directories. The implementation reuses the same structure report already built for doctor and best-effort local Git calls; Git unavailable is non-fatal. A real temporary Git repository test now proves branch commits, branch-changed files, untracked files, modified files, and a naming violation roll up under `src`. The line-limit gate also forced a natural doctor split into `src/cli/doctor_report.rs` instead of shortening `doctor.rs`. | `src/cli/project_review/heatmap.rs`; `src/cli/project_review/heatmap/git.rs`; `src/cli/doctor_report.rs`; `tests/project_review_cli.rs`; `.trellis/spec/assura/compact-project-review.md`; `docs/validation.md`; `docs/support-policy.md`; `website/src/content/docs/reference/api.md`; `cargo test --test project_review_cli --quiet`; `cargo test --test doctor_explain_cli --quiet`; `cargo run --quiet -- check --format json .`. |
| 2026-07-05 | Independent review found four optimization/correctness gaps. Fixed the valid issues before commit: nested Git checkouts now scope status/diff/churn to the Assura project path and strip repo-root prefixes, directory-targeted violations now roll up to the directory itself, Git reads run with optional locks disabled, and normal doctor/review construction no longer clones the full `StructureCheckReport`. Added regressions for nested Git scoping and `unexpected_directory` hot-dir visibility. | `src/cli/project_review/heatmap.rs`; `src/cli/project_review/heatmap/git.rs`; `src/cli/doctor.rs`; `src/cli/doctor_report.rs`; `tests/project_review_cli.rs`; `cargo test --test project_review_cli --quiet` now runs 7 tests; `cargo test --test doctor_explain_cli --quiet`; `cargo run --quiet -- check --format json .`. |

## Copy/Paste Goal Prompt

```text
Execute docs/goals/assura-compact-project-review-common-issues.md.

Start from live state: run the Trellis workflow gate, inspect git status,
review the current roadmap, and run assura check/doctor/content-gap commands
needed to understand the existing first diagnostic experience.

Design and implement the smallest reviewed slice that gives humans and agents
one compact project review path. It must combine structure check status,
doctor/onboarding readiness, next actions, structure-fit guidance for new
files/directories, and high-signal content/reference gaps without surfacing
noisy generated/archive/log findings as blockers.

Do not auto-edit .assura/config.yml or build a new search engine. Reuse
existing Assura truth surfaces, add tests for clean, mismatch, duplicate-path,
inactive-guidance, noisy-gap, and actionable-gap cases, update docs, run the
scoped validation gates, and record reviewer findings before closure.
```
