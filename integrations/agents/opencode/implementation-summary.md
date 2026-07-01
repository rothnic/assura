# OpenCode Prototype Status

This package is historical prototype material. It is not a completed or
supported OpenCode integration for the beta program.

The prototype includes a standalone TypeScript validation engine and hook
simulation code. That architecture is superseded for beta by the shared Assura
CLI contracts:

```bash
assura agent nudge --agent opencode .
assura check --format agent --warn .
assura daemon status --format json .
```

Future OpenCode work should replace this prototype with a thin hook/plugin
adapter over those commands. It must not preserve a private validation engine,
private rule inventory, or independent blocking policy that bypasses Assura
check and daemon contracts.
