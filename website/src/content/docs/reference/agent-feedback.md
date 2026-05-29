---
title: Agent Feedback Delivery
description: How Assura checks become Git hook output, agent nudges, or future low-latency feedback.
---

# Agent Feedback Delivery

Assura has separate feedback surfaces. They share the same structure policy, but
they run at different times and have different delivery guarantees.

## Surface Map

| Surface | Who invokes it | What it runs | How feedback appears | Blocking owner |
| --- | --- | --- | --- | --- |
| Manual CLI proof | Developer or agent command | `assura check`, then `assura-codex-nudge` | Terminal text, status line, or JSON | The caller |
| Git hooks | Git | Installed hook scripts | Git hook stdout/stderr | The hook script |
| Codex nudge package | Wrapper code or CLI | Report parsing and nudge rendering | Library return value or CLI output | The wrapper |
| Native agent hook | Future agent integration | Scoped check plus nudge rendering | Tool result, next agent message, or status line | Hook configuration |
| Warm checker session | Future editor/agent integration | Prepared structure checker or hot daemon | Low-latency check result for changed paths | Integration policy |

`assura-codex-nudge` is a bridge, not an always-on integration by itself. If an
agent calls it directly, that is a manual proof path. If an agent wrapper calls
it after tool use, that wrapper is the integration.

## Current Nudge CLI

Use the CLI when an Assura JSON report already exists:

```bash
assura check --format json . > assura-report.json
assura-codex-nudge --report assura-report.json --format status
```

Or let the nudge CLI run Assura directly:

```bash
assura-codex-nudge --path . --format text
```

Configuration controls what gets shown:

```bash
assura-codex-nudge \
  --report assura-report.json \
  --format status \
  --minimum-severity high \
  --max-messages 3 \
  --blocking
```

| Option | Effect |
| --- | --- |
| `--format status` | Emits one concise line suitable for tool output. |
| `--format text` | Emits bounded human-readable guidance. |
| `--format json` | Emits structured nudge data for wrappers. |
| `--minimum-severity` | Suppresses lower-severity violations from nudge messages. |
| `--max-messages` | Caps the number of displayed nudge messages. |
| `--blocking` | Marks the nudge as blocking metadata. The surrounding wrapper still decides whether to stop. |

## When To Check

| Moment | Recommended check | Why |
| --- | --- | --- |
| Before a commit | Git pre-commit hook | Catch drift before local history changes. |
| Before a push | Git pre-push hook | Catch drift before PR/CI feedback. |
| After an agent edits files | Scoped Assura check, then nudge rendering | Give the agent immediate repair guidance. |
| Before a user-facing agent response | Reuse the latest report or run a final scoped check | Avoid telling the user work is done while structure drift remains. |
| After config or checkout changes | Full project check | Rebuild assumptions after policy or tree shape changes. |

## Warm Sessions And Index Reuse

The current Codex nudge MVP does not keep a daemon or update an agent-facing
index. It runs when a caller invokes the library or CLI.

Assura does have lower-level support intended for future editor and agent
integrations:

- `PreparedStructureCheck` keeps parsed and compiled policy state for repeated
  checks while configuration bytes stay unchanged.
- Changed-path checks can validate a file or directory and its direct aggregate
  scopes without proving whole-project success.
- Performance evidence includes hot daemon and warm editor-session rows to
  measure repeated checks without paying full process startup every time.

A future native agent integration should use that shape:

```text
startup or config change -> load policy and build prepared checker
agent edits files        -> check changed paths and update latest report
before user response     -> show unresolved configured-severity nudges
checkout or policy edit  -> refresh with a full project check
```

That avoids repeating policy parsing and broad validation work on every agent
step, while still giving the agent fresh feedback after edits.

## Agent Integration Matrix

| Integration | Supported now | Expected delivery |
| --- | --- | --- |
| Codex package/CLI | Yes | A wrapper can call the package or CLI and attach status/text/JSON output. |
| Codex native hook | Not yet | A hook should append a status line or bounded nudge after relevant tool calls. |
| Other agents with shell access | Partially | They can call `assura check` and `assura-codex-nudge` manually or through a wrapper. |
| Editor/daemon integration | Not yet as public UX | Should reuse prepared checks or hot daemon state for repeated changed-path feedback. |

