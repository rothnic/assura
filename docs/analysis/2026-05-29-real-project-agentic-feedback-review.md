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
- Advisory feedback:
  `docs/analysis/2026-05-29-real-project-agentic-feedback-agent-feedback.json`
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
3. Run `assura check --format json`.
4. Run `assura check --format agent --warn` to get stable agent feedback JSON.
5. Use `assura check --format agent --agent codex --warn` only when a user has
   manually wired a Codex `UserPromptSubmit` hook.
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

Normalized the checked reports to repo-relative paths:

```bash
node --input-type=module <<'EOF'
import { readFileSync, writeFileSync } from 'node:fs';

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
writeFileSync(
  'docs/analysis/2026-05-29-real-project-agentic-feedback-invalid-report.json',
  JSON.stringify(invalid, null, 2) + '\n'
);
writeFileSync(
  'docs/analysis/2026-05-29-real-project-agentic-feedback-fixed-report.json',
  JSON.stringify(fixed, null, 2) + '\n'
);
EOF
```

Generated the checked stable advisory feedback artifact through the current CLI
surface:

```bash
cargo run --quiet -- check --format agent \
  --warn \
  tests/fixtures/real-project-agentic-feedback/invalid \
  > docs/analysis/2026-05-29-real-project-agentic-feedback-agent-feedback.json
```

Generated optional Codex delivery output through the adapter surface:

```bash
cargo run --quiet -- check --format agent --agent codex \
  --warn \
  tests/fixtures/real-project-agentic-feedback/invalid \
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
cargo run --quiet -- check --format agent \
  --warn \
  target/real-project-agentic-feedback-agent-run/work \
  > target/real-project-agentic-feedback-agent-run/feedback.json
rm target/real-project-agentic-feedback-agent-run/work/scratch.md
mv target/real-project-agentic-feedback-agent-run/work/apps/web/src/BadName.tsx \
  target/real-project-agentic-feedback-agent-run/work/apps/web/src/bad-name.tsx
printf '# UI Agent Guidance\n' \
  > target/real-project-agentic-feedback-agent-run/work/packages/ui/AGENTS.md
cargo run --quiet -- check target/real-project-agentic-feedback-agent-run/work \
  --format json \
  --output target/real-project-agentic-feedback-agent-run/after.json
```

Generated the same-turn observation from the normalized feedback and the observed
after-report:

```bash
node --input-type=module <<'EOF'
import { readFileSync, writeFileSync } from 'node:fs';
import {
  createAgentFeedbackFromReport,
  observeSameTurnFeedback,
} from './integrations/agents/codex/dist/index.js';

const feedback = createAgentFeedbackFromReport(
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
    observeSameTurnFeedback(feedback, after, feedback.messages.length, 0, {
      responseSource: 'codex-main-session',
      turnBoundary: 'same_turn',
      repeatFeedbackCount: 0,
    }),
    null,
    2
  ) + '\n'
);
EOF
```

The observed response source is `codex-main-session`, the turn boundary is
`same_turn`, and no repeat feedback was needed. Raw target files are intentionally
not checked in because `target/` is build output; the normalized observation is
checked in under `docs/analysis/`.

## Drift Covered

The invalid fixture reports the intentional policy drift:

- `apps/web/src/BadName.tsx` -> `file_naming`
- `packages/ui` -> `exists_count` for missing package-local `AGENTS.md`
- `scratch.md` -> `unexpected_file`

The feedback output includes local references to `AGENTS.md`,
`.agents/skills/`, and `.assura/config.yml`.

## Same-Turn Feedback Observation

`observeSameTurnFeedback` recorded one useful feedback per violation class from
the observed repair run above. The after-report had zero remaining violations,
the response source was `codex-main-session`, the turn boundary was
`same_turn`, and no repeat feedback was needed:

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
- Codex delivery is opt-in through `assura check --format agent --agent codex`.
  The proof does not install Codex hooks, mutate `.codex/hooks.json`, or run a
  background service.
- Same-turn observation is local evidence produced by the Codex integration
  library. It is not hosted telemetry.

## PR Review Follow-Up

Gemini Code Assist reviewed PR #13 on 2026-05-29 and opened three medium
priority comments. The follow-up commit addressed them by:

- reading only the first line of worktree `.git` files before parsing
  `gitdir:`;
- reading only the first line of Git `commondir` files before resolving common
  hooks;
- guarding `observeSameTurnFeedback` against a missing `violations` array in an
  after-report.

Focused verification after the review fixes:

```bash
cargo fmt --all -- --check
cargo test hooks --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cd integrations/agents/codex && npm run lint && npm test && npm run build
```

## User-Facing CLI Review Follow-Up

An independent Codex review of the updated user-facing CLI and website copy was
requested on 2026-05-29. It flagged stale API docs, ambiguous roadmap wording,
repo-local fixture assumptions, a pseudo `advice/status` command, and long
rendered command blocks.

Follow-up changes made after that review:

- Treat guided output as general `assura check` formats, not an agent mode:
  `--format advice` and `--format status`.
- Keep display controls general: `--min-severity`, `--max-issues`, and
  `--warn`.
- Document that Git hooks run on Git events only; native agent/editor hooks and
  hot-session management remain future integration work.
- Update API and roadmap docs so guided output and stable agent feedback are
  current, while native agent/editor automation remains future work.
- Add a clone prerequisite for the fixture-based website walkthrough.
- Replace the pseudo `--format advice/status` wording and long flow code block
  with tables.

## Stable Agent Surface Follow-Up

After PR #15 landed, the proof was refreshed to keep `assura check --format
agent` as the stable public feedback surface and `--agent codex` as the only
Codex delivery adapter. The old real-project proof language that centered
`assura hooks install`, `assura hooks status`, and `assura hooks verify` was
removed from the adoption flow. General Git hook documentation remains separate.

Additional focused verification for this follow-up:

```bash
cargo test --test real_project_agentic_feedback_tests --quiet
cargo run --quiet -- check --format agent tests/fixtures/real-project-agentic-feedback/invalid --warn
cargo run --quiet -- check --format agent --agent codex tests/fixtures/real-project-agentic-feedback/invalid --warn
npm run lint && npm test && npm run build && npm pack --dry-run
node --run verify:fast
node --run verify:docs
npx pnpm@10.25.0 build
```

Independent review found two follow-up issues and both were addressed: the
checked feedback artifact reproduction command now writes the stable CLI schema
directly, and the real-project guide now states the required Codex
`features.hooks = true` and one-time `/hooks` approval prerequisites.

Focused verification after this review:

```bash
cargo fmt --all -- --check
cargo test --all-targets --quiet
cargo run --quiet -- check --format json .
cd integrations/agents/codex && npm test && npm run build
cd website && npx pnpm@10.25.0 build
git diff --check
```

Rendered page smoke check:

- URL: `http://127.0.0.1:4321/examples/real-project-feedback/`
- Result: HTTP page loaded, no page-level horizontal overflow, no old
  `--agent`, `advice/status`, or `assura-codex-nudge` wording on the main
  example page, and the clone prerequisite is visible.
