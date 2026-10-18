# Independent Python project adoption trial — 2026-10-17

This was a field trial of the published Assura 0.4.1 binary on a clean tracked copy of an existing Python-led monorepo. It found two high-priority adoption/review problems and two smaller starter/diagnostic gaps. The validation engine caught explicitly configured naming, size and link violations correctly. All original source remained unchanged.

## Identity and isolation

- Original project: `/Users/nroth/workspace/self-hosted-agents`, commit `41b3a28a12436b43bfb152b3347ff3441ea664f5`, 440 tracked files, clean before and after.
- Stack: root `pyproject.toml` packages the Python `awf` CLI from `tools/agent-workflow/src/agent_workflow`; additional Python and TypeScript applications live under `apps`. This is a real Python-led monorepo, not a Python-only fixture. Assura detected `monorepo` with high confidence.
- Original has `.agents/project-policy.json`, 22 existing skill entrypoints, 330-line `AGENTS.md`, `.githooks`, and configured `core.hooksPath=.githooks`.
- Two disposable `git archive HEAD` copies (`minimal`, `onboard`) were initialized as independent Git repositories with `core.hooksPath` pointing to an empty owned directory. No original hooks were executed.
- Exact executable: `/Users/nroth/workspace/.assura-review-20261017/bin/assura`, version `assura 0.4.1`, SHA-256 `05355b9639d0295c51b3ccc7f077311cdb39b90a6cbc4583aae2239e389ac0fe`.
- Adjacent `assura-full`: SHA-256 `4dfdb3dea035ac0fa0f8025864a356074bede59bd0c62dab09a66da93f26650c`.
- README/getting-started/onboarding docs read from `/Users/nroth/.codex/worktrees/assura-new-user-review/assura` at `f3540d4a11d36afa43f69c96fd7f1c233599822a`. Source commentary is corroboration; execution evidence comes from the pinned published binary.
- Scratch backing is APFS `/dev/disk1s2`, 135 GiB available at admission. TMPDIR and requested cache root stayed under this owned directory. Commands were sequential, with no builds, installs, services, child agents or browser sessions. Final process probe found no remaining owned processes.

## Reproducible results

Full argv, cwd, start time, elapsed seconds, exit status and stdout/stderr for product probes are in `transcript.json`; its `name` fields are referenced below. `metadata.json` and `backlog-candidates.json` provide compact machine-readable summaries.

| Path | Result | Evidence names |
|---|---|---|
| Unconfigured check | Exit 4, clear missing-config error, but no suggested init command | `unconfigured-check` |
| Minimal init | Exit 0, config only; explicitly says agent setup is incomplete and trust/approval still needed | `minimal-init` |
| Minimal generated config | Optional Cargo/package manifests, `.rs` naming and open root; Python gets no policy | `minimal-config` |
| Minimal check | Exit 0; 440 files, 132 directories, no violations | `minimal-check` |
| Repeated init | Exit 2, refuses overwrite and identifies `--force` | `minimal-init-repeated` |
| Auto onboarding | Exit 1; files installed, detected monorepo/generic, structure verification fails | `onboard-first` |
| Onboarding check | Exit 1; 129 findings: 1 medium `exists_count`, 3 low `max_lines`, 13 low `limit_children`, 112 low agent-guidance findings | `onboard-baseline-check` |
| Repeated onboarding | Same failure; no file added, removed or changed, including packet/config | `onboard-second`, `idempotency.json` |
| Baseline review | Exit 0 (advisory contract), status fail, summary blocking 1/advisory 0 despite 128 low findings | `onboard-baseline-review` |
| Doctor | Clearly reports inactive content models, collections and reference policies | `onboard-baseline-doctor` |
| Existing content preservation | No tracked original guidance/hook changes; existing AGENTS preserved verbatim | `original-guidance-and-hooks-preserved`, `final-original-guidance-preserved` |
| Explicit Codex integration | Install/activate/doctor/deactivate exit 0; reports structural verification scope and host approval requirement; no live host event claimed | `codex-install`, `codex-activate`, `codex-doctor`, `codex-deactivate` |
| Existing Codex config/hooks | Config model entry preserved; seeded unmanaged SessionStart hook preserved during activation and semantically identical after deactivation | `activated-codex-config`, `deactivated-codex-config`, `activate-with-existing-hooks`, `deactivate-preserving-existing-hooks` |

## Negative probes and policy boundaries

The same probes were added to both isolated copies:

| Probe | Plain init | Agent baseline | Explicit owner policy |
|---|---|---|---|
| `UnexpectedRoot.py` at root | No finding | No finding | Python naming finding |
| `tools/agent-workflow/src/agent_workflow/BadName.py`, 501 lines | No finding | No finding; Explain has no matched source file pattern | Python naming + size findings |
| `docs/review-probe.md` links to missing `./missing-review-probe.md` | No finding | No finding; Explain explicitly says Markdown policy not configured | Missing target found at line 3 |
| `docs/large-review-probe.md`, 501 lines | No finding | One low nonblocking `max_lines` finding | No MD size rule was added in the custom-policy proof |

Evidence: `mutated-minimal-check`, `mutated-onboard-check`, `explain-large-python`, `explain-link-probe`, `owner-policy-check`. The explicit owner policy is preserved in `minimal/.assura/config.yml` and its exact write is recorded as `write-owner-python-link-policy`.

The open root, absent naming rules and inactive link checks are documented baseline boundaries, not missed checks under an active policy. The configured engine handled those probes correctly once enabled. A root closure was not hand-authored because this trial was not authorized to decide the existing project's complete intended shape.

The Python omission is a starter consistency issue: generated broad health policy lists `.md/.js/.jsx/.ts/.tsx` but omits `.py` (and `.rs`); default `init` instead assumes Rust optional paths/naming. Generated instructions correctly require evidence-first specialization, but a Python user has to manually supply language coverage even for simple source health.

## Candidate backlog cards

`backlog-candidates.json` has acceptance criteria, evidence names and dependency fields ready for coordinator deduplication. No Beads tickets were created by this worker.

### PY01 — P1: include configured advisory violations in Review findings/history

The broad baseline has 128 nonblocking diagnostics, while Review reports advisory=0. Adding a new 501-line Markdown file increases Check findings to 130 without adding a Review advisory. Increasing it to 700 lines produces Check text `has 700 lines, exceeding limit 500` while Review history remains `new=0,worsened=0,unchanged=8,resolved=0`. The issue is strongest for users told to use Review as their working radar.

Evidence names: `onboard-baseline-review`, `mutated-onboard-check`, `mutated-review`, `worsened-check`, `worsened-review`. Current source `src/cli/project_review/report.rs:58-59` builds findings from doctor blocking violations and doctor gaps; low configured check findings are not included. Some heatmap totals retain counts, so this is a compact findings/history omission, not complete data loss.

Closure: bounded advisory findings with omission totals; meaningful new/worsened/resolved history for a configured low rule; preserve advisory exit behavior and Check severity contract. No dependency on onboarding fixes.

### PY02 — P1: preserve existing agent policy intent during baseline generation

Onboarding adds `.agents: ./: exists:0-1 | $closed`, then immediately rejects the project's tracked `.agents/project-policy.json` as one forbidden direct file. It also reports 112 low agent-guidance issues against existing skills/AGENTS using Assura-specific frontmatter/heading/routing expectations. The setup preserves bytes, but imposes policy that conflicts with established agent infrastructure before specialization.

Closure: preserve existing project-policy intent and expose unresolved constraints explicitly, avoid automatically applying Assura-specific skill conventions to unmanaged skills, keep deliberate user closures effective, retain rerun idempotency. Do not broadly whitelist every observed future path. No dependency on PY01.

### PY03 — P2: provide consistent Python starter health/specialization coverage

501-line Python probe is silent under baseline while same-size Markdown is advisory; plain init creates Rust-only source naming in a Python project. A supported Python entry path should either provide consistent source-size coverage or explicitly identify uncovered types and direct users to a matching supported recipe. Flat package layout must not be assumed for this monorepo.

Closure: clear Python source/test/generated boundaries, explicit negative source-size proof, evidence-based naming. This is independent adoption improvement; absent unspecialized naming/root checks remain documented inactive behavior.

### PY04 — P2: keep numeric child-count diagnosis alongside custom guidance

The generated folder-health rule replaces the useful numeric diagnosis with only `See docs/process/agent-workflow.md#structure.`. Its root violation uses `path: ""`. Check does not show actual count or configured limit 10 in the finding, creating another lookup before the user can assess the issue.

Closure: actual count, threshold and explicit root label in JSON/agent/text output, with custom project guidance retained. Root and nested examples must prove the message. Independent diagnostic repair.

## Limits and teardown

No LS-Lint config exists in the selected tracked project; migration was skipped rather than using an invented “existing-project” migration case. This worker did not execute the installer, run host trust approval or real event delivery, exercise watch/daemon, build Assura, install project dependencies, reproduce hosted CI or validate other platforms. The companion and binary were supplied by the coordinator and identity-verified here. No claim of CI/release readiness follows from this field trial.

Original project HEAD and clean status were rechecked at the end. Managed Codex activation was deactivated in scratch; unmanaged entries remain. Scratch evidence and disposable snapshots are retained for review. No product source or original repository files changed.
