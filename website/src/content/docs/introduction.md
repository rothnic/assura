---
title: Introduction
description: Welcome to Assura
template: doc
sidebar:
  order: 1
---

Assura is a pre-1.0 local project validation CLI written in Rust. It turns the
project shape described in `.assura/config.yml` into fast deterministic checks,
an advisory branch review, and bounded feedback for coding agents.

- **Structure-First Config**: Describe allowed files, directories, naming
  rules, and existence checks in one config.
- **LS-Lint Migration**: Convert LS-Lint 2.3 naming, regex, exists, ignore,
  wildcard extension, and directory-scope rules into Assura config.
- **CI-Friendly Reports**: Use text locally and JSON or YAML for automation.
- **Agent-Ready Feedback**: Use Review while work is changing, Check as the
  configured gate, and managed project-local integrations for supported hosts.

## Quick Start

```bash
curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
assura init
assura check
```

[Get Started](/guides/getting-started/)

## Current Scope

The current source contract covers deterministic structure,
Markdown, references, severity, suppressions, and agent-guidance checks. Review
summarizes branch and worktree pressure without replacing the authoritative
Check gate.

> **Agent workflow**
>
> `assura agent onboard . --agent codex --activate --format json` creates the
> project-owned baseline and explicitly activates the managed Codex integration.
> OpenCode, Claude, and Pi use the same lifecycle. Full autonomous orchestration,
> remote execution, and inference-based validation remain outside the local
> deterministic contract.

## Help

- [GitHub repository](https://github.com/rothnic/assura)
- [Getting Started](/guides/getting-started/)
- [LS-Lint Migration](/guides/ls-lint-migration/)
