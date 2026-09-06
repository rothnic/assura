# Errors and effects references

Read this for filesystem paths, cached state, rendered output, or subprocess
changes.

## Fallback has an owner and a visible reason

`src/cli/project_review/history.rs` accepts an unreadable *review-history*
cache by starting fresh and records `fallback_reason`. That is a non-authoritative
advisory cache fallback; it is not precedent for losing check findings, report
output, or a subprocess error.

## Do not turn a failed effect into success

`src/main.rs::run_companion` currently distinguishes a missing companion path
from invoking the full CLI; a present companion that cannot launch needs its
own error path and OS diagnostic. `src/cli/agent_onboarding_report.rs::render_report`
currently uses `unwrap_or_default` for JSON/YAML rendering, so a future change
must propagate rendering failure instead of emitting an empty success payload.
Q07 owns those behavior changes; use its contract rather than changing them as
collateral cleanup.

For new work, state which effect may fall back, who observes the reason, and
which exit/report contract remains intact. Preserve `CheckError`,
`StructureCheckReport`, and CLI exit mapping unless the relevant support decision
explicitly changes them.
