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
> Release archives include `assura` and its `assura-full` companion. Keep both
> files on `PATH`; use Cargo only for source builds or local development.

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
      - run: curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sudo env BIN_DIR=/usr/local/bin sh
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
      - run: curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sudo env BIN_DIR=/usr/local/bin sh
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
  image: ubuntu:latest
  before_script:
    - apt-get update && apt-get install -y curl
    - curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | BIN_DIR=/usr/local/bin sh
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
