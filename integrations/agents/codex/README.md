# Assura Codex Integration

This package provides the first Assura Codex/agent nudge MVP. It turns
`assura check --format json` reports into advisory messages that a developer or
agent can use while fixing repository structure failures.

## Current Status

Supported in this MVP:

- parse `StructureCheckReport` JSON from `assura check --format json`
- create actionable nudge messages for structure violations
- run `assura check --format json` and preserve Assura's exit code, including
  non-JSON configuration/runtime failures
- compare evaluation runs for instructions-only, `AGENTS.md`/skills, and
  Assura runtime-nudge workflows
- run a small CLI entrypoint:

  ```bash
  assura-codex-nudge --report assura-report.json --format text
  assura-codex-nudge --path . --format json
  ```

Not supported yet:

- automatic Codex hook installation
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
const nudge = createNudgeFromReport(report);

console.log(nudge.summary);
```

## CLI Usage

Read an existing report:

```bash
assura check --format json . > assura-report.json
assura-codex-nudge --report assura-report.json --format text
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
violations, nudge precision, and deltas from the instructions-only baseline.

## Development

```bash
npm install
npm run lint
npm test
npm run build
```
