

## Current-base PR gate correction (2026-10-20)

The first actual full-PR capture on9908a8c8 completed1 after110 Rust
summary rows (1485passed/0failed/1manualignore) and fast/evidence checks.
The target-state audit rejected the task's full refs/heads branch spelling;
its dependent PRD message did not mean the existing PRD was missing. Three
independent read-only diagnoses support the metadata-only supportedset-branch
correction to the current short branch. Immutable delivery.branch_ref and all
other delivery identity fields remain unchanged. All-target Clippy and docs
were not reached, so this run remains failed/incomplete. Raw capture:
external nu15/current-base-20261020/validation-runs/nu15-full-pr-8d04bd3c-fc94-4c2b-8012-95bfb3c80bea.
No source/runtime gate is waived; corrected clean candidate requires full-PR.
