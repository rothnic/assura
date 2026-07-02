---
title: Agent integration lifecycle
status: active
priority: P0
---

# Agent Integration Lifecycle

## Goal

Execute `docs/goals/assura-agent-integration-lifecycle.md` as the next child of
the post-beta capabilities program.

This slice gives Codex, OpenCode, Claude, and Pi a supported local lifecycle
surface without adding private validation engines: install/update/remove/status
and doctor commands generate reviewable `.assura/integrations/<agent>/`
bundles that call the shared `assura agent nudge`,
`assura check --format agent`, and `assura daemon` contracts.

## User Story Fit

In the parent verification story, a maintainer wants host agents to receive
short, relevant Assura nudges while editing a documentation-heavy repo. They
should be able to install and inspect the Assura wrapper bundle for each host,
verify it delegates to shared contracts, and remove it without hidden host
configuration changes.

## Scope

- Add `assura agent integration install|update|remove|status|doctor`.
- Generate a stable manifest, wrapper script, and README under
  `.assura/integrations/<agent>/`.
- Support Codex, OpenCode, Claude, and Pi labels.
- Keep host-agent config as manual opt-in guidance.
- Extend event-aware nudges to cover `file-read` and `recovery` placements.
- Update command-surface, support, compatibility, and website docs.

## Acceptance Criteria

- [ ] Dry-run install previews every file write without touching disk.
- [ ] Install/update generate reproducible managed files for all four agents.
- [ ] Remove deletes only Assura-managed bundle files.
- [ ] Status and doctor report bundle readiness and shared command delegation.
- [ ] Generated wrappers contain no per-agent validation logic.
- [ ] Nudge events include session-start, before-tool, after-tool, file-read,
      and recovery.
- [ ] Target-state, docs, evidence, and self-check gates pass.
- [ ] Independent review confirms the adapters reuse shared Assura contracts.

## Validation

```bash
cargo fmt --check
cargo test --test agent_surface_cli --quiet
cargo test --test daemon_cli_tests --quiet
cargo test --test real_project_agentic_feedback_tests --quiet
cargo run --quiet -- check --format agent --agent codex .
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
python3 ./.trellis/scripts/task.py validate 07-02-agent-integration-lifecycle
git diff --check
```

## Reviewer Blocking Criteria

Block if any generated adapter embeds independent validation logic, mutates
host-agent config without explicit user action, emits unbounded context, lacks
remove/status/doctor behavior, or revives per-agent feedback commands or
formats.
