---
title: Release Readiness
description: Supported release surfaces, install artifacts, and pre-1.0 compatibility policy
---

Assura's pre-1.0 release surface is intentionally narrow. The supported path is
an installable CLI that validates repository structure from `.assura/config.yml`
and reports results through stable local formats. Project Intelligence is also
supported as a local, source-control-friendly workflow for modeled content,
agent handoffs, editor wrappers, a beta local VS Code package, and safe-fix
previews. It does not require MCP, remote access, hosted services, or editor
marketplace publication.

This page describes the current `v0.3.0` beta-increment surface. `v0.3.0` was
published and live-verified on 2026-07-02 with all expected archives and
checksums reachable. Assura remains pre-1.0 beta.

## Supported Commands

| Command | Status |
| --- | --- |
| `assura check` | Supported structure validation. |
| `assura check --format json` | Supported automation output. |
| `assura check --format yaml` | Supported automation output. |
| `assura check --format agent` | Supported agent feedback output. |
| `assura check --format agent --agent codex` | Supported Codex delivery adapter. |
| `assura review` | Supported next-release advisory branch and worktree review. |
| `assura doctor` | Supported next-release setup and inactive-capability diagnosis. |
| `assura explain` | Supported next-release effective path-policy explanation. |
| `assura init` | Supported starter config creation. |
| `assura status --format json` | Supported project/config summary. |
| `assura migrate` | Supported LS-Lint migration for documented rules. |
| `assura hooks` | Supported local git hook workflow. |
| `assura quality plan` | Supported local quality-gate planning. |
| `assura performance-report` | Supported evidence command. |
| `assura fix markdown --dry-run --format json` | Experimental safe-fix preview contract. |
| `assura fix markdown --apply --format json` | Experimental safe-fix apply/audit contract. |
| `assura agent` | Supported local project-intelligence command group for coding agents. |
| `assura agent onboard` | Supported next-release agent-ready baseline and explicit host activation. |
| `assura agent nudge` | Supported next-release bounded event feedback. |
| `assura agent integration` | Supported next-release managed project-local lifecycle for Codex, OpenCode, Claude, and Pi. |
| `assura content` | Supported local project-intelligence query surface. |
| `assura content context-pack` | Supported bounded project-intelligence handoff packet. |
| `assura content references` | Supported repository-reference graph query. |
| `assura content session` | Supported local JSON-line project-intelligence session. |
| `assura editor session` | Supported local JSON-line editor protocol with LSP-shaped methods. |
| `integrations/editors/vscode` | Supported beta local VS Code package over shared Assura CLI, daemon, and editor-session contracts. |
| `.assura/models/**` | Supported layout for project-intelligence model artifacts stored under `.assura/`. |
| `assura watch` | Supported next-release continuous validation with tested warm and fallback behavior. |

## Project Intelligence Schemas

Stable pre-1.0 schema names used by supported local integrations are checked by
integration tests and release-hardening evidence:

| Schema | Produced By | Status |
| --- | --- | --- |
| `assura.project-intelligence.agent-context.v1` | `assura content agent-context` | Supported local wrapper discovery. |
| `assura.project-intelligence.agent-query.v1` | `assura agent diagnostics`, `assura content agent-query` | Supported local agent query envelope. |
| `assura.project-intelligence.context-pack.v1` | `assura agent context-pack`, `assura content context-pack` | Supported bounded handoff packet with diagnostics, search, graph, repository-reference, relation, and safe-fix preview context. |
| `assura.project-intelligence.session.response.v1` | `assura content session`, `assura agent session` | Supported JSON-line session envelope. |
| `assura.project-intelligence.editor.response.v1` | `assura editor session` | Supported local editor-session envelope. |
| `assura.safe-fix.markdown.v1` | `assura fix markdown --dry-run --format json`, `--apply --format json` | Experimental safe-fix preview/apply audit contract. |

These schemas can change before 1.0, but release notes must call out breaking
changes to documented fields.

## Project Intelligence Adoption

The supported local adoption path is:

```bash
assura init --project-intelligence --no-git-hooks .
assura check --format json .
assura content search "Project Intelligence" . --format json
assura agent context-pack . --collection goals --id goal-project-intelligence-starter --text "Project Intelligence" --limit 5
assura editor session .
```

The starter writes model artifacts below `.assura/models/**` and keeps root
`.assura/` files bounded to well-known entrypoints such as `config.yml` and
command-surface contracts. Projects may still keep runtime schema artifacts
outside `.assura/`, such as `schemas/**`, when that fits their repository
layout.

`assura editor session` is LSP-shaped but is not a full LSP server.
`integrations/editors/vscode` is the supported beta local VS Code package and
uses the same shared CLI, daemon, and editor-session contracts. Full LSP server
framing and editor marketplace publication are roadmap-only; hosted language
servers are unsupported, and required MCP adapters or remote Project
Intelligence providers are unsupported roadmap-only surfaces for this release.

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

Local coding agents should use `assura agent ...` for project-intelligence
handoffs. Editors should use `assura editor session` or wrap lower-level
`assura content ...` contracts. MCP may be added later as an optional adapter,
but it is not part of the required local workflow.

`assura agent integration` is a supported next-release project-local lifecycle
surface. Install and update generate reviewable bundles; explicit activate and
deactivate commands patch only Assura-owned host entries, preserve unmanaged
configuration, and call shared Assura commands rather than embedding per-agent
validation logic.

## Rollback And Repair

Safe-fix apply commands are explicit and audit-oriented. Preview first:

```bash
assura fix markdown --dry-run --format json .
```

Apply only after accepting the planned fix IDs:

```bash
assura fix markdown --apply --format json .
```

Rollback is intentionally VCS-first: inspect the changed paths and use normal
source-control rollback if the accepted edit is not desired. Assura does not
perform automatic repair, background writes, or remote mutation.

## Custom Constraints

`extensions.*` entries are first-party configuration policies in the pre-1.0
release line. They run through `assura check` and report normal diagnostics.
Common source/test and package/doc relationships should use `structure`
captures, `exists:1`, `needs`, and `provides`; specialized custom constraints
remain the exception.

Remote plugin loading, marketplace behavior, shell-executed plugins, and
third-party plugin APIs are not release surfaces.

The canonical boundary is
[Extension API Boundaries](/reference/extension-api-boundaries/). Release
readiness must keep that page, the support policy, compatibility matrix, and
release-surface manifest aligned before claiming a new extension or API
surface.

## Pre-1.0 Policy

Before 1.0, configuration fields and experimental surfaces can change. Release
notes must call out breaking changes, removed experimental surfaces, and the
validation evidence behind compatibility claims.

## Readiness Check

Release PRs run:

```bash
cargo xtask website-demo-data --check --released
cargo xtask release-readiness --format json
```

The marketed-release gate rejects a core site claim unless its manifest row is
supported, carries verified or measured evidence, and ships no later than the
local candidate version. Preview evidence uses the non-strict `--check` mode so
candidate capabilities can be reviewed before version preparation.

The command emits `assura.release-readiness.v1` with the latest GitHub release,
local package version, release-notes version, unreleased public-surface entries
from `docs/data/release-surfaces.json`, missing checklist gates, and a pass/fail
verdict. It exits nonzero when the current branch describes installable
surfaces that have not been assigned to the candidate or an earlier release.
The tag workflow runs both checks before building archives, so publication
cannot bypass this contract. Automation should parse JSON from stdout and treat
stderr as diagnostics.

Maintainer-facing details live in the repository:

- [Release notes](https://github.com/rothnic/assura/blob/master/docs/release-notes.md)
- [Release train](https://github.com/rothnic/assura/blob/master/docs/release-train.md)
- [Release surfaces](https://github.com/rothnic/assura/blob/master/docs/data/release-surfaces.json)
- [Release candidate checklist](https://github.com/rothnic/assura/blob/master/docs/release-candidate-checklist.md)
- [Support policy](https://github.com/rothnic/assura/blob/master/docs/support-policy.md)
- [Compatibility matrix](https://github.com/rothnic/assura/blob/master/docs/compatibility-and-surface.md)
