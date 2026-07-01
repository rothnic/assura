# Assura OpenCode Adapter Notes

This directory is a historical TypeScript prototype and is not a beta-supported
OpenCode plugin package.

Current beta work should build OpenCode hooks over the shared Assura CLI
contracts:

```bash
assura agent nudge --event session-start --agent opencode .
assura agent nudge --event before-tool --changed docs/guide.md --agent opencode .
assura agent nudge --event after-tool --changed docs/guide.md --agent opencode .
assura check --format agent --warn --min-severity medium --max-issues 5 .
assura daemon status --format json .
```

Do not add or depend on a separate OpenCode validation engine. The source under
`src/` is retained only as research material until it is replaced by a thin
adapter that shells out to `assura agent nudge`, `assura check --format agent`,
and daemon JSON commands.

The replacement adapter should:

- call `session-start` once per conversation or project load;
- call `before-tool` only for path-aware write, move, delete, or targeted read
  actions likely to influence edits;
- call `after-tool` with changed paths after file writes or command results
  that mutate the checkout;
- inject nothing when `summary.should_inject` is false;
- fetch detailed feedback with `assura check --format agent --warn` only when a
  nudge requests follow-up; and
- use `assura daemon doctor --format json .` for recovery detail when daemon
  health is unavailable, stale, degraded, or incompatible.

Before promoting any OpenCode surface, add tests that prove:

- the adapter does not duplicate Assura validation logic;
- default injected payloads stay bounded and cache-stable;
- daemon-unavailable paths fall back to `assura check --format agent`; and
- support docs classify the adapter separately from the shared nudge payload.
