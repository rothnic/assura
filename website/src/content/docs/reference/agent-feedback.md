---
title: Agent Feedback Delivery
description: How Assura checks become Git hook output, Codex prompt hook feedback, or future low-latency feedback.
---

# Agent Feedback Delivery

Assura has one check pipeline and several output shapes. Agent integrations use
the same CLI options as humans and hooks; there is no separate agent mode.

## Output Model

| Need | Option | Who uses it |
| --- | --- | --- |
| Raw facts | `--format json` or `--format yaml` | CI, wrappers, reports |
| Repair guidance | `--format advice` | Humans and agents fixing drift |
| Compact status | `--format status` | Git hooks, tool results, final status lines |
| Structured agent feedback | `--format agent` | Agents and wrappers that want stable JSON |
| Codex prompt hook JSON | `--format agent --agent codex` | Optional Codex `UserPromptSubmit` hooks |
| Display limits | `--min-severity` and `--max-issues` | Any noisy workflow |
| Advisory exit | `--warn` | Workflows that should report without blocking |

## Surface Map

| Surface | Who invokes it | What it runs | How feedback appears | Blocking owner |
| --- | --- | --- | --- | --- |
| Manual CLI proof | Developer or agent command | `assura check --format advice` or `--format status` | Guided advice or one-line status | The caller |
| Git hooks | Git | Installed hook scripts | Git hook stdout/stderr | The hook script |
| Agent feedback package | Wrapper code that cannot call the Rust CLI directly | Report parsing and feedback rendering | Library return value or JSON | The wrapper |
| Codex prompt hook | Optional Codex `UserPromptSubmit` command | `assura check --format agent --agent codex` | Codex `hookSpecificOutput.additionalContext` | Hook configuration |
| Future tool/editor hook | Future agent integration | Scoped check plus feedback rendering | Tool result, next agent message, or status line | Hook configuration |
| Project-intelligence session | Local agent/editor wrapper | `assura content session` | JSON-line context/query responses | The wrapper |
| Warm checker session | Future structure-check integration | Prepared structure checker or hot daemon | Low-latency check result for changed paths | Integration policy |

The primary DX is `assura check`. The package is a lower-level bridge for
wrappers that already have an Assura JSON report or cannot shell out to the Rust
CLI directly.

## Current Check Output

Use guided output when a human or agent should fix the result:

```bash
assura check --format advice .
```

Use status output when a hook or tool result needs one concise line:

```bash
assura check --format status .
```

Display controls limit what gets shown without changing what gets checked:

```bash
assura check --format advice . --min-severity medium --max-issues 3
```

Use structured agent output when a wrapper wants stable JSON:

```bash
assura check --format agent . --warn --min-severity medium --max-issues 5
```

Use the Codex delivery adapter only when a Codex `UserPromptSubmit` hook should
inject bounded Assura context:

```bash
assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5
```

| Option | Effect |
| --- | --- |
| `--format advice` | Emits human-readable guidance for fixing violations. |
| `--format status` | Emits one concise line suitable for hooks and tool output. |
| `--format agent` | Emits stable `assura.agent-feedback.v1` JSON. |
| `--agent codex` | Wraps `--format agent` output for Codex `UserPromptSubmit` delivery. |
| `--format json` | Emits the raw structure report. |
| `--format yaml` | Emits the raw structure report as YAML. |
| `--min-severity` | Hides lower-severity feedback items from display. |
| `--max-issues` | Caps displayed feedback items. |
| `--warn` | Reports failures but exits successfully. |

## When To Check

| Moment | Recommended check | Why |
| --- | --- | --- |
| Before a commit | Git pre-commit hook | Catch drift before local history changes. |
| Before a push | Git pre-push hook | Catch drift before PR/CI feedback. |
| Before Codex processes a user prompt | Optional Codex `UserPromptSubmit` hook | Inject bounded Assura context into Codex when the user has opted in. |
| After an agent edits files | Future tool hook or editor integration | Give the agent immediate repair guidance after changed files are known. |
| Before a user-facing agent response | Reuse the latest report or run a final scoped check | Avoid telling the user work is done while structure drift remains. |
| After config or checkout changes | Full project check | Rebuild assumptions after policy or tree shape changes. |

## Warm Sessions And Index Reuse

The current public `assura check` and Git hook paths do not keep a daemon. They
run when a caller invokes Assura or Git fires an installed hook.

For repeated project-intelligence queries, use a local JSON-line session:

```bash
assura content session .
```

Send one request per line:

```json
{"request_id":"ctx-1","type":"context-pack","collection":"assura_goals","id":"goal-assura-project-intelligence-usability-program","text":"Project Intelligence Usability","limit":5}
```

Each response uses `assura.project-intelligence.session.response.v1` and reports
whether the loaded context was `initial_load`, `reused`, or `reloaded`. The
session checks a conservative project fingerprint before every request, so it
does not rely on watcher delivery for correctness. It is still local and
disposable: stop the process to discard state.

Supported request `type` values are `agent-context`, `collections`,
`context-pack`, `diagnostics`, `expand`, `missing-relations`, `safe-fixes`, and
`search`. Failed requests return the same response envelope with `ok: false`,
`response: null`, and an `error.code` such as `invalid_request`,
`request_failed`, or `reload_failed`.

Safe-fix preview responses include both the project-intelligence fact `id` and
the CLI audit `audit_id`. Wrappers should match `audit_id` to
`assura fix markdown --dry-run --format json` `fixes[].id`, then require an
explicit `assura fix markdown --apply --format json` step before writing.

Assura does have lower-level support intended for future editor and agent
integrations:

- `PreparedStructureCheck` keeps parsed and compiled policy state for repeated
  checks while configuration bytes stay unchanged.
- Changed-path checks can validate a file or directory and its direct aggregate
  scopes without proving whole-project success.
- Performance evidence includes hot daemon and warm editor-session rows to
  measure repeated checks without paying full process startup every time.

A future editor or post-tool agent integration should use that shape:

```text
startup or config change -> load policy and build prepared checker
agent edits files        -> check changed paths and update latest report
before user response     -> show unresolved configured-severity feedback
checkout or policy edit  -> refresh with a full project check
```

That avoids repeating policy parsing and broad validation work on every agent
step, while still giving the agent fresh feedback after edits.

## Agent Integration Matrix

| Integration | Supported now | Expected delivery |
| --- | --- | --- |
| Codex package library | Lower-level only | Wrapper code can use library helpers when it already has JSON. |
| Codex `UserPromptSubmit` hook | Yes | A hook runs `assura check --format agent --agent codex` before Codex processes a prompt. |
| Codex post-tool/editor hook | Not yet | A future hook should append a status line or bounded feedback after relevant tool calls. |
| Other agents with shell access | Partially | They can call `assura check --format advice`, `--format status`, or `--format agent` manually or through a wrapper. |
| Project-intelligence session | Yes | Local wrappers can keep `assura content session` open for repeated context/query requests. |
| Editor/daemon structure-check integration | Not yet as public UX | Should reuse prepared checks or hot daemon state for repeated changed-path feedback. |
