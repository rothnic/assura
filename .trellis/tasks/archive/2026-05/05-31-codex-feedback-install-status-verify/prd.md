# Codex Feedback Install Status Verify

## Goal

Make the optional Codex agent-feedback hook user-verifiable through the stable
`assura check` CLI surface. A reviewer should be able to run `assura check`
with agent-oriented formats and filters, verify Codex hook JSON, and understand
which Codex enablement steps remain manual.

## Requirements

- Keep the stable user-facing API under `assura check`; do not add one feedback
  management entrypoint per agent.
- Add `assura check --format agent` for stable structured feedback and
  `--agent codex` as the optional Codex `UserPromptSubmit` delivery adapter.
- Reuse check-level options for feedback filtering and behavior:
  `--min-severity`, `--max-issues`, and `--warn`.
- Verify passing report exit `0`, failing advisory exit `0` with `--warn`, and
  failing blocking exit `1` without `--warn`.
- Document how to append the `assura check --format agent --agent codex` command to
  existing Codex `UserPromptSubmit` hooks without overwriting unrelated hooks.
- Document Codex hook enablement and approval caveats that cannot be automated.
- Update `.trellis/spec/assura/codex-agent-feedback.md` if command signatures
  or hook contracts change.
- Preserve the 2026-05-31 direction correction in durable docs: the stable API
  is `assura check --format agent` plus options such as `--agent codex`; older
  `assura-codex-feedback`, `assura check --format codex-hook`, or per-agent
  CLI/format plans are superseded.

## Non-Goals

- Publish the npm package.
- Add new per-agent feedback management binaries.
- Auto-enable user-level Codex `features.hooks` or run `/hooks` approval.
- Replace repo-local `AGENTS.md` or `.agents/skills/` guidance.
- Add daemon/editor-session reuse or autonomous repair behavior.

## Acceptance Criteria

- `assura check --format agent` emits stable `assura.agent-feedback.v1` JSON.
- `assura check --format agent --agent codex` emits valid Codex hook JSON with
  `hookSpecificOutput.hookEventName = "UserPromptSubmit"`.
- `--warn`, `--min-severity`, and `--max-issues` affect Codex hook feedback the
  same way they affect `advice` and `status` feedback.
- `assura check --format codex-hook` is rejected so agents do not accrete
  per-agent format values.
- Tests cover advisory and blocking Codex hook exit behavior.
- Docs show an append-only `.codex/hooks.json` example that preserves unrelated
  hooks.
- Normal `assura check` and default developer workflows remain unaffected.
- Repo docs and PR text identify the superseded per-agent command/format shapes
  so future sessions do not restart work in that direction.

## Validation

Run and pass, or document exact blockers:

```bash
cd integrations/agents/codex && npm run lint && npm test && npm run build && npm pack --dry-run
node --run verify:fast
node --run verify:docs
cargo run --quiet -- check --format json .
```

If Rust CLI surfaces are touched:

```bash
cargo test --all-targets --quiet
cargo clippy --all-targets --all-features -- -D warnings
```
