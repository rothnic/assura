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
- compare evaluation runs for instructions-only, `AGENTS.md`/skills, and
  Assura runtime-feedback workflows
- run a small CLI entrypoint:

  ```bash
  assura-agent-feedback --report assura-report.json --format text
  assura-agent-feedback --path . --format json
  ```

Not supported yet:

- automatic Codex hook installation
- automatic tool-call blocking or tool-response injection
- hosted telemetry
- complete agent orchestration
- general quality scoring beyond the local evaluation model

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

The package currently produces feedback data. It does not decide when a Codex tool
call should be blocked or where feedback is injected.

The intended hook wrapper contract is:

- before a tool call: optionally run a cheap Assura status check and block only
  when configured as blocking;
- after a tool call: run Assura on the configured scope and attach either
  `renderAgentFeedbackStatusLine` output or bounded feedback text to the next tool result
  or agent message;
- before a user-facing response: include only unresolved feedback that satisfies the
  configured severity and count settings.

## Development

```bash
npm install
npm run lint
npm test
npm run build
```
