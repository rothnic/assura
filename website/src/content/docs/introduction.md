---
title: Introduction
description: Welcome to Assura
template: doc
sidebar:
  order: 1
---

import { Card, CardGrid, Aside, LinkButton } from '@astrojs/starlight/components';

Assura is a pre-1.0 structure validation CLI written in Rust. It checks that a
repository matches the shape described in `.assura/config.yml`.

<CardGrid>
  <Card title="Structure-First Config" icon="document">
    Describe allowed files, directories, naming rules, and existence checks in one config.
  </Card>
  <Card title="LS-Lint Migration" icon="seti:yaml">
    Convert supported LS-Lint 2.3 naming and exists rules into Assura config.
  </Card>
  <Card title="CI-Friendly Reports" icon="list-format">
    Use text locally and JSON or YAML for automation.
  </Card>
  <Card title="Agent Nudge MVP" icon="rocket">
    Turn Assura JSON reports into advisory Codex/agent guidance.
  </Card>
</CardGrid>

## Quick Start

```bash
cargo install assura
assura init
assura check
```

<LinkButton href="/guides/getting-started/" variant="primary">
  Get Started
</LinkButton>

## Current Scope

Assura currently focuses on truthful CLI validation, structure-first project
shape checks, LS-Lint migration, and reproducible benchmark evidence.

<Aside type="note" title="Agent nudge MVP">
  The Codex integration package provides a small advisory nudge MVP. Automatic
  hook installation and complete agent orchestration remain future work.
</Aside>

## Help

- [GitHub repository](https://github.com/rothnic/assura)
- [Getting Started](/guides/getting-started/)
- [LS-Lint Migration](/guides/ls-lint-migration/)
