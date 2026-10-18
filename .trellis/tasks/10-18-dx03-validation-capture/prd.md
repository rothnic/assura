# DX03: Capture the actual validation invocation

## Problem and scope

Bead ae-whr.3. NU16 lost a real outer tool handle and its actual exit remains unknown; per-task copied runners repeatedly complicate capture. Add the smallest internal skill-local wrapper that captures exact argv, separate raw byte streams and an observed invocation exit once. No product API, semantic test parser, orchestration service, global setup, Cargo behavior or CI change. User/source environment and task meaning remain with the caller.

## Authority

Nick authorized the active Beads execution goal, independent GPT-6 Luna implementation review/corrections and reviewed merges, including previously disclosed website deployment coupling. Root owns Beads, pushes, PR creation, merges and production delivery receipts. dx03-validation-capture owns only this clean candidate. No release/tag/private evaluator or human acceptance authority implied.

## Approved design

Implement the corrected design at /Users/nroth/workspace/.assura-review-20261017/dx03/design-v2.md, independently admitted in independent-design-rereview-v2.md in that folder. Original proposal and two findings remain retained. DX03-1 overbroad metadata and DX03-2 imprecise interruption were accepted and addressed; original mandatory generic provenance probes are superseded. One stdlib Python script, focused fake controls and short assura-local-build skill route. Use existing preflight as caller-composed exact command; do not change its interface or execute it twice.

## Acceptance

- Exact single invocation, restrictive fresh evidence directory, no reuse/overwrite, raw separate streams, actual direct child subprocess return code and wrapper status; explicit optional env/hash inputs only. No secrets dump, generic tool-version probing or semantic success claim.
- POSIX-only runtime contract (macOS and Linux): other platforms are rejected before evidence creation or child launch. Do not claim Windows ACL/privacy or execution proof.
- Lifecycle claimed/running/complete/interrupted as specified in approved design. Handled INT/TERM forwards only to owned direct PID, waits at most5seconds and returns nonzero for interruption. Unknown/null when unobserved; hard kill/spawn-to-PID-write window cannot prove never launched or recovered exit. No group signal or descendant-reaping claim.
- Caller owns setup/TMPDIR/targets/export directories and retains full Codex/SSH outer session handle. Wrapper never reconstructs lost exits, retries, repairs, installs or broadly scans/kills processes. Existing NU16 unknown, W02 zero-selection failure and SP04 caller export-parent failure remain honest historical limits.
- Meaningful fake positive/negative controls in design: argv boundaries, byte streams, env, nonzero23, independent fresh immutable runs, prelaunch invalid-input rejection, hash confinement, preflight rejected marker, handled interruption/unknown outcome and zero-test-looking exit0 capture-only. Observe and reap only owned fixture processes.
- Focused controls also fault-inject both running and terminal receipt persistence; cleanup is bounded to the direct child, capture failures return nonzero, and an observed child exit/live status remains distinct. The default focused command must always execute both fault states.
- Run existing local-preflight controls once, one cheap supported real helper composition using Cargo metadata with explicit route and no unrelated profiles, fresh resource/scratch ownership admission first. No cold build necessary. Interpret helper JSON from raw stderr separately from wrapper invocation receipt.
- Cheap workflow/context/diff/structure/evidence gates and independent Luna implementation review before PR; accepted corrections and affected checks with scoped rereview. Appropriate hosted process/evidence jobs remain required; skips uncredited. No full Rust tier solely for skill/Python changes.

## Ownership and proof

Own .agents/skills/assura-local-build/SKILL.md, new scripts/capture-validation.py and scripts/capture-validation-tests.py, plus this task. No local-preflight.py source edits without a separate justified scope amendment. You are not alone; preserve other owners and checkouts. Use task-owned disk-backed0700 scratch, bounded Python workers, explicit argv/environment and actual tool handles. External immutable evidence /Users/nroth/workspace/.assura-review-20261017/dx03/. Preserve source/tool identities, actual exits/counts and PID/teardown; no fixed old-directory overwrite. Expected negative controls are expected, unexpected gate failures require root triage before correction/retry.

## State

Approved designv2, independently admitted for bounded implementation. Base74d9917dd39e599bbd43c3a5760b71644b2cf91b. Review of `4dfc6b8bc37d58350da0699e8b4385ebe7b561b2` admitted POSIX-only platform rejection and owned-child cleanup on receipt persistence failures. The correction is in progress on `chore/dx03-validation-capture`; final focused evidence and scoped rereview are pending. DX01 is closed; DX02 has separate source ownership. Integration must refresh current base and relevant proof.

Final candidate evidence is under `/Users/nroth/workspace/.assura-review-20261017/dx03/impl-20261018T121059Z-a47b269e-1cf6-4c5b-9295-233c9adf79a7/evidence/`:

- `capture-controls-final/`: 13 focused control groups passed; known return-zero interruption, literal signal return mapping, and unknown live-child outcome all returned nonzero wrapper status and verified owned fixture teardown.
- `preflight-controls-canonical-20261018T123053Z/`: unchanged existing controls passed 25/25 with child-only `TMPDIR` and `DX01_CONTROL_ROOT` set to the verified canonical caller-owned APFS scratch directory.
- `real-metadata-composition-final-20261018T123358Z/`: existing preflight helper plus `cargo metadata --offline --locked --no-deps --format-version 1`; helper and wrapper exits were zero, route was `metadata`, and workspace identity matched this checkout.

Earlier failed runs and their raw records remain in sibling evidence directories. The initial local-preflight run's three lexical-versus-canonical path assertions were resolved by changing only caller-owned fixture roots; the helper, its tests, and socket spelling contract were not changed. A prior interrupted-control run remains unknown in its preserved `running` receipt and is not credited; the final focused run passed with the corrected bounded wait loop.

The independent implementation review's pre-correction evidence and F1/F2 fault-injection RED/GREEN artifacts remain immutable under `/Users/nroth/workspace/.assura-review-20261017/dx03/`. Their source binding is path/chronology only (no cryptographic source manifest); they are not final-candidate proof. Do not reuse their filenames or claim they bind to the corrected source.

## Root correction checkpoint

F1 is addressed by early POSIX-only rejection; F2 by bounded direct-child
cleanup and nonzero wrapper capture-failure status. The default aggregate now
executes both persistence fault states. Root strengthened it to observe child
liveness before any fixture cleanup. All 16 groups and both fault states
passed at `/Users/nroth/workspace/.assura-review-20261017/dx03/root-final-controls-20261018T125701Z-0c5d174d/`;
all tracked helper/task/script inputs were hashed before/after unchanged. Both
fault children were already stopped before fixture cleanup; observed exits
were -15 and 0, with wrapper status125 in both cases. This is mocked non-POSIX
coverage only, not Windows execution. Earlier duplicate fault-root caller
failure is retained and receives no fault coverage credit.

Fresh structure, task/context, diff and real metadata helper composition
exited zero at `../root-final-cheap-20261018T125728Z/dx03/` under the external
evidence root. Existing 25/25 preflight proof remains reusable only after
unchanged helper/test input comparison; final scoped Luna rereview is pending.
