---
title: Markdown Engine Selection
status: active
date: 2026-07-02
---

# Markdown Engine Selection

## Decision

Select Assura's native Markdown validation and safe-fix path as the supported
default for this beta increment. Keep `rumdl` as an explicit opt-in
markdownlint-compatible subprocess adapter for broader candidate diagnostics.
Do not promote `rumdl`, `mdlint`, `mado`, or `markdownlint-cli2` to the default
supported engine.

This is a product decision, not a claim that Assura has more commodity
markdownlint rule coverage than every candidate. The supported default must
fit the parent maintainer workflow: structure and coarse file policy first,
stable Assura rule IDs, rule-owned severity and suppression, deterministic
safe fixes, frontmatter and line-ending preservation, daemon/editor/agent
contract reuse, and performance evidence that does not hide startup or adapter
cost.

## Candidate Outcomes

| Candidate | Outcome | Reason |
| --- | --- | --- |
| Assura native Markdown path | Supported default | Fastest accepted path for current supported checks and safe fixes; already owns stable rule IDs, reasoned suppressions, staged output, link/reference checks, frontmatter-aware behavior, and deterministic fix commands. |
| `rumdl 0.2.27` | Supported only as opt-in candidate adapter | Best functional markdownlint-compatible Rust candidate: JSON diagnostics, fix metadata, broad rule overlap, source isolation, and fix-validation passes on all representative profiles. It remains slower than current Assura checks on the accepted local profiles and currently requires a separate binary because its Rust version exceeds Assura's MSRV. |
| `mdlint` | Rejected as supported fixer for this increment | Fastest raw Rust check candidate, but not practical for the supported path because probes found check-mode mutation risk, frontmatter loss on the invalid profile, and overlapping-fix failures on large/fixable profiles. |
| `mado` | Rejected as supported fixer for this increment | Useful comparison row, but the current evidence does not provide the fix/report contract Assura needs for the supported safe-fix workflow. |
| `markdownlint-cli2` | Compatibility baseline only | Broad ecosystem baseline but much slower and Node-backed, so it is not eligible for Assura's supported Rust path. |

## Supported User Surface

For this beta increment, users should rely on:

- `assura check` with native Markdown checks for the supported fast default;
- `markdown.rules.<rule_id>.severity` for modular rule-owned severity;
- reasoned `<!-- assura-ignore <rule>: <reason> -->` suppressions;
- `assura fix markdown --dry-run` and `--apply` for deterministic Assura-owned
  safe fixes;
- optional `markdown.markdownlint_candidate.enabled: true` with `engine: rumdl`
  only when a project intentionally wants experimental markdownlint-compatible
  candidate diagnostics from an installed `rumdl` binary.

The optional adapter must remain isolated from the supported default until a
future slice proves it can meet the no-slower fixture bar or the product
chooses a different supported performance envelope.

## Verification Evidence

The decision is backed by the checked representative probe reports in
`.trellis/tasks/07-02-07-02-markdownlint-compatible-rust-engine/research/`:

- `markdown-engine-probe-2026-07-02-invalid-representative.json`
- `markdown-engine-probe-2026-07-02-frontmatter-link-heavy-representative.json`
- `markdown-engine-probe-2026-07-02-large-doc-representative.json`
- `markdown-engine-probe-2026-07-02-fixable-drift-representative.json`

The reports separate current Assura check timing, Assura safe-fix dry-run/apply
timing, external candidate check timing, external candidate fix timing, and
external candidate fix-validation. The fix-validation rows prove changed files,
frontmatter preservation, line-ending preservation, second-run idempotence, and
post-fix check status on isolated copies.

## Follow-Up

Future Markdown work should add accepted rule coverage to Assura's native path
or promote a candidate only when it satisfies the same staged contracts and
performance bar. The parent program should now continue with the next
post-beta child goal rather than keep researching external Markdown candidates.
