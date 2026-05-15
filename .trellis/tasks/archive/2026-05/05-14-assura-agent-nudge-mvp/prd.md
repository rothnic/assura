# Assura agent nudge MVP

## Goal

Implement the first Codex/agent nudge MVP for Assura: a small, tested Codex
integration package surface that turns `assura check --format json` reports
into actionable nudge messages and measurable workflow metrics.

## Requirements

- Create and execute `docs/goals/assura-agent-nudge-mvp.md`.
- Keep the implementation in `integrations/agents/codex`.
- Provide a typed library API for parsing Assura JSON reports, creating nudge
  messages, running `assura check`, and comparing evaluation runs.
- Provide a CLI entrypoint that can read an existing report file or run Assura.
- Preserve validation failure exit behavior.
- Include tests without adding unnecessary runtime dependencies.
- Update Codex integration docs and website agent-nudge docs.
- Do not implement automatic Codex hook installation in this task.
- Push the branch and create a draft PR after validation.

## Acceptance Criteria

- `npm run lint`, `npm test`, and `npm run build` pass in
  `integrations/agents/codex`.
- `cargo fmt --all -- --check`, `cargo test --all-targets --quiet`, and
  `cargo run --quiet -- check --format json .` pass at repo root.
- `pnpm build` passes in `website`.
- Docs clearly distinguish the MVP nudge package from future hook
  installation and complete autonomous agent behavior.
- Goal completion audit maps every explicit requirement to evidence.

## Out of Scope

- Automatic Codex hook installation.
- Full agent orchestration.
- Changing the `StructureCheckReport` schema.
- Adding hosted telemetry or external services.
