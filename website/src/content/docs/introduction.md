---
title: Introduction
description: Welcome to Assura
template: doc
sidebar:
  order: 1
---

Assura is a pre-1.0 structure validation CLI written in Rust. It checks that a
repository matches the shape described in `.assura/config.yml`.

- **Structure-First Config**: Describe allowed files, directories, naming
  rules, and existence checks in one config.
- **LS-Lint Migration**: Convert LS-Lint 2.3 naming, regex, exists, ignore,
  wildcard extension, and directory-scope rules into Assura config.
- **CI-Friendly Reports**: Use text locally and JSON or YAML for automation.
- **Guided Feedback**: Use `assura check --format advice` or `--format status`
  for concise repair guidance.

## Quick Start

```bash
curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
assura init
assura check
```

[Get Started](/guides/getting-started/)

## Current Scope

Assura currently focuses on truthful CLI validation, structure-first project
shape checks, LS-Lint migration, and reproducible benchmark evidence.

> **Agent feedback MVP**
>
> `assura check` can render guided advice or one-line status output. The
> agent-feedback package also includes an optional native Codex
> `UserPromptSubmit` hook command. Automatic Codex hook installation,
> daemon/editor support, and complete agent orchestration remain future work.

## Help

- [GitHub repository](https://github.com/rothnic/assura)
- [Getting Started](/guides/getting-started/)
- [LS-Lint Migration](/guides/ls-lint-migration/)
