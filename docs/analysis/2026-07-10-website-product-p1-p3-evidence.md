---
id: analysis-2026-07-10-website-product-p1-p3-evidence
type: analysis
title: Website roadmap product P1-P3 evidence
status: active
created: 2026-07-10
owners:
  - assura-maintainers
---

# Website Roadmap Product P1-P3 Evidence

## Outcome

The product surfaces needed by the website roadmap are implemented and covered
by deterministic tests. Review remains an advisory radar; Check remains the
policy gate.

## Product P1

| Outcome | Evidence |
| --- | --- |
| Explicit automatic or named Git base | `assura review --base <auto|ref>` plus invalid-ref and no-common-ancestor tests |
| Compact scan-first tree | Text renderer reports branch, worktree, thresholds, and nested hot directories without serializing the internal ranking score |
| Stable bounded finding state | SHA-256 fingerprints, rule-aware pressure, `new/worsened/unchanged/resolved`, five hot directories, 12 agent findings, and six next actions maximum |
| Advisory command semantics | A report with blocking findings exits `0`; `assura check` retains nonzero gate behavior |
| Onboarding verification | `agent onboard` embeds Review without reading or mutating finding history |
| Versioned automation | Review JSON is `assura.project-review.v2`; agent JSON is `assura.project-review.agent.v2` |

Focused proof:

```bash
cargo test -p assura --test project_review_cli
cargo test -p assura --test project_intelligence_onboarding
cargo test -p assura cli::project_review::history::tests --lib
cargo xtask website-demo-data --check
```

## Product P2 Warm Loop And Cache

| Outcome | Evidence |
| --- | --- |
| Safe file-local reuse | Direct file checks persist size, modification time, and content hash; `PreparedStructureCheck` and daemon changed-path checks validate a touched path and affected parents |
| Directory invalidation | Cache schema v4 fingerprints directory modification time, child count, child names, and child types; rename/type/config/corruption tests force safe recomputation |
| Worktree isolation | Local reports use `worktrees/<project-key>/...` namespaces |
| Shared immutable reuse | Clean worktrees at the same Git object reuse `shared/<common-repository-key>-<HEAD>/...`; an integration test proves a sibling worktree returns without creating a local record |
| Observable fallback | `assura cache status --format json` reports root, worktree/shared namespaces, entry count, bytes, and the reason shared reuse is unavailable |
| Explicit cleanup | `assura cache clean --format json` reports removed entries/bytes and leaves an empty selected root |
| Repeated-message cooldown | `assura agent nudge` suppresses identical event messages for 300 seconds by default; `--cooldown-seconds 0` disables it |
| Five checked latency budgets | `no-change-warm-review`, `one-file-change`, `directory-create-delete`, `config-change`, and `agent-nudge` have versioned p95 budgets, current/history JSON, and a CI regression gate |

Performance proof:

```bash
cargo build --release --bin assura-full
cargo xtask warm-loop-benchmark \
  --binary target/release/assura-full \
  --iterations 20 \
  --output benches/history/warm-loop-current.json \
  --history benches/history/warm-loop-history.jsonl
cargo xtask warm-loop-no-regression benches/history/warm-loop-current.json
```

CI runs the same five rows on its stable runner and uploads current/history
artifacts. Local checked-in measurements are reviewable baseline evidence, not
a cross-machine performance guarantee.

## Product P2 Lifecycle

| Roadmap behavior | Current surface |
| --- | --- |
| Map agent events to bounded feedback | `assura agent nudge --event session-start|before-tool|after-tool|file-read|idle|recovery` |
| Cool down repeated messages | Stable message fingerprints and configurable cooldown state |
| Show approaching thresholds | Review serializes measured values beside blocking, worktree-file, untracked-file, churn, and commit thresholds |
| Keep advisory events exit-zero | Review and agent nudge return success after a report is assembled |
| Route gates through Check | Only `assura check` controls policy pass/fail |
| Diagnose integrations | `assura agent integration status|doctor` and daemon status/doctor commands |

## Product P3 Universal Signals

The language-agnostic stack now covers every prioritized family in the imported
roadmap: naming and placement, required/forbidden paths, child pressure, line
and Markdown-section thresholds, generated-output boundaries, branch/worktree
churn, Markdown/reference health, agent-guidance contracts, frontmatter
references, typed records and relationships, binary source custody,
requirements/evidence traceability, and bounded computed checks.

These families remain layered. Structure and review provide the broad starting
point; project-specific and language-specific checks attach through existing
configuration and computed-check boundaries rather than a second validation
engine.
