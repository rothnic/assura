# Real Project Agentic Feedback Proof

## Goal

Complete `docs/goals/assura-real-project-policy-proof.md` end to end: a realistic Assura policy scenario with valid and invalid project states, reproducible check/report/feedback evidence through `assura check --format agent`, user-facing docs, tests, performance notes for the same-turn agent path, and review evidence.

## What I Already Know

- The current goal title is "Assura real project agentic feedback proof" and it is `status: planned`.
- The goal requires a single canonical scenario that proves Assura can protect a real project shape and give useful agent/developer feedback.
- Supported v0.1 surfaces include `assura check`, `assura init`, `assura migrate`, `assura status`, JSON/YAML/text reports, and advisory agent feedback.
- Agent feedback should use `assura check --format agent`; Codex delivery should use `assura check --format agent --agent codex`.
- The agent feedback package parses `assura check --format json` reports as a library helper, not as a separate user-facing CLI.
- Structure enforcement supports direct file/directory contracts, forbidden patterns, and Assura extension `exists` direct-count rules.
- Trellis is the canonical workflow/source-of-truth layer; the scenario must not introduce a competing task system.

## Assumptions

- A deterministic generated fixture is preferable to a pinned external checkout for this goal because it keeps ordinary local validation fast and reproducible.
- The smallest acceptable feedback path is proving `assura check --format agent` rather than adding a separate daemon, hook-management CLI, or hidden agent service.
- Same-turn performance proof can be documented with measured `assura check` runs on the scenario fixture and a hot-path design note, without regenerating the broader LS-Lint performance report unless checked-in performance data or headline claims change.
- Website docs should present the supported workflow as manual/advisory feedback, not automatic agent enforcement.

## Requirements

- Add deterministic valid and invalid real-project fixture states for a modern multi-package repo shape.
- Include a readable scenario `.assura/config.yml` using supported structure-first fields.
- Include intentional invalid drift covering unexpected direct contents, naming drift, and existence/count drift, including an Assura-specific exact `exists:1` rule for project guidance such as `AGENTS.md`.
- Add focused tests proving the valid fixture passes and invalid fixture fails with the intended rule categories.
- Add JSON report and feedback proof that are generated or reproducible from commands.
- Extend the local feedback loop with `assura check --format agent` behavior that an agent can run and reason about.
- Add tests for agent feedback output, including advisory and blocking behavior where relevant.
- Add docs or website content showing install Assura, define policy, run check, inspect failure, receive feedback, fix drift, and rerun.
- Record review evidence under `docs/analysis/` with exact commands, result paths, limitations, and user-facing notes.
- Keep broader roadmap claims out of the user-facing content.

## Acceptance Criteria

- [ ] `docs/goals/assura-real-project-policy-proof.md` is implemented, or any deliberate scope adjustment is explicitly justified in the PR.
- [ ] Valid and invalid fixtures are deterministic and do not depend on an untracked local checkout.
- [ ] The invalid fixture failure includes `unexpected_file` or `unexpected_directory`, `file_naming` or `directory_naming`, and `exists_count`.
- [ ] Scenario docs label LS-Lint-compatible behavior separately from Assura-specific exact `exists:1` behavior.
- [ ] `assura check --format agent` and relevant delivery adapters are documented and tested.
- [ ] Codex feedback output for the invalid fixture references useful local guidance and records same-turn feedback metrics.
- [ ] Performance evidence describes the repeated same-turn check path and does not make unsupported aggregate claims.
- [ ] Minimum validation commands from the goal pass or have a concrete platform blocker recorded.
- [ ] PR description links the goal doc and review record.

## Definition Of Done

- Rust and TypeScript tests pass for touched areas.
- `cargo fmt --all -- --check`, `git diff --check`, `cargo test --all-targets --quiet`, and `cargo run --quiet -- check --format json .` have been run.
- If website content changes, `cd website && pnpm build` has been run.
- If Codex integration changes, `cd integrations/agents/codex && npm install && npm run lint && npm test && npm run build` has been run or the exact blocker is recorded.
- A review agent and independent Codex reviewer have inspected the user-facing changes and claims.
- The PR is open, Gemini review comments have been addressed, and no unresolved review comments remain.

## Out Of Scope

- Daemon/watch architecture.
- Dependency graph validation.
- Hosted telemetry or autonomous agent orchestration.
- Broad performance benchmark suite regeneration unless this task changes performance claims or data.

## Technical Notes

- Goal: `docs/goals/assura-real-project-policy-proof.md`
- Specs: `.trellis/spec/assura/index.md`, `.trellis/spec/assura/structure-enforcement.md`, `.trellis/spec/assura/tooling-stabilization.md`
- Existing hook manager: `src/cli/hooks.rs`, `src/cli/args.rs`, `src/cli/full_entry.rs`
- Existing feedback package: `integrations/agents/codex/src/index.ts`
- Existing website routes: `website/src/content/docs/examples/` and `website/src/content/docs/guides/`
