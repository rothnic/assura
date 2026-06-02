---
title: Goal 06 review evidence gates review
date: 2026-06-02
status: evidence
---

# Goal 06 Review Evidence Gates Review

Goal file: `docs/goals/assura-goal-06-review-evidence-and-quality-gates.md`.

## Scope Review

- Active Trellis task:
  `.trellis/tasks/06-01-roadmap-phase-01-execution`.
- Branch: `codex/phase-01-goal-06-review-evidence-gates`.
- Public surfaces changed: PR template, validation docs, and CI workflow.
- Public CLI surface preserved: stable feedback remains
  `assura check --format agent`, with Codex delivery only through
  `--agent codex`.
- Workflow boundary: Trellis remains the task/spec workflow; this goal adds
  evidence requirements and checks around Trellis rather than replacing it.

## Evidence Inventory

| Evidence | Location | Checked In | Reproduction Command |
| --- | --- | --- | --- |
| Review record template | `docs/analysis/review-record-template.md` | Yes | `node --run verify:evidence` |
| Evidence policy | `docs/analysis/evidence-and-review-policy.md` | Yes | `node --run verify:evidence` |
| PR template | `.github/PULL_REQUEST_TEMPLATE.md` | Yes | `node --run verify:evidence` |
| Evidence gate implementation | `scripts/verify.sh` | Yes | `node --run verify:evidence` |
| CI evidence job | `.github/workflows/ci.yml` | Yes | PR check `Evidence Gates` |
| Goal 06 review record | `docs/analysis/2026-06-02-goal-06-review-evidence-gates-review.md` | Yes | N/A |
| Local build/test output | `target/` and terminal output | No | `node --run verify:fast` |

Generated outputs under `target/` are intentionally not checked in. PR evidence
must name the command that regenerates them when they support a completion
claim.

## Validation Commands

| Command | Status | Notes |
| --- | --- | --- |
| `bash -n scripts/verify.sh` | Passed | Verifies shell syntax after adding `verify:evidence`. |
| `ruby -e 'require "yaml"; ARGV.each { |f| YAML.load_file(f); puts "parsed #{f}" }' .github/workflows/ci.yml .github/workflows/docs.yml .github/workflows/release.yml .github/workflows/security.yml` | Passed | Verifies edited workflow YAML parses. |
| `node --run verify:evidence` | Passed | Runs Trellis state check plus Goal 06 evidence policy. |
| `cargo run --quiet -- check --format json .` | Passed | Assura self-check reports zero violations. |
| `git diff --check` | Passed | No whitespace errors. |
| `node --run verify:fast` | Passed | Includes formatting, compile, Rust tests, self-check, Trellis state, and evidence gate. |
| `cargo fmt --all -- --check` | Passed | Required Goal 06 formatting gate. |
| `cargo clippy --all-targets --all-features -- -D warnings` | Passed | No warnings after script/docs/CI changes. |
| `cargo test --all-targets --quiet` | Passed | Includes benchmark-style harness targets. |
| `node --run verify:docs` | Passed | Website static build completed with 28 pages. |
| `node --run verify:fast` after review fixes | Passed | Re-ran after Hubble findings were fixed and review closure was recorded. |
| `node --run verify:evidence` after PR review fixes | Passed | Re-ran after Gemini link parsing and case-insensitive stale-surface hardening. |

## Backfilled PR Evidence Examples

| PR | Existing Evidence | Template Coverage |
| --- | --- | --- |
| PR #21, Goal 04 | `docs/analysis/2026-06-01-goal-04-fast-incremental-check-review.md`, checked performance JSON, CI performance report, Iteration 01 ledger entry. | The template makes the performance artifacts and reproduction commands explicit, and the evidence policy clarifies which benchmark outputs belong in `target/` versus checked JSON. |
| PR #22, Goal 05 | `docs/analysis/2026-06-02-goal-05-installable-adoption-review.md`, release-style archive smoke CI artifacts, local release-smoke evidence, review-agent and Gemini closure notes. | The template would have caught the need to record Gemini findings and the stale macOS runner-label fix before merge; the new CI evidence gate keeps the current PR template and stale command-surface checks in place. |

## Review Tasks

| Task | Status | Evidence |
| --- | --- | --- |
| R0. Scope and source-of-truth review | Complete | Trellis stays canonical; `.trellis/spec/assura/roadmap.md` and the Iteration 01 ledger now point to Goal 06. |
| R1. CI/script false-positive review | Complete | `verify:evidence` scopes goal metadata and markdown links to Iteration 01/current evidence while scanning current user-facing docs, workflows, and package manifests for stale command surfaces. |
| R2. Documentation example review | Complete locally | PR template and `docs/validation.md` now require goal docs, review records, evidence, validation, review feedback, and next-goal handoff. |
| R3. Evidence reproduction review | Complete locally | `node --run verify:evidence` and `node --run verify:fast` passed locally. |
| R4. Completed PR review | Complete locally | PR #21 and PR #22 were mapped against the new template above. |
| R5. Next-goal handoff | Complete locally | Iteration 01 handoff now points to `docs/goals/assura-goal-06-review-evidence-and-quality-gates.md`; Goal 06 PR should hand off to Goal 07. |

## Review Feedback Closure

| Source | Finding | Decision | Evidence |
| --- | --- | --- | --- |
| Review agent Hubble (`019e8636-de87-7941-b958-48433238b284`) | Stale-surface gate was too narrow and missed future per-agent CLIs/formats outside the initial scan roots. | Fixed | Added pattern-based detection for named package feedback CLIs, per-agent hook formats, per-agent format values, package-manifest `bin` entries, current user-facing docs, workflows, README, and Codex integration docs/manifests. Added self-test samples proving representative forbidden strings are rejected and valid stable surfaces are allowed. |
| Review agent Hubble (`019e8636-de87-7941-b958-48433238b284`) | Markdown link checking covered historical goal and analysis docs, creating future false-positive risk. | Fixed | Scoped local link checks to the Iteration 01 goal set, current review/evidence policy artifacts, validation docs, PR template, and active Assura spec files. |
| Review agent Hubble (`019e8636-de87-7941-b958-48433238b284`) | Review record still marked R1 and review feedback closure as pending. | Fixed | Updated R1 to complete and recorded each review finding with a decision and evidence. |
| Gemini Code Assist on PR #23 | Markdown link checker did not decode URL-encoded paths or handle link titles before local path resolution. | Fixed | Added `urllib.parse.unquote` and title-aware target extraction before resolving local markdown links. |
| Gemini Code Assist on PR #23 | Stale command-surface and package `bin` checks were case-sensitive. | Fixed | Added case-insensitive regex matching, normalized bin-name allowlist comparison, and mixed-case stale-surface self-test samples. |

## Handoff

- PR: https://github.com/rothnic/assura/pull/23.
- Next goal after completion:
  `/goal docs/goals/assura-goal-07-extension-and-plugin-foundation.md`
- Known risks: the markdown link checker is intentionally scoped to local
  checked docs and ignores generated `target/` paths. Broader website route
  validation remains covered by `node --run verify:docs`.
