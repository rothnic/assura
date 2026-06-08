---
title: Deslopify Dead Path Classification
status: active
---

# Deslopify Dead Path Classification

## Summary

This classifies the code areas named by the deslopify PRD. No removal is safe
in this slice without changing tests, benchmarks, or supported command
evidence. The corrective action is containment: support docs now classify Rust
exports as unstable internal APIs, and follow-up rule goals define deterministic
detectors for future cleanup.

## Area Classification

| Area | Classification | Evidence | Action |
| --- | --- | --- | --- |
| `src/cli/check` | Current product | Implements `assura check`, self-check, LS-Lint-compatible structure validation, changed-path planning support, and performance timing hooks. | Keep. Continue module topology enforcement in `.assura/config.yml`. |
| `src/cli/performance_report` | Supported evidence command | `assura performance-report` is listed as supported in `docs/support-policy.md` and feeds CI/website performance evidence. | Keep. Changes require performance-reporting skill and performance gates. |
| `src/intelligence/**` | Experimental internal/test surface | Exported from `src/lib.rs`, used by `tests/intelligence_graph_tests.rs` and `benches/graph_benchmarks.rs`, but dependency graph validation is unsupported. | Keep contained as unstable internal API. Future removal requires moving or deleting tests/benches first. |
| `src/maturity/**` | Experimental internal surface | Used by maturity tests, constraint severity/trigger internals, CLI config/output helpers, and optional git signal code, but maturity detection is unsupported. | Keep contained as unstable internal API. Future cleanup should separate severity policy from maturity detection. |
| `src/validation/**` | Internal broad validation API | Used by compatibility and unit tests; includes pairing/resolver paths with ignored roadmap tests. | Keep contained. Pairing/resolver roadmap paths should be revisited by a test-relationship/public-surface rule task before removal. |
| `assura watch` | Experimental CLI wrapper | Support docs classify watch as experimental; current command is a truthful one-shot wrapper over `assura check`, not a release-grade long-running watch mode. | Keep experimental wording; do not advertise long-running behavior. |
| `crates/assura-check-cli` | Internal hot-check/performance support | Provides check-only binaries, compiled config runners, hot daemon/session clients, and tests used by performance and incremental-check evidence. | Keep internal. Do not present as a primary user-facing release surface until a dedicated goal promotes it. |

## Deterministic Follow-Up Signals

- Public-surface support drift can be detected by scanning manifest/CLI claim
  fields and `src/lib.rs` unstable markers.
- Abandoned module families can be detected by combining public export scans,
  test/bench import scans, and support-matrix rows.
- Ignored roadmap tests in internal modules are a useful dead-path signal, but
  they need a configurable test-relationship rule before they can block.
