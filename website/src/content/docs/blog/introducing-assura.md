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
curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
assura init
assura check
```

## Structure-First Config

```yaml
rules:
  "@source-file":
    naming: kebab-case
    max_lines: 500

structure:
  ./:
    .ts: "@source-file"
    .tsx: "@source-file"
    src/: exists:1
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
      - run: curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sudo env BIN_DIR=/usr/local/bin sh
      - run: assura check --format text
```

## Roadmap

Agent feedback, richer quality measurement, and editor/agent integrations are
future work. They are not required for the current pre-1.0 onboarding release.
