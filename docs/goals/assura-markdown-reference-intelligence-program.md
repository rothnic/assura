---
id: goal-assura-markdown-reference-intelligence-program
type: goal
title: Assura Markdown reference intelligence program
status: planned
created: 2026-06-30
owners:
  - assura-maintainers
related:
  - ./assura-markdown-lint-link-reference-engine.md
  - ./assura-reference-daemon-readiness.md
  - ./assura-daemon-management-cli.md
  - ./assura-vscode-daemon-integration.md
  - ./assura-agent-daemon-awareness.md
  - ./assura-incremental-release-train.md
  - ./assura-public-roadmap-artifact.md
  - ../project-intelligence-facts.md
  - ../support-policy.md
---

# Assura Markdown Reference Intelligence Program

## Objective

Create the parent program for Markdown validation, repository-reference
intelligence, local daemon readiness, editor integration, and agent-aware
workflows. The program keeps the CLI and local protocol as the source of truth
so VS Code, future editors, hooks, and agent plugins use the same behavior
instead of each inventing its own scanner or daemon lifecycle.

## Current State

Assura already supports local structure checks, JSON/agent output, project
intelligence content queries, local JSON-line agent/editor sessions, and a
narrow Markdown lint/fix slice. `assura watch` is still experimental, and there
is no supported daemon lifecycle, VS Code package, or agent daemon-health
contract.

The next work should not skip directly to an editor extension. First, the
underlying CLI and daemon contracts need to be stable enough that an editor or
agent wrapper is thin and predictable.

The program also needs a release train. The current public GitHub release is
`v0.1.0` from 2026-05-24, while later Project Intelligence and Markdown
planning work has continued on `master`. New user-facing slices should move
through pre-1.0 version bumps and GitHub release artifacts before docs present
them as installable behavior.

## Program Principles

- CLI first: every daemon lifecycle and health operation must be available
  through a future JSON-capable daemon command family.
- Shared core: CLI, VS Code, and agent/plugin integrations must call the same
  daemon client/protocol layer.
- Local only by default: no MCP, hosted service, or remote access is required
  for daemon health, validation, or editor diagnostics.
- Bounded context: agents should receive health summaries, affected paths,
  rule IDs, and remediation commands, not full repository dumps.
- Progressive recovery: when the daemon is missing or unhealthy, agents and
  editors should know whether to start it, restart it, inspect logs, or fall
  back to one-shot `assura check`.
- Organized state: daemon cache, socket metadata, and logs must not litter the
  `.assura/` root. Project-local state belongs under an organized subtree such
  as `.assura/cache/daemon/` when project-local state is required.
- Release increments: supported or experimental user-facing slices should bump
  the pre-1.0 version, update release notes, and publish GitHub release
  artifacts through CI/CD.
- Public roadmap: website roadmap labels should be two to four words and
  generated from a repo-maintained artifact instead of copied prose.

## Major Sub-Goals

| Order | Sub-goal | Purpose | Status |
| --- | --- | --- | --- |
| 1 | [Markdown lint and repository reference engine](./assura-markdown-lint-link-reference-engine.md) | Validate Markdown, internal links, code/comment references, and inbound/outbound reference graph edges. | Planned |
| 2 | [Reference daemon readiness](./assura-reference-daemon-readiness.md) | Make the daemon/session layer reliable enough for repeated checks, file events, stale-cache detection, and bounded affected-reference feedback. | Planned |
| 3 | [Daemon management CLI](./assura-daemon-management-cli.md) | Provide daemon lifecycle/status/doctor/log commands as the shared control plane for humans, editors, and agents. | Planned |
| 4 | [VS Code daemon integration](./assura-vscode-daemon-integration.md) | Build the first editor integration over the shared CLI/client contracts, reporting diagnostics and daemon health in VS Code. | Planned |
| 5 | [Agent daemon awareness](./assura-agent-daemon-awareness.md) | Define how agents detect daemon health, recover when it is down, and receive bounded context through tools, hooks, or context injection. | Planned |
| 6 | [Incremental release train](./assura-incremental-release-train.md) | Ensure meaningful pre-1.0 slices produce version bumps, release notes, tags, and GitHub release artifacts. | Planned |
| 7 | [Public roadmap artifact](./assura-public-roadmap-artifact.md) | Render a concise Done/Now/Next website roadmap from the same repo-owned roadmap source. | Planned |
| 8 | Future Zed integration | Reuse the daemon CLI/client protocol after VS Code proves the editor contract. | Future |
| 9 | Future JetBrains integration | Reuse the daemon CLI/client protocol after VS Code proves the editor contract. | Future |

## Recommended Architecture

1. `assura check` and `assura content` remain the one-shot truth path.
2. A local daemon reuses the same validation and project-intelligence core,
   adds file watching, caches prepared state, and exposes a local socket or
   equivalent platform transport.
3. The future daemon command family is the public lifecycle and health surface:
   status, start, stop, restart, doctor, logs, and a bounded
   `check-changed` or equivalent affected-path request.
4. The VS Code extension uses the daemon client or CLI JSON contract. It should
   show diagnostics in the Problems panel, daemon state in the status bar, and
   lifecycle commands in the command palette.
5. Agents use the same JSON contracts through CLI calls, approved hooks, or
   plugin tools. If richer tool integrations exist, they should wrap the CLI
   contract rather than bypass it.
6. Release PRs bump pre-1.0 versions and publish GitHub artifacts before new
   daemon/editor/agent features are advertised as installable.
7. The public website renders a concise roadmap from a repository artifact,
   with two-to-four-word item labels and links to detail pages.

## Agent Integration Model

Agents should be able to answer these questions cheaply:

- Is the daemon running for this workspace?
- Is the daemon healthy, stale, warming, or unavailable?
- Which project root and config fingerprint is it serving?
- What changed paths or targets are dirty?
- What command should repair the current failure mode?
- Should the agent fall back to `assura check --format agent`?

The default agent path should be:

1. Call the future daemon status command in JSON mode.
2. If healthy, request bounded affected-reference or diagnostics context.
3. If missing or unhealthy, call the future daemon doctor command in JSON mode.
4. If the doctor says it is safe, call the future daemon start command.
5. If start is unavailable or fails, fall back to one-shot
   `assura check --format agent --agent codex` or the generic agent format.

Hooks and context injection should include only a compact daemon-health block
and remediation command. On-demand tool calls should fetch detailed diagnostics
only when the agent needs them.

## Definition Of Done

- Parent and sub-goal docs exist and are linked from the roadmap.
- Each sub-goal has objective, scope, non-goals, validation commands, review
  tasks, and blocking criteria.
- The roadmap names this parent program as the current recommended goal.
- A release-train child goal defines how incremental versions and GitHub
  releases should be produced before 1.0.
- A public-roadmap child goal defines the concise website roadmap artifact and
  validation rules.
- Existing support policy is not contradicted: daemon, watch, marketplace
  editor packages, MCP, and remote services remain future or experimental until
  their own goals prove support.

## Validation Commands

```bash
python3 ./.trellis/scripts/task.py validate .trellis/tasks/06-30-markdown-lint-link-reference-engine
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm the parent goal does not claim daemon/watch/editor support exists
  today.
- R2: Confirm VS Code is the first editor integration and Zed/JetBrains remain
  future follow-ups.
- R3: Confirm CLI management is the shared control plane for humans, editors,
  and agents.
- R4: Confirm the agent integration model prefers bounded status/context and
  explicit recovery commands over broad context injection.
- R5: Confirm the release-train goal prevents future supported daemon/editor
  claims from staying unreleased on `master`.
- R6: Confirm the public-roadmap goal keeps website labels short and generated
  from a repo artifact.

## Reviewer Blocking Criteria

Block if the program makes MCP or remote access mandatory, bypasses CLI-first
daemon management, promotes VS Code before daemon lifecycle contracts exist,
claims `assura watch` is supported, or fails to give agents a deterministic
daemon-health and recovery path, or omits incremental release/versioning work
from the program, or allows the website roadmap to drift from repo state.
