---
title: API Reference
description: Current supported integration surface for Assura
template: doc
sidebar:
  order: 2
---

The supported API is the local command-line interface and its report formats.
Rust library APIs, TypeScript plugin APIs, agent profiles, and custom runtime
constraints are not stable public surfaces in this release. See
[Extension API Boundaries](/reference/extension-api-boundaries/) for the
canonical distinction between first-party `extensions.*` config policies,
supported local JSON contracts, internal Rust modules, and deferred public
plugin APIs.

> **Current scope**
>
> Build automation against `assura check`, `assura status`, `assura agent`,
> `assura editor`, `assura content`, and the JSON/YAML report fields
> documented here.

## Command Surface

| Command | Purpose |
| --- | --- |
| `assura check [path]` | Validate a project or subpath |
| `assura status [path]` | Print discovered config and rule summary |
| `assura init [path]` | Create a starter `.assura/config.yml` |
| `assura migrate [.ls-lint.yml ...]` | Convert LS-Lint 2.3 rule config |
| `assura agent ...` | Run local project-intelligence commands for coding agents |
| `assura editor ...` | Run local project-intelligence commands for editor integrations |
| `assura content ...` | Query project-intelligence facts and context |
| `assura info [path]` | Print text configuration details |
| `assura watch [path]` | Run one check as a current watch wrapper |

Supported check formats are `text`, `json`, `yaml`, `advice`, and `status`.
Supported status formats are `text`, `json`, and `yaml`.

## Init Options

| Option | Purpose |
| --- | --- |
| `--project-intelligence` | Create starter project-intelligence schema, collections, modeled records, and a broken-state example |
| `--force` | Overwrite an existing starter config and starter files |
| `--no-git-hooks` | Skip the optional hook setup message |

## Check Options

| Option | Purpose |
| --- | --- |
| `--warn` | Print violations but exit successfully, useful for advisory agent feedback and gradual adoption |
| `--fail-fast` | Stop after the first violation |
| `--no-parallel` | Run validation without parallel traversal |
| `--ls-lint-target-semantics` | Match LS-Lint path-argument behavior by checking only the explicit target path |
| `--min-severity low|medium|high|critical` | Hide lower-severity feedback items without changing what is checked |
| `--max-issues <count>` | Cap displayed feedback items without changing what is checked |
| `--agent generic|codex` | Select a delivery adapter for `--format agent`; Codex wraps feedback for `UserPromptSubmit` |

## Agent Surface

`assura agent` is the supported local project-intelligence command group for
coding agents. It defaults to JSON output and delegates to the same contracts as
the lower-level content-query commands.

| Command | Purpose |
| --- | --- |
| `assura agent context` | Summarize project-intelligence capabilities |
| `assura agent diagnostics` | Return diagnostics through the shared agent envelope |
| `assura agent context-pack` | Build one bounded project-intelligence handoff packet |
| `assura agent search` | Search modeled content facts |
| `assura agent show` | Show one modeled content instance |
| `assura agent expand` | Expand graph context around one modeled object |
| `assura agent missing-relations` | Report unresolved modeled relations |
| `assura agent safe-fixes` | Preview safe fixes through the shared agent envelope |
| `assura agent nudge` | Emit bounded event-aware nudge JSON for local host wrappers |
| `assura agent integration install|update|remove|status|doctor` | Manage reviewable local Codex, OpenCode, Claude, and Pi integration bundles |
| `assura agent session` | Run a persistent JSON-line local query session |

Examples:

```bash
assura agent context .
assura agent diagnostics tests/fixtures/content_runtime/missing_reference
assura agent context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program --text "Project Intelligence Usability" --limit 5
assura agent safe-fixes tests/fixtures/project_intelligence_real_repo/beacon_crm/invalid
assura agent integration install codex .
assura agent integration doctor codex .
```

MCP is not required for local agent usage. If an MCP adapter is added later, it
should wrap these same CLI/library contracts.

## Extension/API Boundary

Current `extensions.*` configuration families are first-party policies executed
by `assura check`. They are not a public third-party plugin API and they do not
provide remote plugin loading, shell-executed validation plugins, plugin
marketplaces, TypeScript plugin APIs, or semver-stable Rust library APIs.

Automation should integrate through the local CLI, daemon, content, agent, and
editor contracts documented on this page. Public plugin API work is roadmap-only
until a separate goal proves sandboxing, versioning, distribution, security,
diagnostics, and performance gates.

## Editor Surface

`assura editor session [path]` is the supported local editor protocol. It reads
one JSON request per stdin line and emits one
`assura.project-intelligence.editor.response.v1` JSON response per stdout line.
The protocol is LSP-shaped for wrapper simplicity, but it is not a full
`Content-Length` framed language server and does not include editor marketplace
packaging.

| Method | Purpose |
| --- | --- |
| `textDocument/diagnostics` | Return LSP-shaped diagnostics for a file URI or path |
| `textDocument/context` | Return a bounded project-intelligence context pack for the file or explicit object |
| `textDocument/codeAction` | Return safe-fix preview code actions without writing files |

Example requests:

```json
{"request_id":"diag-1","method":"textDocument/diagnostics","params":{"textDocument":{"uri":"docs/goals/goal_portable_structure.md"}}}
{"request_id":"ctx-1","method":"textDocument/context","params":{"uri":"docs/goals/goal_portable_structure.md","text":"portable","limit":5}}
{"request_id":"fix-1","method":"textDocument/codeAction","params":{"uri":"docs/goals/goal_portable_structure.md"}}
```

Every response has this envelope:

```json
{
  "schema": "assura.project-intelligence.editor.response.v1",
  "sequence": 1,
  "request_id": "diag-1",
  "method": "textDocument/diagnostics",
  "reload": {
    "state": "initial_load",
    "reason": "session context loaded",
    "project_root": ".",
    "config_path": "./.assura/config.yml"
  },
  "ok": true,
  "result": {},
  "error": null
}
```

`reload.state` is `initial_load`, `reused`, `reloaded`, `reload_failed`, or
`not_checked`. Code-action responses include an `apply_command` string, but
wrappers must still require explicit user approval before running
`assura fix markdown --apply --format json`.

## Content Query Surface

| Command | Purpose |
| --- | --- |
| `assura content agent-context` | Summarize project-intelligence capabilities |
| `assura content agent-query` | Wrap one query in the shared agent envelope |
| `assura content context-pack` | Build one bounded project-intelligence handoff packet |
| `assura content session` | Run a persistent JSON-line local query session |
| `assura content search` | Search modeled content facts with lexical scores |
| `assura content expand` | Expand graph context around one modeled object |
| `assura content missing-relations` | Report unresolved modeled relations |
| `assura content references` | Report inbound repository references by target path or outbound references by source path |

Object-mode context-pack JSON includes `repository_references.path`,
`repository_references.inbound`, and `repository_references.outbound` for the
modeled object's repository path. Those arrays use the same edge shape as
`assura content references`, including source position, target path, anchor or
line range, existence, rule, kind, and confidence.

`assura content session [path]` reads one JSON request per stdin line and emits
one `assura.project-intelligence.session.response.v1` JSON response per stdout
line. Use it when an agent, editor wrapper, or local integration needs repeated
diagnostics, context-pack, graph, search, relation, or safe-fix preview queries
without restarting the CLI process. The session reloads conservatively when the
project fingerprint changes; `assura watch` remains experimental.

Safe-fix previews returned by `safe-fixes` include an `audit_id` that matches
`assura fix markdown --dry-run --format json` `fixes[].id`. Apply still happens
through `assura fix markdown --apply --format json`, and integrations must not
write repairs implicitly.

Session request fields:

| Field | Required | Purpose |
| --- | --- | --- |
| `type` | Yes | One of `agent-context`, `collections`, `context-pack`, `diagnostics`, `expand`, `missing-relations`, `safe-fixes`, or `search` |
| `request_id` | No | Caller-provided correlation string returned unchanged |
| `collection` | For `context-pack` object mode and `expand` | Modeled collection name |
| `id` | For `context-pack` object mode and `expand` | Modeled object id inside the collection |
| `text` | For `context-pack` search context and `search` | Keyword query text |
| `limit` | No | Bound for context-pack and graph expansion results; defaults to `20` |

Every response has this envelope:

```json
{
  "schema": "assura.project-intelligence.session.response.v1",
  "sequence": 1,
  "request_id": "ctx-1",
  "request_type": "context-pack",
  "reload": {
    "state": "initial_load",
    "reason": "session context loaded",
    "project_root": ".",
    "config_path": "./.assura/config.yml"
  },
  "ok": true,
  "response": {},
  "error": null
}
```

`reload.state` is `initial_load`, `reused`, `reloaded`, `reload_failed`, or
`not_checked`. Failed requests keep the same envelope with `ok: false`,
`response: null`, and an error object:

```json
{
  "code": "request_failed",
  "message": "expand request requires `id`"
}
```

`invalid_request` means the JSON line did not parse, `request_failed` means the
request parsed but failed validation or execution, and `reload_failed` means
the project changed but the rebuilt context could not be loaded.

## Check JSON Shape

```json
{
  "success": true,
  "project_root": "/work/example",
  "config_path": "/work/example/.assura/config.yml",
  "checked_path": "/work/example",
  "files_checked": 12,
  "dirs_checked": 4,
  "violations": []
}
```

Violation entries use this shape:

```json
{
  "path": "/work/example/src/BadName.rs",
  "rule": "file_naming",
  "message": "File name 'BadName' does not match kebab-case",
  "severity": "medium",
  "severity_label": "Medium",
  "blocking": true,
  "corrective_context": "Rename the file to match the effective naming rule, or update files.naming/naming_patterns when the policy is stale."
}
```

`low` findings are advisory and use `"blocking": false`. `medium`, `high`, and
`critical` findings are blocking unless the caller explicitly passes `--warn`.

## Status JSON Shape

```json
{
  "project_root": "/work/example",
  "config_path": "/work/example/.assura/config.yml",
  "configured_directories": 2,
  "configured_file_rules": 3,
  "configured_markdown_rules": 0,
  "exclusions": ["target/**", "node_modules/**"]
}
```

## Recommended Automation

```bash
assura check --format json . > assura-report.json
```

Fail the job when the command exits nonzero. Parse `violations` only when you
need a custom summary or uploaded artifact.

For guided local output, use:

```bash
assura check --format advice .
assura check --format status .
assura check --format agent .
assura check --format agent --agent codex . --warn
```

## Future APIs

The roadmap includes native agent hooks, hot editor sessions, quality
measurement, and extension points. Those capabilities should be documented as
future work until the repo contains a tested public API for them.
