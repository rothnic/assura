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
- `assura agent integration activate codex .` generates the adapter and wires
  `SessionStart`, `UserPromptSubmit`, `PreToolUse`, and `PostToolUse`.

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
- A `max_lines` finding includes the actual count, effective limit, and lines
  to cut or move. Guidance recommends splitting the file or escalating a
  legitimate exception to the policy owner.
- Agent output exposes `line_budget`. Raw JSON and YAML expose the same
  fields in the violation's `metadata`. When Assura can reproduce the policy
  context for a POSIX file target, `actions` lists explain then recheck,
  each with a working directory and shell command. Status stays summary-only.
- `--format agent` is the stable structured feedback format. `--agent codex`
  only wraps that feedback in Codex `UserPromptSubmit` hook JSON for delivery.
- Codex `PostToolUse` delivery is an optional local adapter over
  `assura agent nudge --agent codex --event after-tool`; it must not create a
  second stable check format or a package-specific feedback binary.
- The generated adapter extracts paths from known tool fields and from Codex
  `apply_patch` headers, then calls the shared nudge command. Pathless shell
  commands provide no changed-path hints. Event intent alone does not warrant a
  nudge. Nudge audit records live under `.assura/agent-sessions/`.
- Routine context is a single selected finding or trajectory line inside an
  `<assura-feedback>` block, <=256 UTF-8 bytes including the wrapper. The adapter
  emits it only when Assura sets `summary.should_inject`.
- The Assura checkout uses the exact installed Codex adapter and its activation
  groups. Preserve its separate Trellis prompt hook. Bundle bytes are portable
  between worktrees. Regenerate through the lifecycle CLI and verify drift with
  `assura agent integration doctor codex .`.
- Codex sessions that cached the previous checkout command still reach the
  generated adapter through a forwarding launcher. New registrations point
  directly at the generated adapter.
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
| Codex `PostToolUse` supplies paths with eligible violations | Emit `PostToolUse` hook JSON with an `<assura-feedback>` block and a nudge audit record. |
| Codex `PostToolUse` supplies no eligible path or other signal | Emit no stdout context. |
| `assura check --format codex-hook` | Reject as an unsupported format. |
| `assura check --format status --agent codex` | Reject because delivery adapters require `--format agent`. |
| Unsupported `assura check` arguments | Follow normal `assura check` parse errors and exit `2`. |

## 5. Good / Base / Bad Cases

- Good: A project appends `assura check --format agent --agent codex . --warn`
  to existing `UserPromptSubmit` hooks and receives advisory context on each
  prompt.
- Good: A project activates the generated adapter alongside its own hooks.
  The adapter forwards path-bearing edits and emits only selected nudges.
- Base: A user runs `assura check --format agent . --warn` manually and sees
  stable Assura agent feedback JSON on stdout.
- Bad: Documentation implies hooks run without Codex hook enablement/approval,
  the hook edits `.codex/hooks.json` automatically, a new per-agent feedback
  management binary is the stable API, each agent needs a distinct
  `--format <agent>-hook` value, or post-tool hooks inject on every tool call
  without considering changed paths, severity, or auditability.
- Blocking regression: A PR reintroduces `assura-codex-feedback`, documents
  `assura check --format codex-hook` as valid, or presents package executable
  entrypoints as the normal user-facing path instead of `assura check`.

## 6. Tests Required

- Unit tests for hook JSON shape and `additionalContext`.
- Unit tests for severity filtering, max-message limiting, and violation
  blocking thresholds.
- Regression tests for `assura check --format agent --agent codex` advisory and
  blocking exit behavior.
- Integration tests check exact generated bundle and `.codex/hooks.json`
  parity in a second checkout while preserving the Trellis prompt hook.
- Verify that an `apply_patch` event forwards changed paths, logs to
  `.assura/agent-sessions/nudges.jsonl`, and emits bounded context. Verify a
  pathless Git command remains quiet without an eligible signal.
- Delivery tests must assert the routine context byte ceiling, valid UTF-8,
  one selected finding, and omission of repeated log/state metadata.
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

Running the full project check after every tool call produces avoidable context.
The generated adapter requests event-specific nudges.

### Correct

```json
{
  "command": "assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5"
}
```

Pair hook snippets with Codex hook enablement/approval prerequisites and explain
that `--warn` makes validation feedback advisory while omitting `--warn` uses
normal `assura check` blocking behavior.

```sh
assura agent integration activate codex .
assura agent integration doctor codex .
```

Use the host's hook approval when activating or refreshing its commands.
