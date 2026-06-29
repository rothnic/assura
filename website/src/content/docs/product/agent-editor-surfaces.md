---
title: Agent And Editor Surfaces
description: Current agent feedback and planned shared APIs for diagnostics, safe fixes, graph queries, and search.
---

Agent and editor surfaces reuse the same local validation and query core. The
current public path is `assura check`; future daemon, LSP, MCP, and editor
surfaces should call the same contracts rather than invent parallel validation.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| `assura check --format agent` | Supported | Stable JSON shape for wrappers and agent integrations. |
| `assura check --format agent --agent codex` | Supported adapter | Optional Codex `UserPromptSubmit` delivery adapter. |
| `assura content agent-context` | Supported | Generic project-intelligence capability schema for wrappers; not agent-specific. |
| `assura content agent-query` | Supported | Generic request/response envelope over diagnostics, graph/search, semantic candidates, and code-symbol queries. |
| Git hook feedback | Experimental | Hooks can call the CLI and render bounded status or advice. |
| Daemon/editor session | Planned | Future surfaces should reuse prepared checks or a hot local session. |
| LSP and MCP surfaces | Planned | Future surfaces should share diagnostics, safe fixes, graph queries, and search contracts. |
| Automatic agent orchestration | Unsupported | Assura does not currently install or manage complete agent workflows. |

See [Agent Feedback Delivery](/reference/agent-feedback/) for current CLI output
formats and future integration direction.

Generic project-intelligence wrappers should start with:

```bash
assura content agent-context . --format json
```

This reports the shared
`assura.project-intelligence.agent-context.v1` schema and available diagnostics,
safe-fix, graph/search, semantic-candidate, and code-symbol capabilities.

Wrappers that need one stable envelope around specific query results can use:

```bash
assura content agent-query keyword-search . --text portable --format json
assura content agent-query graph-expand . --collection goals --id goal-1 --format json
```

The query envelope uses `assura.project-intelligence.agent-query.v1` and wraps
the same content-query results used by human CLI commands.

Safe-fix wrappers can preview bounded writes before applying them:

```bash
assura fix markdown --rule trailing-spaces --dry-run --format json .
```

The dry-run report uses `assura.safe-fix.markdown.v1` and counts proposed files
and line fixes without writing.
