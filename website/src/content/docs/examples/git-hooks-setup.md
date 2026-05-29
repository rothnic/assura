---
title: Git Hooks Setup
description: Run Assura before commits or pushes
template: doc
sidebar:
  order: 4
---

Assura can install Git hooks when you want local commits or pushes to run the
same structure checks used in CI. The installed hooks are advisory on ordinary
feature branches by default.

> **Note**
>
> `assura check` validates the configured project path. It does not currently
> offer staged-file-only validation.

## Install And Verify

From a Git repository with `.assura/config.yml`:

```bash
assura hooks install
assura hooks status
assura hooks verify
```

`status` shows whether each hook is managed by Assura and runnable. `verify`
exits nonzero if a hook is missing, unmanaged, or not executable, which makes it
suitable for an agent or setup script to run before continuing work.

Re-running `assura hooks install` is idempotent. It does not overwrite an
existing custom hook unless you pass `--force`.

The generated pre-commit hook blocks on `main` and `master`; on other branches
it prints warnings so local work is not trapped mid-iteration. The generated
pre-push hook is advisory unless `ASSURA_BLOCKING_PUSH=1` is set.

If the check fails, fix the reported files and commit again.

## Hooks Versus Agent Wrappers

Git hooks and agent nudges are separate delivery paths:

- Git hooks are executed by Git before commit, before push, or after checkout.
  They can block only when their script exits nonzero in a blocking mode.
- Agent nudges are produced by the Codex nudge package or
  `assura-codex-nudge` after an Assura JSON report exists. A wrapper decides
  whether to show the nudge as a status line, text guidance, or JSON.

Current Assura installs local Git hooks. It does not yet install a Codex tool
hook that automatically injects nudge text into tool-call responses.

See [Agent Feedback Delivery](/reference/agent-feedback/) for the distinction
between manual CLI proof, Git hooks, nudge wrappers, and future native agent
hooks.

## Manual Hook Alternative

If you prefer to manage hooks yourself, create `.git/hooks/pre-push`:

```bash
#!/usr/bin/env bash
set -euo pipefail

assura check --format text .
```

Then make it executable:

```bash
chmod +x .git/hooks/pre-push
```

## Pre-Commit Framework

If your project uses [pre-commit](https://pre-commit.com/), add a local hook:

```yaml
repos:
  - repo: local
    hooks:
      - id: assura-check
        name: Assura check
        entry: assura check --format text .
        language: system
        pass_filenames: false
        always_run: true
```

Then install it:

```bash
pre-commit install
```

## CI Still Matters

Local hooks are easy to bypass with `--no-verify`, so keep `assura check` in CI
as the source of truth for pull requests.
