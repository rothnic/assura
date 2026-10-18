# Pinned release red proof

Date: 2026-10-17. This is isolated local behavior evidence from the released executable, not a source build or test.

- Executable: `/Users/nroth/workspace/.assura-review-20261017/bin/assura`
- Version: `assura 0.4.1`
- SHA-256: `05355b9639d0295c51b3ccc7f077311cdb39b90a6cbc4583aae2239e389ac0fe`
- Disposable Git fixture: `/Users/nroth/workspace/.assura-review-20261017/nu01-red-20261017-b`
- Fixture HEAD: `346f9681156112a7d34697c69a2bf9c5085609fa`
- Policy: `structure./.children.src/.files.max_lines=500`, `severity=low`; `src/oversized-probe.ts` crossed 501 -> 700 -> 499 lines.

Observed command receipts, all from the exact executable above in the fixture root:

| State | Command | Exit | Observation |
| --- | --- | ---: | --- |
| 501 lines | `assura check --format json .` | 0 | `success=true`; one `max_lines` finding; `severity=low`, `blocking=false`. |
| 501 lines | `assura review . --format json` | 0 | `summary.advisory=4`; no finding identifies `src/oversized-probe.ts` or `max_lines`; history `new=11`, `worsened=0`, `unchanged=0`, `resolved=0`. |
| 700 lines | `assura check --format json .` | 0 | `success=true`; one `max_lines` finding; `severity=low`, `blocking=false`. |
| 700 lines | `assura review . --format json` | 0 | `summary.advisory=4`; no `max_lines` finding; history `new=0`, `worsened=0`, `unchanged=11`, `resolved=0`. |
| 499 lines | `assura check --format json .` | 0 | `success=true`; zero violations. |
| 499 lines | `assura review . --format json` | 0 | `summary.advisory=4`; no `max_lines` resolved finding; history `new=0`, `worsened=0`, `unchanged=11`, `resolved=0`. |

The earlier disposable directory ending `nu01-red-20261017-a` omitted `severity: low`, so its `max_lines` finding was medium/blocking and Check correctly exited 1. It is retained here only as a fixture setup miss, not as evidence for the acceptance behavior.

Both exact disposable directories were removed after confirming no executable process remained. No Cargo command, source test, VPS command, or global binary modification was performed for this receipt.
