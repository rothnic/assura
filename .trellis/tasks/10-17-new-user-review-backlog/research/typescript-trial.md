# Assura 0.4.1 field trial: existing TypeScript/Next.js project

## Outcome

The published CLI onboarded an existing Node/Bun/Next.js project, preserved local policy and guidance on repeat runs, and correctly enforced explicit naming, Markdown-link, and closed-scope policies. The main product gaps concern the quality plan generated for actual source locations, advisory review visibility, and repairable setup/closed-scope diagnostics. No Assura source changes were made.

This was a bounded CLI field trial on macOS x86_64, not a performance benchmark or proof of live host-hook delivery. All CLI invocations used the exact published candidate path; the original project was read only.

## Identities and scope

- Candidate: `/Users/nroth/workspace/.assura-review-20261017/bin/assura`, `assura 0.4.1`, Mach-O x86_64.
- Candidate SHA-256: `05355b9639d0295c51b3ccc7f077311cdb39b90a6cbc4583aae2239e389ac0fe`.
- Adjacent `assura-full` SHA-256: `4dfdb3dea035ac0fa0f8025864a356074bede59bd0c62dab09a66da93f26650c`.
- Existing project: `/Users/nroth/workspace/resume-chatbot`, revision `64823b9196f09057919430b9f9bd8668570dbc40`.
- Original dirty paths: modified `README.md`, `docs/project-specification.md`; untracked `.github/workflows/roam.yml`, `.roam/`, `docs/technical-overview.md`, `resume-chatbot-architecture.excalidraw`. Final original status is identical. These local changes were excluded from the trial by using `git archive HEAD`.
- Disposable copy: `project/` adjacent to this report. Git setup used `core.hooksPath=/dev/null`; no project source hooks were executed. No dependency install or build occurred.
- Disk-backed scratch: `/dev/disk1s2` at `/System/Volumes/Data`, with 136 GiB available when checked. `TMPDIR` was set to owned `tmp/` for child commands.
- Docs read from Assura checkout `/Users/nroth/.codex/worktrees/assura-new-user-review/assura`, revision `f3540d4a11d36afa43f69c96fd7f1c233599822a`: README, getting-started, agent-ready-onboarding, Markdown validation and agent-feedback reference.
- `transcript.json` contains 65 command receipts with exact argv, cwd, exit, stdout/stderr and elapsed time, plus identity/environment and mutation metadata. These are local elapsed samples; the longest Assura call was 0.329 s.

## New-user journey and probes

| Step | Receipt label | Observed behavior |
| --- | --- | --- |
| Discover CLI | `help`, `onboard-help` | Commands and onboarding options discoverable; help succeeds. |
| Before config | `doctor-before`, `review-before`, `check-before` | Each exits 4; stderr only says no `.assura/config.yml` found. No recovery command or JSON error object. |
| Documented start | `onboard-auto` | Exit 0 in 0.329 s; node/high confidence, Bun available, generic host/low confidence. Installed config, agent skills, docs and onboarding packet. |
| Read handoff | `read-.assura/onboarding/agent-next.md` | Clear evidence-first specialization and negative-test procedure. Does not overclaim complete project specialization. |
| Baseline check | `check-after` | Exit 0, 271 files/67 directories, 18 low/nonblocking violations: 11 max-lines and 7 child-limit findings. |
| Baseline review | `review-after` | `needs-review`, advisory=0, inactive=5. Raw/heatmap structure still reports the 18 violations. Main next action is deciding whether to use content models. |
| Common drift without extra policy | `baseline-mutation-check` | Unexpected root file, BadProbeName TSX, and broken Markdown link produce no direct diagnostic because these policies are unconfigured. A 501-line TSX adds one low max-lines finding; check stays exit 0. |
| Explain coverage | `baseline-mutation-explain` | Correctly states Markdown policy `not_configured` for broken-link file. |
| Review after extra advisory | `baseline-mutation-review` | Raw violations increase 18→19, but advisory=0 and history reports new=0/worsened=0, unchanged=7. No finding identifies the new oversized source file. |
| Repeat with owner edits | `onboard-repeat` | Exit 0; config comment and AGENTS owner rule retained byte-for-byte. Original `.agent`/`.cursor` guidance and `lefthook.yml`, plus seeded unmanaged `.git/hooks/pre-commit`, unchanged. |
| Activate concrete local host | `codex-onboard-activate` | Codex bundle and project hook wiring structurally verified. Report explicitly says runtime delivery unavailable pending user trust/approval. Existing `.codex/config.toml` retained. |
| Preserve unrelated host hook | `codex-activate-repeat`, `codex-deactivate` | Added an unrelated UserPromptSubmit entry before repeat activation; deactivation retains exactly that owner entry and removes Assura entries. Bundle remains. |
| Name/link specialization | `specialized-policy-clean`, `specialized-mutation-check` | Added documented component kebab naming and docs local-link policy. Clean project passes; BadProbeName and missing target generate precise medium errors and exit 1. |
| Advisory versus gate | `specialized-mutation-review`, `specialized-warn-agent` | Review stays advisory exit 0 with status=fail/blocking=2; warn-mode check exit 0. Gate check exits 1. This is intentional and works. |
| Closed root/source scope | `closed-policy-count-allowance-*` | Explicit root allowlist plus `.tsx: exists:0-100 \| kebab-case` in closed components scope: clean=0, root/source mutation=1, cleanup=0. Diagnostic identifies parent directory and count but omits unexpected filenames. |
| Quality routing | `quality-app`, `quality-test` | `app/(chat)/page.tsx` triggers only `assura check`; `tests/e2e/chat.test.ts` triggers `assura check`, `bun run lint`, `bun run test`. |
| Cleanup | `cleanup-check`, `final-tracked-diff` | All temporary bad files removed, main config restored, baseline passes with original 18 advisory findings. Tracked source diff empty. Project-local Assura integration deactivated. |

The initial closed-scope attempt used `.tsx: kebab-case` without an `exists` range. This names files but does not exempt matching files from a closed wildcard count: all 44 existing direct TSX files were rejected. Adding `exists:0-100` supplied explicit cardinality intent and made the policy work. The failed attempt remains in the transcript; it is configuration friction, not a claim that the validator is broken.

## Supported defaults and honest inactive state

- Leaving root/source naming and Markdown-link policy open at initial onboarding is documented: the handoff explicitly asks the agent to specialize from evidence. The baseline is not meant to infer every existing path as intended or silently enforce arbitrary naming.
- `project_specialization=configured_unverified`, content models inactive, and generic/no activated host were reported truthfully. A passing baseline is not complete specialization.
- `check` reports low violations without failing; `review` advisory exit 0 can contain blocking findings. These exit contracts worked.
- Native Codex trust and hook execution were not tested. The activation report clearly separates structural verification from unavailable host approval.
- Preservation results are in `preservation.json`: repeated onboarding and host activation/deactivation retained all seeded owner text/configuration and original project guidance.

## Suggested backlog cards

Each card below is independently implementable unless its acceptance section explicitly says otherwise. Shared release/regression gates are validation, not artificial dependency edges.

### TS-01 — P1: Include established source roots in generated native quality scopes

**User impact:** A new user following generated quality guidance can edit the main Next.js application without being told to run lint or tests, while equivalent test-directory changes get the native checks.

**Evidence:** `onboard-auto` generated `quality.scopes.bun.paths` limited to `src/**`, `tests/**`, `package.json`, `bun.lock`. This project uses established `app/`, `components/`, `lib/`, `hooks/`, `artifacts/` source roots. `quality-app` returns only policy/`assura check`; `quality-test` returns Bun lint/test too. No native commands were executed in this trial.

**Smallest scope:** When generating a new quality scope, derive source boundaries from project evidence or use a safe language-file scope with generated/dependency exclusions. Preserve existing user scopes and keep inferred coverage reviewable. The agent handoff should expose unresolved source roots before native-check coverage looks complete.

**Acceptance:** Existing Next/Bun fixture with app/components/lib sources and generated outputs; changes to each established source root select native lint/test for the applicable phase; generated/dependency files do not; repeat onboarding preserves user-authored scope values. A src-based Bun fixture continues to work.

**True prerequisites:** None. Do not block on content models, hook activation, or native test execution.

### TS-02 — P2: Surface active advisory regressions in compact Review

**User impact:** Review's top-level advisory count and history can remain unchanged when a configured source-size check becomes worse; the user must inspect raw heatmap counters or run a lower-level check to discover the new violation. Inactive optional content capabilities dominate the next actions.

**Evidence:** `check-after` has 18 low findings; the 501-line `components/long-probe.tsx` adds one in `baseline-mutation-check`. `baseline-mutation-review` still has advisory=0, new=0, worsened=0; no named finding for the added violation. The raw count is 19, so data is present but not promoted.

**Smallest scope:** Aggregate configured nonblocking structure findings into a bounded advisory signal, or explicitly distinguish omitted configured advisories and link to them. Include worsening/new active-policy evidence in next actions ahead of purely optional setup choices when appropriate.

**Acceptance:** On a passing project with a low max-lines rule, add one violating file: Review remains advisory exit 0, but summary/history exposes the new active-policy problem and offers a bounded path to its precise finding; remove the file and observe resolution. Existing inactive capability reporting stays truthful.

**True prerequisites:** None. If suppression is deliberate product policy, clarify the count/omission contract in Review docs instead of claiming no advisories.

### TS-03 — P2: Offer a concrete recovery path when first-run commands lack configuration

**User impact:** Users naturally try doctor, review or check first; they receive the same missing-file error without a next step. `doctor` cannot currently diagnose this most basic setup state.

**Evidence:** `doctor-before`, `review-before`, `check-before`: exit 4, empty stdout even with `--format json`, only missing `.assura/config.yml` stderr.

**Smallest scope:** Preserve the error exit contract while suggesting `assura agent onboard .` (and optionally minimal `assura init`) with the applicable project path. Define machine-readable failure behavior instead of silently treating `--format json` as inapplicable.

**Acceptance:** Empty existing Git project with no config: all three commands expose a usable recovery command; documented JSON behavior is tested; running recovery then check succeeds. Avoid writing files from read-only diagnosis commands.

**True prerequisites:** None.

### TS-04 — P2: Name unmatched children in closed-scope count diagnostics

**User impact:** A closed policy catches unexpected files but gives only a parent directory and aggregate wildcard count. Root violations have empty report path, so agent feedback requires another discovery pass to identify what to move/remove.

**Evidence:** `closed-policy-count-allowance-mutations` returns `Directory '.' has 1 files matching '*', expected 0` at path `""` and the equivalent components count, omitting `unexpected-probe.txt` and `components/unexpected-probe.rs` entirely.

**Smallest scope:** Include a bounded set of unmatched child paths in count-rule diagnostics/corrective context, retaining the aggregate cardinality rule and truncation count.

**Acceptance:** One and many unexpected files under root and nested closed scope; diagnostics name bounded offending paths with correct counts, deterministic ordering and honest omitted count. Explicit allowed file patterns remain excluded from unmatched results.

**True prerequisites:** None. A whole new closed-scope feature is unnecessary.

### TS-05 — P2: Reconcile integration availability labels in the agent-feedback reference

**User impact:** Users see supported local lifecycle commands described as “Supported next-release” after README and onboarding guide say v0.4.1 supports them, making the recommended setup path look unreleased.

**Evidence:** Docs revision `f3540d4...`, `website/src/content/docs/reference/agent-feedback.md` lines 257–270 use next-release for install/activate/update/deactivate/remove/status/doctor and event nudges. The same verified release successfully activates, inspects and deactivates Codex in this trial.

**Acceptance:** Supported released lifecycle/event rows cite the release-surface contract consistently; unreleased portions stay explicitly marked; docs link validation passes.

**True prerequisites:** None. Published binary/runtime feature availability is already proved for exercised Codex lifecycle commands; do not claim untested host delivery.

### TS-06 — P3: Add an existing TypeScript adoption example through negative proof

**User impact:** The generic getting-started minimal policy uses Cargo/Rust examples; a Next/Bun user still has to translate roots, closed-extension cardinality, generated boundaries, and native quality scopes from reference docs.

**Evidence:** The ordered handoff is useful, but manual specialization needed source/file naming, `markdown.check_links`, and explicit extension `exists` counts before a closed component scope passed. Starter quality routing misses the observed source roots (TS-01).

**Smallest scope:** One source-backed TS/Next/Bun walkthrough with existing config/guidance preservation, clear open-baseline versus closed-policy state, direct-extension count allowance, native-quality planning, negative cases and recovery. Avoid a new abstraction or template family without need.

**Acceptance:** Copyable example applies to an existing fixture without dependency installs; clean → unexpected root/source file, bad name, long file, broken local link → cleanup; exact exit behavior and inactive boundaries stated. Quality example should be validated after TS-01 if it depends on generated source coverage; otherwise use explicit project-owned paths.

**True prerequisites:** None for explicit existing-policy documentation. TS-01 only if claiming automatic generated quality coverage.

## Additional CI vendor-path diagnostic

Requested separately by the review orchestrator. Full failed-job log is `ci-failed.log`; command receipt is `ci-failed-log`.

Run [37214120576](https://github.com/rothnic/assura/actions/runs/37214120576) checked source `f3540d4a11d36afa43f69c96fd7f1c233599822a` on Ubuntu 24.04.5. The failed job was Performance Report. The native build succeeded; LS-Lint/native/warm report generation succeeded; native and warm gates passed. The failing command was:

```text
cargo xtask performance-no-slower target/performance/ls-lint-comparison.json
Performance no-slower gate failed for accepted fixtures in cohort realistic-equivalent (assura-cli must be <= ls-lint-cli).
many_configured_scopes_regression: assura-cli 11.812 ms > ls-lint-cli 11.650 ms
```

This is an actual failure of the source-defined strict performance contract, not an installer/download/toolchain setup failure. The observed gap is 0.162 ms (about 1.39%). A single hosted run does not distinguish a small source regression from runner variance; causation remains unproven. `.github/workflows/ci.yml` lines 715–721 generate 16-iteration comparison evidence and apply this exact no-slower gate. Do not relax its threshold or relabel it environmental without reviewing the attached comparison samples and a bounded equivalent reproduction. No benchmark or repair was run here.

## Limits and retained evidence

No application build, dependency installation, native lint/test execution, long-running watch/daemon, full Rust tests, global agent configuration change, or public install-script test was performed. Host-approved hook delivery remains unproven. The sandbox retains its generated baseline, review cache, deactivated integration bundle, unrelated seeded host hook/settings, and two policy examples for review; all intentionally bad files were removed. The original repository and Assura source/Trellis files were not changed.
