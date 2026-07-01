# Reference Daemon Readiness

## Problem

The beta program has completed Reference Graph. Assura can now build local
repository-reference facts, query inbound/outbound affected context, and report
opt-in source-reference breakage through one-shot checks. The next beta gap is
a warm local daemon/session layer that can reuse that truth for repeated
checks, editor diagnostics, hooks, and agent nudges without requiring a hosted
service.

## Goal

Execute `docs/goals/assura-reference-daemon-readiness.md` as Epic 6 of the beta
program. Start by revalidating the goal against current Project Intelligence,
Reference Graph, editor-session, and prepared-check surfaces, then implement the
smallest daemon-core slice that proves warm state matches one-shot truth.

## Scope

- Refresh the daemon readiness goal with current gaps, user certainty bar,
  proof gates, and review criteria before implementation.
- Reuse existing prepared check, content-query session, config fingerprint,
  and `RepositoryReference` fact behavior where possible.
- Keep daemon state local, explicit, inspectable, and safe to discard.
- Prove changed-source and changed-target feedback is bounded and conservative.
- Preserve one-shot `assura check` as the fallback truth path.

## Non-Goals

- No VS Code extension UI in this task.
- No remote daemon manager or hosted service.
- No per-agent validation logic.
- No hidden automatic repair.

## Validation

Use narrow checks while iterating:

```bash
cargo fmt --check
cargo test daemon --quiet
cargo test --test repository_reference_graph_tests --quiet
cargo run --quiet -- check --format json .
git diff --check
```

Before committing implementation slices, also run:

```bash
cargo xtask evidence
cargo xtask target-state
```

## Review

Complex implementation slices require independent review. Ask reviewers to
focus on daemon state freshness, parity with one-shot truth, bounded affected
context, explicit degradation, and whether editor/agent callers can consume the
result without new per-agent contracts.
