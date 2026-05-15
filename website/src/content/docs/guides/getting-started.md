---
title: Getting Started
description: Install Assura, initialize config, run checks, and wire CI
template: doc
sidebar:
  order: 1
---

import { Steps, Aside } from '@astrojs/starlight/components';

Assura validates repository structure from `.assura/config.yml`. The primary
supported command is `assura check`.

## First Run

<Steps>

1. **Install or build Assura**

   ```bash
   cargo install assura
   ```

   Source build:

   ```bash
   git clone https://github.com/rothnic/assura
   cd assura
   cargo build --release
   ```

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

</Steps>

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
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install assura
      - run: assura check --format text
```

## LS-Lint Migration

If a project already has `.ls-lint.yml`, convert it and then run Assura:

```bash
assura migrate .ls-lint.yml --output .assura/config.yml
assura check
```

See [LS-Lint Migration](/guides/ls-lint-migration/) for a complete example.

## Agent Nudge MVP

The Codex integration package now provides the first advisory nudge MVP. It can
turn Assura JSON output into targeted guidance for a developer or agent:

```bash
assura check --format json . > assura-report.json
assura-codex-nudge --report assura-report.json --format text
```

<Aside type="note" title="Current release scope">
  This MVP does not install Codex hooks automatically and does not replace
  repo-local `.agents/skills/` guidance. Treat it as advisory unless your
  workflow enforces the command exit code.
</Aside>
