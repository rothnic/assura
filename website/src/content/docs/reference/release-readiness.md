---
title: Release Readiness
description: Supported release surfaces, install artifacts, and pre-1.0 compatibility policy
---

Assura's pre-1.0 release surface is intentionally narrow. The supported path is
an installable CLI that validates repository structure from `.assura/config.yml`
and reports results through stable local formats.

## Supported Commands

| Command | Status |
| --- | --- |
| `assura check` | Supported structure validation. |
| `assura check --format json` | Supported automation output. |
| `assura check --format yaml` | Supported automation output. |
| `assura check --format agent` | Supported agent feedback output. |
| `assura check --format agent --agent codex` | Supported Codex delivery adapter. |
| `assura init` | Supported starter config creation. |
| `assura status --format json` | Supported project/config summary. |
| `assura migrate` | Supported LS-Lint migration for documented rules. |
| `assura hooks` | Supported local git hook workflow. |
| `assura performance-report` | Supported evidence command. |
| `assura watch` | Experimental until watch-mode tests and docs are added. |

## Install Artifacts

Release archives include `assura` and its internal `assura-full` companion.
Keep both files together and run `assura`.

| Platform | Archive |
| --- | --- |
| Linux x86_64 | `assura-linux-amd64.tar.gz` |
| macOS Apple Silicon | `assura-macos-arm64.tar.gz` |
| macOS Intel | `assura-macos-amd64.tar.gz` |
| Windows x86_64 | `assura-windows-amd64.zip` |

Release automation also builds `assura-linux-musl-amd64.tar.gz` for Linux musl.
Each release archive has a sibling `.sha256` checksum file generated and
verified by release automation.

## Agent Feedback

The stable agent feedback command is:

```bash
assura check --format agent .
```

Codex delivery stays on the same format:

```bash
assura check --format agent --agent codex .
```

Assura does not ship package feedback CLIs, per-agent command names, or
per-agent `--format` values.

## Custom Constraints

`extensions.custom_constraints` is experimental and first-party in the pre-1.0
release line. The supported example is `paired_file_exists`, which runs through
`assura check` and reports normal diagnostics with `custom:<id>` rule names.

Remote plugin loading, marketplace behavior, shell-executed plugins, and
third-party plugin APIs are not release surfaces.

## Pre-1.0 Policy

Before 1.0, configuration fields and experimental surfaces can change. Release
notes must call out breaking changes, removed experimental surfaces, and the
validation evidence behind compatibility claims.

Maintainer-facing details live in the repository:

- [Release notes](https://github.com/rothnic/assura/blob/master/docs/release-notes.md)
- [Release candidate checklist](https://github.com/rothnic/assura/blob/master/docs/release-candidate-checklist.md)
- [Support policy](https://github.com/rothnic/assura/blob/master/docs/support-policy.md)
- [Compatibility matrix](https://github.com/rothnic/assura/blob/master/docs/compatibility-and-surface.md)
