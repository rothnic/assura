# Backlog reconciliation at f3540d4a

Reviewed source: `/Users/nroth/.codex/worktrees/assura-new-user-review/assura`, full HEAD `f3540d4a11d36afa43f69c96fd7f1c233599822a`. Parked records: `archive/assura-prior-records-20261017` / `b69f4a3745834c2f2fbd53b30a212d9950ec1255`. Read-only analysis; only this report was written. Hindsight search/list returned no pages.

## Parked A04 note

Disposition: preserve as historical research, do not import its original reporting defect as open work. The narrow defect is fixed in current source and has focused contract tests.

- `src/cli/agent_integration.rs:69-141` now calls lifecycle, then post-activation status and integration doctor; returns doctor facts plus explicit `host_approval_status = unavailable` when approval is required. `:145-172` uses `structurally_verified` and gives host-specific follow-up.
- `src/cli/agent_onboarding.rs:482-529` serializes structural verification separately, post-activation doctor facts, and host approval. `src/cli/agent_onboarding_report.rs:232-269` contains the corresponding schema. The legacy `verified` bool remains a structural alias (`agent_onboarding.rs:505`), so consumers must not reinterpret it as runtime proof.
- `tests/agent_integration_cli.rs:393-465` requires structural verification, post-activation doctor pass/facts, unavailable host approval, and the concrete Codex approval follow-up, in JSON and text.
- `src/cli/agent_integration_bundle.rs:433-435` requires host approval for Codex/Claude/Pi, not OpenCode. Actual host trust is not observed: the report remains unavailable even if a human approved the host outside Assura. That is an honest observability limit, not a newly reproduced failure.
- `.trellis/tasks/09-04-maturity-portfolio-strategy/research/evidence/A04.md:3-26` explicitly closes the LOCAL lifecycle card after supported pre-commit integration; it explicitly does not claim real host delivery. The report fix is attributed to PR #208 at `:178-198`. Historical later “A04 remains active” paragraphs are superseded by the leading state, not reasons to reopen the card.
- Current tests retain real Git event coverage (`tests/git_hook_lifecycle.rs:747`, advisory/opt-in blocking push) and host-adapter event tests (`tests/agent_integration_cli.rs:207-317`). Adapter execution in a test is still not host trust/real host dispatch proof.
- `.trellis/spec/assura/agent-harness-hooks.md:54-66` distinguishes project-local activation, structural file doctor, and logs. `research/initialization-packet.md:53-59` retains the full acceptance wording; A07 owns whole-journey acceptance (`:98-115`), where required unavailable evidence cannot pass.

Remaining boundary: independently observe real approved host dispatch if a new backlog item claims runtime activation/soak. Do not schedule the already-fixed reporting slice again or promote doctor output to runtime proof.

## Parked versioned-artifact task

Disposition: completed implementation, preserve parked task as historical completion evidence; do not reopen the versioned-preview feature.

GitHub compare API read on 2026-10-17 for `69e6ca109d27360f90d7cbd886586324f79003c2...f3540d4a11d36afa43f69c96fd7f1c233599822a` returned `status=ahead`, `ahead_by=72`, `behind_by=0`, and merge-base `69e6ca109d27360f90d7cbd886586324f79003c2`. Thus #337 is current-head ancestry. Local history initially contained only three commits and `--is-shallow-repository=true`; its initial negative merge-base exit was not ancestry evidence.

Current implementation persists: `.github/workflows/ci.yml:279-299` invokes the artifact contract, names Linux previews with package version plus full source SHA, and asserts BOTH binary versions; `:350-370` does the Windows equivalent; `:322-330` and `:428-436` retain previews for 14 days. `xtask/src/main.rs:6932-6944` contains the contract test including CRLF behavior. `docs/release-train.md:51-59` explains temporary versioned preview artifacts.

Historical statements are stale today: `Cargo.toml:3` is 0.4.1; local v0.4.0 points to `ad6d104d`, v0.4.1 to `7ab1d826`; `docs/release-notes.md:8-15` and `.trellis/spec/assura/roadmap.md:49-53` record v0.4.1 published/live-verified on 2026-10-01 with source/run identity. This review did not freshly install release assets or certify all current release checks. The old “release workflow is tag-driven only” statement also needs historical qualification: `.github/workflows/release.yml:3-12` now allows explicit workflow_dispatch publication recovery for a named immutable tag.

## Existing backlog map / prevent duplicate beads

Authoritative queue file: `.trellis/tasks/09-04-maturity-portfolio-strategy/research/backlog.json`, `updated=2026-09-12`, historical `source_snapshot=ed093668...` (`:1-8`). Statuses below are RECORDED states, not new certification. Counts: 21 done, 1 not_needed, 1 active, 1 verified, 2 blocked, 6 pending. The checkpoint’s “unfinished=4” describes active/verified/held rows; it does not eliminate six dependent pending rows.

| Existing ID | Recorded state / dependencies | Relevant ownership and current evidence |
|---|---|---|
| A01-A06 | done; A01←P01, A02←A01, A03←A02, A04/A05←A03, A06←R01+A04 | Existing evaluator, patterns, specialization, local lifecycle, native quality, bounded feedback work. Queue `:151-233`; map new concrete bugs to owning surface, do not recreate broad epics. A04 source/test confirmation above. |
| A07 | active; A04+A05+A06 | Whole initialization and follow-up acceptance, queue `:236-249`. Evidence `evidence/A07.md:3-28` is explicitly preparation/no-credit; latest source pointer 74f565c is stale against f3540d4a. Packet `initialization-packet.md:98-115` preserves independent/blinded/holdout requirements. Continue this outcome; agent-led exploratory review does not complete it. |
| R05 | done; B00 | Installer hardening, queue `:93-104`. Evidence header says implemented/pending, but its final `evidence/R05.md:125-134` records merged closure; do not read only its header or recreate installer epic. Current CI contains replacement/version/checksum checks. |
| R06 | pending; P01,R01,R02,R03,R04,R05,R07,Q02,W01,W02 | Public release/install proof, queue `:355-375`; referenced `evidence/R06.md` is absent. Packet `reliability-packet.md:91-101` requires one candidate, platform/install proof, displayed commands, post-publication installed path. Reconcile against v0.4.1 release evidence before importing pending status. A released version alone does not prove all retained R06 acceptance. |
| W02 | blocked; P01+R02+W01 | Release-aware installation/docs/product story, queue `:339-352`, packet `adoption-packet.md:16-27`. Blocker and evidence are dated Sep 8 PR #187/no-deploy uncertainty; current release docs have since advanced. Re-audit exact remaining diff/authority rather than importing old deployment hold as current fact or duplicating locally prepared work. |
| P01 | done; B00 | Product boundary/support ledger/roadmap. Use as scope owner for new support claims. Public roadmap and release status drift is a narrow reconciliation child, not a second product roadmap. |
| Q02 | done; P01+Q01+R04 | Contributor docs and review/enforcement authority; `quality-packet.md:36-48`. Specific fresh-user setup instruction failures belong in a narrow follow-up, with reproducible steps. |
| Q03 | done; Q01+P01 | Config authority; queue `:252-264`, packet `quality-packet.md:50-60`. Current `tests/config_authority_contract.rs:61` retains explicit-config agreement coverage. A concrete parser/inspection inconsistency warrants a regression child, not another consolidation epic. |
| Q07 | done; Q01 | Error/fallback contracts; queue `:311-322`; `tests/launcher_error_contract.rs:8` retains companion-boundary coverage. Map newly reproduced misleading exits/output separately. |
| R02/R03 | done; B00 / R02 | Performance evidence and many-scope repair. Current-head hosted no-slower failure below means historical done is not current gate pass. Create narrow regression/investigation child linked to these IDs; do not erase their prior completion. |
| R07 | done; B00 | Self-check advisory dispositions. New current findings need current self-check triage rather than automatically reopening all historical exceptions. |
| W03/W04 | verified / pending; B00+P01 / W02+W03 | Existing case study and discovery; W03 publication separately held (`backlog.json:378-405`). Avoid duplicating article work. Outside the core install/config review. |
| F01 | blocked; B00 | Interview kit prepared; actual participant selection/invitations owned by Nick, queue `:408-419`; packet `adoption-packet.md:54-64`. Internal project exercises are useful exploratory evidence, not participant demand. |
| F02 | pending; R06+A07+F01 | External independent pilot/retention, queue `:422-435`, packet `adoption-packet.md:66-76`. Preserve dependencies; internal agent trials do not satisfy external retention acceptance. |
| F03/F04/F05 | pending; F02+W03+W04 / F03 / R06+A07+F02+Q02+Q07 | Launch, authorized publication/feedback, v1 decision. Keep downstream outcomes dependency-bound; packet `adoption-packet.md:78-111`. |

Public artifact drift: `docs/data/public-roadmap.json:3` still says 2026-09-05; its Now/Next rows at `:19-29` remain coarse umbrellas for reliability, safe initialization, evaluation and release. `website/src/content/docs/roadmap.mdx:10-13` says “moving toward a beta release” even though canonical roadmap/release notes now record beta v0.4.1 published. `progress-current.md:3-36` latest checkpoint is Sep 12 and references historical source. Before Beads import, create one bounded reconciliation item to attach fresh source/release/CI evidence and map old IDs; preserve append-only history and explicit external authority. Migration is not permission to execute A07, publish, invite users, or weaken tests.

## Extra diagnostic: task owner gate

Recommendation: keep the already-registered research task in planning and record its precise hold; do not change its existing owner string merely to pass start. New task intent is owner nroth; old P05 correction is also owner nroth. `common/delivery_cli.py:161-195,197-245` checks unfinished candidates of the SAME coordinator, and `:877-883` intentionally rejects starting another one. Existing registration explicitly rejects changing owner/candidate (`:368-370`). `delivery-lifecycle.md:102-116` treats owner as receipt binding identity with state-preserving rejection. I found no supported owner-transfer command. A genuinely separate coordinator can own a distinct new candidate from outset; that is not evidence that relabeling this existing owner after rejection is supported. Do not mutate old receipts or claim old integration complete to free the lane.

Research can be completed while remaining a planning artifact. There is a narrower supported closure path worth using: unchanged existing `delivery register` ensures this artifact receipt without invoking `pending_owned_candidate` (`common/delivery_cli.py:368-390`); close explicitly allows “delivery register or start” (`:715-718`). Artifact delivery requires genuine `acceptance_verified` and `durable_evidence_verified`, not integration completion (`common/delivery_status.py:539-547,757-758`). Therefore register the unchanged artifact intent, record actual exact-tip durable output and acceptance evidence, and close only this artifact when its predicates pass. This does not change the old P05 candidate. I did not execute or live-test that mutation sequence. If the CLI unexpectedly blocks it, preserve the rejection and report the research as delivered evidence with formal task closure held; do not relabel owner or weaken predicates.

## Extra diagnostic: current-head CI

[Run 37214120576](https://github.com/rothnic/assura/actions/runs/37214120576), Rust CI, push head f3540d4a: conclusion failure. Every reported job except Performance Report succeeded. Failed step: Enforce no-slower gate. Exact log:

`Performance no-slower gate failed for accepted fixtures in cohort realistic-equivalent (assura-cli must be <= ls-lint-cli).`

`many_configured_scopes_regression: assura-cli 11.812 ms > ls-lint-cli 11.650 ms`

This is about 1.39% slower. Small magnitude does not establish runner noise; no cause is proven here. Logs still contain successful artifact uploads after failure, which must not be mistaken for a green gate. No rerun, threshold change, or performance repair was attempted.

Skipped: builds, test execution, release installation, host trust/runtime observation, task/receipt mutations, Beads import, deployment, and external outreach. All test statements above describe present assertions or dated historical evidence, not fresh passes.
