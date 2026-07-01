# Assura OpenCode Adapter Notes

This directory is a historical TypeScript prototype and is not a beta-supported
OpenCode plugin package.

Current beta work should build OpenCode hooks over the shared Assura CLI
contracts:

```bash
assura agent nudge --event after-tool --changed docs/guide.md --agent opencode .
assura check --format agent --warn .
assura daemon status --format json .
```

Do not add or depend on a separate OpenCode validation engine. The source under
`src/` is retained only as research material until it is replaced by a thin
adapter that shells out to `assura agent nudge`, `assura check --format agent`,
and daemon JSON commands.

Before promoting any OpenCode surface, add tests that prove:

- the adapter does not duplicate Assura validation logic;
- default injected payloads stay bounded and cache-stable;
- daemon-unavailable paths fall back to `assura check --format agent`; and
- support docs classify the adapter separately from the shared nudge payload.
