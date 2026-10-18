# SP01 measurement checkpoint

Status: mechanism and correctness evidence are available; the public performance decision is held because the VPS was actively contended. The source change remains provisional and has not met its retention criteria. No speedup is claimed.

## Provenance

- Repo: `/home/ubuntu/data/projects/assura-sp01-filesystem-attribution-20261018-01/source/checkout` on verified `vps-9cb01956` (Ubuntu).
- Baseline source: `a4a0d23b4d5eabd5f62fb7bbfb8ce73fced4308f`, tree `b3841d82...`; candidate source: `d85f65ab23120f387f3b36166c80553fbc480b1e`, tree `d502559a...`.
- Compiler/build: `cargo +stable`, release profile, `CARGO_BUILD_JOBS=2`, private `CARGO_HOME` and `CARGO_TARGET_DIR`, disk-backed `/var/tmp/agent-runs-1000/sp01-20261018-01` as `TMPDIR`. Candidate release executable SHA-256 `cf1b99f45f4437c401d27de2e169728963d507d867446977658b18a83cb53177`; adjacent `assura-full` SHA-256 `00077c9f21438b09aca9df83fd59753f6752a0a35f3e4b33e4d50aa9cf2445da`.
- Fixture arms contain 5,000 files each. Config SHA-256: no-size `49916f46a13fa84f69c0a08f2cdb51dfbbf349b09d2fd7c7155360dd93f07c9f`; size-policy `eea78d3d956eef051a8bb7b473813a1e7d5abeb91a006a67dbf7d2c52cde05fd`; naming-only `f0e66a0fd77abf57e98b5483a4dccba18404eaa858c4f9ae65a85b60f0ae9440`.
- Raw, command, and gate receipts remain in the owned VPS run directory under `evidence/`; this report summarizes the key results without copying bulky traces into Git.

## Attribution and correctness

Temporary instrumentation on the baseline no-size arm counted 5,000 calls to `validate_file_size` with no effective max-size rule, and 5,000 metadata requests. The size-policy arm also counted 5,000 calls and requests. The naming-only fast path made no size-validator calls. Instrumented timings are diagnostic only.

Uninstrumented `strace -c` runs confirmed the candidate removes exactly 5,000 `statx` calls from the no-size arm (10,006 to 5,006). The size-policy arm stayed at 10,006, and naming-only stayed at six. All six commands exited zero. Tracer elapsed times are not performance evidence.

Base and candidate `assura check --format json .` matched stdout, stderr, and exit code byte-for-byte on all three 5,000-file arms. An edge fixture also matched exactly: 1,024-byte default-boundary passes; 1,025-byte default fails; a 1,025-byte pattern override under a 2 KB limit passes; zero-byte passes; a symlink to an oversized target and an unreadable mode-000 oversized file report the same findings; a broken symlink is skipped. Missing-target invocation returned the same exit code 3 and exact error text on both binaries. Candidate `cargo +stable test --locked --test pattern_scoped_file_directives` passed 5/5 tests.

## Public measurement hold

No uninstrumented public timing samples were taken. Host observations before profiling/timing showed unstable unrelated work: load average moved from `12.14, 10.98, 8.17` at 03:33:34Z to `8.60, 10.18, 8.03` at 03:34:16Z; CPU PSI `some avg10` was 11.09% then 15.83%. High-CPU unrelated processes included multiple `gc` sessions, `nogo_actual`, `du`, Java, Dolt, and later `bd`; no cargo/rustc work was active during these observations. An 8-second per-CPU sample at 03:36:03Z ranged from 22.7% to 96.7% busy, while high-CPU processes retained broad CPU affinity. Therefore there is no demonstrated quiet CPU or stable host interval to support the required balanced paired public comparison. Foreign work was not stopped or modified.

Next decision: take one bounded paired timing attempt only after an observed quiet interval with stable pre/post host activity; if that cannot be obtained, keep the public gain unproven and disposition the candidate as deferred rather than treating syscall reduction as a sufficient benefit. Strict no-slower/native/warm gates remain unrun because public timing is held.
