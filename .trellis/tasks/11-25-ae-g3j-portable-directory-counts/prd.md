# Portable directory counts

Beads issue `ae-g3j` under `ae-e6t` owns priority, live state and closure. This task holds the native delivery contract.

## Acceptance

Live and compiled checks must agree on literal and wildcard self-directory counts when the matched directory is absent, empty or populated. Preserve count ranges, severity and finding semantics from `ae-dc7`, the supported artifact schema and source validation, fail-fast behavior and the fast-only route. Exercise both CLI profiles. Omit redundant work on the naming-only path and verify the existing startup and no-slower gates.

The full-profile CLI baseline demonstrated two false passes. An existing wildcard match under a zero-count policy bypasses validation in the compiled walk. A missing literal or wildcard scope under a positive minimum bypasses the configured-directory check in a fast-only artifact. A live literal zero-count failure also emits two identical findings. Keep the proof for these cases separate, then compare builds against the actual same-version baseline artifacts.
