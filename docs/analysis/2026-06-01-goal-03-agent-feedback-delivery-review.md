---
status: current
---

# Goal 03 Agent Feedback Delivery Review

## Scope

This record covers Agentic Adoption Phase 01 Goal 03:
`docs/goals/assura-goal-03-agent-feedback-delivery-loop.md`.

The proof uses the existing deterministic real-project fixture:

- valid fixture:
  `tests/fixtures/real-project-agentic-feedback/valid`
- invalid fixture:
  `tests/fixtures/real-project-agentic-feedback/invalid`
- scenario config:
  `tests/fixtures/real-project-agentic-feedback/valid/.assura/config.yml`

The public surface remains `assura check --format agent`. Codex delivery remains
only the optional adapter `assura check --format agent --agent codex`.

## Checked Evidence

- Invalid report:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-invalid-report.json`
- Fixed report:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-fixed-report.json`
- Generic agent JSON:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-agent-feedback.json`
- Codex `UserPromptSubmit` JSON:
  `docs/analysis/2026-06-01-goal-03-agent-feedback-codex-hook.json`
- Goal 03 proof summary:
  `docs/analysis/2026-06-01-goal-03-agent-feedback-delivery-proof.json`
- Same-turn observation:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-same-turn-observation.json`

The proof summary records 12 seeded violations, 11 displayed feedback messages,
all Critical/High violations shown, 7 of 8 Medium violations shown before
truncation, Codex `additionalContext` at 5,835 bytes, deterministic Codex output
across two runs, and 12 violations fixed before a new turn.

## Commands

Passed locally:

```bash
cargo fmt --all -- --check
cargo test --test cli_command_surface_tests --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo test --all-targets --quiet
cargo run --quiet -- check --format agent . --warn
cargo run --quiet -- check --format agent --agent codex . --warn
cd integrations/agents/codex && npm run lint && npm test && npm run build && npm pack --dry-run
git diff --check
```

Artifact generation used the same supported surfaces:

```bash
cargo run --quiet -- check tests/fixtures/real-project-agentic-feedback/invalid \
  --config tests/fixtures/real-project-agentic-feedback/invalid/.assura/config.yml \
  --format json --output target/goal-03-agent-feedback-proof/invalid.json
cargo run --quiet -- check tests/fixtures/real-project-agentic-feedback/invalid \
  --config tests/fixtures/real-project-agentic-feedback/invalid/.assura/config.yml \
  --format agent --warn --min-severity medium --max-issues 11
cargo run --quiet -- check tests/fixtures/real-project-agentic-feedback/invalid \
  --config tests/fixtures/real-project-agentic-feedback/invalid/.assura/config.yml \
  --format agent --agent codex --warn --min-severity medium --max-issues 11
```

## Review Notes

- R0: `.trellis/spec/assura/codex-agent-feedback.md` remains current and still
  forbids package feedback CLIs, per-agent CLI entrypoints, and per-agent
  `--format` values.
- R1: Generic agent JSON remains `assura.agent-feedback.v1`; Codex output is a
  `UserPromptSubmit` wrapper around the same feedback.
- R2: Tests cover advisory mode, blocking behavior, severity filtering,
  max-issue filtering, deterministic Codex output, and old `codex-hook`
  rejection.
- R3: Checked artifacts are generated from documented commands and normalized to
  repo-relative paths where needed.
- R4: Website docs state Codex prerequisites and avoid automatic hook mutation,
  daemon, hosted telemetry, and autonomous repair claims.
- R5: Review agent `019e8589-8038-7833-b105-52c2e09caf10` found no blocking
  issues; its low-severity stale phase-label wording finding was addressed
  before PR publication. Gemini's deterministic sorting comment was addressed
  with path, rule, and message tie-breakers before max-issue truncation.
