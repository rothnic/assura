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
| Display limits | `--min-severity` and `--max-issues` | Any noisy workflow |
| Advisory exit | `--warn` | Workflows that should report without blocking |

## Surface Map

| Surface | Who invokes it | What it runs | How feedback appears | Blocking owner |
| --- | --- | --- | --- | --- |
| Manual CLI proof | Developer or agent command | `assura check --format advice` or `--format status` | Guided advice or one-line status | The caller |
| Git hooks | Git | Installed hook scripts | Git hook stdout/stderr | The hook script |
| Agent feedback package | Wrapper code that cannot call the Rust CLI directly | Report parsing and feedback rendering | Library return value or JSON | The wrapper |
| Codex prompt hook | Optional Codex `UserPromptSubmit` command | Reused JSON report or `assura check --format json <path>` | Codex `hookSpecificOutput.additionalContext` | Hook configuration |
| Future tool/editor hook | Future agent integration | Scoped check plus feedback rendering | Tool result, next agent message, or status line | Hook configuration |
| Warm checker session | Future editor/agent integration | Prepared structure checker or hot daemon | Low-latency check result for changed paths | Integration policy |

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

| Option | Effect |
| --- | --- |
| `--format advice` | Emits human-readable guidance for fixing violations. |
| `--format status` | Emits one concise line suitable for hooks and tool output. |
| `--format json` | Emits the raw structure report. |
| `--format yaml` | Emits the raw structure report as YAML. |
| `--min-severity` | Hides lower-severity advice and status items from display. |
| `--max-issues` | Caps displayed advice and status items. |
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

The current public `assura check` and Git hook paths do not keep a daemon or
update an agent-facing index. They run when a caller invokes Assura or Git fires
an installed hook.

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
| Codex package/CLI | Yes | A wrapper can call the package or CLI and attach status/text/JSON output. |
| Codex `UserPromptSubmit` hook | Yes, optional source-checkout proof | A hook emits Codex JSON with bounded `additionalContext` before Codex processes a prompt. |
| Codex post-tool/editor hook | Not yet | A future hook should append a status line or bounded feedback after relevant tool calls. |
| Other agents with shell access | Partially | They can call `assura check --format advice` or `--format status` manually or through a wrapper. |
| Editor/daemon integration | Not yet as public UX | Should reuse prepared checks or hot daemon state for repeated changed-path feedback. |
