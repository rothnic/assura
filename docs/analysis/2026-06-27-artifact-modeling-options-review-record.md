---
id: analysis-2026-06-27-artifact-modeling-options-review-record
type: analysis
title: Artifact modeling options review record
status: active
created: 2026-06-27
owners:
  - assura-maintainers
related:
  - docs/goals/assura-artifact-modeling-options-comparison.md
  - docs/analysis/2026-06-27-artifact-modeling-options-comparison.md
---

# Artifact Modeling Options Review Record

This record preserves the independent review evidence for the artifact
modeling comparison. The reviews were read-only and did not edit files.

## Candidate Coverage

| Candidate | Review coverage |
| --- | --- |
| TypeSpec | Epicurus reviewed TypeSpec against LinkML and identified generated-artifact and reference-decorator gaps. |
| LinkML | Epicurus reviewed LinkML against TypeSpec and identified profile, generated-artifact, and editor-DX requirements. |
| JSON Schema/JTD | Banach reviewed JSON Schema/JTD as the runtime-target candidate. |
| CUE | Banach reviewed CUE as a constraint-authoring candidate and blocked CUE/Go from the runtime path. |
| Zod/TypeBox | Aquinas reviewed Zod/TypeBox and corrected the runtime score when JSON Schema is checked in. |
| Assura control | Aquinas reviewed the control format and warned not to dismiss it before testing a minimal generator/profile. |
| Full completion audit | Dirac reviewed the updated work against every goal requirement and returned blockers that were then addressed. |

## Reviewer Findings

### Epicurus

Agent ID: `019f0ae3-38b2-7ec3-b44c-2c78edb57148`

Verdict: provisional pass only. LinkML-first was defensible as the next
prototype, but not as a permanent choice. The review required actual TypeSpec
and LinkML generation commands, warned that TypeSpec relation decorators could
change the score, and noted that identical normalized schemas did not prove
candidate-specific generation.

Resolution:

- Added TypeSpec and LinkML generated-output snapshots.
- Updated LinkML to use the same user-facing `status` field as other
  candidates.
- Reworded the recommendation as a next-prototype decision.

### Banach

Agent ID: `019f0ae3-4e01-71b0-a11c-7aef056f342b`

Verdict: provisional pass for JSON Schema/JTD as runtime target; block for CUE
as runtime engine. The review required generated-output proof, safe-write
evidence, and real Rust validation against generated or compiled artifacts.

Resolution:

- Added generated/compiled artifact validation in
  `tests/artifact_modeling_options_comparison.rs`.
- Added CUE per-class generated snapshots.
- Added safe-update proof for Markdown frontmatter.
- Kept CUE as authoring-time only, never a runtime dependency.

### Aquinas

Agent ID: `019f0ae3-5eac-7661-a983-508e183ceac7`

Verdict: block until generated-output proof, source constraints, and safe-write
evidence were improved. The review also corrected that Zod's generated JSON
Schema should have a better native-runtime score, while remaining weak as a
cross-language source of truth.

Resolution:

- Updated source sketches so basic non-empty constraints are represented before
  normalization where the candidate syntax supports it.
- Added Zod generated-output snapshot.
- Raised Zod/TypeBox native runtime score while keeping cross-language score
  low.
- Preserved Assura control as a benchmark rather than a recommended core.

### Dirac

Agent ID: `019f0aec-c783-7332-93da-366d2ba1a804`

Verdict: block on three auditability gaps: Rust was not validating generated
outputs, native single-binary performance was asserted rather than measured, and
review outputs were not preserved enough for audit.

Resolution:

- Added generated/compiled artifact validation for TypeSpec, LinkML, CUE, JSON
  Schema/JTD, Zod, and Assura control.
- Added a cached native Rust validation loop that reuses loaded schema artifacts
  and runs without subprocesses.
- Added this review record with reviewer IDs, candidate coverage, findings, and
  resolutions.

## Post-Review Validation

```bash
cargo test --test artifact_modeling_options_comparison --quiet
```

Observed result:

```text
7 passed; 0 failed
```
