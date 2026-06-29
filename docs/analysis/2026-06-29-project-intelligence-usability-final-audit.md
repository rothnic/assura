---
title: Project Intelligence Usability Final Audit
status: active
date: 2026-06-29
---

# Project Intelligence Usability Final Audit

This audit maps
`docs/goals/assura-project-intelligence-usability-program.md` definition of
done items to current repo evidence. It is release-hardening evidence for the
local Project Intelligence usability slice, not a 1.0 compatibility guarantee.

## Result

The Project Intelligence usability program is complete locally when paired with
the validation and independent review recorded in
`docs/goals/assura-project-intelligence-release-hardening.md`.

## Evidence Map

| Program requirement | Evidence | Status |
| --- | --- | --- |
| install/init to first useful content query | `assura init --project-intelligence`, `assura check --format json`, and `assura content search` are covered by `tests/project_intelligence_onboarding.rs` and `tests/project_intelligence_release_hardening.rs`. | Proven locally. |
| starter template | The starter writes `.assura/models/project-intelligence/starter.schema.json`, modeled goals/specs/ADRs, and a broken-state example; see `tests/project_intelligence_onboarding.rs`. | Proven locally. |
| non-Assura project package | Beacon CRM fixtures under `tests/fixtures/project_intelligence_real_repo/beacon_crm/` are covered by `tests/project_intelligence_real_repo_proof.rs`, `tests/project_intelligence_context_pack.rs`, `tests/agent_surface_cli.rs`, and `tests/editor_surface_cli.rs`. | Proven locally. |
| bounded context-pack workflow | `assura content context-pack` and `assura agent context-pack` emit `assura.project-intelligence.context-pack.v1`; covered by `tests/project_intelligence_context_pack.rs`, `tests/agent_surface_cli.rs`, and release-hardening schema tests. | Proven locally. |
| warm-session path | `assura content session` and `assura agent session` emit `assura.project-intelligence.session.response.v1` and reload conservatively on file changes; covered by `tests/project_intelligence_session.rs` and `tests/agent_surface_cli.rs`. | Proven locally. |
| Agent and editor integrations | `assura agent ...` is covered by `tests/agent_surface_cli.rs`; `assura editor session` is covered by `tests/editor_surface_cli.rs`; release docs classify MCP, remote access, full LSP server framing, and editor marketplace packaging as not required or not supported. | Proven locally. |
| Safe fixes | `assura fix markdown --dry-run --format json` previews and `--apply --format json` writes only after explicit opt-in; safe-fix audit IDs flow through context-pack, content session, agent, and editor code-action previews. Covered by safe-fix, context-pack, session, agent, and editor tests. | Proven locally. |
| .assura/ | Project-intelligence artifacts under `.assura/` must live under `.assura/models/**`; root files stay bounded. Covered by content runtime validation tests and Assura self-check. | Proven locally. |
| schemas and support levels | `docs/support-policy.md`, `docs/compatibility-and-surface.md`, `docs/release-notes.md`, and `website/src/content/docs/reference/release-readiness.md` classify supported, experimental, roadmap, and unsupported project-intelligence surfaces. `tests/project_intelligence_release_hardening.rs` checks live schema names and release-readiness coverage. | Proven locally. |
| independent review evidence | Completed successors record independent review in their goal progress logs. The editor slice review agent fixed absolute file URI handling and parity tests. Release-hardening review agent `019f14e9-a9b3-73e0-86ab-5ac72ec14d3d` found support-matrix, release-note, schema-example, safe-fix, and task-branch gaps; fixes are recorded in `docs/goals/assura-project-intelligence-release-hardening.md` and checked by release-hardening tests. | Proven locally. |

## Supported Local Workflow

The release-hardening smoke path is intentionally local:

```bash
assura init --project-intelligence --no-git-hooks .
assura check --format json .
assura content search "Project Intelligence" . --format json
assura content context-pack . --collection goals --id goal-project-intelligence-starter --text "Project Intelligence" --limit 5 --format json
assura agent context-pack . --collection goals --id goal-project-intelligence-starter --text "Project Intelligence" --limit 5
assura editor session .
```

The checked version of this smoke runs in
`tests/project_intelligence_release_hardening.rs` through the Cargo-built
`assura` binary. Release-candidate archive smoke remains covered by
`cargo xtask release-smoke`, `scripts/smoke-install-adoption.sh`, and CI
installable adoption jobs for generic install behavior.

## Unsupported Or Roadmap-Only Boundaries

- MCP is optional future adapter work and is not required for local agent
  usability.
- Full LSP server framing and editor marketplace packages are not supported by
  the current editor surface.
- Hosted services, remote providers, and automatic repair are not part of the
  supported local Project Intelligence workflow.
- Semantic candidates and code-symbol evidence provide context only; they do
  not decide validation correctness.

## Validation Commands

Current release-hardening validation should include:

```bash
cargo fmt --check
cargo test --test project_intelligence_release_hardening --quiet
cargo test -p assura --test editor_surface_cli --quiet
cargo test -p assura --test agent_surface_cli --quiet
cargo test --test project_intelligence_context_pack --quiet
cargo test --test project_intelligence_session --quiet
cargo run --quiet -- check --format json .
cargo xtask docs
cargo xtask evidence
git diff --check
```
