# NU15 current-base validation

The first actual full-PR capture on `9908a8c8` exited 1 after 110 Rust
summary rows (1,485 passed, 0 failed, 1 intentional ignore) and fast/evidence
checks. The target-state audit rejected the task's full `refs/heads/` branch
spelling; its dependent PRD message did not mean the existing PRD was missing.
Three independent read-only diagnoses supported the metadata-only
`task.py set-branch` correction to the current short branch. Immutable
`delivery.branch_ref` and all other delivery identity fields stayed unchanged.
All-target Clippy and docs were not reached in that failed run. Raw capture:
external `nu15/current-base-20261020/validation-runs/nu15-full-pr-8d04bd3c-fc94-4c2b-8012-95bfb3c80bea`.

The single corrected full-PR run on `a3b42f82` completed with exit 0 on
2026-10-20. Capture direct child PID 95651 and outer session 81922 both
terminated; the capture receipt is complete with return code 0. It reached
target-state audit, all-target/all-feature Clippy, documentation validation
(54 YAML examples), and the Astro/Pagefind website build (48 pages). Rust
output contains 110 summary rows: 1,485 passed, 0 failed, 1 intentional ignore.
Raw capture and census: external
`nu15/current-base-20261020/validation-runs/nu15-full-pr-9440bb25-46fe-4cec-b5a3-a5f582b727b4`.

The tested source stayed clean and unchanged. The five generated demo revisions
remain `7ca5e4e6`; manifest/lock and both protected performance payload hashes
remain unchanged. Retained stderr includes the already-reviewed optional
Starlight custom-404 lookup warning and existing dead-code warnings; no output
was suppressed. This later metadata note changes no gate or runtime input.

These results support the reviewed observability increment only. Current hosted
checks and final independent review are still required before root integration.
The historical Windows deadline cause and fix remain unproven; NU15 acceptance
and dependent NU13/NU14 closure are not claimed.
