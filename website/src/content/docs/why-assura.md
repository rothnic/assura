---
title: Why Assura?
description: When to use the current Assura pre-1.0 release
template: doc
sidebar:
  order: 2
---

Assura is a structure-first project validator for AI-assisted development. It
is useful when project shape, content expectations, and agent guidance should
be explicit, checked locally, and enforced consistently before merge.

The supported workflow has distinct jobs:

- create `.assura/config.yml` with `assura init`
- inspect branch and worktree pressure with `assura review`
- enforce configured policy with `assura check`
- onboard and explicitly activate supported project-local agent integrations
  with `assura agent onboard --activate`
- migrate LS-Lint 2.3 projects with `assura migrate`
- consume human, JSON, YAML, and agent reports in local tools and CI

> **Pre-1.0 scope**
>
> The current source contract keeps deterministic validation separate from
> advisory review and inferred context. Managed Codex, OpenCode, Claude, and Pi
> activation is explicit and project-local. Remote orchestration, public plugin
> APIs, hosted execution, and automatic repair remain outside the supported
> core product.

## The Problem

Repository structure rules are often scattered across READMEs, review comments,
and team habits. That makes drift easy:

- files use different naming conventions in different directories
- generated output accidentally enters validation paths
- required files are noticed only after a review
- existing LS-Lint users cannot easily compare a replacement tool against the
  same fixtures

Assura turns those expectations into a checked project shape.

## Current Strengths

- **Structure Rules**: Validate file names, directory names, required files,
  forbidden files, and direct-child count rules from one config file.
- **LS-Lint Migration**: Convert LS-Lint 2.3 naming, regex, exists, ignore,
  wildcard extension, and directory-scope rules into `.assura/config.yml`.
- **Automation Output**: Use text output for people and JSON/YAML reports for
  scripts, CI jobs, and release evidence.
- **Agent Workflow**: Use bounded event feedback while editing, compact Review
  before handoff, and Check as the configured merge gate.
- **Benchmark Evidence**: The checked benchmark report compares the current
  `assura check` path against `@ls-lint/ls-lint@2.3.0` on the
  `realistic-equivalent` LS-Lint-compatible fixture cohort. Optional pinned
  real-repository rows can be generated as extended evidence, but they are not
  part of the current checked public report.

## When Assura Fits

Use Assura now when you need:

- repository naming and structure rules to be executable
- Markdown, local links, references, severity, suppressions, and agent guidance
  to use the same deterministic report path
- CI to reject files that do not match the allowed project shape
- migration from LS-Lint configuration into a more explicit Assura config
- JSON output that can be inspected by scripts or uploaded as a CI artifact

Assura is especially useful when multiple coding agents need timely shared
project constraints without repeatedly inferring them from prose.

## When Another Tool May Fit Better

Use another tool alongside Assura when you need:

- language semantics such as TypeScript types, Rust borrow checking, or lint
  diagnostics
- deep security scanning
- code formatting
- custom runtime plugins
- hosted monitoring or remote policy execution

`assura watch` stays resident, coalesces edit bursts, respects configured
exclusions, and reuses a prepared policy. Its JSON stream identifies cold full,
warm affected-path, and warm full-project runs so an agent can distinguish a
fast local signal from a complete project result.

## Agent Feedback Direction

`assura agent nudge` provides bounded event-aware feedback, `assura review`
summarizes advisory branch and worktree state, and
`assura check --format agent` remains the authoritative policy report for
agents. Managed project-local integrations activate those shared commands for
Codex, OpenCode, Claude, and Pi without creating host-specific validation
engines.

## Start Here

- [Getting Started](/guides/getting-started/): Install from a release archive,
  initialize config, run checks, fix a failure, and wire CI.
- [LS-Lint Migration](/guides/ls-lint-migration/): Convert an LS-Lint 2.3
  `.ls-lint.yml` file and validate the migrated project.
- [Configuration Reference](/reference/configuration/): Supported fields for
  the current structure-first config.
