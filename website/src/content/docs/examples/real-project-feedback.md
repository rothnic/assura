---
title: Real Project Feedback
description: Protect a realistic project shape and turn failures into agent guidance
template: doc
sidebar:
  order: 2
---

This example shows the current supported Assura workflow for a modern
multi-package project. It protects the project shape, runs the stable agent
feedback check, shows guided output, and reruns after drift is fixed.

> **No background service in this workflow**
>
> This walkthrough does not run a daemon or maintain a live repository index.
> Checks run only when you run `assura check`, when Git invokes a hook that you
> configured separately, or when an integration calls Assura.

This walkthrough is reproducible from a clone of the Assura repository. The
fixture used by Assura's tests lives at
`tests/fixtures/real-project-agentic-feedback/`.

## Policy Shape

The policy models a product platform with:

- root `README.md`, root `AGENTS.md`, and workspace manifests
- `apps/web` with source, tests, docs, and app config files
- `packages/ui` with source, tests, and package-local `AGENTS.md`
- ignored generated directories such as `node_modules`, `dist`, `coverage`,
  `.next`, and `.turbo`

The root and package guidance files use Assura's exact direct-count behavior:

```yaml
structure:
  AGENTS.md: exists:1
```

Exact file counts such as `AGENTS.md: exists:1` are Assura behavior. Extension
counts such as `*.md`-style direct file counts are the LS-Lint-compatible part
of this policy family.

## Run Local Feedback

Install Assura, initialize or copy a policy into `.assura/config.yml`, then run
the local feedback loop through the stable check surface:

```bash
assura check --format agent . --warn
```

`--format agent` emits stable `assura.agent-feedback.v1` JSON. `--warn` keeps
the command advisory so an agent can inspect feedback without blocking the
surrounding workflow.

Codex delivery uses the same agent-format report through an explicitly
activated project-local integration:

```bash
assura agent integration activate codex .
assura check --format agent --agent codex . --warn
```

Codex only shows this prompt-hook feedback after the user enables
`features.hooks = true` in Codex config and approves hooks once with `/hooks`.
Assura does not automate those Codex user-level settings.

The current flow is intentionally one command at each trigger:

| Path | Command or integration | Output |
| --- | --- | --- |
| Manual check | `assura check --format advice` or `--format status` | Terminal output |
| Stable agent JSON | `assura check --format agent` | `assura.agent-feedback.v1` JSON |
| Codex prompt hook | `assura check --format agent --agent codex` | Codex `UserPromptSubmit` JSON |
| Git commit/push | User-configured Git hook runs Assura | Hook output |
| Native post-tool agent hook | Planned integration | Tool/result status |
| Warm session | Planned integration | Reused check state |

## Run The Valid Case

```bash
assura check --format json tests/fixtures/real-project-agentic-feedback/valid
```

The valid fixture exits `0` and reports no violations.

## Inspect Drift

```bash
work=/tmp/assura-real-project-feedback
rm -rf "$work"
cp -R tests/fixtures/real-project-agentic-feedback/invalid "$work"
assura check --format advice "$work"
```

The invalid fixture intentionally seeds 12 policy violations:

- two root scratch/draft files with Critical closed-world file drift
- one app-level notes file with High closed-world file drift
- one missing `packages/ui/AGENTS.md` with High exact-count drift
- source and test files with Medium naming drift
- JavaScript source files with Medium extension and forbidden-file drift

The feedback is advisory unless your workflow enforces the Assura exit code. It
points back to project-local guidance such as `AGENTS.md` and
`.assura/config.yml`.

For compact output, render only the status line:

```bash
assura check --format status "$work"
```

For noisier projects, limit what gets displayed without changing what Assura
checks:

```bash
assura check --format agent "$work" \
  --min-severity medium \
  --max-issues 11 \
  --warn
```

`--min-severity` and `--max-issues` only control displayed feedback
items.
The check still evaluates the configured project policy, and the CLI still exits
with Assura's result: `0` when the report passes and `1` when the report
contains validation failures. Use `--warn` for advisory reporting that exits
successfully.

## Output Model

There is no separate agent mode. Assura runs the same check and changes only the
output shape:

| Need | Option | Result |
| --- | --- | --- |
| Machine facts | `--format json` | Raw report |
| Repair guidance | `--format advice` | Bounded next steps |
| Hook/tool status | `--format status` | One-line summary |
| Stable agent feedback | `--format agent` | `assura.agent-feedback.v1` JSON |
| Codex prompt delivery | `--format agent --agent codex` | Codex `UserPromptSubmit` JSON |
| Advisory exit | `--warn` | Reports drift but exits `0` |

This example proves the supported paths: manual CLI output, stable agent JSON,
optional managed Codex delivery, configurable guided output, and same-turn
observation. Codex still requires the user to enable and approve project hooks;
Assura manages only its project-local entries.

The checked Goal 03 proof caps feedback at 11 messages for 12 seeded
violations. Critical and High violations are prioritized before Medium
violations, so all 4 Critical/High violations and 7 of 8 Medium violations
appear before truncation. The checked Codex `additionalContext` is under 24 KiB
and deterministic across repeated runs.

For the full delivery model, including warm sessions and index reuse for future
agent integrations, see [Agent Feedback Delivery](/reference/agent-feedback/).

## Observe Same-Turn Feedback

The lower-level Codex library helper exposes `observeSameTurnFeedback` for
recording whether feedback helped before a new turn was needed. The observation
records:

- violation class
- feedback count
- whether the class was fixed before a new turn
- useful, noisy, or mixed classification
- remaining violations
- response source
- turn boundary
- repeat feedback count

This is local measurement data. It is not hosted telemetry and it does not
claim complete autonomous repair.

## Rerun After Fixing

Fix the drift in the disposable copy by removing unexpected files, renaming
source and test files to kebab-case, removing forbidden JavaScript files, and
restoring package guidance:

```bash
cd /tmp/assura-real-project-feedback
rm scratch.md draft-plan.md apps/web/notes.txt
rm apps/web/src/legacy.js apps/web/src/old-helper.js
rm apps/web/tests/BadSpec.ts apps/web/tests/HomePage.test.ts
mv apps/web/src/BadName.tsx apps/web/src/bad-name.tsx
mv apps/web/src/AnotherBad.tsx apps/web/src/another-bad.tsx
printf '# UI Agent Guidance\n' > packages/ui/AGENTS.md
assura check --format text .
```

The same policy now passes without changing the project workflow source of
truth.
