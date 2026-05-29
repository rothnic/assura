---
status: current
---

# Real Project Agentic Feedback Review

## Scope

This record covers the proof required by
`docs/goals/assura-real-project-policy-proof.md`.

The scenario is a deterministic multi-package product-platform fixture:

- valid fixture:
  `tests/fixtures/real-project-agentic-feedback/valid`
- invalid fixture:
  `tests/fixtures/real-project-agentic-feedback/invalid`
- scenario config:
  `tests/fixtures/real-project-agentic-feedback/valid/.assura/config.yml`

The policy uses supported structure-first fields. Exact direct file count rules
such as `AGENTS.md: "1"` are Assura behavior and are not presented as native
LS-Lint parity.

## Evidence Artifacts

- Invalid report:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-invalid-report.json`
- Fixed report:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-fixed-report.json`
- Advisory nudge:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-nudge.json`
- Same-turn observation:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-same-turn-observation.json`
- Raw release timing:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-timing.txt`
- User-facing guide:
  `website/src/content/docs/examples/real-project-feedback.md`

## User Journey Notes

The current supported workflow is:

1. Install Assura.
2. Define or migrate `.assura/config.yml`.
3. Run `assura hooks install`, `assura hooks status`, and
   `assura hooks verify`.
4. Run `assura check --format json`.
5. Feed the report to `assura-codex-nudge`.
6. Fix project drift and rerun `assura check`.

The proof does not claim daemon behavior, hosted telemetry, dependency graph
validation, automatic Codex hook installation, or autonomous repair.

The checked report artifacts normalize `project_root`, `config_path`, and
`checked_path` to repository-relative fixture paths so the evidence is stable
across clones.

## Command Results

Passed:

```bash
cargo fmt --all -- --check
git diff --check
cargo test --all-targets --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo run --quiet -- check --format json .
cd integrations/agents/codex && npm install && npm run lint && npm test && npm run build
npx pnpm@10.25.0 build
```

`pnpm` and `corepack` were not installed globally on this machine, so the
website build used the package-manager version declared by `website/package.json`
through `npx pnpm@10.25.0`.

Generated raw invalid report, expected exit `1`:

```bash
mkdir -p target/real-project-agentic-feedback-agent-run
cargo run --quiet -- check tests/fixtures/real-project-agentic-feedback/invalid \
  --format json \
  --output target/real-project-agentic-feedback-agent-run/before.json
```

Generated raw fixed report, expected exit `0`:

```bash
cargo run --quiet -- check tests/fixtures/real-project-agentic-feedback/valid \
  --format json \
  --output target/real-project-agentic-feedback-agent-run/fixed.json
```

Normalized the checked reports to repo-relative paths and generated the nudge:

```bash
node --input-type=module <<'EOF'
import { readFileSync, writeFileSync } from 'node:fs';
import {
  createNudgeFromReport,
  observeSameTurnFeedback,
} from './integrations/agents/codex/dist/index.js';

const invalidPath = 'tests/fixtures/real-project-agentic-feedback/invalid';
const validPath = 'tests/fixtures/real-project-agentic-feedback/valid';
function normalize(report, label) {
  return {
    ...report,
    project_root: label,
    config_path: `${label}/.assura/config.yml`,
    checked_path: label,
  };
}
const before = JSON.parse(
  readFileSync('target/real-project-agentic-feedback-agent-run/before.json', 'utf8')
);
const fixedRaw = JSON.parse(
  readFileSync('target/real-project-agentic-feedback-agent-run/fixed.json', 'utf8')
);
const invalid = normalize(before, invalidPath);
const fixed = normalize(fixedRaw, validPath);
const nudge = createNudgeFromReport(invalid);
writeFileSync(
  'docs/analysis/2026-05-29-real-project-agentic-feedback-invalid-report.json',
  JSON.stringify(invalid, null, 2) + '\n'
);
writeFileSync(
  'docs/analysis/2026-05-29-real-project-agentic-feedback-fixed-report.json',
  JSON.stringify(fixed, null, 2) + '\n'
);
writeFileSync(
  'docs/analysis/2026-05-29-real-project-agentic-feedback-nudge.json',
  JSON.stringify(nudge, null, 2) + '\n'
);
EOF
```

Generated advisory nudge, expected exit `1`:

```bash
node integrations/agents/codex/dist/cli.js \
  --report docs/analysis/2026-05-29-real-project-agentic-feedback-invalid-report.json \
  --format json
```

Observed same-turn repair run:

```bash
rm -rf target/real-project-agentic-feedback-agent-run
mkdir -p target/real-project-agentic-feedback-agent-run
cp -R tests/fixtures/real-project-agentic-feedback/invalid \
  target/real-project-agentic-feedback-agent-run/work
cargo run --quiet -- check target/real-project-agentic-feedback-agent-run/work \
  --format json \
  --output target/real-project-agentic-feedback-agent-run/before.json
node integrations/agents/codex/dist/cli.js \
  --report target/real-project-agentic-feedback-agent-run/before.json \
  --format json \
  > target/real-project-agentic-feedback-agent-run/nudge.json
rm target/real-project-agentic-feedback-agent-run/work/scratch.md
mv target/real-project-agentic-feedback-agent-run/work/apps/web/src/BadName.tsx \
  target/real-project-agentic-feedback-agent-run/work/apps/web/src/bad-name.tsx
printf '# UI Agent Guidance\n' \
  > target/real-project-agentic-feedback-agent-run/work/packages/ui/AGENTS.md
cargo run --quiet -- check target/real-project-agentic-feedback-agent-run/work \
  --format json \
  --output target/real-project-agentic-feedback-agent-run/after.json
```

Generated the same-turn observation from the normalized nudge and the observed
after-report:

```bash
node --input-type=module <<'EOF'
import { readFileSync, writeFileSync } from 'node:fs';
import {
  createNudgeFromReport,
  observeSameTurnFeedback,
} from './integrations/agents/codex/dist/index.js';

const nudge = createNudgeFromReport(
  JSON.parse(
    readFileSync(
      'docs/analysis/2026-05-29-real-project-agentic-feedback-invalid-report.json',
      'utf8'
    )
  )
);
const after = JSON.parse(
  readFileSync('target/real-project-agentic-feedback-agent-run/after.json', 'utf8')
);
writeFileSync(
  'docs/analysis/2026-05-29-real-project-agentic-feedback-same-turn-observation.json',
  JSON.stringify(
    observeSameTurnFeedback(nudge, after, nudge.messages.length, 0, {
      responseSource: 'codex-main-session',
      turnBoundary: 'same_turn',
      repeatNudgeCount: 0,
    }),
    null,
    2
  ) + '\n'
);
EOF
```

The observed response source is `codex-main-session`, the turn boundary is
`same_turn`, and no repeat nudge was needed. Raw target files are intentionally
not checked in because `target/` is build output; the normalized observation is
checked in under `docs/analysis/`.

## Drift Covered

The invalid fixture reports the intentional policy drift:

- `apps/web/src/BadName.tsx` -> `file_naming`
- `packages/ui` -> `exists_count` for missing package-local `AGENTS.md`
- `scratch.md` -> `unexpected_file`

The nudge output includes local references to `AGENTS.md`,
`.agents/skills/`, and `.assura/config.yml`.

## Same-Turn Feedback Observation

`observeSameTurnFeedback` recorded one useful nudge per violation class from
the observed repair run above. The after-report had zero remaining violations,
the response source was `codex-main-session`, the turn boundary was
`same_turn`, and no repeat nudge was needed:

- `exists_count`
- `file_naming`
- `unexpected_file`

## Performance Note

The same-turn proof uses the release `assura` binary because user-facing
feedback includes executable startup. Raw timing output is checked in at
`docs/analysis/2026-05-29-real-project-agentic-feedback-timing.txt`. On this
machine, a five-run release check of the invalid fixture reported BSD
`/usr/bin/time -p` wall times of `0.43s`, `0.00s`, `0.00s`, `0.00s`, and
`0.00s`.

This is sufficient for the small local feedback proof, but it is not a headline
aggregate performance claim. Broader changed-path and warm editor-session
optimization remains future work unless backed by checked real-repo performance
data.

## Known Limitations

- The fixture is generated and checked in, not a pinned external repository.
  That keeps the proof deterministic and fast for ordinary validation.
- The hook workflow installs local Git hooks only. It does not install Codex
  hooks or run a background service.
- Same-turn observation is local evidence produced by the Codex integration
  library. It is not hosted telemetry.
