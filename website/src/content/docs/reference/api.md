---
title: API Reference
description: Current supported integration surface for Assura
template: doc
sidebar:
  order: 2
---

The supported v0.1 API is the command-line interface and its report formats.
Rust library APIs, TypeScript plugin APIs, agent profiles, and custom runtime
constraints are not stable public surfaces in this release.

> **Current scope**
>
> Build automation against `assura check`, `assura status`, and the JSON/YAML
> report fields documented here.

## Command Surface

| Command | Purpose |
| --- | --- |
| `assura check [path]` | Validate a project or subpath |
| `assura status [path]` | Print discovered config and rule summary |
| `assura init [path]` | Create a starter `.assura/config.yml` |
| `assura migrate [.ls-lint.yml]` | Convert supported LS-Lint config |
| `assura info [path]` | Print text configuration details |
| `assura watch [path]` | Run one check as a current watch wrapper |

Supported check and status formats are `text`, `json`, and `yaml`.

## Check JSON Shape

```json
{
  "success": true,
  "project_root": "/work/example",
  "config_path": "/work/example/.assura/config.yml",
  "checked_path": "/work/example",
  "files_checked": 12,
  "dirs_checked": 4,
  "violations": []
}
```

Violation entries use this shape:

```json
{
  "path": "/work/example/src/BadName.rs",
  "rule": "file_naming",
  "message": "File name 'BadName' does not match kebab-case",
  "severity": "medium"
}
```

## Status JSON Shape

```json
{
  "project_root": "/work/example",
  "config_path": "/work/example/.assura/config.yml",
  "configured_directories": 2,
  "configured_file_rules": 3,
  "configured_markdown_rules": 0,
  "exclusions": ["target/**", "node_modules/**"]
}
```

## Recommended Automation

```bash
assura check --format json . > assura-report.json
```

Fail the job when the command exits nonzero. Parse `violations` only when you
need a custom summary or uploaded artifact.

## Future APIs

The roadmap includes richer agent nudges, quality measurement, and extension
points. Those capabilities should be documented as future work until the repo
contains a tested public API for them.
