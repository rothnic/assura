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
| `assura migrate [.ls-lint.yml ...]` | Convert LS-Lint 2.3 rule config |
| `assura info [path]` | Print text configuration details |
| `assura watch [path]` | Run one check as a current watch wrapper |

Supported check formats are `text`, `json`, `yaml`, `advice`, and `status`.
Supported status formats are `text`, `json`, and `yaml`.

## Check Options

| Option | Purpose |
| --- | --- |
| `--warn` | Print violations but exit successfully, useful for advisory agent feedback and gradual adoption |
| `--fail-fast` | Stop after the first violation |
| `--no-parallel` | Run validation without parallel traversal |
| `--ls-lint-target-semantics` | Match LS-Lint path-argument behavior by checking only the explicit target path |
| `--min-severity low|medium|high|critical` | Hide lower-severity feedback items without changing what is checked |
| `--max-issues <count>` | Cap displayed feedback items without changing what is checked |
| `--agent generic|codex` | Select a delivery adapter for `--format agent`; Codex wraps feedback for `UserPromptSubmit` |

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

For guided local output, use:

```bash
assura check --format advice .
assura check --format status .
assura check --format agent .
assura check --format agent --agent codex . --warn
```

## Future APIs

The roadmap includes native agent hooks, hot editor sessions, quality
measurement, and extension points. Those capabilities should be documented as
future work until the repo contains a tested public API for them.
