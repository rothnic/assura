---
id: goal-assura-post-beta-support-release-hardening
type: goal
title: Assura post-beta support and release hardening
status: planned
created: 2026-07-01
owners:
  - assura-maintainers
related:
  - ./assura-post-beta-capabilities-program.md
  - ../support-policy.md
  - ../compatibility-and-surface.md
  - ../data/release-surfaces.json
---

# Assura Post-Beta Support And Release Hardening

## Objective

Close the post-beta program with release-grade support classifications,
compatibility notes, target-state checks, documentation, and evidence for all
newly supported code-agnostic capabilities.

This remains a beta-track hardening goal. It should prepare a versioned beta
increment and must not claim GA or post-beta status.

## Current Gap

The next program spans daemon process support, document graph support,
Markdown lint/fix integration, agent hooks, VS Code packaging, performance
gates, and extension API boundaries. A final hardening goal should reconcile
their support levels before any beta or post-beta release claim is made.

## Scope

- Audit public docs, release notes, support policy, compatibility matrix,
  command surface, release surfaces, and website roadmap for consistent
  support wording.
- Build or identify the final verification package for the parent
  north-star scenario: a documentation-heavy Rust repository branch with
  renamed architecture docs, moved source paths, Markdown/reference drift,
  invalid content/frontmatter, safe and unsafe Markdown findings, stale daemon
  state, agent nudge transcripts, VS Code parity evidence, and a negative
  LS-Lint no-slower fixture.
- Ensure supported, experimental, internal, planned, and unsupported surfaces
  are classified consistently.
- Add target-state checks for the new supported surfaces and for known
  overclaim risks.
- Confirm all child-goal validation evidence is current.
- Prepare release readiness notes and a rollback/support plan for daemon,
  agent, VS Code, document graph, Markdown, and performance surfaces.
- Define the version number for the beta increment and ensure release notes
  describe the increment without implying Assura has exited beta.

## Non-Goals

- No new feature implementation unless needed to fix a release blocker.
- No support promotion for a surface that lacks tests, docs, and review.
- No hosted service or marketplace claim without corresponding release proof.

## Definition Of Done

- Support policy, compatibility docs, release surfaces, website docs, and public
  roadmap agree on the status of every post-beta capability.
- The parent [North-Star Verification Scenario](./assura-post-beta-capabilities-program.md#north-star-verification-scenario)
  is executable or explicitly deferred with a release-blocking reason.
- Target-state checks prevent the most likely unsupported claims.
- All child goals have completion evidence or explicit deferral notes.
- Release readiness commands pass on a clean branch.
- A beta version increment is documented with accurate supported,
  experimental, internal, planned, and unsupported surface wording.
- Independent review confirms no public overclaim or missing support caveat.

## Release Verification Use Case Picture

Use this child goal to prove the release is useful to a specific maintainer,
not only internally consistent. The maintainer is reviewing a multi-agent branch
in a documentation-heavy Rust CLI repository. The branch moved source paths,
renamed architecture docs, changed goal/frontmatter records, edited Markdown
reference pages, added VS Code and agent integration metadata, and refreshed
performance fixtures.

Before approving the release, the maintainer should be able to run one
documented verification package and make a branch-safety decision:

1. `assura check` stages findings from structure and root hygiene through
   coarse file policy before Markdown internals, content collections,
   references, daemon/editor surfaces, and optional language-specific checks.
2. Every finding reports a stable rule, rule-owned severity, supported
   suppression shape, path or heading context, and merge impact.
3. `assura content` answers which goals, ADRs, analysis notes, source files,
   tests, headings, benchmark rows, and release docs are affected by the rename
   or move.
4. Markdown safe-fix preview/apply repairs only deterministic supported drift
   and leaves human-judgment issues visible.
5. The daemon returns fresh IPC answers that match one-shot CLI truth, and it
   rejects, marks stale, or falls back when config or file fingerprints change.
6. Codex, OpenCode, Claude, and Pi receive bounded Assura nudges only around
   useful events such as broad edits, changed-path checks, or stale-reference
   blockers.
7. VS Code diagnostics, commands, safe-fix previews, and daemon doctor output
   agree with the shared CLI/daemon contracts.
8. Release notes, support policy, compatibility docs, release surfaces, public
   roadmap, and website docs describe those exact surfaces as supported,
   experimental, internal, planned, or unsupported without overclaiming beta
   status.
9. CI or the documented local equivalent blocks the release if the intentionally
   broken verification state passes, if support docs drift from implementation,
   or if any accepted LS-Lint-equivalent fixture is slower than native LS-Lint
   without actionable attribution.

This child goal is complete only when the release evidence shows that the
maintainer can merge, block, or send targeted repair instructions from that
single coherent picture.

## Validation Commands

```bash
cargo fmt --check
cargo test --workspace --all-targets --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

## Review Tasks

- R1: Confirm support classifications are consistent across docs and data.
- R2: Confirm release evidence covers every promoted surface.
- R3: Confirm unsupported or deferred items remain clearly marked.
- R4: Confirm target-state checks cover the highest-risk overclaims.
- R5: Confirm the final verification package proves the maintainer can make a
  merge, block, or targeted-repair decision from one coherent scenario.

## Reviewer Blocking Criteria

Block if public docs claim unsupported daemon, editor, agent, graph, Markdown,
extension, or non-beta behavior; if release evidence is stale; if target-state
permits a known overclaim; or if a child goal is marked complete without
independent review. Also block if the release can proceed without proving, or
explicitly deferring as a release blocker, the parent north-star verification
scenario.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-02 | Started Support Hardening after PR #135 merged and the branch was created from `origin/master`. Live release readiness failed because `v0.2.0` is already published while daemon mode, VS Code support, and extension API boundaries were still marked `unreleased`; this slice prepares the next beta increment as `v0.3.0` without claiming GA. | PR #135 merge commit `a4b5e8ba4b6382d271ca5e9eea30e2f5ad2e29da`; `.trellis/tasks/07-02-post-beta-support-release-hardening/prd.md`; `cargo xtask release-readiness --format json` before edits. |
| 2026-07-02 | Prepared initial `v0.3.0` release-hardening metadata. Package versions, release notes, release checklist examples, release-surface first-release values, and a new target-state release-hardening guard now agree on the beta increment; local release readiness passes with no unreleased user-facing surfaces. | `Cargo.toml`; `crates/assura-check-cli/Cargo.toml`; `crates/assura-stable-hash/Cargo.toml`; `xtask/Cargo.toml`; `docs/release-notes.md`; `docs/data/release-surfaces.json`; `xtask/src/main.rs`; `cargo xtask release-readiness --format json`; `cargo xtask target-state`. |
| 2026-07-02 | Added the release verification use-case picture for this child so support hardening is reviewed against a maintainer's end-to-end branch-safety decision, not only against release metadata consistency. Local release smoke passed with the generated preview binary reporting version `0.3.0` and completing adoption smoke. | [Release Verification Use Case Picture](#release-verification-use-case-picture); `cargo xtask release-smoke`; preview binary version output `0.3.0`; adoption smoke evidence path printed under `target/assura-macos-amd64-preview.tar.gz` install workflow. |
| 2026-07-02 | Addressed independent review findings before PR creation. The `v0.3.0` release checklist now gates daemon, agent integration, VS Code local package, and extension-boundary support explicitly; target-state fails if those checklist gates drift; Trellis metadata compares the task branch against `master`. | Review agent `019f231f-8703-7120-a15c-44553b6eb3a3`; `docs/release-candidate-checklist.md`; `xtask/src/main.rs`; `.trellis/tasks/07-02-post-beta-support-release-hardening/task.json`; `cargo xtask target-state`; `cargo test --test daemon_cli_tests --quiet`; `cargo test --test agent_surface_cli --quiet`; `pnpm --dir integrations/editors/vscode test && pnpm --dir integrations/editors/vscode run build && pnpm --dir integrations/editors/vscode run doctor && pnpm --dir integrations/editors/vscode run package`. |
| 2026-07-02 | Fixed the PR #136 CI performance blocker without weakening the no-slower gate. The failing accepted fixture was `many_configured_scopes_regression`; Assura lost by 0.186 ms on Linux because config loading repeated the generic `validator` tree walk after Assura's semantic validator had already checked the same config. Removing that duplicate loader pass keeps semantic validation and restored local no-slower evidence. | PR #136 Performance Report job `84793692639`; `src/config/loader.rs`; `target/performance/pr136-loader-fast.json`; `cargo xtask performance-no-slower target/performance/pr136-loader-fast.json`; `cargo test --lib config --quiet`; `cargo test --test ls_lint_parity_regression_tests --quiet`. |
