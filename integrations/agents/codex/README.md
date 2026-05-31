# Assura Agent Feedback

This package is the lower-level Assura agent feedback library. The stable
user-facing CLI path is `assura check` with feedback formats such as
`--format advice`, `--format status`, and `--format agent`. This package does
not publish separate feedback CLI binaries; use it from wrapper code only when
that wrapper already has `assura check --format json` output.

## Current Status

Supported in this MVP:

- parse `StructureCheckReport` JSON from `assura check --format json`
- create actionable feedback messages for structure violations
- filter feedback messages by minimum severity and cap displayed message count
- render concise status lines for tool-result or agent-message wrappers
- parse `assura check --format json` output produced by the Rust CLI
- observe same-turn feedback by violation class after guidance is applied
- render optional native Codex `UserPromptSubmit` hook feedback through
  `hookSpecificOutput.additionalContext`
- configure hook severity filtering, injected message count, and opt-in
  blocking behavior
- compare evaluation runs for instructions-only, `AGENTS.md`/skills, and
  Assura runtime-feedback workflows

Not supported yet:

- package executable binaries
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

## Stable CLI Usage

For normal CLI use, prefer:

```bash
assura check --format advice .
assura check --format status .
assura check --format advice . --min-severity medium --max-issues 3
assura check --format agent . --warn --min-severity medium --max-issues 5
assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5
```

Create reusable JSON for wrapper code with the Rust CLI:

```bash
assura check --format json . > assura-report.json
assura check --format agent . > assura-feedback.json
```

## Optional Native Codex Hook

Use the Rust CLI directly for Codex hook feedback:

```bash
assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5
```

`--format agent` is the stable Assura feedback format. `--agent codex` wraps
that feedback in native Codex `UserPromptSubmit` hook JSON. `--warn` keeps the
hook advisory with exit `0`; omit `--warn` when the surrounding workflow should
block on Assura validation failures.

Prerequisites:

- Install the `assura` CLI. The Assura release installer installs `assura` and
  `assura-full`.
- Enable Codex hooks in user config with `features.hooks = true`, then run
  `/hooks` once in Codex and approve the project hook command.

When configured with the Rust CLI command above, Codex invokes `assura check
--format agent --agent codex` for the `UserPromptSubmit` hook event used by this
repository's `.codex/hooks.json`. Feedback is injected by writing Codex hook JSON
to stdout:

```json
{
  "hookSpecificOutput": {
    "hookEventName": "UserPromptSubmit",
    "additionalContext": "<assura-feedback>...</assura-feedback>"
  }
}
```

The injected context includes the check source, severity filter, issue limit,
validation counts, and path-specific Assura feedback messages.

Example optional hook entry:

```json
{
  "hooks": {
    "UserPromptSubmit": [
      {
        "hooks": [
          {
            "type": "command",
            "command": "assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5",
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

The package currently produces feedback data and Codex `UserPromptSubmit` JSON
rendering helpers. It does not install hooks, expose CLI commands, intercept
tool calls, mutate tool responses, keep a warm process, or decide
editor-session policy.

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
