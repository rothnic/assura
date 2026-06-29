---
title: Project intelligence usability gap evaluation
status: active
---

# Project Intelligence Usability Gap Evaluation

## Evidence Reviewed

- `docs/goals/assura-project-intelligence-usability-program.md`
- `docs/goals/assura-project-intelligence-adoption-blueprint.md`
- `docs/goals/assura-project-intelligence-real-repo-proof.md`
- `docs/goals/assura-project-intelligence-onboarding-template.md`
- `docs/goals/assura-project-intelligence-context-pack.md`
- `docs/goals/assura-project-intelligence-persistent-session.md`
- `docs/goals/assura-project-intelligence-safe-fix-workflow.md`
- `docs/analysis/2026-06-29-project-intelligence-real-repo-proof.md`
- `website/src/content/docs/examples/project-intelligence-demo.md`
- `.trellis/spec/assura/roadmap.md`

## Outcome Assessment

The latest updates meet the intended proof bar for the completed usability
slices. Project intelligence now has a visual documentation demo, Assura-local
goal modeling, a deterministic non-Assura Beacon CRM fixture, a first-run
`assura init --project-intelligence` starter, and a bounded
`assura content context-pack` handoff.

We know this because the completed goal logs and roadmap point to concrete
artifacts: the demo page shows starter setup, graph/search, agent envelopes,
context-pack handoff, and safe-fix previews; the Beacon CRM fixture proves valid
and invalid states; the onboarding goal added starter-generation regressions;
and the context-pack goal added focused tests for diagnostic and object
handoff modes, including truncation evidence.

That makes the runtime credible and the first handoff/session loop usable, but
not yet broadly usable as an everyday product. The remaining work is now
narrower: applying safe fixes with audit and recovery, exposing the same
contracts through concrete agent and editor transports, and locking
release/support expectations.

## Remaining Gaps

| Gap | Evidence | User Impact | Next Goal |
| --- | --- | --- | --- |
| Safe-fix support stops at preview | The current safe-fix contract and context pack expose proposed Markdown repairs, but do not own apply, audit, or rollback behavior. | Users can understand a repair but cannot safely complete an accepted repair workflow through Assura. | `docs/goals/assura-project-intelligence-safe-fix-workflow.md` |
| Agent transport is not a supported product contract | Agents can shell out to CLI/context-pack commands, but there is no supported MCP-style tool surface over the same contracts. | Agent integrations still need bespoke wrappers and cannot rely on stable tool names or schemas. | `docs/goals/assura-project-intelligence-mcp-agent-transport.md` |
| Editor transport is not a supported product contract | Docs classify daemon/editor sessions and LSP behavior as future work. | Maintainers cannot get diagnostics, context, or safe-fix previews in an editor without custom glue. | `docs/goals/assura-project-intelligence-lsp-editor-transport.md` |
| Release status is not consolidated | Runtime, docs, fixtures, support policy, and release readiness do not yet agree on the promoted usability slice. | Users cannot tell which schemas and surfaces are supported, experimental, or roadmap-only. | `docs/goals/assura-project-intelligence-release-hardening.md` |

## Ordered Goal Set

Completed prerequisites:

1. Adoption blueprint: documented the first product path and visual demo.
2. Real-repo proof: proved the path on Assura plus the Beacon CRM package.
3. Onboarding template: added `assura init --project-intelligence`.
4. Context pack: added `assura content context-pack`.
5. Persistent session: added `assura content session`.

Remaining executable goals:

1. Safe-fix workflow: extend preview into explicit apply, audit, and recovery.
2. MCP agent transport: expose diagnostics, context packs, queries, and safe-fix
   previews through supported local agent tools.
3. LSP editor transport: expose diagnostics, project-intelligence context, and
   safe-fix code actions through a supported local editor protocol.
4. Release hardening: lock schemas, support status, docs, and final evidence.

This replaces the older transport bundle with two smaller transport goals. MCP
agent transport can prove tool contracts first; LSP editor transport can then
prove editor diagnostics and code actions without mixing agent-tool concerns
into the same review gate.
