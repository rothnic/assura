# Assura Agent Feedback

This package is the lower-level Assura agent feedback bridge. The preferred
user-facing CLI path is `assura check --format advice` or
`assura check --format status`; use this package when a wrapper already has
`assura check --format json` output or cannot call the Rust CLI directly.

## Current Status

Supported in this MVP:

- parse `StructureCheckReport` JSON from `assura check --format json`
- create actionable feedback messages for structure violations
- filter feedback messages by minimum severity and cap displayed message count
- render concise status lines for tool-result or agent-message wrappers
- run `assura check --format json` and preserve Assura's exit code, including
  non-JSON configuration/runtime failures
- observe same-turn feedback by violation class after guidance is applied
- render optional native Codex `UserPromptSubmit` hook feedback through
  `hookSpecificOutput.additionalContext`
- configure hook severity filtering, injected message count, and opt-in
  blocking behavior
- compare evaluation runs for instructions-only, `AGENTS.md`/skills, and
  Assura runtime-feedback workflows
- run small CLI entrypoints:

  ```bash
  assura-agent-feedback --report assura-report.json --format text
  assura-agent-feedback --path . --format json
  node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js --path . --min-severity medium --max-messages 5 --block-mode off
  ```

Not supported yet:

- automatic Codex hook installation
- automatic tool-call blocking or tool-response injection
- automatic mutation of Codex hook configuration
- hosted telemetry
- complete agent orchestration
- general quality scoring beyond the local evaluation model
- daemon/editor-session feedback reuse

## Library Usage

```ts
import {
  createAgentFeedbackFromReport,
  parseStructureCheckReport,
} from "@assura/agent-feedback";

const report = parseStructureCheckReport(jsonFromAssura);
const feedback = createAgentFeedbackFromReport(report, {
  minimumSeverity: "high",
  maxMessages: 3,
});

console.log(feedback.summary);
```

Same-turn feedback observation:

```ts
import { observeSameTurnFeedback } from "@assura/agent-feedback";

const observations = observeSameTurnFeedback(feedback, reportAfterFix, 2, 0, {
  responseSource: "codex-main-session",
  turnBoundary: "same_turn",
  repeatFeedbackCount: 0,
});
console.log(observations);
```

## CLI Usage

For normal CLI use, prefer:

```bash
assura check --format advice .
assura check --format status .
assura check --format advice . --min-severity medium --max-issues 3
```

Read an existing report from wrapper code:

```bash
assura check --format json . > assura-report.json
assura-agent-feedback --report assura-report.json --format text
assura-agent-feedback --report assura-report.json --format status
assura-agent-feedback \
  --report assura-report.json \
  --format status \
  --minimum-severity high \
  --max-messages 3 \
  --blocking
```

Run Assura directly:

```bash
assura-agent-feedback --path . --format json
```

Exit codes:

- `0`: the Assura report passed
- `1`: the Assura report contained validation failures
- `2`: the feedback CLI failed or the report was invalid

## Optional Native Codex Hook

`assura-codex-hook` is a small proof of the native Codex hook path. It is not
installed automatically. Users opt in by adding it to a Codex
`UserPromptSubmit` hook alongside any existing hooks.

Prerequisites:

- Install the `assura` CLI separately. The Assura release installer installs
  `assura` and `assura-full`; it does not install `assura-codex-hook`.
- Install or build this npm package so the hook command is available to the
  Codex process. This package is not part of the Rust release installer. From
  this source checkout, run `npm install && npm run build` in
  `integrations/agents/codex/` and point Codex at
  `node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js`.
  After `@assura/agent-feedback` is published, users can install it globally or
  use `npm exec --yes --package @assura/agent-feedback -- assura-codex-hook`.
- Enable Codex hooks in user config with `features.hooks = true`, then run
  `/hooks` once in Codex and approve the project hook command.

The hook runs when Codex invokes `UserPromptSubmit`, which is the per-prompt
hook event used by this repository's `.codex/hooks.json`. On each run it either:

- reuses a precomputed Assura report passed with `--report`, or
- runs `assura check --format json <path>` when no report is supplied.

Feedback is injected by writing Codex hook JSON to stdout:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "UserPromptSubmit",
    "additionalContext": "<assura-feedback>...</assura-feedback>"
  }
}
```

The injected context includes the check source, severity filter, message limit,
blocking mode, violation counts, and path-specific Assura feedback messages.

Example optional hook entry:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js --path . --min-severity medium --max-messages 5 --block-mode off",
            "timeout": 10
          }
        ]
      }
    ]
  }
}
```

If a project already has `UserPromptSubmit` hooks, append the command to the
existing hook list instead of replacing the file. This package intentionally
does not edit `.codex/hooks.json` for you.

### Hook Configuration

```bash
node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js --path . \
  --min-severity medium \
  --max-messages 5 \
  --block-mode off \
  --block-count 1
```

After `@assura/agent-feedback` is published and installed, use
`assura-codex-hook` with the same options.

- `--report <path>` reuses an existing `assura check --format json` report.
- `--path <path>` is checked when no report is supplied. Default: `.`.
- `--assura-bin <bin>` selects the Assura executable. Default: `assura`.
- `--min-severity info|low|medium|high|critical` filters which violations are
  shown and considered for violation blocking. Default: `info`.
- `--max-messages <count>` limits injected path-specific messages. Default: `5`.
- `--block-mode off|violations|errors|all` controls whether the hook command can
  return nonzero. Default: `off`.
- `--block-count <count>` sets the matching violation threshold for
  `violations` or `all` blocking. Default: `1`.

Default behavior is advisory: validation failures and hook execution errors are
reported to Codex context but the hook exits `0`. Blocking is opt-in:

- `--block-mode violations` exits `1` when matching violations meet
  `--block-count`.
- `--block-mode errors` exits `2` when the hook cannot produce a valid Assura
  report.
- `--block-mode all` enables both violation and error blocking.

This hook does not start or reuse a daemon, watch files, integrate with an
editor, or perform autonomous repair. It only emits per-prompt Codex hook
context from one Assura JSON report or one `assura check --format json` run.

## Measurement Model

Use `compareEvaluationRuns` to compare:

- `instructions_only`
- `agents_skills`
- `assura_runtime_feedback`

Tracked metrics include structural violations introduced, correction loops,
instruction adherence, feedback count, useful feedback, noisy feedback, missed
violations, feedback precision, same-turn fixed/remaining observations by
violation class, response source, turn boundary, repeat feedback count, and deltas
from the instructions-only baseline.

## Delivery Model

The package currently produces feedback data and a focused Codex
`UserPromptSubmit` hook command. It does not install hooks, intercept tool
calls, mutate tool responses, keep a warm process, or decide editor-session
policy.

The intended wrapper contract for integrations beyond the implemented Codex
prompt hook is:

- before a tool call: optionally run a cheap Assura status check and block only
  when configured as blocking;
- after a tool call: run Assura on the configured scope and attach either
  `renderAgentFeedbackStatusLine` output or bounded feedback text to the next
  tool result or agent message;
- before a user-facing response: include only unresolved feedback that satisfies
  the configured severity and count settings.

## Development

```bash
npm install
npm run lint
npm test
npm run build
```
