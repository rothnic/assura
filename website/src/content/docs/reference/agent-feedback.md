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
| Event-aware nudge JSON | `assura agent nudge` | Codex, OpenCode, Claude, and Pi wrappers around relevant events |
| Display limits | `--min-severity` and `--max-issues` | Any noisy workflow |
| Advisory exit | `--warn` | Workflows that should report without blocking |

## Surface Map

| Surface | Who invokes it | What it runs | How feedback appears | Blocking owner |
| --- | --- | --- | --- | --- |
| Manual CLI proof | Developer or agent command | `assura check --format advice` or `--format status` | Guided advice or one-line status | The caller |
| Git hooks | Git | Installed hook scripts | Git hook stdout/stderr | The hook script |
| Agent feedback package | Wrapper code that cannot call the Rust CLI directly | Report parsing and feedback rendering | Library return value or JSON | The wrapper |
| Codex prompt hook | Optional Codex `UserPromptSubmit` command | `assura check --format agent --agent codex` | Codex `hookSpecificOutput.additionalContext` | Hook configuration |
| Event-aware nudge | Local agent hook or wrapper | `assura agent nudge --event ... --agent <agent>` | Bounded `assura.agent-nudge.v1` JSON | The wrapper |
| Agent integration lifecycle | Maintainer or setup script | `assura agent integration ...` | Reviewable `.assura/integrations/<agent>/` bundle | The maintainer |
| Project-intelligence agent CLI | Local coding agent with shell access | `assura agent ...` | JSON diagnostics, context packs, graph/search, relation checks, and safe-fix previews | The caller |
| Project-intelligence editor session | Local editor wrapper | `assura editor session` | LSP-shaped JSON-line diagnostics, context, and code-action previews | The wrapper |
| Future tool/editor hook | Future agent integration | Scoped check plus feedback rendering | Tool result, next agent message, or status line | Hook configuration |
| Project-intelligence session | Local agent/editor wrapper | `assura content session` | JSON-line context/query responses | The wrapper |
| Warm checker session | Future structure-check integration | Prepared structure checker or hot daemon | Low-latency check result for changed paths | Integration policy |

The primary DX is `assura check`. The package is a lower-level bridge for
wrappers that already have an Assura JSON report or cannot shell out to the Rust
CLI directly.

## Review Before Check

Use `assura review .` while work is changing. It summarizes the branch and
worktree scope, actionable findings, crossed advisory thresholds, the most
specific hot path, and the first concrete repair. A successfully assembled
review is advisory and exits zero even when it reports `needs attention`.

Use `assura check --format agent .` when the configured project contract must
make the final pass/fail decision. Check preserves blocking exit behavior for
hooks and CI. Add global `--verbose` to Review only when a human needs the full
diagnostic inventory; integrations should prefer Review JSON or agent output
instead of scraping verbose text.

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

Structure feedback uses one shared severity contract across text, JSON, YAML,
advice, status, agent, and Codex output. `low` findings are advisory and emit
`"blocking": false`; `medium`, `high`, and `critical` findings emit
`"blocking": true` and make `assura check` exit 1 unless `--warn` is present.
Agent messages include `path`, `rule`, `severity`, `severity_label`,
`blocking`, `problem`, and `corrective_context` so integrations do not need to
parse human prose.

## Event-Aware Nudges

Use `assura agent nudge` when a wrapper can observe session or tool events and
needs a concise decision about whether Assura context should be injected.
Identical messages are suppressed during the default 300-second cooldown;
wrappers can set `--cooldown-seconds 0` for measurement or deliberate replay:

```bash
assura agent nudge --event session-start --agent codex .
assura agent nudge --event before-tool --agent opencode --changed docs/guide.md .
assura agent nudge --event after-tool --agent claude --changed docs/guide.md .
assura agent nudge --event file-read --agent codex --changed docs/guide.md .
assura agent nudge --event recovery --agent opencode .
assura agent nudge --event after-tool --agent pi --changed src/cli/check/rules.rs .
```

The command emits `assura.agent-nudge.v1` JSON by default. Its `target_agent`
field labels the host wrapper; it does not select a private validation engine.
`codex` may use the supported Codex delivery wrapper for deeper check output,
while `opencode`, `claude`, and `pi` should fetch generic
`assura check --format agent --warn` output when detail is needed.

| Event | Expected use | Cache and context policy |
| --- | --- | --- |
| `session-start` | Compact daemon health and project readiness. | Inject only when `summary.should_inject` is true. |
| `before-tool` | Path-aware edit, move, delete, or targeted read operations likely to influence edits. | Pass only paths relevant to the pending tool call. |
| `after-tool` | Changed files produced new findings or affected references. | Keep `--max-issues` small and defer full reports to explicit commands. |
| `file-read` | High-value reads that should account for affected docs, headings, references, or content records. | Pass the read path through `--changed`; inject only when Assura finds actionable context. |
| `recovery` | Resume, stale daemon state, failed tool call, or context-repair moments. | Prefer compact daemon and fallback commands over broad reports. |

Daemon-aware nudges include exact follow-up commands such as
`assura daemon status --format json .`, `assura daemon doctor --format json .`,
and the appropriate `assura check --format agent` fallback. Wrappers should use
those commands for detail instead of injecting daemon state into every event.

## When To Check

Agent-ready onboarding writes `.assura/onboarding/lifecycle.md` with the same
three lifecycle modes used by the JSON onboarding report:

- `nudge`: path-aware working-loop feedback from `assura agent nudge`.
- `warn`: advisory checks with `assura check --format agent --warn`.
- `gate`: pre-push or CI checks without `--warn`, preserving blocking exit
  behavior for configured medium, high, or critical findings.

| Moment | Recommended check | Why |
| --- | --- | --- |
| Before a commit | Git pre-commit hook | Catch drift before local history changes. |
| Before a push | Git pre-push hook | Catch drift before PR/CI feedback. |
| Before Codex processes a user prompt | Optional Codex `UserPromptSubmit` hook | Inject bounded Assura context into Codex when the user has opted in. |
| Before or after path-aware agent tool calls | `assura agent nudge --event before-tool|after-tool --changed <path>` | Give the agent immediate, bounded guidance only when changed paths matter. |
| Before high-value file reads | `assura agent nudge --event file-read --changed <path>` | Surface affected-reference or content context before the agent trusts a file in isolation. |
| During recovery or resume | `assura agent nudge --event recovery` | Re-anchor the agent on daemon health and fallback commands without flooding context. |
| Before a user-facing agent response | Reuse the latest report or run a final scoped check | Avoid telling the user work is done while structure drift remains. |
| After config or checkout changes | Full project check | Rebuild assumptions after policy or tree shape changes. |

## Warm Sessions And Index Reuse

The current public `assura check` and Git hook paths do not keep a daemon. They
run when a caller invokes Assura or Git fires an installed hook.

For repeated project-intelligence queries, use a local JSON-line session:

```bash
assura content session .
```

For one-shot project-intelligence handoffs, use the local agent command group:

```bash
assura agent context .
assura agent diagnostics .
assura agent context-pack . --collection assura_goals --id goal-assura-project-intelligence-usability-program --text "Project Intelligence Usability" --limit 5
assura agent safe-fixes .
```

These commands default to JSON and reuse the same content-query and safe-fix
preview contracts as `assura content ...`.

For editor wrappers, use the local editor session:

```bash
assura editor session .
```

It accepts LSP-shaped JSON-line methods:

```json
{"request_id":"diag-1","method":"textDocument/diagnostics","params":{"textDocument":{"uri":"docs/goals/goal_portable_structure.md"}}}
{"request_id":"ctx-1","method":"textDocument/context","params":{"uri":"docs/goals/goal_portable_structure.md","text":"portable","limit":5}}
{"request_id":"fix-1","method":"textDocument/codeAction","params":{"uri":"docs/goals/goal_portable_structure.md"}}
```

Editor responses use `assura.project-intelligence.editor.response.v1` and
include the same conservative reload states as `assura content session`.
`textDocument/codeAction` returns preview actions only; wrappers must require an
explicit `assura fix markdown --apply --format json` step before writing.

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

Managed editor and post-tool agent integrations use that shape:

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
| `assura agent integration install <agent>` | Supported next-release | Generates a reviewable `.assura/integrations/<agent>/` manifest, wrapper, and README for Codex, OpenCode, Claude, or Pi. |
| `assura agent integration activate <agent>` | Supported next-release | Explicitly patches only Assura-owned project-local host entries, verifies the result, and rolls back partial writes. |
| `assura agent integration update <agent>` | Supported next-release | Regenerates managed files and refreshes an already active integration when the shared wrapper contract changes. |
| `assura agent integration deactivate <agent>` | Supported next-release | Removes only Assura-owned host entries while retaining the reviewable bundle. |
| `assura agent integration remove <agent>` | Supported next-release | Deactivates Assura-owned entries and removes only the managed bundle. |
| `assura agent integration status <agent>` | Supported next-release | Reports generated, activated, verified, conflicted, and managed-file state. |
| `assura agent integration doctor <agent>` | Supported next-release | Checks project config, bundle and activation state, and delegation to nudge/check/daemon commands. |
| Codex package library | Lower-level only | Wrapper code can use library helpers when it already has JSON. |
| Codex `UserPromptSubmit` hook | Yes | A hook runs `assura check --format agent --agent codex` before Codex processes a prompt. |
| Codex event nudges | Supported next-release | The managed hook calls `assura agent nudge --agent codex` around relevant session and tool events. |
| OpenCode event nudges | Supported next-release | The managed plugin calls `assura agent nudge --agent opencode` and shared agent-format checks for detail. |
| Claude event nudges | Supported next-release | Managed project hooks call `assura agent nudge --agent claude` around path-aware tool events. |
| Pi event nudges | Supported next-release | The managed local extension calls `assura agent nudge --agent pi`, including performance-gate nudges for structure hot paths. |
| Other agents with shell access | Yes | They can call `assura agent ...` for project-intelligence context and `assura check --format agent` for structure feedback. |
| Project-intelligence session | Yes | Local wrappers can keep `assura content session` open for repeated context/query requests. |
| Project-intelligence editor session | Yes | Local editor wrappers can keep `assura editor session` open for LSP-shaped diagnostics, context, and code-action previews. |
| Full LSP/editor marketplace package | Not yet | A future package should wrap the local editor session or shared content-query contracts. |
| Editor/daemon structure-check integration | Not yet as public UX | Should reuse prepared checks or hot daemon state for repeated changed-path feedback. |
