# Assura Codex Integration

This package provides the first Assura Codex/agent nudge MVP. It turns
`assura check --format json` reports into advisory messages that a developer or
agent can use while fixing repository structure failures.

## Current Status

Supported in this MVP:

- parse `StructureCheckReport` JSON from `assura check --format json`
- create actionable nudge messages for structure violations
- filter nudge messages by minimum severity and cap displayed message count
- render concise status lines for tool-result or agent-message wrappers
- run `assura check --format json` and preserve Assura's exit code, including
  non-JSON configuration/runtime failures
- observe same-turn feedback by violation class after a nudge is applied
- compare evaluation runs for instructions-only, `AGENTS.md`/skills, and
  Assura runtime-nudge workflows
- run a small CLI entrypoint:

  ```bash
  assura-codex-nudge --report assura-report.json --format text
  assura-codex-nudge --path . --format json
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
  createNudgeFromReport,
  parseStructureCheckReport,
} from "@assura/codex-integration";

const report = parseStructureCheckReport(jsonFromAssura);
const nudge = createNudgeFromReport(report, {
  minimumSeverity: "high",
  maxMessages: 3,
});

console.log(nudge.summary);
```

Same-turn feedback observation:

```ts
import { observeSameTurnFeedback } from "@assura/codex-integration";

const observations = observeSameTurnFeedback(nudge, reportAfterFix, 2, 0, {
  responseSource: "codex-main-session",
  turnBoundary: "same_turn",
  repeatNudgeCount: 0,
});
console.log(observations);
```

## CLI Usage

Read an existing report:

```bash
assura check --format json . > assura-report.json
assura-codex-nudge --report assura-report.json --format text
assura-codex-nudge --report assura-report.json --format status
assura-codex-nudge \
  --report assura-report.json \
  --format status \
  --minimum-severity high \
  --max-messages 3 \
  --blocking
```

Run Assura directly:

```bash
assura-codex-nudge --path . --format json
```

Exit codes:

- `0`: the Assura report passed
- `1`: the Assura report contained validation failures
- `2`: the nudge CLI failed or the report was invalid

## Measurement Model

Use `compareEvaluationRuns` to compare:

- `instructions_only`
- `agents_skills`
- `assura_runtime_nudges`

Tracked metrics include structural violations introduced, correction loops,
instruction adherence, nudge count, useful nudges, noisy nudges, missed
violations, nudge precision, same-turn fixed/remaining observations by
violation class, response source, turn boundary, repeat nudge count, and deltas
from the instructions-only baseline.

## Delivery Model

The package currently produces nudge data. It does not decide when a Codex tool
call should be blocked or where a nudge is injected.

The intended hook wrapper contract is:

- before a tool call: optionally run a cheap Assura status check and block only
  when configured as blocking;
- after a tool call: run Assura on the configured scope and attach either
  `renderNudgeStatusLine` output or bounded nudge text to the next tool result
  or agent message;
- before a user-facing response: include only unresolved nudges that satisfy the
  configured severity and count settings.

## Development

```bash
npm install
npm run lint
npm test
npm run build
```
