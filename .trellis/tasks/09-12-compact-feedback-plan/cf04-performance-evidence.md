# CF04 performance evidence

Measured 2026-09-12 on macOS x86_64 with the CF03 merge `6c70e23640229cb784bded5f3406e32a90fa2f25`, Rust 1.94.1, Node 25.6.0, and native LS-Lint 2.3.0. The clean current report was generated from docs-only CF04 commit `a42ef40aab8960ce5bb5aaacd1f3edaefd0228ae` on the same code base.

The exact installed Codex adapter `.codex/hooks/assura-agent-nudge.py` was run 100 times per case from a clean clone with the release Assura binary. All runs returned zero and emitted zero routine bytes. Wall-time p50/p95/max in milliseconds:

| case | p50 | p95 | max |
| --- | ---: | ---: | ---: |
| idle prompt | 126.267 | 140.938 | 160.263 |
| one edit | 123.917 | 131.257 | 136.869 |
| five-file burst | 125.746 | 136.979 | 146.368 |
| config-shaped edit | 126.048 | 134.703 | 147.477 |

The current release CLI report uses 5 process-to-process samples per equivalent fixture and native LS-Lint 2.3.0. It remains a separate operation from installed-hook entry; the page must not use CLI rows as hook-latency proof. The clean current report measures 8/8 Assura-faster rows, aggregate ratio `1.241x`, and `1/8` strict 2x rows. Earlier same-host runs measured `1.263x` and `1.196x`; that variation is why the page uses the current checked report and avoids stronger universal or 2x wording.

No hook optimization is claimed in CF04. The measured hook p95 is above the CF03 aspirational 25 ms unchanged-hook target, so wording distinguishes the installed adapter from release CLI and persistent-session benchmarks.

## Fixed six-pair project trial

On 2026-09-12, a fixed six-pair trial used the release binary from this clean
candidate, isolated temporary Git clones, and explicit `agent_feedback` config
for the on arm (`periodic`, one-second cadence, 256-byte line cap). The off arm
used the same fixture operation with feedback disabled. Pair order was balanced
on/off, off/on, on/off, off/on, off/on, on/off. Each command returned zero; the
expected fixture operation was accepted in all 12 arms. Small unintegrated and
coordination-only cases remained unintegrated; healthy-delivery and the docs-only
negative control were fast-forward integrated in their isolated clone.

| pair | matched activity | on result | off result | routine text bytes (on/off) | nuisance reports (on/off) |
| ---: | --- | --- | --- | ---: | ---: |
| 1 | small unintegrated edit | trajectory | silent | 133 / 0 | 0 / 0 |
| 2 | coordination-only commit | trajectory | silent | 133 / 0 | 0 / 0 |
| 3 | healthy delivery | trajectory | silent | 122 / 0 | 0 / 0 |
| 4 | docs-only negative control | silent | silent | 1 / 0 | 0 / 0 |
| 5 | small unintegrated edit | silent | silent | 1 / 0 | 0 / 0 |
| 6 | healthy delivery | silent | silent | 1 / 0 | 0 / 0 |

The JSON envelope was 2,576–2,592 bytes when the automatic trajectory was
present; that is inspect/transport output, not injected routine text. Observed
automatic command wall time was 167–493 ms, and no model/provider cost or
productivity score was collected. The docs-only case produced no trajectory
line, as intended. This is a bounded implementation trial, not statistical
evidence; the decision is **keep** the explicit opt-in, default-silent contract
and tune only from future current evidence.

For a bounded resource probe, `/usr/bin/time -l` around 10 exact installed-hook
idle invocations reported 3.07 s real, 1.59 s user, 1.48 s system, 25,661,440
bytes maximum resident size, 626,688 bytes peak footprint, and zero block input
or output operations. The adapter is a short-lived hook process; no resident
collector remained after the batch, so there is no separate background CPU/I/O
consumer to attribute. This host probe is evidence for the adapter operation,
not a universal resource guarantee.
