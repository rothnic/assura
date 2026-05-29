---
title: Real Project Feedback
description: Protect a realistic project shape and turn failures into agent guidance
template: doc
sidebar:
  order: 2
---

This example shows the current supported Assura workflow for a modern
multi-package project. It protects the project shape, installs local feedback
hooks, runs a check, turns failures into a Codex nudge, and reruns after drift
is fixed.

> **No background service in this workflow**
>
> This walkthrough does not run a daemon or maintain a live repository index.
> Checks run only when Git invokes an installed hook, when you run
> `assura check`, or when a script calls `assura-codex-nudge`.

The fixture used by Assura's tests lives at
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

## Run The Valid Case

```bash
assura check --format json tests/fixtures/real-project-agentic-feedback/valid
```

The valid fixture exits `0` and reports no violations.

## Inspect Drift

```bash
rm -rf /tmp/assura-real-project-feedback
cp -R tests/fixtures/real-project-agentic-feedback/invalid /tmp/assura-real-project-feedback
assura check --format json /tmp/assura-real-project-feedback > assura-report.json
assura-codex-nudge --report assura-report.json --format text
```

The invalid fixture intentionally includes:

- `scratch.md` at the root, which violates the closed direct-file list
- `apps/web/src/BadName.tsx`, which violates kebab-case source naming
- a missing `packages/ui/AGENTS.md`, which violates the exact direct count

The nudge is advisory unless your workflow enforces the Assura exit code. It
points back to project-local guidance such as `AGENTS.md` and
`.assura/config.yml`.

For compact agent output, render only the status line:

```bash
assura-codex-nudge --report assura-report.json --format status
```

For stricter workflows, configure the nudge surface:

```bash
assura-codex-nudge \
  --report assura-report.json \
  --format status \
  --minimum-severity high \
  --max-messages 3 \
  --blocking
```

`--minimum-severity` controls which violation severities become nudge messages,
`--max-messages` caps the number of messages shown, and `--blocking` marks the
nudge as blocking metadata for the surrounding workflow. The CLI still exits
with Assura's result: `0` when the report passes and `1` when the report
contains validation failures.

## When Feedback Runs

| Delivery path | Status | Who triggers it | What runs | What you see | Blocking behavior |
| --- | --- | --- | --- | --- | --- |
| Manual CLI proof | Supported | A developer or agent explicitly runs commands | `assura check`, then `assura-codex-nudge` | Terminal `status`, `text`, or `json` output | The command can fail with Assura's exit code; no agent tool call is intercepted |
| Git hooks | Supported | Git invokes `pre-commit`, `pre-push`, or `post-checkout` | Installed hook script runs `assura check` | Git hook output plus Assura report | The hook script decides pass/fail |
| Nudge package or CLI | Supported | Wrapper code calls the package or `assura-codex-nudge` | Report parsing and nudge rendering | Status line, text guidance, or JSON | The wrapper decides whether to treat the nudge as blocking |
| Native Codex hook | Planned | Codex tool-call lifecycle | Scoped check plus bounded nudge rendering | Tool result, next agent message, or status line | Not implemented in this example |

`--blocking` changes the rendered nudge from advisory to blocking metadata. It
does not install a hook or intercept Codex tool calls by itself.

This example proves the supported rows: manual CLI output, installed Git hook
behavior, report-to-nudge conversion, configurable nudge output, and same-turn
observation. It does not install the planned native Codex hook row.

For the full delivery model, including warm sessions and index reuse for future
agent integrations, see [Agent Feedback Delivery](/reference/agent-feedback/).

## Observe Same-Turn Feedback

The Codex integration exposes `observeSameTurnFeedback` for recording whether a
nudge helped before a new turn was needed. The observation records:

- violation class
- nudge count
- whether the class was fixed before a new turn
- useful, noisy, or mixed classification
- remaining violations
- response source
- turn boundary
- repeat nudge count

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
