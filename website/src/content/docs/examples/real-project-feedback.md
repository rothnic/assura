---
title: Real Project Feedback
description: Protect a realistic project shape and turn failures into agent guidance
template: doc
sidebar:
  order: 2
---

This example shows the current supported Assura workflow for a modern
multi-package project. It protects the project shape, installs local feedback
hooks, runs a check, shows guided output, and reruns after drift is fixed.

> **No background service in this workflow**
>
> This walkthrough does not run a daemon or maintain a live repository index.
> Checks run only when Git invokes an installed hook, when you run
> `assura check`, or when an integration calls Assura.

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
files:
  exists:
    AGENTS.md: "1"
```

Exact file counts such as `AGENTS.md: "1"` are Assura behavior. Extension
counts such as `*.md`-style direct file counts are the LS-Lint-compatible part
of this policy family.

## Install Local Feedback

Install Assura, initialize or copy a policy into `.assura/config.yml`, then wire
the local feedback loop:

```bash
assura hooks install
assura hooks status
assura hooks verify
```

`hooks verify` gives an agent a clear pass/fail signal before it starts editing.
The hooks remain local Git hooks; they are not a daemon, hosted telemetry, or
autonomous agent orchestration.

Installed Git hooks rerun Assura on Git events:

| Trigger | What reruns | Hot session? |
| --- | --- | --- |
| `git commit` | `pre-commit` runs `assura check --format advice` | No |
| `git push` | `pre-push` runs `assura check --format advice` | No |
| `git checkout` | `post-checkout` runs `assura status` | No |
| Agent edits a file | Nothing from Git hooks alone | Future native agent/editor integration |

The hot daemon/session work exists below this UX as performance infrastructure,
but `assura hooks install` does not start or manage that session yet.

The current flow is intentionally one command at each trigger:

| Path | Command or integration | Output |
| --- | --- | --- |
| Manual check | `assura check --format advice` or `--format status` | Terminal output |
| Git commit/push | Installed Git hook runs Assura | Hook output |
| Native agent hook | Planned integration | Tool/result status |
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

The invalid fixture intentionally includes:

- `scratch.md` at the root, which violates the closed direct-file list
- `apps/web/src/BadName.tsx`, which violates kebab-case source naming
- a missing `packages/ui/AGENTS.md`, which violates the exact direct count

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
assura check --format advice "$work" \
  --min-severity medium \
  --max-issues 3
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
| Advisory exit | `--warn` | Reports drift but exits `0` |

This example proves the supported paths: manual CLI output, installed Git hook
behavior, configurable guided output, and same-turn observation. Codex
`UserPromptSubmit` delivery uses
`assura check --format agent --agent codex` when users wire that hook manually.

For the full delivery model, including warm sessions and index reuse for future
agent integrations, see [Agent Feedback Delivery](/reference/agent-feedback/).

## Observe Same-Turn Feedback

The agent feedback package exposes `observeSameTurnFeedback` for recording
whether feedback helped before a new turn was needed. The observation records:

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

Fix the drift in the disposable copy by removing the unexpected file, renaming
the source file to kebab-case, and restoring package guidance:

```bash
cd /tmp/assura-real-project-feedback
rm scratch.md
mv apps/web/src/BadName.tsx apps/web/src/bad-name.tsx
printf '# UI Agent Guidance\n' > packages/ui/AGENTS.md
assura check --format text .
```

The same policy now passes without changing the project workflow source of
truth.
