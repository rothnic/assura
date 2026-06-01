---
id: goal-assura-real-project-policy-proof
type: goal
title: Assura real project agentic feedback proof
status: active
created: 2026-05-26
owners:
  - assura-maintainers
related:
  - .trellis/spec/assura/index.md
  - .trellis/spec/assura/roadmap.md
  - .trellis/spec/assura/structure-enforcement.md
  - .trellis/spec/assura/tooling-stabilization.md
  - docs/goals/assura-v0-1-polished.md
  - docs/goals/assura-agent-nudge-mvp.md
  - docs/goals/assura-native-ls-lint-performance-rearchitecture.md
  - docs/goals/assura-pinned-ls-lint-fixture-benchmark-suite.md
  - docs/analysis/2026-05-19-ls-lint-performance-progress-ledger.md
---

# Assura Real Project Agentic Feedback Proof

## Objective

Prove that a new Assura user can take a realistic project structure policy from
intent to working agentic feedback in one coherent end-to-end slice.

The target outcome is not another isolated benchmark or docs update. The target
outcome is a reviewer can inspect a representative project policy, run Assura
against valid and invalid project states, see clear failures, see useful agent
or developer guidance, install and verify the local feedback loop, and
understand how the same workflow would be adopted by a real repo without relying
on unsupported roadmap claims.

This work should take a focused engineer about 1-2 weeks. It should produce a
single reviewable PR or a tightly sequenced pair of PRs only if the fixture and
website/doc work cannot stay readable in one branch.

## Why This Goal Exists

Recent work proved several important pieces independently:

- v0.1 onboarding is now tied to supported `assura check`, `assura init`,
  `assura migrate`, `assura status`, and website flows.
- The agent feedback MVP can turn Assura JSON reports into advisory feedback.
- The CLI-to-CLI performance path can make fair LS-Lint comparisons.
- The lightweight `assura-check` path can support a truthful performance claim
  for LS-Lint-compatible structure checks.

The next useful proof is that these pieces compose into a product workflow a new
agentic-coding adopter would actually trust. Assura should be able to express a
real project shape, catch meaningful drift, explain the fix, and keep feedback
fast enough that an agent can correct course before it has to start a new turn.

## User Journey And Gap Analysis

This goal is grounded in the questions a real first-time user will ask while
evaluating Assura for agentic coding:

- "Can I migrate or write a policy that protects the shape of my actual repo,
  not just a demo tree?"
- "Can I require project-specific guidance files such as package-level
  `AGENTS.md` without confusing that with upstream LS-Lint behavior?"
- "Can I run the feedback loop through the stable `assura check` surface in a
  way an agent can discover and verify without guessing?"
- "Will the feedback arrive quickly enough to keep the agent inside the same
  implementation turn?"
- "When Assura provides feedback to the agent, can I see whether the feedback was useful,
  ignored, noisy, or fixed?"
- "Do the website and docs show the current supported path plainly, without
  hiding the real commands behind roadmap language?"

The known gaps are the next work, not claim-softening:

- Hook setup is currently documentation/manual-workflow oriented. This goal
  should use `assura check --format agent` and optional delivery adapters as
  the supported command path that agents can run for themselves.
- The agent feedback MVP exists, but it is not yet packaged as the primary
  same-turn feedback loop that records whether the agent handled the feedback.
- Real-project policy examples exist through benchmark fixtures, but the user
  adoption story needs one canonical walkthrough that protects the project shape
  as-is and includes meaningful drift cases.
- Assura-extended LS-Lint notation such as `exists:1` for exact file or
  directory requirements must be documented and tested as a positive capability.
- Public performance evidence must connect to the agent hot path: repeated
  checks, changed-path checks, warm/editor-session checks, and the observed
  behavior of agents that received feedback.

## End-to-End Outcome

At the end of this goal, the repo should contain one canonical "real project
agentic feedback proof" scenario with all of these pieces connected:

1. A representative project policy that looks like a real modern repository,
   not a toy extension fixture.
2. Valid and invalid materialized project states for that policy.
3. Assura configuration that validates the policy through supported v0.1
   surfaces.
4. CLI evidence showing clean projects pass and drifted projects fail with
   stable, useful reports.
5. A supported `assura check --format agent` path for the local feedback loop.
6. Agent/developer feedback evidence showing the failure report can be turned into
   concise corrective guidance and that the agent response can be observed.
7. Website or docs guidance that explains the scenario as a user-facing
   adoption example.
8. Tests and checked-in evidence that make the scenario reproducible for future
   agents.

## Starting Repo Truth

- `assura check` is the supported structure-first validation path.
- `.assura/config.yml` dogfoods the current repository structure rules.
- Closed-world direct file and directory contracts are documented in
  `.trellis/spec/assura/structure-enforcement.md`.
- The stable agent feedback path is `assura check --format agent`; Codex
  delivery uses `--agent codex`.
- Performance comparison claims must use the executable and row-family
  contracts in `.trellis/spec/assura/tooling-stabilization.md`.
- Existing performance goal docs already cover CLI-to-CLI fairness, lightweight
  check binaries, and pinned fixture benchmark expansion.

## Scope

This goal covers:

- selecting or creating one realistic policy scenario,
- implementing or updating fixtures for valid and invalid project states,
- ensuring the policy uses supported Assura v0.1 behavior,
- implementing supported agent feedback through `assura check --format agent`,
- adding tests that lock the expected pass/fail behavior,
- producing JSON report examples for the invalid state,
- producing feedback examples from the invalid report,
- recording whether the agent handled, ignored, or needed repeat feedback for the
  same violation class,
- updating docs or website content so the scenario is useful to a first-time
  adopter,
- recording reviewer evidence in `docs/analysis/`.

This goal does not cover:

- daemon or watch-mode architecture,
- dependency graph validation,
- broad new Assura notation features beyond the policy proof,
- a broader performance benchmark suite beyond this scenario,
- claiming complete agent automation or general AI quality scoring.

## Policy Scenario Requirements

The scenario should model a project that Assura is meant to help with:

- a clean root with only declared well-known files,
- app or package directories with explicit allowed names or patterns,
- source, test, docs, scripts, and generated-output areas,
- ignored generated paths such as `node_modules`, `dist`, `coverage`, `.next`,
  or equivalent,
- at least one direct-content rule that catches a well-named but unexpected
  file or directory,
- at least one naming rule that catches an incorrectly named file,
- at least one existence/count rule that catches missing or duplicated required
  content,
- at least one Assura-extended exact `exists:1` rule that models a real
  agentic-coding policy, such as requiring an `AGENTS.md` or project guidance
  file per package.

Prefer a generated fixture when it keeps the test deterministic. If a pinned
external repo is used, keep materialization opt-in unless the checkout is small,
stable, and fast enough for ordinary local validation.

## Definition Of Done

This goal is done only when all of the following are true.

### 1. Policy Fixture Proof

- A valid fixture passes through the supported Assura check path.
- An invalid fixture fails through the supported Assura check path.
- The invalid fixture produces violations for the intended drift categories,
  including unexpected direct contents, naming drift, and existence/count drift.
- Fixture generation or materialization is deterministic and documented.
- The fixture does not depend on an untracked local checkout.

### 2. Assura Configuration Proof

- The scenario config uses supported v0.1 structure-first fields.
- Any LS-Lint-compatible parts are labeled as such.
- Any Assura-specific extensions are labeled as Assura behavior and are not
  presented as native LS-Lint parity.
- The config remains readable enough to serve as a user-facing example.

### 3. Report And Feedback Proof

- The failing scenario has a checked-in JSON report example or a reproducible
  command that writes one.
- The report includes stable paths, rule names, counts, and success status.
- The agent feedback MVP can consume the failing report and emit actionable
  guidance.
- Feedback output points to useful project-local guidance such as `AGENTS.md`,
  `.assura/config.yml`, or scenario docs.
- A measured feedback loop records at least: violation class, feedback count, whether
  the agent fixed the issue before a new turn, useful/noisy classification, and
  remaining violations.
- Advisory behavior is stated clearly when the caller is not enforcing exit
  codes.

### 4. Hook And Agent Feedback Proof

- Use `assura check --format agent` as the supported command path for local
  feedback.
- Use `assura check --format agent --agent codex` only as a Codex delivery
  adapter for users who manually wire a `UserPromptSubmit` hook.
- The proof must produce clear success/failure output an agent can use before
  continuing work.
- Hook examples must be append-only and must not overwrite unrelated custom hook
  content.
- The commands must include instructions an agent can follow without relying on
  hidden local state.

### 5. User-Facing Proof

- Website or docs content shows the scenario from a user's perspective:
  install Assura, define policy, run `assura check`, inspect failure, receive
  feedback, fix project drift, rerun check.
- The page or guide uses supported commands only.
- The content distinguishes the current supported workflow from broader roadmap
  items such as dependency graphs, hosted telemetry, and autonomous agent
  orchestration.
- The scenario is discoverable from an existing relevant docs path or index.

### 6. Performance And Hot-Path Proof

- Evidence includes a repeated-check or changed-path case that represents the
  same-turn agent feedback path.
- The evidence uses release binaries where executable startup or user-facing
  performance is part of the claim.
- Any aggregate performance language is backed by the checked real-repo data or
  by this scenario's measured artifact. Do not use synthetic-only evidence as a
  headline user claim.
- Wildcard configured-scope checks must not add a fresh full-tree traversal per
  scope. Derive matched wildcard scopes from the main traversal, a cached scope
  index, or equivalent measured design so same-turn agent feedback does not
  multiply filesystem walks as policies grow.
- The review record explains what is fast enough for this workflow and what
  remains a future optimization target.

### 7. Test And Regression Proof

- Tests cover valid and invalid fixture behavior.
- Tests assert the expected violation rule categories for the invalid fixture.
- Tests cover JSON report shape or parseability where practical.
- If feedback output is generated in tests, assertions check useful content rather
  than brittle full strings.
- Tests cover the supported agent feedback command path, including advisory and
  blocking behavior where relevant.
- Existing LS-Lint parity and structure-first behavior is not weakened.

### 8. Review Evidence

- `docs/analysis/` contains a short review record with:
  - exact commands run,
  - pass/fail results,
  - paths to fixture/config/report/feedback/hook evidence,
  - user-journey notes describing what a new user would experience,
  - any screenshots if website pages changed,
  - known limitations and follow-up recommendations.
- The PR description links the goal document and the review record.
- A reviewer can reproduce the core valid and invalid checks without reading
  this conversation.

## Minimum Validation Commands

Run these before calling the goal complete, adjusting only when the PR explains
an exact platform blocker:

```bash
cargo fmt --all -- --check
git diff --check
cargo test --all-targets --quiet
cargo run --quiet -- check --format json .
```

If hook or Codex feedback integration code is changed, also run the relevant
package checks and the supported agent feedback command:

```bash
cd integrations/agents/codex && npm install && npm run lint && npm test && npm run build
cargo run --quiet -- check --format agent --agent codex . --warn
```

If website docs are changed, also run:

```bash
cd website && pnpm build
```

If performance data or website performance claims are touched, also follow
`.agents/skills/assura-performance-reporting/SKILL.md` and regenerate the
checked-in performance data through the established report workflow.

## Reviewer Blocking Criteria

Reviewers should block completion if any of these are true:

- the scenario is too synthetic to prove real adoption usefulness,
- the invalid fixture fails for incidental reasons instead of intentional
  policy drift,
- docs introduce a separate feedback install/status/verify CLI instead of the
  stable `assura check --format agent` surface,
- docs imply unsupported daemon behavior, dependency graph validation, hosted
  telemetry, or autonomous agent enforcement,
- the report or feedback examples are hand-written instead of generated or
  reproducible,
- feedback evidence does not show whether the agent handled the feedback before a
  new turn,
- performance evidence does not reflect the real same-turn feedback path,
- the scenario introduces a second source of truth for project workflow next to
  Trellis,
- validation commands are missing, stale, or only described in conversation.

## Suggested Work Plan

1. Select the policy scenario and write a short design note under
   `docs/analysis/`.
2. Add valid and invalid fixtures plus supported Assura config.
3. Implement or verify the `assura check --format agent` feedback path.
4. Add fixture tests, stable agent feedback tests, and JSON report proof.
5. Add feedback proof using the existing agent feedback package, including
   observed agent response metrics.
6. Update website or docs adoption content around the full user journey.
7. Run the validation commands and record a review artifact.

## Progress Log

| Date | Phase | Evidence |
| --- | --- | --- |
| 2026-05-29 | Execution started | Created Trellis task `.trellis/tasks/05-29-real-project-agentic-feedback-proof`, branched `codex/real-project-agentic-feedback-proof`, and seeded PRD from this goal. |
| 2026-05-29 | Implementation slice | Added checked valid/invalid real-project fixtures, same-turn observation, user-facing docs, JSON/feedback evidence, and review record. |
| 2026-05-29 | PR review follow-up | Opened PR #13, received Gemini Code Assist review, addressed three comments on worktree hook parsing and defensive same-turn observation handling, and recorded the follow-up in `docs/analysis/2026-05-29-real-project-agentic-feedback-review.md`. |
| 2026-05-31 | Stable surface follow-up | Synced to `master` after PR #15, archived the merged 05-31 install/status/verify task, refreshed the real-project proof around `assura check --format agent`, and kept Codex delivery only under `--agent codex`. |
| 2026-05-31 | Validation and review | Iteration 1; context health: goal tracker exposed `tokensUsed=174387`, no remaining-token budget exposed. Review agent findings were resolved by making the checked feedback artifact reproducible from the stable CLI command and documenting Codex `features.hooks = true` plus `/hooks` approval. Passed `cargo fmt --all -- --check`, `cargo test --all-targets --quiet`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo run --quiet -- check --format json .`, `cargo run --quiet -- check --format agent tests/fixtures/real-project-agentic-feedback/invalid --warn`, `cargo run --quiet -- check --format agent --agent codex tests/fixtures/real-project-agentic-feedback/invalid --warn`, `npm run lint && npm test && npm run build && npm pack --dry-run` in `integrations/agents/codex`, `node --run verify:fast`, `node --run verify:docs`, `npx pnpm@10.25.0 build` in `website`, and `git diff --check`. |

## Handoff Prompt

Use this prompt to start implementation:

```text
Execute docs/goals/assura-real-project-policy-proof.md. Start by reading the
goal document, AGENTS.md, .agents/skills/assura-goal-execution/SKILL.md,
.trellis/spec/assura/index.md, .trellis/spec/assura/structure-enforcement.md,
and .trellis/spec/assura/tooling-stabilization.md. Implement the smallest
end-to-end real project agentic feedback proof that satisfies the Definition Of
Done: realistic policy, stable `assura check --format agent` feedback, optional
Codex delivery only through `--agent codex`, same-turn feedback evidence,
hot-path performance evidence, docs, tests, and a review record under
docs/analysis/.
```
