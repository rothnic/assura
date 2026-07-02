---
title: Extension API Boundaries
description: What Assura means by extensions, plugins, and API support
template: doc
sidebar:
  order: 3
---

Assura uses extension language in a narrow way during the beta line. Current
`extensions.*` configuration entries are first-party policy families executed
by `assura check`. They are not a public third-party plugin API.

## Current Categories

| Category | Status | Use it for |
| --- | --- | --- |
| `extensions.*` config policies | Experimental first-party | Deterministic local repository policies that do not fit ordinary directory-tree validation. |
| CLI, daemon, content, agent, and editor JSON contracts | Supported or experimental by command | Local automation, CI, editor adapters, and agent hooks. |
| Rust modules exported from `src/lib.rs` | Internal/unstable | Assura binaries, tests, and benchmarks before a separate API stabilization decision. |
| Public plugin API or SDK | Roadmap only | Future work that still needs sandboxing, versioning, distribution, security, and performance proof gates. |

## First-Party Config Policies

Current config families under `extensions.*` are local and deterministic:

- `extensions.custom_constraints`
- `extensions.release_contracts`
- `extensions.support_matrices`
- `extensions.manifest_semantics`
- `extensions.test_relationships`
- `extensions.module_topologies`
- `extensions.docs_lifecycles`
- `extensions.repository_references`
- `extensions.relationships`

These policies parse configured paths, surfaces, expected values, and evidence
files. They do not load repository-provided code and they do not run arbitrary
shell commands.

## Not Supported Today

Assura does not currently support:

- remote plugin loading;
- shell-executed validation plugins;
- plugin marketplaces;
- TypeScript plugin APIs;
- semver-stable Rust library APIs;
- hosted extension registries.

Use the supported local command contracts for integration work. If a public
plugin API is reopened later, it needs a separate goal with security,
sandboxing, versioning, distribution, diagnostics, and performance gates.

Repository source of truth:
[`docs/extension-api-boundaries.md`](https://github.com/rothnic/assura/blob/master/docs/extension-api-boundaries.md).
