---
title: "Introducing Assura"
description: "Assura is a pre-1.0 structure validation CLI for repository shape checks."
published: 2026-03-19
author: "Assura Team"
---

# Introducing Assura

Assura is a Rust CLI for validating that a repository matches the structure
described in `.assura/config.yml`.

## Current Supported Flow

```bash
curl -fsSL https://assura.dev/install.sh | sh
assura init --recipe agentic-core --recipe structure-health
assura check
```

## Structure-First Config

```yaml
rules:
  source-file:
    naming: kebab-case
    max_lines: 500

structure:
  src/: exists:1
  ./**/:
    .{ts,tsx}: $source-file
exclude:
  - "target/**"
```

## LS-Lint Migration

```bash
assura migrate .ls-lint.yml --output .assura/config.yml
assura check
```

## CI

```yaml
name: Assura
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: curl -fsSL https://assura.dev/install.sh | sudo env BIN_DIR=/usr/local/bin sh
      - run: assura check --format text
```

## Roadmap

Project review, agent feedback formats, and editor integrations build on the
same deterministic structure policy. The CLI remains local and pre-1.0.
