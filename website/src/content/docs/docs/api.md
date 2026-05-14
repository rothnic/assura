---
title: API Reference
description: Supported command and report surfaces for Assura v0.1
---

import { Aside } from '@astrojs/starlight/components';

Assura v0.1 supports the CLI as the public integration surface. Treat Rust
library internals as unstable until a later release documents a stable API.

<Aside type="caution">
  Do not build integrations against undocumented Rust structs or plugin traits.
  Use `assura check --format json` or `assura check --format yaml` for v0.1
  automation.
</Aside>

## Commands

```bash
assura check [path] [--format text|json|yaml]
assura status [path] [--format text|json|yaml]
assura init [path] [--force] [--no-git-hooks]
assura migrate [.ls-lint.yml] --output .assura/config.yml
assura info [path]
assura watch [path]
```

`assura check` is the primary validation command. `assura watch` currently runs
one check and exits with the same status as `check`.

## Exit Codes

| Code | Meaning |
| --- | --- |
| `0` | Validation succeeded |
| `1` | Validation completed and found violations |
| `2` | Configuration error |
| `3` | Runtime error |
| `4` | No config found |

## JSON Check Report

```json
{
  "success": false,
  "project_root": "/work/example",
  "config_path": "/work/example/.assura/config.yml",
  "checked_path": "/work/example",
  "files_checked": 3,
  "dirs_checked": 1,
  "violations": [
    {
      "path": "/work/example/BadName.ts",
      "rule": "file_naming",
      "message": "File name 'BadName' does not match kebab-case",
      "severity": "medium"
    }
  ]
}
```

The same fields are emitted for YAML reports.

## Status Report

`assura status --format json` reports the project root, config path, configured
directory count, configured rule count, markdown rule count, and exclusions.
Use it to confirm that Assura found the expected config before running checks
in CI.

## Stable Integration Pattern

```bash
assura check --format json . > assura-report.json
```

Then parse `.success` and `.violations` in your CI or local script.
