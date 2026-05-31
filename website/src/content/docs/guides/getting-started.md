---
title: Getting Started
description: Install Assura, initialize config, run checks, and wire CI
template: doc
sidebar:
  order: 1
---

Assura validates repository structure from `.assura/config.yml`. The primary
supported command is `assura check`.

## First Run

1. **Install Assura**

   ```bash
   curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
   ```

   The installer supports Linux x64, macOS Apple Silicon, and macOS Intel.
   Windows users can run the PowerShell installer from the
   [Installation guide](/guides/installation/) or download the zip from
   [GitHub Releases](https://github.com/rothnic/assura/releases/latest).

2. **Create a project config**

   From your project root:

   ```bash
   assura init
   ```

   `init` creates `.assura/config.yml` and refuses to overwrite an existing
   config unless `--force` is provided.

3. **Run validation**

   ```bash
   assura check
   ```

4. **Create an intentional failure**

   Add a file that violates your generated naming rules, then run:

   ```bash
   assura check
   ```

   Assura exits nonzero when violations are present.

5. **Fix and re-run**

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

The supported `assura check` formats are `text`, `json`, and `yaml`.

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

## Guided Feedback

Assura can render either raw reports or guided output from the same check:

```bash
assura check --format advice .
assura check --format status .
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
node /absolute/path/to/assura/integrations/agents/codex/dist/hook-cli.js --path . --min-severity medium --max-messages 5 --block-mode off
```

The Assura release installer installs only the `assura` CLI. The hook command
comes from the separate `@assura/agent-feedback` npm package once published or
a local build of `integrations/agents/codex`. Codex must also have hooks enabled
in user config with `features.hooks = true`; run `/hooks` once and approve the
project hook command before expecting feedback.

> **Current release scope**
>
> Git hooks rerun Assura on Git events such as commit and push. They do not run
> after every file edit, install Codex hooks automatically, start a daemon, or
> replace repo-local `.agents/skills/` guidance. For Git hooks, use `--warn`
> when you want advisory reporting that exits successfully.
>
> The Codex hook command does not install itself, reuse a daemon/editor session,
> or replace repo-local `.agents/skills/` guidance. Treat it as advisory unless
> your workflow opts into hook blocking.

See [Real Project Feedback](/examples/real-project-feedback/) for a complete
policy, hook, check, feedback, and rerun walkthrough. See
[Agent Feedback Delivery](/reference/agent-feedback/) for the difference
between manual CLI proof, Git hooks, feedback wrappers, optional Codex prompt
hooks, future editor/daemon integrations, and warm-session reuse.
