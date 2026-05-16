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
- migrate simple LS-Lint projects with `assura migrate`
- consume `text`, `json`, or `yaml` reports in local scripts and CI

> **Pre-1.0 scope**
>
> This release focuses on truthful onboarding and LS-Lint-compatible structure
> checks. Agent nudges, custom plugin APIs, role profiles, and quality scoring
> are roadmap items, not supported v0.1 features.

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
- **LS-Lint Migration**: Convert supported `.ls-lint.yml` rules into
  `.assura/config.yml` and get clear errors for unsupported directory-scope
  patterns.
- **Automation Output**: Use text output for people and JSON/YAML reports for
  scripts, CI jobs, and release evidence.
- **Benchmark Evidence**: The benchmark suite compares the current
  `assura check` path against `@ls-lint/ls-lint@2.3.0` on the same generated
  fixtures.

## When Assura Fits

Use Assura now when you need:

- repository naming and structure rules to be executable
- CI to reject files that do not match the allowed project shape
- migration from simple LS-Lint configuration into a more explicit Assura config
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
- continuous file watching that stays resident and reacts to every change

The `assura watch` command in this release is a truthful one-shot wrapper over
`assura check`; it is not a long-running file watcher yet.

## Agent Nudge Direction

The Codex integration package provides the first advisory nudge MVP. It can
consume `assura check --format json` output, explain structural failures, and
track comparison metrics for instructions-only, `AGENTS.md`/skills, and Assura
runtime-nudge workflows. Automatic hook installation and complete agent
orchestration remain future work.

## Start Here

- [Getting Started](/guides/getting-started/): Install from source,
  initialize config, run checks, fix a failure, and wire CI.
- [LS-Lint Migration](/guides/ls-lint-migration/): Convert a supported
  `.ls-lint.yml` file and validate the migrated project.
- [Configuration Reference](/reference/configuration/): Supported fields for
  the current structure-first config.
