# DX03: Capture the actual validation invocation

## Problem and scope

Bead ae-whr.3. NU16 lost a real outer tool handle and its actual exit remains unknown; per-task copied runners repeatedly complicate capture. Add the smallest internal skill-local wrapper that captures exact argv, separate raw byte streams and an observed invocation exit once. No product API, semantic test parser, orchestration service, global setup, Cargo behavior or CI change. User/source environment and task meaning remain with the caller.

## Authority

Nick authorized the active Beads execution goal, independent GPT-6 Luna implementation review/corrections and reviewed merges, including previously disclosed website deployment coupling. Root owns Beads, pushes, PR creation, merges and production delivery receipts. dx03-validation-capture owns only this clean candidate. No release/tag/private evaluator or human acceptance authority implied.

## Approved design

Implement the corrected design at /Users/nroth/workspace/.assura-review-20261017/dx03/design-v2.md, independently admitted in independent-design-rereview-v2.md in that folder. Original proposal and two findings remain retained. DX03-1 overbroad metadata and DX03-2 imprecise interruption were accepted and addressed; original mandatory generic provenance probes are superseded. One stdlib Python script, focused fake controls and short assura-local-build skill route. Use existing preflight as caller-composed exact command; do not change its interface or execute it twice.

## Acceptance

- Exact single invocation, restrictive fresh evidence directory, no reuse/overwrite, raw separate streams, actual direct child subprocess return code and wrapper status; explicit optional env/hash inputs only. No secrets dump, generic tool-version probing or semantic success claim.
- Lifecycle claimed/running/complete/interrupted as specified in approved design. Handled INT/TERM forwards only to owned direct PID, waits at most5seconds and returns nonzero for interruption. Unknown/null when unobserved; hard kill/spawn-to-PID-write window cannot prove never launched or recovered exit. No group signal or descendant-reaping claim.
- Caller owns setup/TMPDIR/targets/export directories and retains full Codex/SSH outer session handle. Wrapper never reconstructs lost exits, retries, repairs, installs or broadly scans/kills processes. Existing NU16 unknown, W02 zero-selection failure and SP04 caller export-parent failure remain honest historical limits.
- Meaningful fake positive/negative controls in design: argv boundaries, byte streams, env, nonzero23, independent fresh immutable runs, prelaunch invalid-input rejection, hash confinement, preflight rejected marker, handled interruption/unknown outcome and zero-test-looking exit0 capture-only. Observe and reap only owned fixture processes.
- Run existing local-preflight controls once, one cheap supported real helper composition using Cargo metadata with explicit route and no unrelated profiles, fresh resource/scratch ownership admission first. No cold build necessary. Interpret helper JSON from raw stderr separately from wrapper invocation receipt.
- Cheap workflow/context/diff/structure/evidence gates and independent Luna implementation review before PR; accepted corrections and affected checks with scoped rereview. Appropriate hosted process/evidence jobs remain required; skips uncredited. No full Rust tier solely for skill/Python changes.

## Ownership and proof

Own .agents/skills/assura-local-build/SKILL.md, new scripts/capture-validation.py and scripts/capture-validation-tests.py, plus this task. No local-preflight.py source edits without a separate justified scope amendment. You are not alone; preserve other owners and checkouts. Use task-owned disk-backed0700 scratch, bounded Python workers, explicit argv/environment and actual tool handles. External immutable evidence /Users/nroth/workspace/.assura-review-20261017/dx03/. Preserve source/tool identities, actual exits/counts and PID/teardown; no fixed old-directory overwrite. Expected negative controls are expected, unexpected gate failures require root triage before correction/retry.

## State

Approved designv2, independently admitted for bounded implementation. Base74d9917dd39e599bbd43c3a5760b71644b2cf91b. No implementation or validation credit yet. DX01 dependency is already closed; DX02 separate source ownership may proceed concurrently. Integration must refresh current base and relevant proof.
