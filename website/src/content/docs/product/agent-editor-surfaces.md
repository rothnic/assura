---
title: Agent And Editor Surfaces
description: Current agent feedback and planned shared APIs for diagnostics, safe fixes, graph queries, and search.
---

Agent and editor surfaces reuse the same local validation and query core. The
current public paths are `assura check` for structure feedback and
`assura agent` for local project-intelligence handoffs and
`assura editor session` for local editor wrappers. Optional protocol adapters
should call the same contracts rather than invent parallel validation.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| `assura check --format agent` | Supported | Stable JSON shape for wrappers and agent integrations. |
| `assura check --format agent --agent codex` | Supported adapter | Optional Codex `UserPromptSubmit` delivery adapter. |
| `assura agent` | Supported | Local coding-agent command group with JSON defaults for diagnostics, context packs, graph/search queries, relation checks, and safe-fix previews. |
| `assura editor session` | Supported | Local JSON-line editor protocol with LSP-shaped diagnostics, context, code-action preview methods, and conservative reload metadata. |
| `assura content agent-context` | Supported | Generic project-intelligence capability schema for wrappers; not agent-specific. |
| `assura content agent-query` | Supported | Generic request/response envelope over diagnostics, graph/search, semantic candidates, and code-symbol queries. |
| Git hook feedback | Experimental | Hooks can call the CLI and render bounded status or advice. |
| Full LSP server and editor packages | Roadmap only | `Content-Length` framed language-server transport and marketplace plugins are not part of the current supported surface. |
| MCP adapter | Roadmap only | Optional future stdio adapter over the same local contracts; no remote access is required for current agent usability. |
| Automatic agent orchestration | Unsupported | Assura does not currently install or manage complete agent workflows. |

See [Agent Feedback Delivery](/reference/agent-feedback/) for current CLI output
formats and future integration direction.

Local coding agents should start with:

```bash
assura agent context .
```

This reports the shared
`assura.project-intelligence.agent-context.v1` schema and available diagnostics,
safe-fix, graph/search, semantic-candidate, and code-symbol capabilities.

Agents that need one stable envelope around diagnostics or safe-fix previews can
use:

```bash
assura agent diagnostics .
assura agent safe-fixes .
```

The query envelope uses `assura.project-intelligence.agent-query.v1` and wraps
the same content-query results used by human CLI commands. For direct project
knowledge inspection, use the agent commands with JSON defaults:

```bash
assura agent context-pack . --collection goals --id goal-1 --text portable
assura agent search "portable" .
assura agent expand goals goal-1 .
assura agent missing-relations .
```

Safe-fix wrappers can preview bounded writes before applying them:

```bash
assura fix markdown --rule trailing-spaces --dry-run --format json .
```

The dry-run report uses `assura.safe-fix.markdown.v1` and counts proposed files
and line fixes without writing.

Editor wrappers can keep one local process open and send LSP-shaped JSON-line
requests:

```bash
assura editor session .
```

```json
{"request_id":"diag-1","method":"textDocument/diagnostics","params":{"textDocument":{"uri":"docs/goals/goal_portable_structure.md"}}}
{"request_id":"ctx-1","method":"textDocument/context","params":{"uri":"docs/goals/goal_portable_structure.md","text":"portable","limit":5}}
{"request_id":"fix-1","method":"textDocument/codeAction","params":{"uri":"docs/goals/goal_portable_structure.md"}}
```

Responses use `assura.project-intelligence.editor.response.v1`. Code actions
are previews only; applying a repair still requires an explicit
`assura fix markdown --apply --format json` command.

For an end-to-end example that starts with repository files and ends with an
agent diagnostic and safe-fix preview, see
[Project Intelligence Demo](/examples/project-intelligence-demo/).
