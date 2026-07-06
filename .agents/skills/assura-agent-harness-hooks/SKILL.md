---
name: assura-agent-harness-hooks
description: "Research, implement, compare, and update Assura agent harness hook integrations for Codex, OpenCode, Claude, Pi, OpenClaw, or similar coding-agent runtimes. Use when changing agent hook bundles, adding a harness target, validating hook lifecycle coverage, or capturing hook API learnings."
---

# Assura Agent Harness Hooks

## Overview

Use this skill to keep Assura hook integrations consistent across agent
harnesses without pretending every harness exposes the same API. The goal is a
shared Assura nudge contract with thin, versioned, harness-specific delivery
adapters.

## Workflow

1. **Classify the harness**: Identify whether the target supports native hooks,
   plugins, extensions, command wrappers, or only documented manual wiring.
2. **Verify the current API**: Prefer local repo source or official docs. Do not
   rely on old task notes for hook names, payload fields, or install paths.
3. **Map events to Assura**: Convert available lifecycle events to Assura
   `session-start`, `before-tool`, `after-tool`, `file-read`, `idle`, and
   `recovery` events. Mark unsupported events explicitly.
4. **Choose the distribution boundary**: Classify code as Rust core logic,
   shared adapter contract, generated shared helper, or harness-specific
   adapter. Do not duplicate validation or nudge policy in TypeScript/Python
   adapters.
5. **Preserve the shared contract**: Keep validation in
   `assura agent nudge` and `assura check --format agent`; adapters only bridge
   harness protocol, changed-path detection, and context injection.
6. **Update lifecycle behavior**: If generated hook bundles or adapters are
   managed by Assura, install/update should refresh managed stale files and
   doctor/status should warn or fail clearly on drift.
7. **Capture learnings**: Update
   `references/harness-hook-matrix.md` whenever a hook API, payload shape,
   install path, or limitation is verified.

## Required Checks

- Search current code for every target enum, CLI value list, generated file
  list, config allowlist, and test fixture before adding a harness.
- Keep line-limit fixes natural. If a hook adapter or generator grows too large,
  split by responsibility instead of shortening names or collapsing logic.
- Treat "not available" as a real implementation state. Generate a wrapper and
  documented fallback rather than claiming native post-tool hooks exist.
- Record adapter artifact version, adapter contract version, minimum Assura
  version, runtime requirements, and update path before adding generated code.
- Runtime logs belong under ignored `.assura/agent-sessions/*.jsonl`; managed
  hook bundles belong under `.assura/integrations/<harness>/`.

## References

- Read `.trellis/spec/assura/agent-harness-hooks.md` for the durable product
  contract on distribution, versioning, lifecycle, and update behavior.
- Read `references/harness-hook-matrix.md` before changing or claiming support
  for Codex, OpenCode, Claude, Pi, or OpenClaw.
- When a harness has dedicated local instructions, read those too:
  OpenCode plugin work should use `create-opencode-plugin`; Pi-agent work should
  use `pi-agent-rust`.

## Done Criteria

- Harness support has a manifest entry, wrapper behavior, install/update/status
  semantics, doctor drift checks, and tests.
- Generated artifacts declare compatibility, runtime needs, managed ownership,
  and whether updates are tied to the Assura binary or a harness package.
- The matrix says which hooks are native, adapted, or unsupported and cites the
  source of truth used for that claim.
- Agent-facing output remains compact and advisory unless the existing Assura
  validation command is intentionally run in blocking mode.
