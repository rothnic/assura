---
title: Project intelligence usability gap evaluation
status: active
---

# Project Intelligence Usability Gap Evaluation

## Evidence Reviewed

- `docs/goals/assura-project-intelligence-usability-program.md`
- `docs/goals/assura-project-intelligence-adoption-blueprint.md`
- `docs/goals/assura-project-intelligence-real-repo-proof.md`
- `docs/analysis/2026-06-29-project-intelligence-real-repo-proof.md`
- `website/src/content/docs/examples/project-intelligence-demo.md`
- `.trellis/spec/assura/roadmap.md`

## Outcome Assessment

The latest updates meet the intended proof bar for the current slice: project
intelligence now has a visual documentation demo, Assura-local goal modeling,
and a deterministic non-Assura Beacon CRM fixture. The proof covers valid and
invalid checks, search, graph expansion, missing relations, agent-query
diagnostics, and safe-fix previews without network access.

That makes the runtime credible, but not yet broadly usable. The remaining
work is productization: reducing hand setup, providing one useful context
bundle instead of a sequence of commands, keeping repeated checks warm, wiring
transports over shared contracts, making safe fixes apply safely, and locking
release/support expectations.

## Remaining Gaps

| Gap | Evidence | User Impact | Next Goal |
| --- | --- | --- | --- |
| First-run setup is still hand-authored | The demo says to add typed collections and models, but there is no starter project-intelligence template or guided init profile. | New users must infer config, schema, and collection structure before the first useful query. | `docs/goals/assura-project-intelligence-onboarding-template.md` |
| Useful context requires command choreography | The demo asks users to run check, search, missing-relations, expand, agent-context, and agent-query separately. | Agents and maintainers need custom wrappers to gather one editing context. | `docs/goals/assura-project-intelligence-context-pack.md` |
| Repeated editing loops are cold | Public project-intelligence commands are one-shot CLI calls, while warm session evidence is still internal/performance-oriented. | Editor and agent loops pay avoidable startup/model load costs or depend on internal binaries. | `docs/goals/assura-project-intelligence-persistent-session.md` |
| Safe-fix support stops at dry-run | The current safe-fix contract reports proposed Markdown repairs but does not own apply, audit, or rollback behavior. | Users can preview but cannot safely complete an accepted repair workflow through Assura. | `docs/goals/assura-project-intelligence-safe-fix-workflow.md` |
| Editor and agent transports are planned only | Docs classify daemon/editor/LSP/MCP surfaces as future work. | Integrations can call CLI contracts but cannot depend on a supported transport. | `docs/goals/assura-project-intelligence-editor-agent-transports.md` |
| Release status is not consolidated | Runtime, docs, fixtures, support policy, and release readiness do not yet agree on the promoted usability slice. | Users cannot tell which schemas and surfaces are supported, experimental, or roadmap-only. | `docs/goals/assura-project-intelligence-release-hardening.md` |

## Ordered Goal Set

1. Onboarding template: create a reproducible first-run starter path from a
   normal repo to a clean project-intelligence model.
2. Context pack: provide one bounded command or contract that gathers the
   diagnostic, graph, search, and safe-fix preview context an agent needs.
3. Persistent session: productize the warm local reuse path for repeated
   project-intelligence checks and queries.
4. Safe-fix workflow: extend dry-run into explicit apply, audit, and recovery.
5. Editor and agent transports: expose the shared contracts through concrete
   editor and agent integration surfaces.
6. Release hardening: lock schemas, support status, docs, and final evidence.

This replaces the older four-bucket ordering. Persistent sessions remain
important, but they are not the next missing user-facing piece; setup and
context assembly block adoption first.
