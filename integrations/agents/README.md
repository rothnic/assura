# Assura Agent Integrations

This directory contains agent-adapter code and historical prototypes for
downstream users' agent environments. Beta-supported behavior must be routed
through the shared Assura CLI contracts before an adapter is promoted.

Project-local agent configuration, such as this repository's `.codex/` Trellis
support, remains in the platform-specific configuration directories at the repo
root. Installable integration packages live here so OpenCode, Codex, and future
agent adapters are developed under one source tree.

## Packages

- `opencode/`: Historical OpenCode prototype notes; replace with a thin
  `assura agent nudge --agent opencode` adapter before any beta claim.
- `codex/`: Codex integration package with advisory nudge and optional native
  `UserPromptSubmit` hook feedback commands.

## Shared Nudge Contract

All agent adapters should prefer the repo-owned CLI contracts instead of
reimplementing validation:

```bash
assura agent nudge --event after-tool --changed docs/guide.md --agent codex .
assura check --format agent --warn .
assura daemon status --format json .
```

`assura agent nudge` emits `assura.agent-nudge.v1`, a bounded payload with
daemon health, changed-path findings, affected-reference context, and one
suggested command for deeper diagnostics. The `--agent` value labels the host
wrapper (`codex`, `opencode`, `claude`, or `pi`); it must not create a private
validation path.
