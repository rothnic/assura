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
curl -L https://github.com/rothnic/assura/releases/latest/download/assura-linux-amd64.tar.gz | tar xz
sudo install -m 755 assura assura-full /usr/local/bin/
assura init
assura check
```

## Structure-First Config

```yaml
structure:
  ./:
    files:
      naming: kebab-case
    directories:
      naming: kebab-case
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
      - run: curl -L https://github.com/rothnic/assura/releases/latest/download/assura-linux-amd64.tar.gz | tar xz
      - run: sudo install -m 755 assura assura-full /usr/local/bin/
      - run: assura check --format text
```

## Roadmap

Agent nudges, richer quality measurement, and editor/agent integrations are
future work. They are not required for the current pre-1.0 onboarding release.
