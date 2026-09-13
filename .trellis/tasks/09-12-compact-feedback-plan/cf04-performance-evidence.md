# CF04 performance evidence

Measured 2026-09-12 on macOS x86_64 with the CF03 merge `6c70e23640229cb784bded5f3406e32a90fa2f25`, Rust 1.94.1, Node 25.6.0, and native LS-Lint 2.3.0. The clean current report was generated from docs-only CF04 commit `4351891f6cf6fb0ddd620298a9f477cfb69b7360` on the same code base.

The exact installed Codex adapter `.codex/hooks/assura-agent-nudge.py` was run 100 times per case from a clean clone with the release Assura binary. All runs returned zero and emitted zero routine bytes. Wall-time p50/p95/max in milliseconds:

| case | p50 | p95 | max |
| --- | ---: | ---: | ---: |
| idle prompt | 126.267 | 140.938 | 160.263 |
| one edit | 123.917 | 131.257 | 136.869 |
| five-file burst | 125.746 | 136.979 | 146.368 |
| config-shaped edit | 126.048 | 134.703 | 147.477 |

The current release CLI report uses 5 process-to-process samples per equivalent fixture and native LS-Lint 2.3.0. It remains a separate operation from installed-hook entry; the page must not use CLI rows as hook-latency proof. The clean current report measures 8/8 Assura-faster rows, aggregate ratio `1.183x`, and `1/8` strict 2x rows. A prior same-host 3-sample report measured `1.263x`, and the pre-clean 5-sample run measured `1.196x`; that variation is why the page uses the current checked report and avoids stronger universal or 2x wording.

No hook optimization is claimed in CF04. The measured hook p95 is above the CF03 aspirational 25 ms unchanged-hook target, so wording distinguishes the installed adapter from release CLI and persistent-session benchmarks.
