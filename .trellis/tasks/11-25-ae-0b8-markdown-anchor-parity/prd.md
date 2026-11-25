# Markdown anchor parity

Beads issue `ae-0b8` under `ae-e6t` owns priority, status and closure. This task holds the native delivery contract.

Both Markdown links and opt-in repository references must resolve heading anchors from the existing Markdown link check's heading and slug rules. Preserve path and line-range handling, rule IDs, severity and suppression. Keep one slug implementation and leave reference extraction and policy eligibility unchanged.

CLI acceptance exercises linked heading text, duplicate suffixes, fenced and frontmatter headings, a real missing heading, distinct severity and a Markdown-only suppression. Prove the fixture fails against the registered base for the expected repository-reference regressions, then run affected integration tests and the required candidate review and merge verification.
