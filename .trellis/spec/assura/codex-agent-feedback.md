# Codex Agent Feedback Contract

## 1. Scope / Trigger

This spec applies when changing Assura agent feedback output, especially
`assura check` feedback formats, optional delivery adapters, native Codex hook
output, Codex post-tool nudges, install docs, and advisory/blocking behavior.
Codex integration is optional and must not make normal developer workflows
depend on Codex hooks.

## 2. Signatures

- `assura check [path] --format advice [--min-severity <severity>] [--max-issues <count>] [--warn]`
- `assura check [path] --format status [--min-severity <severity>] [--max-issues <count>] [--warn]`
- `assura check [path] --format agent [--agent generic|codex] [--min-severity <severity>] [--max-issues <count>] [--warn]`
- `assura agent nudge [path] --agent codex --event after-tool --changed <path> --format json [--min-severity <severity>] [--max-issues <count>]`
- Codex hook config may call `.codex/hooks/assura-agent-nudge.py` from
  `UserPromptSubmit` and `PostToolUse`.

Primary check feedback options:

- `--min-severity low|medium|high|critical`
- `--max-issues <non-negative-integer>`
- `--warn`
- `--agent generic|codex` only applies with `--format agent`

## 3. Contracts

- The stable user-facing feedback API is `assura check`; do not add one
  management entrypoint or one `--format <agent>-hook` value per agent.
- Direction lock, clarified on 2026-05-31: `assura-codex-feedback`,
  `assura check --format codex-hook`, and future per-agent feedback CLI or
  format names are superseded by `assura check --format agent` plus optional
  delivery options such as `--agent codex`.
- Before changing this surface, read this spec, the active goal doc, and the
  current PR body/comments. Treat older branch history, task notes, or package
  docs that point to per-agent commands/formats as stale unless this spec is
  explicitly updated first.
- `assura check --format advice|status|agent` must share the same filtering and
  advisory options where practical.
- `--format agent` is the stable structured feedback format. `--agent codex`
  only wraps that feedback in Codex `UserPromptSubmit` hook JSON for delivery.
- Codex `PostToolUse` delivery is an optional local adapter over
  `assura agent nudge --agent codex --event after-tool`; it must not create a
  second stable check format or a package-specific feedback binary.
- Post-tool nudges compare the current Git worktree snapshot with the previous
  prompt/tool snapshot in the same session, pass changed paths to
  `assura agent nudge`, and append audit records under
  `.assura/agent-sessions/*.jsonl`.
- Post-tool injection is advisory and should be selective: inject when Assura
  returns `summary.should_inject`, when high-risk Git intent such as commit,
  merge, rebase, push, reset, clean, checkout, switch, pull, or stash is
  detected against a dirty worktree, or when the changed-path delta is large
  enough to warrant interruption.
- Codex post-tool context must include the hook event, tool name, detected tool
  intent, changed-path delta since the previous hook/message, dirty-path count,
  nudge summary, and log/state file locations.
- The Assura release installer installs the Rust CLI only. The npm package may
  provide library helpers for wrappers, but it must not publish separate
  feedback CLI binaries.
- User-facing hook snippets must use `assura check --format agent --agent codex`
  rather than package-specific feedback binaries.
- Codex hook docs must say that users need `features.hooks = true` and one-time
  `/hooks` approval before `UserPromptSubmit` or `PostToolUse` hook feedback can
  appear.
- `assura check --format agent` stdout must be Assura agent feedback JSON with
  schema `assura.agent-feedback.v1`.
- `assura check --format agent --agent codex` stdout must be Codex hook JSON with
  `hookSpecificOutput.hookEventName = "UserPromptSubmit"` and
  `hookSpecificOutput.additionalContext` containing an `<assura-feedback>`
  block.
- `assura check --format agent --agent codex --warn` is advisory and exits `0`
  for validation failures; omitting `--warn` preserves normal `assura check`
  blocking behavior and exits `1` for validation failures.

## 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| `assura check --format agent` passes | Emit `assura.agent-feedback.v1` JSON and exit `0`. |
| `assura check --format agent --agent codex` passes | Emit `UserPromptSubmit` hook JSON and exit `0`. |
| `assura check --format agent --agent codex --warn` has violations | Emit `UserPromptSubmit` hook JSON and exit `0`. |
| `assura check --format agent --agent codex` has violations without `--warn` | Emit `UserPromptSubmit` hook JSON and exit `1`. |
| `assura check --format agent --agent codex --min-severity medium --max-issues 1` | Emit hook JSON with filtered/bounded feedback context. |
| Codex `PostToolUse` follows a mutating tool and changed paths violate Assura policy | Emit `PostToolUse` hook JSON with an `<assura-nudge>` block and append nudge/state JSONL records. |
| Codex `PostToolUse` follows a non-mutating tool with no changed-path delta and no high-risk Git intent | Append an audit record if the wrapper runs, but emit no stdout context. |
| Codex `PostToolUse` detects `git commit`, `git merge`, or similar high-risk Git intent while the worktree is dirty | Emit `PostToolUse` hook JSON even when the changed-path delta is zero, so the agent sees commit/merge pressure. |
| `assura check --format codex-hook` | Reject as an unsupported format. |
| `assura check --format status --agent codex` | Reject because delivery adapters require `--format agent`. |
| Unsupported `assura check` arguments | Follow normal `assura check` parse errors and exit `2`. |

## 5. Good / Base / Bad Cases

- Good: A project appends `assura check --format agent --agent codex . --warn`
  to existing `UserPromptSubmit` hooks and receives advisory context on each
  prompt.
- Good: A project appends `.codex/hooks/assura-agent-nudge.py` to Codex
  `PostToolUse` with matcher `*`; after a mutating tool call it logs an
  `after_tool` nudge and injects only when policy warrants interruption.
- Base: A user runs `assura check --format agent . --warn` manually and sees
  stable Assura agent feedback JSON on stdout.
- Bad: Documentation implies hooks run without Codex hook enablement/approval,
  the hook edits `.codex/hooks.json` automatically, a new per-agent feedback
  management binary is the stable API, each agent needs a distinct
  `--format <agent>-hook` value, or post-tool hooks inject on every tool call
  without considering changed paths, severity, Git intent, or auditability.
- Blocking regression: A PR reintroduces `assura-codex-feedback`, documents
  `assura check --format codex-hook` as valid, or presents package executable
  entrypoints as the normal user-facing path instead of `assura check`.

## 6. Tests Required

- Unit tests for hook JSON shape and `additionalContext`.
- Unit tests for severity filtering, max-message limiting, and violation
  blocking thresholds.
- Regression tests for `assura check --format agent --agent codex` advisory and
  blocking exit behavior.
- Integration tests for `.codex/hooks.json` wiring of Codex `PostToolUse`.
- Integration tests that simulate Codex `PostToolUse` JSON, verify changed-path
  delta detection, verify high-risk Git intent injection, and assert
  `.assura/agent-sessions/nudges.jsonl` plus
  `.assura/agent-sessions/codex-hook-state.jsonl` audit records.
- Packaging smoke with `npm pack --dry-run` must not expose package executable
  binaries.

## 7. Wrong vs Correct

### Wrong

```json
{
  "command": "assura check --format codex-hook . --warn"
}
```

Documenting either the package hook as the primary API or a Codex-specific check
format leaves users unsure which surface is stable and encourages one command
shape per agent.

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "assura check --format agent --agent codex . --warn"
          }
        ]
      }
    ]
  }
}
```

Running the full project check after every tool call is too noisy and lacks
changed-path, Git-intent, and audit-state context.

### Correct

```json
{
  "command": "assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5"
}
```

Pair hook snippets with Codex hook enablement/approval prerequisites and explain
that `--warn` makes validation feedback advisory while omitting `--warn` uses
normal `assura check` blocking behavior.

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "*",
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"$(git rev-parse --show-toplevel)/.codex/hooks/assura-agent-nudge.py\"",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

The adapter should call `assura agent nudge --agent codex --event after-tool`,
log the nudge payload, track the previous Git snapshot, and inject compact
context only when severity, changed paths, or high-risk Git intent justify it.
