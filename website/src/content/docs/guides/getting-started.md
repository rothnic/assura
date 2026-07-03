---
title: Getting Started
description: Install Assura, initialize config, run checks, and wire CI
template: doc
sidebar:
  order: 1
---

Assura validates repository structure from `.assura/config.yml`. The primary
supported command is `assura check`.

If a coding agent is entering a new repository, use the dedicated
[Agent-Ready Onboarding](/guides/agent-ready-onboarding/) path first. It creates
a broad baseline, writes `.assura/onboarding/agent-next.md`, and separates
checked capabilities from choices that still need user answers.

## First Run

1. **Install Assura**

   ```bash
   curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
   ```

   The installer downloads a release archive. It does not require a Rust
   toolchain or a source checkout. Windows users can run the PowerShell
   installer from the [Installation guide](/guides/installation/) or download
   the zip from [GitHub Releases](https://github.com/rothnic/assura/releases/latest).

2. **Create a project config**

   From your project root:

   ```bash
   assura init
   ```

   `init` creates `.assura/config.yml` and refuses to overwrite an existing
   config unless `--force` is provided.

3. **Author a minimal policy**

   Replace the generated starter with a focused policy for the project shape
   you want to keep:

   ```yaml
   structure:
     ./:
       extra: false
       README.md: exists:1
       Cargo.toml: exists:1
       src/: exists:1
       "*.lock": exists:0-1
     src/:
       .rs: snake_case
   exclude:
     - "target/**"
   ```

   Use `exists:1` for required direct children, `exists:0-1` for optional
   singletons, extension keys such as `.rs` for direct file naming, and
   `extra: false` when unexpected root files should fail the check.

4. **Run validation**

   ```bash
   assura check
   ```

5. **Create an intentional failure**

   Add a file that violates your generated naming rules, then run:

   ```bash
   assura check
   ```

   Assura exits nonzero when violations are present.

6. **Fix and re-run**

   Rename or move the file so it matches `.assura/config.yml`, then run:

   ```bash
   assura check --format text
   ```

## JSON Output

Use JSON when CI or another tool needs a machine-readable report:

```bash
assura check --format json .
```

Example shape:

```json
{
  "success": true,
  "project_root": "/workspace/my-project",
  "config_path": "/workspace/my-project/.assura/config.yml",
  "checked_path": "/workspace/my-project",
  "files_checked": 12,
  "dirs_checked": 4,
  "violations": []
}
```

The raw report formats for `assura check` are `text`, `json`, and `yaml`.

## CI

Use the same command locally and in CI:

```yaml
name: Assura

on:
  pull_request:
  push:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sudo env BIN_DIR=/usr/local/bin sh
      - run: assura check --format text
```

## LS-Lint Migration

If a project already has `.ls-lint.yml`, convert it and then run Assura:

```bash
assura migrate .ls-lint.yml --output .assura/config.yml
assura check
```

See [LS-Lint Migration](/guides/ls-lint-migration/) for a complete example.
See [Adoption Walkthrough](/guides/adoption-walkthrough/) for the release-smoke
journey that covers empty projects, LS-Lint migration, and recovery cases.

## Guided Feedback

Assura can render either raw reports or guided output from the same check:

```bash
assura check --format advice .
assura check --format status .
assura check --format agent . --warn --min-severity medium --max-issues 5
```

For local Git feedback, run:

```bash
assura hooks install
assura hooks status
assura hooks verify
```

Codex users can opt into native hook feedback by wiring the hook command into
their Codex `UserPromptSubmit` hooks:

```bash
assura check --format agent --agent codex . --warn --min-severity medium --max-issues 5
```

Codex must also have hooks enabled in user config with `features.hooks = true`;
run `/hooks` once and approve the project hook command before expecting
feedback from `assura check --format agent --agent codex`.

> **Current release scope**
>
> Git hooks rerun Assura on Git events such as commit and push. They do not run
> after every file edit, install Codex hooks automatically, start a daemon, or
> replace repo-local `.agents/skills/` guidance. For Git hooks, use `--warn`
> when you want advisory reporting that exits successfully.
>
> The Codex hook command does not install itself, reuse a daemon/editor session,
> or replace repo-local `.agents/skills/` guidance. `--warn` makes it advisory;
> omit `--warn` only when your workflow should block on validation failures.

See [Real Project Feedback](/examples/real-project-feedback/) for a complete
policy, hook, check, feedback, and rerun walkthrough. See
[Agent Feedback Delivery](/reference/agent-feedback/) for the difference
between manual CLI proof, Git hooks, feedback wrappers, optional Codex prompt
hooks, future editor/daemon integrations, and warm-session reuse.
