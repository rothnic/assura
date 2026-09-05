# Assura Maturity Execution Backlog

> For agentic workers: use `superpowers:executing-plans` when available and follow the repository's inline Trellis workflow. Execute one card at a time. The complete standalone instructions are in [executor-prompt.md](executor-prompt.md). For a cross-session release train, start with the [stable E2E goal prompt](e2e-goal-prompt.md).

**Goal:** Deliver a dependable repository-policy tool and a credible demonstration of Nick Roth's technical product and AI systems leadership.

**Architecture:** One authoritative configured-policy engine, thin CLI/harness adapters, project-owned patterns, bounded feedback and independently evaluated initialization. Existing language tools execute their own checks. Retain supported broader features while freezing speculative expansion.

**Tech stack:** Rust/Cargo, existing xtask and GitHub Actions; Astro/Playwright on the Assura site; separate Astro NickRoth repository.

**Spec:** [strategy](../prd.md), [code findings](code-quality.md), [positioning](positioning-launch.md).

**Planning review:** [review disposition and validation limitations](backlog-review.md).

## How to execute

The machine-readable queue is [backlog.json](backlog.json). Each ID has a solution card in the packet linked below. `pending` means not implemented, not ready by itself. Select the first pending row whose dependencies are done and whose required authority/environment is available. An implementation-dependent row must also have its dependencies in the current Git ancestry. Prefer the listed order; independent rows can proceed when an earlier row is externally blocked.

1. Start with **B00**, fresh GitHub state and execution isolation. The strategy checkout is older than master; do not implement on it.
2. Then **P01**, make the product boundary and support contract explicit.
3. Next **R01** and **R02**, restore trustworthy watch behavior and performance evidence collection. R03 resolves the actual slow fixture.
4. **F01** can prepare interviews immediately after B00; it does not need a released product. An agent may prepare materials, but Nick authorizes invitations.
5. Follow dependencies thereafter. Never mark a dependent release/pilot/outreach outcome complete because its draft or test fixture exists.

A card is normally one reviewable PR. If its changed-file scope exceeds the packet ownership or its solution requires a new public contract, stop that card with the concrete discovery and proposed amendment. Continue another ready card if possible. Do not ask an agent to invent a roadmap while fixing a test.

## Queue and ownership

| ID | Deliverable | Dependencies | Packet |
| --- | --- | --- | --- |
| B00 | Current-source execution baseline and worktree | none | [Reliability](reliability-packet.md#b00) |
| P01 | Product boundary, support ledger and canonical roadmap | B00 | [Reliability](reliability-packet.md#p01) |
| R01 | Watch scope regression fixed with deterministic proof | B00 | [Reliability](reliability-packet.md#r01) |
| R02 | Independent performance reports and honest failure artifacts | B00 | [Reliability](reliability-packet.md#r02) |
| R03 | Many-scope performance regression resolved | R02 | [Reliability](reliability-packet.md#r03) |
| R04 | Supported Rust minimum and feature matrix enforced | B00 | [Reliability](reliability-packet.md#r04) |
| R05 | Existing installer work completed, without duplication | B00 | [Reliability](reliability-packet.md#r05) |
| R07 | All current self-check advisories resolved or justified | B00 | [Reliability](reliability-packet.md#r07) |
| Q01 | Project Rust quality skill and routed reference examples | B00 | [Quality](quality-packet.md#q01) |
| Q02 | Human/agent contribution contract and review safeguards | P01, Q01, R04 | [Quality](quality-packet.md#q02) |
| A01 | Independent initialization evaluator and fixture contract | P01 | [Initialization](initialization-packet.md#a01) |
| A02 | Small reusable/local pattern catalog and safe application | A01 | [Initialization](initialization-packet.md#a02) |
| A03 | Agent-led specialization with explicit decision state | A02 | [Initialization](initialization-packet.md#a03) |
| A04 | Hook setup, preservation and real activation verification | A03 | [Initialization](initialization-packet.md#a04) |
| A05 | Useful native quality plans and authoritative CI recipe | A03 | [Initialization](initialization-packet.md#a05) |
| A06 | Feedback latency/output budgets and quiet idle behavior | R01, A04 | [Initialization](initialization-packet.md#a06) |
| A07 | Blinded candidate and holdout evaluation passes | A04, A05, A06 | [Initialization](initialization-packet.md#a07) |
| Q03 | Config authority and parser consumers consolidated | Q01, P01 | [Quality](quality-packet.md#q03) |
| Q04 | Cohesive policy modules and canonical visibility | Q03 | [Quality](quality-packet.md#q04) |
| Q05 | Experimental maturity scoring contained or retired safely | P01, Q01 | [Quality](quality-packet.md#q05) |
| Q06 | Focused xtask/performance plumbing simplification | Q01, R02 | [Quality](quality-packet.md#q06) |
| Q07 | Observable subprocess/report errors and safe fallbacks | Q01 | [Quality](quality-packet.md#q07) |
| W01 | All marketing Start links resolve to usable setup | B00 | [Adoption](adoption-packet.md#w01) |
| W02 | Focused, release-aware product story and demonstration | P01, R02, W01 | [Adoption](adoption-packet.md#w02) |
| R06 | Same-candidate release and public-install proof | P01, R01, R02, R03, R04, R05, R07, Q02, W01, W02 | [Reliability](reliability-packet.md#r06) |
| W03 | Existing NickRoth case study revised and ready to publish | B00, P01 | [Adoption](adoption-packet.md#w03) |
| W04 | Cross-links, accessibility, discovery and measurement | W02, W03 | [Adoption](adoption-packet.md#w04) |
| F01 | Interview kit and demand decision record | B00 | [Adoption](adoption-packet.md#f01) |
| F02 | External pilot evidence and retention findings | R06, A07, F01 | [Adoption](adoption-packet.md#f02) |
| F03 | LinkedIn/demo/community launch package | F02, W03, W04 | [Adoption](adoption-packet.md#f03) |
| F04 | Approved publication and feedback response loop | F03 | [Adoption](adoption-packet.md#f04) |
| F05 | Evidence-based v1 decision and roadmap pruning | R06, A07, F02, Q02, Q07 | [Adoption](adoption-packet.md#f05) |

R06 can release the existing narrowed contract before A07 only if it does not advertise unattended specialization. After A07, release any new profile/init functionality through the same R06 procedure against the new candidate SHA; earlier install proof cannot certify later changes. Q03–Q06 are bounded cleanup candidates, not automatic release blockers: the cards require a concrete cost/consumer finding before refactoring. W03 can publish a truthfully labeled work-in-progress case study before the pilot; F03 updates it with measured outcomes.

## Global constraints and proof rules

- Baseline on 2026-09-04: GitHub master `ed093668918bc271fc98b9112acaf7c1bf3eb314`, source `0.4.0`; refresh before execution. Never infer public installation availability from Cargo version.
- Keep `assura check --format agent --agent codex` as the stable feedback route. Reuse existing integration lifecycle commands. No per-host validation engines.
- Agent decides structure from explicit local intent and evidence; profiles are editable choices. Project intelligence expansion, remote pattern execution and arbitrary automatic repair are out of scope.
- Preserve unrelated changes, user configuration, hooks and local pattern overrides. A source/behavior patch must not weaken policy to pass. Any justified policy adjustment is a visible separate decision.
- Rust changes: focused red/green contract test, then `cargo xtask fast`; before a PR run `cargo xtask pr` and relevant feature/OS/release gates. Inspect tier definitions before running; report baseline failures separately. Docs-only changes use structure/evidence and docs checks as applicable.
- Native test commands must run from the correct project cwd. Record cwd, SHA, exact binary path/version, command, exit status and relevant result. No “passed” from a zero-test run when tests were expected.
- Use the exact candidate's compiled binary for evaluation. Prefer `cargo run --bin assura-full -- …` while developing full-CLI behavior; packaged two-binary tests must use packaged artifacts. Routine user install instructions must not depend on a temporary build path.
- Independent review before complex PRs; reviewer findings are evidence for maintainer judgment. Public support removals, CI policy relaxation, merges, releases, deployments and external communication require the corresponding explicit authority.
- Offload sustained benchmark/evaluation runs to an approved isolated checkout on vps-dev when available. Keep local work bounded. Never move production services or copy credentials for this backlog.

## Completion record

After a card, create `evidence/<ID>.md` alongside this file, using:

```markdown
# R01 completion evidence
State: verified
Source SHA: actual commit
Worktree: actual absolute path
Problem reproduced: command, exit, observed failure
Change: files and behavior
Checks: exact commands, cwd, exit, meaningful counts
Negative control: why the old/broken behavior is rejected
Review: findings and disposition
Remaining: unresolved limitations, or none
Integration: commit/PR and whether merged
Next ready ID: ID with dependencies present
```

Use `blocked` with a specific missing dependency/decision and completed independent work. `not_needed` requires proof of existing equivalent behavior, consumer/churn evidence, or an explicitly approved scope exclusion; it cannot replace a failed gate. `done` requires the card's actual observable outcome. Store source evidence locally and publish only redacted material.

## Concern-to-solution coverage

| Concern from review | Closing cards |
| --- | --- |
| Latest CI fails; check success includes advisories | R01, R02, R03, R07 |
| Public release/site mismatch and stale setup instructions | R04, R05, R06, Q02, W01, W02 |
| Duplicate config, CLI/domain boundary, include splitting, reexports | Q03, Q04 |
| Maturity proxies, low-value tests, hypothetical abstractions | P01, Q01, Q05 |
| xtask size, performance arguments, swallowed errors | Q06, Q07 |
| Agents can make a permissive passing setup | A01, A02, A03, A07 |
| Hooks skipped or merely generated; ineffective/expensive gates | A04, A05, A06 |
| Contributions exceed reviewer capacity or bypass policy | Q01, Q02 |
| Portfolio gap, duplicated content, broken site journeys | W01, W02, W03, W04 |
| Unsupported marketing, Reddit reception, no feedback loop | F01, F02, F03, F04 |
| Scope creep and vague maturity endpoint | P01, F05 |
