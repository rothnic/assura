---
id: goal-assura-agent-nudge-mvp
type: goal
title: Assura agent nudge MVP
status: completed
created: 2026-05-14
owners:
  - assura-maintainers
related:
  - .trellis/spec/assura/index.md
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/structure-enforcement.md
  - .trellis/spec/assura/tooling-stabilization.md
  - docs/goals/assura-v0-1-polished.md
---

# Assura agent nudge MVP

## Objective

Ship the smallest Codex/agent nudge MVP that proves Assura can turn
`assura check --format json` output into actionable, measured feedback for a
developer or agent, without claiming a complete autonomous agent system.

## Starting Repo Truth

- `assura check --format json` returns `StructureCheckReport` with `success`,
  paths, counts, and `violations`.
- `integrations/agents/codex` exists as the canonical Codex integration
  package location.
- At goal start, the Codex package was a skeleton and did not install hooks.
- At goal start, website docs labeled agent nudges as future work.
- Repo-local `.agents/skills/` is the durable progressive-disclosure guidance
  surface for agent workflows.

## Acceptance Criteria

### 1. Runtime Nudge Path

- Add a supported Codex integration entrypoint that can consume Assura JSON.
- Provide a CLI or library function that runs `assura check --format json` and
  emits nudge output.
- Preserve nonzero exit behavior when Assura reports validation failures.
- Do not install Codex hooks automatically in this goal.

### 2. Actionable Nudge Content

Every nudge for a failing report must include:

- A short summary.
- The violation count and affected rules.
- File/path-specific corrective guidance.
- References to repo-local guidance surfaces such as `AGENTS.md`,
  `.agents/skills/`, or `.assura/config.yml`.
- A clear statement that the nudge is advisory unless the caller enforces the
  command exit code.

### 3. Measurement

Add a small evaluation data model that can compare:

- instructions-only workflows
- `AGENTS.md`/skills workflows
- Assura runtime nudges

Metrics must include:

- structural violations introduced
- correction loops
- instruction adherence
- nudge count
- useful nudges
- noisy nudges
- missed violations
- nudge precision

### 4. Tests

Add fixture-style tests proving:

- Passing Assura JSON produces no blocking nudge.
- Failing Assura JSON produces actionable nudge content.
- Invalid JSON is rejected with a clear error.
- Measurement comparison computes nudge precision and relative correction-loop
  changes.
- The CLI can read a report file and output JSON nudge data.

### 5. Documentation

- Update `integrations/agents/codex/README.md` from skeleton status to MVP
  status.
- Update website docs that describe future agent nudges so they point to the
  MVP surface without implying automatic hook installation.
- Keep unsupported Codex hook installation and complete agent automation marked
  future-only.

### 6. Validation Commands

Run and pass, or document exact blockers:

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo run --quiet -- check --format json .
cd integrations/agents/codex && npm install && npm run lint && npm test && npm run build
cd website && pnpm build
```

## Non-Goals

- Do not implement automatic Codex hook installation.
- Do not claim general agent quality scoring is complete.
- Do not add external runtime dependencies unless they are required.
- Do not change the primary `assura check` JSON schema unless necessary.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-05-14 | Goal created after syncing `master` and branching `codex/assura-agent-nudge-mvp`. | `docs/goals/assura-agent-nudge-mvp.md`; `git status -sb` |
| 2026-05-14 | Iteration 1 implemented the Codex nudge library, CLI file mode, evaluation metrics, fixture-style Node tests, and docs updates. Initial sandboxed `npm install` hit `EAI_AGAIN`; approved registry access installed `@types/node`. | `integrations/agents/codex/src/index.ts`; `integrations/agents/codex/src/cli.ts`; `integrations/agents/codex/src/nudge-test.ts`; `npm run lint`; `npm test`; `npm run build` |
| 2026-05-14 | Context checkpoint before broader validation: goal tool reported `tokensUsed=75339`, no token budget exposed. No new reusable project skill needed; the existing `assura-local-build` skill covers the npm network issue. | `get_goal`; `.agents/skills/assura-local-build/SKILL.md` |
| 2026-05-14 | Required validation passed. A first self-check immediately after `pnpm build` saw transient ignored `website/.agents` and `website/.codex` directories; after the build cleaned up, rerunning `assura check` passed with zero violations. A Clippy run without OpenSSL variables failed for local `pkg-config`/OpenSSL discovery, then passed with the documented WSL OpenSSL variables. | `cargo fmt --all -- --check`; `cargo test --all-targets --quiet`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo run --quiet -- check --format json .`; `pnpm build`; `npm run lint && npm test && npm run build` |
| 2026-05-14 | Updated Assura roadmap so `Agent Nudge MVP` is an active epic with this Trellis task as owner and hook installation as the next follow-up. | `.trellis/spec/assura/roadmap.md` |
| 2026-05-14 | Branch pushed and draft PR opened. | `codex/assura-agent-nudge-mvp`; `https://github.com/rothnic/assura/pull/10` |
| 2026-05-14 | Addressed PR review gaps: added an injectable Assura process runner, direct-run tests for success/failure/non-JSON exit behavior, CLI preservation of Assura non-JSON exit codes, README wording aligned to behavior, and package dry-run cleanup so compiled test files are excluded. | `integrations/agents/codex/src/index.ts`; `integrations/agents/codex/src/cli.ts`; `integrations/agents/codex/src/nudge-test.ts`; `npm test`; `npm pack --dry-run` |

## Completion Audit Checklist

| Requirement | Evidence |
| --- | --- |
| Goal file exists and defines outcomes | `docs/goals/assura-agent-nudge-mvp.md` |
| Codex runtime nudge path implemented | `integrations/agents/codex/src/index.ts` exposes `parseStructureCheckReport`, `createNudgeFromReport`, `runAssuraCheck`, and render helpers; `integrations/agents/codex/src/cli.ts` exposes `assura-codex-nudge` |
| Nudge content is actionable | `createNudgeFromReport` includes summary, violation count, affected rules, per-path guidance, references, and advisory wording |
| Measurement model compares three workflow modes | `WorkflowMode`, `EvaluationRun`, and `compareEvaluationRuns` cover `instructions_only`, `agents_skills`, and `assura_runtime_nudges` |
| Tests cover parser, nudge, metrics, CLI file mode, and direct Assura-run mode | `integrations/agents/codex/src/nudge-test.ts`; `npm test` |
| Docs updated truthfully | `integrations/agents/codex/README.md`; `website/src/content/docs/examples/multi-agent-config.md`; getting started, introduction, and why pages |
| Roadmap updated | `.trellis/spec/assura/roadmap.md` |
| Validation commands pass | `cargo fmt --all -- --check`; `cargo test --all-targets --quiet`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo run --quiet -- check --format json .`; `pnpm build`; `npm run lint && npm test && npm run build` |
| Branch pushed and PR opened | Draft PR `https://github.com/rothnic/assura/pull/10` from `codex/assura-agent-nudge-mvp` to `master` |
