---
title: Why Assura?
description: When to use the current Assura pre-1.0 release
template: doc
sidebar:
  order: 2
---

Assura v0.1 is a structure-first repository validator. It is useful when you
want a repository shape to be explicit, checked locally, and enforced in CI.

The current supported workflow is intentionally small:

- create `.assura/config.yml` with `assura init`
- validate with `assura check`
- migrate LS-Lint 2.3 projects with `assura migrate`
- consume `text`, `json`, or `yaml` reports in local scripts and CI

> **Pre-1.0 scope**
>
> This release focuses on truthful onboarding and LS-Lint-compatible structure
> checks. Advisory guided output and stable agent feedback are supported through
> `assura check` formats. Optional Codex hook feedback is available only when
> users wire `assura check --format agent --agent codex` into Codex hooks.
> Automatic Codex hook installation, custom plugin APIs, role profiles,
> daemon/editor support, and quality scoring are roadmap items, not supported
> v0.1 features.

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
- **Benchmark Evidence**: The checked benchmark report compares the current
  `assura check` path against `@ls-lint/ls-lint@2.3.0` on the
  `realistic-equivalent` LS-Lint-compatible fixture cohort. Optional pinned
  real-repository rows can be generated as extended evidence, but they are not
  part of the current checked public report.

## When Assura Fits

Use Assura now when you need:

- repository naming and structure rules to be executable
- CI to reject files that do not match the allowed project shape
- migration from LS-Lint configuration into a more explicit Assura config
- JSON output that can be inspected by scripts or uploaded as a CI artifact

Assura is especially useful for projects that want to define the allowed
repository shape before adding more agent- or quality-oriented feedback.

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

`assura check --format advice`, `--format status`, and `--format agent` provide
the supported guided and agent feedback paths. Codex `UserPromptSubmit` hooks
can inject feedback through `assura check --format agent --agent codex` when
users wire that command manually. The lower-level integration package is a
library helper for wrappers that already have JSON reports; it does not expose
a separate feedback CLI. Automatic Codex hook installation, daemon/editor
support, and complete agent orchestration remain future work.

## Start Here

- [Getting Started](/guides/getting-started/): Install from a release archive,
  initialize config, run checks, fix a failure, and wire CI.
- [LS-Lint Migration](/guides/ls-lint-migration/): Convert an LS-Lint 2.3
  `.ls-lint.yml` file and validate the migrated project.
- [Configuration Reference](/reference/configuration/): Supported fields for
  the current structure-first config.
