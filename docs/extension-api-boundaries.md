---
title: Extension API Boundaries
status: active
---

# Extension API Boundaries

This page is the canonical boundary for Assura extension and plugin language in
the beta line.

Assura currently has three different integration categories that must not be
conflated:

1. First-party config extension policies under `extensions.*`.
2. Supported local CLI, daemon, content, agent, and editor JSON contracts.
3. Internal Rust modules used by Assura's binaries, tests, and benchmarks.

Assura does not currently provide a public third-party plugin API, remote
plugin loader, shell-executed validator system, plugin marketplace, TypeScript
plugin SDK, or semver-stable Rust library API.

## First-Party Config Extension Policies

`extensions.*` means first-party configuration policy families that are parsed
from `.assura/config.yml` and executed by `assura check`. These policies are
deterministic, local, and bounded by configured files, paths, surfaces, and
expected values.

They are "extensions" because they model cross-file policy that does not fit
the directory tree itself. They are not third-party plugins and they do not load
untrusted code from a repository.

| Config family | Status | Boundary |
| --- | --- | --- |
| `extensions.custom_constraints` | Experimental first-party | Specialized built-in constraints only. Prefer `structure` captures, `exists:1`, `needs`, and `provides` for common relationships. No shell execution or remote loading. |
| `extensions.release_contracts` | Experimental first-party | Release artifact and documentation synchronization checks for this repo's release train. Not a package publisher or release automation service. |
| `extensions.support_matrices` | Experimental first-party | Explicit support classification for commands, Rust export families, docs tables, packages, and binaries. Not a semantic-versioning guarantee. |
| `extensions.manifest_semantics` | Experimental first-party | Cargo manifest metadata checks for configured packages and binaries. Not a replacement for Cargo, license scanning, dependency hygiene, or semver tooling. |
| `extensions.test_relationships` | Experimental first-party | Source/test evidence, manual-test exceptions, and fixture-family ownership checks. Not a coverage percentage or semantic test-adequacy claim. |
| `extensions.module_topologies` | Experimental first-party | Rust module-family ownership, root existence, public export classification, and internal visibility checks. Not a full Rust parser or public API guarantee. |
| `extensions.docs_lifecycles` | Experimental first-party | Documentation lifecycle, frontmatter status, historical exception, and deterministic claim-evidence checks. Not broad natural-language stale-prose detection. |
| `extensions.repository_references` | Experimental first-party | Opt-in diagnostics for locally provable repository-reference target, heading-anchor, and line-anchor failures. Lower-confidence candidates remain graph context, not validation truth. |
| `extensions.agent_guidance` | Experimental first-party | Opt-in `AGENTS.md` and project-local `SKILL.md` routing, progressive-disclosure, and supporting-reference checks. Not a global skill registry or host-agent-specific validation engine. |
| `extensions.relationships` | Internal generated policy | Normalized capture relationships produced from concise `structure` notation. Users should author captures, `exists:1`, `needs`, and `provides` instead of hand-writing this family. |

## Supported Local Contracts

Stable automation should target Assura's local command contracts rather than a
plugin API:

- `assura check --format json|yaml|agent|advice|status`
- `assura content ...`
- `assura content session`
- `assura agent ...`
- `assura editor session`
- `assura daemon ... --format json`

These contracts can be wrapped by local agent hooks, local editor adapters, CI,
or scripts. Wrappers must call the shared contracts and must not embed private
validators or create per-agent validation logic.

## Internal Rust APIs

Public Rust module visibility in `src/lib.rs` exists for Assura's binaries,
tests, benchmark harnesses, and internal code organization. It does not create
a semver-stable library API before 1.0.

The current `rust:*` support-matrix rows are support classifications for this
repository, not a public SDK promise.

## Deferred Public Plugin API

A real public plugin API is deferred for this beta increment. Reopening it
requires a separate implementation goal with proof gates for:

- sandboxing and security review;
- versioned plugin protocol and compatibility policy;
- distribution and provenance, without implying a marketplace by default;
- deterministic diagnostics, severity, suppression, and reporting contracts;
- performance budgets and no-slower fixture evidence;
- tests proving plugins cannot bypass staged validation or mutate files
  implicitly.

Until that goal exists and is completed, Assura docs must describe public
plugins, remote plugin loading, shell-executed validators, plugin marketplaces,
TypeScript plugin APIs, and semver-stable Rust APIs as unsupported or deferred.
