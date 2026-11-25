# Shared session freshness

Beads issue `ae-wi8` under `ae-e6t` owns live state and closure.

## Acceptance

One private helper owns the context, fingerprint and refresh state for content and editor sessions. The protocol adapters retain their request and response schemas, diagnostics and sequence. Verify initial load, a source change before the first valid request, reuse, malformed request sequencing, failed reload and recovery through both real CLI sessions.
