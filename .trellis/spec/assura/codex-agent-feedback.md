# Codex Agent Feedback Contract

## 1. Scope / Trigger

This spec applies when changing `integrations/agents/codex`, especially command
signatures, native Codex hook output, install docs, and advisory/blocking
behavior. The integration is optional and must not mutate `.codex/hooks.json`
or make normal developer workflows depend on Codex hooks.

## 2. Signatures

- `assura-agent-feedback --report <path> [--format text|status|json]`
- `assura-agent-feedback [--path <path>] [--assura-bin <bin>] [--format text|status|json]`
- `assura-codex-hook --report <path> [hook-options]`
- `assura-codex-hook [--path <path>] [--assura-bin <bin>] [hook-options]`

Hook options:

- `--min-severity info|low|medium|high|critical`
- `--max-messages <positive-integer>`
- `--block-mode off|violations|errors|all`
- `--block-count <positive-integer>`

## 3. Contracts

- The Assura release installer installs the Rust CLI only. Codex feedback
  commands come from the separate `@assura/agent-feedback` npm package or a
  local build of `integrations/agents/codex`.
- Until `@assura/agent-feedback` is published, user-facing hook snippets should
  show the local source-build command path instead of claiming npm availability.
- Codex hook docs must say that users need `features.hooks = true` and one-time
  `/hooks` approval before `UserPromptSubmit` hook feedback can appear.
- Hook stdout must be Codex hook JSON with
  `hookSpecificOutput.hookEventName = "UserPromptSubmit"` and
  `hookSpecificOutput.additionalContext` containing an `<assura-feedback>`
  block.
- Hook context must identify whether it reused `--report <path>` or attempted
  `assura check --format json <path>`.
- Default hook behavior is advisory: validation failures and hook execution
  errors exit `0` unless users opt into blocking.

## 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Report JSON passes | Emit additional context and exit `0`. |
| Report JSON has violations, `--block-mode off` | Emit additional context and exit `0`. |
| Matching violations meet `--block-count`, `--block-mode violations` | Emit additional context and exit `1`. |
| Assura cannot produce a valid report, `--block-mode off` | Emit hook error context and exit `0`. |
| Assura cannot produce a valid report, `--block-mode errors` | Emit hook error context and exit `2`. |
| `--report` is stale or invalid | Hook context says report reuse failed, not that Assura ran. |
| Hook CLI arguments are malformed, `--block-mode off` | Emit hook error context and exit `0`. |
| Hook CLI arguments are malformed, `--block-mode errors` | Emit hook error context and exit `2`. |

## 5. Good / Base / Bad Cases

- Good: A project appends an `assura-codex-hook` command to existing
  `UserPromptSubmit` hooks, keeps `--block-mode off`, and receives advisory
  context on each prompt.
- Base: A user runs `assura-codex-hook --report assura-report.json` manually and
  sees Codex hook JSON on stdout.
- Bad: Documentation implies the Rust installer provides `assura-codex-hook`,
  hooks run without Codex hook enablement/approval, or the hook edits
  `.codex/hooks.json` automatically.

## 6. Tests Required

- Unit tests for hook JSON shape and `additionalContext`.
- Unit tests for report reuse versus direct check source descriptions.
- Unit tests for severity filtering, max-message limiting, and violation
  blocking thresholds.
- Regression tests for invalid report errors and malformed argument errors.
- Packaging smoke with `npm pack --dry-run` to prove both npm binaries are
  included.

## 7. Wrong vs Correct

### Wrong

```json
{
  "command": "assura-codex-hook --path ."
}
```

Documenting only this snippet leaves users unsure whether the command is
installed, whether Codex hooks are enabled, and whether the hook blocks.

### Correct

```json
{
  "command": "node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js --path . --block-mode off"
}
```

Pair hook snippets with local build instructions or the separate
`@assura/agent-feedback` npm-package path once published, Codex hook
enablement/approval prerequisites, and the advisory default.
