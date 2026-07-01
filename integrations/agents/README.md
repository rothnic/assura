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

## Event Recipes

Adapters should invoke Assura only at event points where repository feedback is
likely to change the next action:

| Event | Recipe | Default injection policy |
| --- | --- | --- |
| Session start | `assura agent nudge --event session-start --agent <agent> .` | Read daemon health and inject only when Assura reports a recovery nudge. |
| Before tool call | `assura agent nudge --event before-tool --agent <agent> --changed <path> .` | Use for edit, move, delete, or path-targeted read operations likely to influence edits. |
| After tool call | `assura agent nudge --event after-tool --agent <agent> --changed <path> .` | Inject bounded findings only when changed paths create relevant Assura feedback. |
| Deep diagnostics | `assura check --format agent --warn --min-severity medium --max-issues 5 .` | Fetch on demand instead of injecting large diagnostics into every event. |
| Daemon recovery | `assura daemon status --format json .` then `assura daemon doctor --format json .` | Prefer machine-readable status and exact recovery commands. |

Codex is the only current host with a supported delivery wrapper for native
prompt-hook JSON:

```bash
assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5
```

OpenCode, Claude, and Pi wrappers should consume the generic
`assura.agent-nudge.v1` JSON and render their own status line or tool-result
annotation. They should not add `assura check --agent <host>` variants unless a
future Assura spec promotes that host-specific delivery format.

## Target Agent Notes

| Agent | Local integration path | Beta smoke |
| --- | --- | --- |
| Codex | Optional `UserPromptSubmit` hook for `assura check --format agent --agent codex`, plus pre/post-tool wrappers over `assura agent nudge --agent codex`. | CLI tests verify Codex hook JSON and shared nudge output. |
| OpenCode | Thin plugin or hook that shells out to `assura agent nudge --agent opencode` and `assura check --format agent --warn` for detail. | Shared nudge CLI tests accept the label; the existing TypeScript package remains historical until replaced. |
| Claude | Local command hook or wrapper that calls `assura agent nudge --agent claude` around path-aware tool events. | Shared nudge CLI tests accept the label and prove cache-stable session-start output. |
| Pi | Local extension or hook wrapper that calls `assura agent nudge --agent pi` and keeps performance-gate nudges visible for structure hot paths. | Shared nudge CLI tests accept the label and prove performance-gate path nudges. |

All adapters should pass changed paths relative to the project root, keep
`--max-issues` small by default, and defer full reports to explicit follow-up
commands.
