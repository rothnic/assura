---
id: goal-assura-beta-code-agnostic-capabilities-program
type: goal
title: Assura beta code-agnostic capabilities program
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-public-roadmap-artifact.md
  - ./assura-incremental-release-train.md
  - ./assura-beta-structure-severity-contract.md
  - ./assura-beta-content-collections-querying.md
  - ./assura-markdown-lint-link-reference-engine.md
  - ./assura-code-doc-reference-validation.md
  - ./assura-reference-daemon-readiness.md
  - ./assura-daemon-management-cli.md
  - ./assura-beta-agent-nudge-integrations.md
  - ./assura-vscode-daemon-integration.md
  - ./assura-ls-lint-no-slower-performance-gate.md
  - ../../.trellis/spec/assura/roadmap.md
---

# Assura Beta Code-Agnostic Capabilities Program

## Objective

Drive Assura from the current Project Intelligence and Markdown Reference
planning state to a beta release that provides code-agnostic repository quality
capabilities with daemon-aware local workflows.

This is the overarching goal to kick off when the next large chunk of work
should proceed goal by goal. A future agent should start here, pick the first
incomplete major iteration, execute only that iteration's referenced goal file
or files to their proof gates, update this program, then continue to the next
iteration.

## Beta Product Bar

Beta means Assura can be used locally by humans, CI, editors, and agents to
validate and understand a repository without being tied to one programming
language. The beta surface must include:

- structure validation with stable rule IDs, severity, and concise messages;
- frontmatter and Assura collection modeling, validation, querying, and bounded
  context packs;
- high-performance Markdown linting and heading validation;
- repository-internal code/doc reference validation;
- daemon mode for warm, incremental, affected-path feedback;
- daemon management commands for humans, editors, hooks, and agents;
- concise nudges for Codex, OpenCode, Claude, and Pi agents at relevant events;
- a VS Code integration over shared daemon/client contracts;
- pre-1.0 release artifacts and a public roadmap;
- LS-Lint-equivalent performance gates where Assura is never slower than native
  LS-Lint on any headline fixture.

## Ten Major Iterations

| Order | Epic | Primary goal file(s) | Exit bar |
| --- | --- | --- | --- |
| 1 | Roadmap And Releases | [Public roadmap artifact](./assura-public-roadmap-artifact.md), [Incremental release train](./assura-incremental-release-train.md) | Public roadmap and release train are repo-backed, validated, and ready to report beta progress. |
| 2 | Structure Severity | [Beta structure severity contract](./assura-beta-structure-severity-contract.md) | Structure findings have stable severity, rule IDs, messages, and agent-friendly remediation. |
| 3 | Collections Querying | [Beta content collections and querying](./assura-beta-content-collections-querying.md) | Frontmatter collections can be modeled, validated, queried, expanded, and packed for agents. |
| 4 | Markdown Quality | [Markdown lint and repository reference engine](./assura-markdown-lint-link-reference-engine.md) | Markdown linting, required headings, severity, suppressions, and safe fixes work locally and quickly. |
| 5 | Reference Graph | [Code and documentation reference validation](./assura-code-doc-reference-validation.md) | Markdown, code comment, docstring, file, heading, and line/range references are validated with inbound/outbound edges. |
| 6 | Daemon Core | [Reference daemon readiness](./assura-reference-daemon-readiness.md) | Warm daemon/session checks match one-shot truth and provide bounded affected-path feedback. |
| 7 | Daemon CLI | [Daemon management CLI](./assura-daemon-management-cli.md) | Status, start, stop, restart, doctor, logs, and fallback commands are JSON-capable and shared. |
| 8 | Agent Nudges | [Beta agent nudge integrations](./assura-beta-agent-nudge-integrations.md), [Agent daemon awareness](./assura-agent-daemon-awareness.md) | Codex, OpenCode, Claude, and Pi agents can receive concise event-aware nudges without context bloat. |
| 9 | VS Code | [VS Code daemon integration](./assura-vscode-daemon-integration.md) | VS Code diagnostics, status, and commands use the shared daemon/client contract. |
| 10 | LS-Lint Gate | [LS-Lint no-slower performance gate](./assura-ls-lint-no-slower-performance-gate.md) | No headline LS-Lint-equivalent fixture is slower than native LS-Lint; CI/review blocks regressions. |

## Execution Rules

- Execute epics in order unless a skipped epic is already complete or blocked
  with a recorded reviewer-accepted reason.
- Each epic should land as one or more PRs scoped to its referenced goal files.
- Complex implementation epics require independent review before PR creation.
- After each epic, update this program's progress log with PRs, validation
  commands, review artifacts, and the next epic.
- Use release-train work after any user-facing supported or experimental
  capability, not only at the end.
- Do not claim daemon, hook, editor, or performance support publicly until the
  relevant goal and release evidence proves it.
- The LS-Lint no-slower gate applies to every PR that changes
  LS-Lint-equivalent structure validation, traversal, ignore handling, rule
  planning, performance reporting, or fixture classification.

## Performance Gate Policy

The beta program rejects aggregate-only performance claims. For accepted
headline LS-Lint-equivalent fixtures:

- every fixture must be measured against native LS-Lint;
- every accepted cold CLI row must be no slower than LS-Lint;
- slower rows block merge even if the aggregate is faster;
- warm daemon/session rows are valuable but cannot pass the cold CLI gate;
- "CLI floor" is an attribution topic, not an excuse to merge slower behavior;
- any fixture removed from the headline set needs written rationale and review.

## Kickoff Prompt

```text
Execute docs/goals/assura-beta-code-agnostic-capabilities-program.md as the
master beta goal. Start with the workflow gate, git status, live roadmap,
current PRs, release state, and current performance claim summary from
benches/history/current.json. Then select the first incomplete epic from the
Ten Major Iterations table, read its referenced goal file(s), execute only that
epic to its proof gates with independent review for complex implementation,
update the beta program progress log, and report the next epic and goal path.
Do not skip the LS-Lint no-slower gate for any structure/performance change.
```

## Definition Of Done

- All ten major iterations are completed or explicitly deferred with a
  reviewer-accepted replacement path.
- Public docs and release artifacts classify beta-supported, experimental,
  future, and unsupported surfaces consistently.
- `assura check`, daemon workflows, agent nudges, VS Code, and content/query
  commands share the same core validation and finding contracts.
- Agent nudges stay bounded and event-relevant.
- The checked performance gate fails if any headline LS-Lint-equivalent fixture
  is slower than native LS-Lint.
- A beta release tag and GitHub release artifact exist with validation evidence.

## Validation Commands

Planning-only updates to this program should run:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

Implementation epics add their own validation commands.

## Review Tasks

- R1: Confirm the ten iterations reflect the actual missing beta capabilities.
- R2: Confirm every iteration points to an executable goal file.
- R3: Confirm agent integrations do not revive per-agent validation logic.
- R4: Confirm daemon and editor support are not claimed before proof exists.
- R5: Confirm the LS-Lint no-slower gate blocks per-fixture regressions.

## Reviewer Blocking Criteria

Block if the program is only a roadmap without executable goal files, omits
daemon mode, omits Codex/OpenCode/Claude/Pi agent nudges, omits frontmatter and
collection querying, omits Markdown/reference validation, lets VS Code bypass
the shared daemon contract, or allows any headline LS-Lint fixture to be slower
than native LS-Lint without blocking the merge.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-06-30 | Created the beta master program with ten major iterations and a hard no-slower LS-Lint performance gate after review clarified the desired beta destination. | User request; [.trellis/spec/assura/roadmap.md](../../.trellis/spec/assura/roadmap.md); `jq '.claim_summary,.warm_claim_summary' benches/history/current.json`. |
