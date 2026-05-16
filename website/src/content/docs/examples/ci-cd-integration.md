---
title: CI/CD Integration
description: Run Assura in continuous integration
template: doc
sidebar:
  order: 3
---

Run `assura check` in CI to reject repository shape drift before merge.

> **Note**
>
> Until a published binary or action is available for your environment, install
> Assura with Cargo in the job. In this repository, use `cargo install --path .`
> from the checked-out source.

## GitHub Actions

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
      - uses: Swatinem/rust-cache@v2
      - run: cargo install --path .
      - run: assura check --format text .
```

## JSON Artifact

Use JSON when you want a machine-readable report:

```yaml
name: Assura JSON

on:
  pull_request:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo install --path .
      - name: Run Assura
        run: assura check --format json . > assura-report.json
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: assura-report
          path: assura-report.json
```

The JSON report contains:

```json
{
  "success": false,
  "project_root": "/home/runner/work/example/example",
  "config_path": "/home/runner/work/example/example/.assura/config.yml",
  "checked_path": "/home/runner/work/example/example",
  "files_checked": 4,
  "dirs_checked": 2,
  "violations": [
    {
      "path": "/home/runner/work/example/example/BadName.ts",
      "rule": "file_naming",
      "message": "File name 'BadName' does not match kebab-case",
      "severity": "medium"
    }
  ]
}
```

## GitLab CI

```yaml
stages:
  - validate

assura:
  stage: validate
  image: rust:latest
  before_script:
    - cargo install --path .
  script:
    - assura check --format text .
```

## Parsing JSON

When you need a custom summary, parse `.violations`:

```bash
assura check --format json . > assura-report.json || status=$?
jq '.violations | length' assura-report.json
exit "${status:-0}"
```

- `--format text`: human-readable CI logs.
- `--format json`: artifacts and scripted summaries.
- `--format yaml`: automation that prefers YAML.
