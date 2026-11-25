# Shared Rust export scanner

Beads issue `ae-tv6` under `ae-e6t` owns live state and closure.

## Acceptance

Topology and support matrices use one internal scanner for distinct sorted root Rust export families. Keep the existing scan grammar, including grouped and multiline imports, comments, nested module exclusion, and public re-exports. The matrix adds its own `rust:` identifiers. Preserve both consumers' rule IDs and output. Exercise the shared fixture through native and compiled checks and run their existing suites.
