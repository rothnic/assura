---
title: Assura new-user review and dependency backlog
status: active
updated: 2026-10-17
---

# Assura new-user review and dependency backlog

Assura's configured validation engine and preservation behavior worked in
these trials. The first-day experience still has important gaps: Review omits
active nonblocking violations, onboarding can impose incompatible policy on
existing agent infrastructure, generated native-quality coverage misses real
application roots, the quickstart rejects its own generated guidance, and an
install can leave the old executable selected. Fix those before treating a
successful setup command as evidence of a dependable adoption journey.

This review proposes **12 new bounded items** and reconciles **10 unfinished
existing outcomes**. It preserves 22 historical terminal records without
recreating them as open work. No product fixes or Beads writes were performed.

## Evidence and workspace state

- Refreshed `origin/master`: `f3540d4a11d36afa43f69c96fd7f1c233599822a`.
  Fetched full history: the repository is no longer shallow. The original
  `docs/maturity-portfolio-strategy` branch is an ancestor, 621 commits behind.
- Review branch: `docs/assura-new-user-review-20261017`, based on that master.
  The old branch is retained. Its two untracked records were committed only
  to `archive/assura-prior-records-20261017` at
  `b69f4a3745834c2f2fbd53b30a212d9950ec1255`, as requested.
- Latest published release was freshly queried: [v0.4.1, published October 1](https://github.com/rothnic/assura/releases/tag/v0.4.1).
  Intel macOS archive checksum passed. Exact tested launcher SHA-256:
  `05355b9639d0295c51b3ccc7f077311cdb39b90a6cbc4583aae2239e389ac0fe`;
  adjacent companion:
  `4dfdb3dea035ac0fa0f8025864a356074bede59bd0c62dab09a66da93f26650c`.
  `src`, `crates`, `integrations`, and Cargo metadata are unchanged between
  the release tag and current master; runtime trials used the published pair.
- Two independent agents tested tracked snapshots of existing projects:
  `resume-chatbot` at `64823b9196f09057919430b9f9bd8668570dbc40`
  (Next.js/Bun, 65 receipts), and `self-hosted-agents` at
  `41b3a28a12436b43bfb152b3347ff3441ea664f5`
  (Python-led monorepo, 80 receipts). The first project's pre-existing dirty
  work was excluded from the snapshot and preserved; the second stayed clean.
- CLI experiments used explicit absolute executables and disk-backed scratch
  under `/Users/nroth/workspace/.assura-review-20261017`. Full raw receipts
  remain there; the [hash manifest](../../.trellis/tasks/10-17-new-user-review-backlog/research/raw-evidence-manifest.json)
  makes them independently inspectable. No project dependencies were installed.
- The native [public installer](https://assura.dev/install.sh) was executed
  with an isolated `BIN_DIR` and passed. An initial Python HTTP client got
  403, while the documented curl path returned 200 and installed successfully;
  that client difference is not an installer defect.

## What worked and what did not

| Journey | Current observation |
| --- | --- |
| Discover commands | Help and onboarding options are available. |
| Diagnose before setup | Check, Review and Doctor exit 4 for missing config, with no concrete recovery command; JSON stdout is empty. |
| Minimal init | Creates config and refuses accidental overwrite. The generic starter assumes optional Rust paths. |
| Existing-project onboarding | TypeScript project succeeds; Python-led project fails on existing `.agents/project-policy.json`. Assura self-onboarding also fails on existing `.agents/README.md` and inherited doc naming. |
| Preserve user work | Repeat runs retain owner config comments, guidance and hooks. Unmanaged Codex entries survive activation/deactivation. |
| Specialize policy | Explicit naming, local-link and closed-scope negative cases are detected. Unspecialized root/naming/link gaps are reported as inactive and are not claimed as validator bugs. |
| Advisory Review | Both projects retain advisory=0 despite configured low findings. A 501-line Markdown file growing to 700 lines produces no new/worsened Review finding. Raw violation/heatmap counts still contain the evidence. |
| Native quality routing | Generated Bun paths miss `app/`, `components/`, `lib/`, `hooks/`, `artifacts/`. An app edit selects only Assura; a test edit selects native lint/test too. These commands were planned, not executed. |
| Managed integration | Local Codex generation, activation, inspection and deactivation work; structural verification and unavailable host approval remain distinct. No live trusted host event was observed. |
| Watch | Separate bounded fixture emitted an initial full report, then a warm affected-path naming failure after a real file change. The owned watcher was terminated and reaped. |
| Quickstart | The exact Getting Started YAML rejects `AGENTS.md` produced by its own preceding recipe-init step; check exits 1. |
| Upgrade discovery | Installer reports success while `command -v assura` and `assura --version` still select earlier `/usr/local/bin/assura` 0.4.0. Both old/new locations were already on PATH. |

## Assura on Assura

The clean current-source policy passes with **13 nonblocking violations**;
11 are in `src`. Examples include `agent_nudge_delivery.rs` at 1,981 lines
under a 500-line limit. Low-severity findings intentionally do not make Check
fail. Review reports two recommended baseline folders and three inactive
capabilities, while omitting those configured violations from findings/history.

Controlled changes in a disposable source copy established these boundaries:

| Change | Actual result |
| --- | --- |
| Unexpected root file | Two medium findings; Check exits 1. |
| Bad Rust filename | Low naming/rustdoc findings; Check exits 0 by configured severity. |
| 501-line Rust file | Low line-limit/rustdoc findings; Check exits 0; Review still omits the advisory. |
| Markdown without frontmatter | Frontmatter failure; this alone is not link-check proof. |
| Markdown with valid frontmatter and missing local target | No finding for that file: maintained analysis docs have no active link policy. |
| Remove the first mutation set | Original 13 nonblocking findings return. |
| Follow Doctor's recommended onboard action | Recipe status says applied, but verification fails after new `.agents` closure and doc naming conflicts. Doctor's two folder gaps disappear; they were not the important failure. |

Doctor truthfully marks repository-reference policy unchecked. Review's 1,116
raw unresolved reference candidates are informational; they must be filtered
before any configured policy promotes them. NU10 asks for a bounded maintained
docs policy, not a blanket gate over that noisy inventory.

Current-master [Rust CI run 37214120576](https://github.com/rothnic/assura/actions/runs/37214120576)
is **failed**, specifically the strict LS-Lint no-slower comparison:
`many_configured_scopes_regression` measured 11.812 ms Assura versus
11.650 ms LS-Lint (0.162 ms, approximately 1.39%). Other test, install,
native-performance and warm-loop jobs succeeded. One run cannot establish
source regression versus runner variance. No threshold was relaxed or rerun
requested; NU11 retains the exact unresolved gate.

The topology audit found 46 prunable registrations, 15 unreadable worktrees
and 18 unmerged goal branches. Those are ownership/recovery obligations,
not permission to prune. `task start` for this research artifact was held by
the older P05 `integrated_pending_verification` receipt. Its owner was not
renamed and the old receipt was not closed to bypass that guard.

## Prioritized new work

Every item includes reproduction evidence, closure criteria, owning surface
and links to prior cards in the [structured backlog](../../.trellis/tasks/10-17-new-user-review-backlog/research/backlog.json).
All 12 new items are independently startable; shared release checks are not
invented sequencing dependencies.

| Priority | ID | Outcome | Prior related work |
| --- | --- | --- | --- |
| P1 | NU01 | Show active nonblocking violations in Review/history | R07, A06 |
| P1 | NU02 | Preserve established policy during onboarding | A02, A03 |
| P1 | NU03 | Cover actual source roots in native quality plans | A05, A03 |
| P1 | NU04 | Make the documented quickstart pass | Q02, W02 |
| P1 | NU05 | Detect an installation shadowed by an old executable | R05, R06 |
| P1 | NU11 | Resolve current-head performance evidence failure | R02, R03 |
| P2 | NU06 | Give missing-config recovery guidance | Q07 |
| P2 | NU07 | Name count-rule offenders and preserve actual limits | Q07, A06 |
| P2 | NU08 | Make Python/Rust starter health coverage explicit | A02, A03 |
| P2 | NU09 | Dispose of current self-check advisory debt | R07 |
| P2 | NU10 | Enable scoped local-link checks for maintained Assura docs | R07, P01 |
| P2 | NU12 | Reconcile legacy delivery holds and workspace ownership | Existing delivery records |

Start NU01, NU02 and NU03 in separate owned lanes when resources allow;
NU04 and NU05 are small independent first-day repairs. Investigate NU11
alongside them on a suitable benchmark host. Repeat the existing-project
negative/preservation checks after fixes. Those reruns are regression evidence,
not independent/blinded A07 credit or external pilot retention.

## Existing unfinished outcomes and dependencies

These are refreshed recommendations against dated Trellis records, not a
rewrite of their recorded lifecycle. Completed dependencies stay historical;
they are not recreated as open blockers.

| ID | Previously recorded state | Remaining action / actual dependency |
| --- | --- | --- |
| A07 | active, no-credit | Refresh source/protocol and independently evaluate initialization. Preserve A04/A05/A06 prerequisites; do not count these exploratory trials as acceptance. |
| W02 | blocked | Refresh release story, `next-release` integration labels and public-roadmap copy against v0.4.1; reassess its dated deployment hold. |
| R06 | pending | Reconcile platform/same-candidate install proof with today's Mac result; depends on W02, NU05 and NU11 plus previously terminal prerequisites. |
| W03 | verified | Reuse the existing local case study; publication remains separately held. |
| W04 | pending | Discovery/measurement after W02 and W03. |
| F01 | blocked | Prepared interview kit still needs Nick's participant/invitation authority. |
| F02 | pending | Independent external pilot/retention after R06, A07 and F01. |
| F03 | pending | Launch package after F02, W03 and W04. |
| F04 | pending | Approved publication/feedback after F03; no outreach authorized here. |
| F05 | pending | Evidence-based v1 decision after R06, A07 and F02, retaining Q02/Q07 evidence. |

The parked A04 reporting defect is fixed in current source and focused tests;
both field trials corroborate the structural/host-approval boundary. Do not
recreate that defect. The parked versioned-preview task is already merged as
#337 and ancestral to current master. Its old v0.3.0/v0.4.0 statements are
historical. [Reconciliation detail](../../.trellis/tasks/10-17-new-user-review-backlog/research/prior-work-reconciliation.md)
records those findings and the existing card map.

## Beads handoff

Local `bd` is 1.2.2. `bd where`, `context` and `info` find no authoritative
repository database; none was initialized. The native
[graph proposal](../../.trellis/tasks/10-17-new-user-review-backlog/research/beads-graph.json)
contains 23 nodes: one adoption epic, 12 new findings, 10 carried unfinished
outcomes, and 15 real blocking edges. Symbolic keys let the selected store
allocate its own issue IDs. Historical terminal work is excluded.

Graph fields and edge direction were checked against the
[installed-version source](https://github.com/gastownhall/beads/blob/6c124203e771433a3550c348771a5b5e27fd3c21/cmd/bd/graph_apply.go).
`from_key` is the dependent; `to_key` is its prerequisite. Parent membership
does not sequence children. Static validation found no duplicate keys,
missing dependencies or cycles. Native store validation remains unrun.

Before applying: select the authoritative store, inspect its existing issues,
reuse matching IDs, and reconcile the carried statuses/owners. The graph is
a one-time proposal; all native graph-created issues start open, so applying
it repeatedly would create duplicates and would not preserve an existing
verified/active lifecycle automatically. Then run the installed tool's
`bd create --graph <graph-file> --dry-run --json`; create only after that
store-specific review. No new scheduler, participant outreach or product
implementation is authorized by this import artifact.

## Review materials and limits

- [TypeScript trial](../../.trellis/tasks/10-17-new-user-review-backlog/research/typescript-trial.md)
- [Python-led monorepo trial](../../.trellis/tasks/10-17-new-user-review-backlog/research/python-trial.md)
- [Backlog validation](../../.trellis/tasks/10-17-new-user-review-backlog/research/backlog-validation.json)

Evidence checks passed via `cargo xtask evidence`. Product Rust/Cargo and
integration sources were not changed. No full test/clippy/benchmark rebuild,
cross-platform rerun, daemon/content-runtime journey, live trusted host event,
external pilot, publication or deployment was performed. Full existing-project
receipts remain outside the repository to avoid copying unrelated source
material into this review. These limits prevent release-readiness and external
user-acceptance claims.
