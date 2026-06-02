---
title: Goal 08 Release Readiness Review
date: 2026-06-02
status: evidence
---

# Goal 08 Release Readiness Review

Goal file:
`docs/goals/assura-goal-08-release-readiness-and-ecosystem.md`.

## Scope Review

- Active Trellis task:
  `.trellis/tasks/06-01-roadmap-phase-01-execution`.
- Branch: `codex/phase-01-goal-08-release-readiness`.
- Goal 07 was completed through merged PR #24 before this work started.
- Public agent feedback surface remains `assura check --format agent`, with
  Codex delivery only through `--agent codex`.
- Release readiness work is documentation, evidence, support policy, and
  roadmap handoff work. It does not add new CLI commands or package feedback
  CLIs.

## Evidence Inventory

| Evidence | Location | Checked In | Reproduction Command |
| --- | --- | --- | --- |
| Release notes | `docs/release-notes.md` | Yes | `node --run verify:evidence` |
| Release candidate checklist | `docs/release-candidate-checklist.md` | Yes | `node --run verify:evidence` |
| Support policy | `docs/support-policy.md` | Yes | `node --run verify:evidence` |
| Compatibility matrix | `docs/compatibility-and-surface.md` | Yes | `node --run verify:evidence` |
| Stale project memory cleanup | `docs/project-memories.md` | Yes | `node --run verify:evidence` |
| Checksum workflow support | `.github/workflows/release.yml`, `.github/workflows/ci.yml`, `scripts/verify.sh` | Yes | `node --run verify:release-smoke` |
| Website release readiness page | `website/src/content/docs/reference/release-readiness.md` | Yes | `node --run verify:docs` |
| Planned next iteration | `docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md` | Yes | `node --run verify:evidence` |
| Iteration ledger and Trellis routing | `docs/goals/assura-roadmap-phase-01-agentic-adoption-foundation.md`, `.trellis/spec/assura/roadmap.md` | Yes | `node --run verify:evidence` |

## Validation Commands

| Command | Status | Notes |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Passed | Required by Goal 08. |
| `cargo test --all-targets --quiet` | Passed | Required by Goal 08; includes benchmark-style harnesses. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed | Required by Goal 08. |
| `cargo build --release --bin assura --no-default-features --features json-output,yaml-config` | Passed | Required by Goal 08. |
| `cargo run --quiet -- check --format json .` | Passed | Self-check reports zero violations after release docs were added. |
| `node --run verify:fast` | Passed | Required by Goal 08. |
| `node --run verify:docs` | Passed | Website builds 29 pages, including `/reference/release-readiness/`. |
| `cd website && npx pnpm@10.25.0 build` | Passed | Required by Goal 08; exact command built 29 pages. |
| `node --run verify:evidence` | Passed | Trellis task, goal status, review evidence, and stale-surface checks pass after release docs were registered. |
| `node --run verify:release-smoke` | Passed | Built local release archive, verified checksum, installed through `website/public/install.sh`, and completed first-run adoption smoke. |
| `git diff --check` | Passed | No whitespace errors after release docs and review record edits. |

## Review Tasks

| Task | Status | Evidence |
| --- | --- | --- |
| R0. Scope and source-of-truth review | Complete locally | Goal 08 is active; Goal 07 is completed; Iteration 02 is planned but not active. |
| R1. Compatibility and performance claim review | Complete locally | Release notes and compatibility matrix cite only supported commands, CI smoke lanes, and checked evidence surfaces. |
| R2. Release artifact and installer smoke review | Complete locally | Checklist names exact archives, CI smoke jobs, installer scripts, and post-tag live verification. |
| R3. Install and first-check reproduction review | Complete locally | `node --run verify:release-smoke` verified the archive checksum, installed from the local release archive, and completed the adoption smoke. |
| R4. Website and release notes stale-claim review | Complete locally | Release readiness page and release notes remove unsupported hooks/package feedback/plugin marketplace claims. |
| R5. Next iteration and support policy review | Complete locally | Support policy and planned Iteration 02 are checked in and linked from roadmap state. |

## Review Feedback Closure

| Source | Finding | Decision | Evidence |
| --- | --- | --- | --- |
| Local review | Old release notes advertised unsupported dependency graph validation, IDE plugin, marketplace, and package-like feedback ideas. | Fixed | Rewrote `docs/release-notes.md` around current supported, experimental, and unsupported surfaces. |
| Local review | Goal 08 needed an explicit next roadmap iteration so Iteration 01 completion would not be confused with product roadmap completion. | Fixed | Added planned `docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md` and linked it from `.trellis/spec/assura/roadmap.md`. |
| Review agent Faraday (`019e8686-3058-7592-b6b8-040dda60389f`) | Required artifacts were still untracked, so PR creation would omit core Goal 08 files. | Fixed | Staged-file risk is addressed before commit/PR; review record now tracks this as a blocking pre-PR checklist item. |
| Review agent Faraday (`019e8686-3058-7592-b6b8-040dda60389f`) | Goal 08 promised checksum verification, but release workflow/checklist evidence did not generate or verify checksums. | Fixed | Added checksum generation and verification to `scripts/verify.sh`, `.github/workflows/release.yml`, and the CI installable adoption matrix; docs now require `.sha256` assets. |
| Review agent Faraday (`019e8686-3058-7592-b6b8-040dda60389f`) | Goal 08 required validation omitted `node --run verify:release-smoke` and `node --run verify:evidence`. | Fixed | Added both commands to `docs/goals/assura-goal-08-release-readiness-and-ecosystem.md`. |
| Review agent Faraday (`019e8686-3058-7592-b6b8-040dda60389f`) | `docs/project-memories.md` contained stale release and package/plugin claims and was outside evidence checks. | Fixed | Rewrote `docs/project-memories.md` to defer release truth to current docs and added it to `scripts/verify.sh` checked markdown files. |
| Review agent Faraday (`019e8686-3058-7592-b6b8-040dda60389f`) | Live release verification omitted the musl release asset. | Fixed | Added musl archive and all checksum URLs to `run_release_live`. |
| Review agent Faraday (`019e8686-3058-7592-b6b8-040dda60389f`) | Review-agent closure was pending. | Fixed | Recorded findings and decisions in this closure table. |
| Gemini or PR review | Pending after PR opens. | Pending | PR template requires review feedback closure. |

## Handoff

- PR: pending.
- Iteration 01 completion condition: Goal 08 PR merged, ledger updated, and
  next roadmap iteration identified without marking the product roadmap done.
- Next planned iteration:
  `/goal docs/goals/assura-roadmap-iteration-02-policy-depth-and-ecosystem.md`
