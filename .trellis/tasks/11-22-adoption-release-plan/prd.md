# Assura adoption release plan

## Outcome

A developer can adopt Assura on an existing repository, make a change without
growing inherited policy debt, and get the information needed to repair a
failure. A user who repeats this across repositories can eventually apply the
same budgets while each project keeps ownership of its layout.

## Delivery goal and definition of done

Ship v0.4.2 as a reliable upgrade for existing users, then ship v0.5.0
Workspace Flexibility so a developer can configure Assura once and check,
repair and commit their changes across repositories and linked worktrees.
The goal is complete when both versions are live and verified:

- **v0.4.2 upgrade.** Audit the full candidate against v0.4.1, including
  configuration, CLI, output and behavior. Replay the affected install,
  onboarding and targeted-check journeys using the same packaged candidate.
- **v0.5.0 workflow.** Pass the integrated acceptance below with one packaged
  candidate. Verify a growing oversized file blocks the real commit, the
  repair commands work from the reported directory, and clean CI reproduces
  the verdict for the same bytes, policy and baseline.
- **Merged source.** Each PR has independent review, resolved findings, and
  required local and hosted checks tied to the final candidate on the current
  base. Verify the merge and repeat affected acceptance on the merged source.
- **Live release.** For each version, pass release readiness and smoke, verify
  the merged SHA's previews, then publish an authorized annotated tag through
  the release workflow. Verify its receipt and all five checksummed archives
  across Linux GNU and musl, Windows, and Intel and Arm macOS. Check versions
  for both bundled binaries, exercise the published installer for the explicit
  version and latest, and smoke the advertised workflow from downloaded assets.
  Confirm live documentation and install links show the shipped capabilities.
- **Handoff.** Record PRs, merged SHAs, tags, workflow and artifact receipts,
  and actual install and workflow results on the existing delivery cards.
  Close owned worktree and branch records after verifying reachability.

The coordinator owns dependency order, integration and publication. Give
each implementation worker one worktree, explicit file ownership and a
bounded test brief. Run the patch compatibility audit alongside independent
guidance work once the delivery gate admits it. Sequence snapshot enforcement
before its use in policy resolution and final hook and guidance integration.
Use the least costly capable worker for scoped edits, fixtures and audits,
and escalate for Git snapshot or policy-authority decisions when needed.
Use a fresh GPT-6 Luna reviewer for independent review and scoped rereview.
Keep long Rust builds off the Mac by using hosted CI or a verified vps-dev
workspace with disk-backed scratch and available resources.

## Baseline

- The 2026-11-22 fast-forward-only pull found `origin/master` at `9ae7d816`
  and the checkout already matched it. The current branch had no same-named
  remote or tracking branch. GitHub's latest packaged release is
  [v0.4.1](https://github.com/rothnic/assura/releases/tag/v0.4.1).
- An installed v0.4.1 check from two directories below the policy found
  the ancestor config and correctly reported a 3-line file against a 2-line
  cap. Its correction offered raising the limit. Advice and agent formats
  added generic prose. Text, JSON and YAML exposed the correction. Status
  stayed summary-only. The filename contained spaces.
- Current source has scoped check, file-specific explain, review with a Git
  base, a prepared path check and managed project-local hooks. Review's base
  feeds advisory history and heat, while the quality planner selects native
  commands from changed paths. Change-budget enforcement belongs in check
  and should reuse those Git and policy inputs. Existing ancestor discovery
  already respects project boundaries.
- Post-v0.4.1 source includes adoption fixes for quickstart, install
  shadowing, onboarding policy preservation, missing-config discovery,
  count diagnostics, targeted relationship coverage, starter coverage and
  Markdown explain. Package proof and native issue closure remain separate.

## Release targets

Beads owns priority, dependencies and live status. These are planning targets
for CLI artifacts. A release earns its number by shipping an independently
useful workflow. Hold a target at preview if its contract is still changing.
Use patches to learn and fix regressions within a released contract. The
[release train](../../../docs/release-train.md) governs candidate validation
and publication.

| Target | Priority | Outcome and work | Exit evidence |
| --- | --- | --- | --- |
| v0.4.2 | P1 | Package compatible fixes to the existing install, onboarding and validation experience using R06 (`ae-we5`). Eligible changes include installer shadow detection, policy-preserving setup, missing-config diagnostics, and targeted-check fixes that pass the candidate compatibility audit. | Verify clean install and upgrade from the same candidate on supported platforms. Replay the affected existing behaviors and document the public-surface delta from v0.4.1. |
| v0.5.0 | P1 | Deliver workspace flexibility as one increment. Select policy once across repos and worktrees (`ae-whr.9`), gate the intended working, staged or committed change without making inherited violations block that change (`ae-whr.8`), and explain and recheck the repair in that same scope (`ae-whr.7`). | The same packaged candidate proves the combined workflow from setup through repair and CI. Test a tracked-policy repo and a private-policy repo, a linked checkout outside the group, and a real path-limited commit using the existing pre-commit route. |
| v0.5.x | As evidenced | Ship compatible fixes and improvements to the released workspace workflow. Examples include clearer policy-origin diagnostics and faster repeated checks. | Pin the prior release and candidate. Prove the improvement and preserve config meaning, Git scope, verdicts and automation contracts for unaffected inputs. |
| Unassigned | P2, conditional | Add narrowly configured added-line or addition-budget rules (`ae-whr.10`) if replay exposes a material gap in existing tools. | The real workflow shows which rule is needed and when it should run. Separate tests prove additions versus net growth, pre-existing debt and exemptions. |
| Unassigned | P2, conditional | Reduce repeated setup through user-installed runtime-resolving hooks (`ae-whr.11`) if portable-policy replay still shows setup friction. Start with one native agent and one Git runner. Add pre-push enforcement only after its ref contract is proved. | Real edit and Git events select the right inputs, leave unrelated projects quiet, preserve other hooks and survive update and removal. Extend coverage host by host after native proof. |

Audit the complete candidate against the last public release before calling it
v0.4.2. Package eligible reliability fixes independently of the workspace
increment. Reclassify the candidate if its actual public delta requires a
minor version.

Build and validate the workspace pieces in SHA-pinned CI previews. First
prove change scopes with an existing project config, then exercise persistent
policy selection, then replay the guidance and actual scoped recheck on the
combined artifact. Guidance work on existing commands can proceed in parallel.
Promote v0.5.0 when the integrated outcome passes. Use the last version
component for compatible changes to the supported workflow. New public
contracts go through minor-release selection.

## v0.5.0 Workspace Flexibility

A developer can bring the same Assura policy to existing repos, keep any
project policy in place, and check and repair the change they intend to commit.
They configure the workspace once and use the same short commands from a
regular checkout or a linked worktree.

The packaged release must demonstrate one connected workflow:

1. Assign shared policy to two repositories. Keep one repository's checked-in
   layout config, and select an external config for the other without adding
   tracked files. Verify that an out-of-tree linked worktree resolves the same
   owner policy and uses its own checkout and index.
2. Make a legitimate change in an established repo with existing size debt.
   Prove that the existing pre-commit route checks the intended staged bytes,
   that an untouched violation stays outside the verdict, and that making a
   touched oversized file grow triggers a finding.
3. Follow the finding's inspect and recheck commands, fix the problem, and
   pass the same scoped check. In a clean CI checkout, reproduce the verdict
   for the same staged tree after committing, using the explicitly selected
   policy and matching baseline.

The existing per-repo hook route is enough for this workflow. One-time global
hook installation, pre-push enforcement and extra diff rules remain
conditional follow-ups with their own evidence.

## What Assura 1.0 Would Promise

Assura 1.0 should make a compatibility commitment for a bounded
repository-policy workflow. A maintainer can install or upgrade Assura, set
explicit policy safely, understand a failure and repair it, and rely on the
same declared policy and Git snapshot producing the same verdict locally and
in CI. The stable contract would specify:

- The supported config notation, source selection and precedence, rule IDs,
  enforcement semantics, Git scope and baseline modes, and path explanations.
  Include private and shared policy selection if the rollout validates it as
  part of the core workflow.
- The documented CLI arguments, exit behavior and machine-readable schemas
  that automation can depend on, including bounded agent feedback. State the
  supported platforms, harnesses and installation, update and removal paths.
- An upgrade example from the last supported pre-1.0 release and a public
  migration and deprecation policy. Use the existing support and compatibility
  ledgers to name the surfaces that receive the v1 stability commitment.
  Global automation and broader integrations qualify on their own evidence.
  Existing supported surfaces and four-host obligations stay in force unless
  the maintainer separately approves a documented change.

The existing [F05 readiness decision](../09-04-maturity-portfolio-strategy/research/adoption-packet.md#f05)
(`ae-220`) owns the go, narrow, or continue-beta decision. It requires R06
same-candidate install proof, A07 independent correctness, F02 retained use,
and the documented 30-day, 50-session, three-repository soak, including four
hosts if the existing matrix is retained. Preserve Q02 and Q07 prerequisites.
Wrong verdicts, unsafe setup, failed upgrades and silent enforcement gaps
hold a stability claim. Completing the adoption targets supplies evidence
to F05. Continue the 0.x train while the contract or proof is still moving.

## Architecture decisions for the slices

- **Guide the repair from the finding.** Start with effective limit, actual
  count, amount to cut, an exact path-scoped explain or recheck, and brief
  owner escalation when a legitimate edit cannot fit. Reuse existing
  corrective context. Add structured actions for the change-budget workflow
  and render the same policy result in hooks.
  Use the same comparison context for the rerun and for any optional review
  link. Preserve the summary contract of status, document any structured
  schema delta, and keep existing hook output caps. Use the packaged CLI to
  prove that commands run from the stated directory for a path with spaces.
- **Compare the content being checked.** An explicit base resolves to a
  commit and comparison point. Local repair needs working edits and new
  non-ignored paths. Commit checks need the index, and CI and push checks
  need the relevant committed trees. Share the resolver with review,
  report the resolved object IDs. A branch comparison is shared work and
  carries no author attribution. Keep full checks for repository-wide
  layout and relationship contracts. Use NUL-delimited Git path records
  and preserve path bytes through selection and blob lookup. Diagnose an
  unmerged index or an unsupported path explicitly. Git documents the distinct
  [tree, index and working-tree comparisons](https://git-scm.com/docs/git-diff).
- **Compose policy by authority.** Keep project layout with the existing
  loader. Add a deliberate source of owner budget constraints that are
  independently enforced, so project exceptions, exclusions, severity,
  explicit project config, or local guidance cannot silently cancel them.
  Compute owner eligibility from the selected snapshot paths before
  project filtering, apply owner exemptions to owner rules, and keep any
  owner blocking finding in the final verdict. Expose the source, rule
  and active exception in explain. Use built-in action IDs and bounded
  plain text that renders control characters safely. On a shared OS
  account this is a policy authority convention. Independent CI gate
  inputs are controlled and pinned by the CI operator.
- **Keep hooks thin.** Reuse the existing install, verify, update and remove
  lifecycle and one policy engine. Resolve context at event time. With no
  policy, automatic hooks take a fast quiet path. Preserve existing hook
  execution and the difference between advisory feedback and a selected
  blocking gate. Use A06 to establish cold and warm no-policy and edit
  latency and output budgets before promotion. Measure those independently
  of branch validation.
- **Defer speculative scope.** A second scripting engine, arbitrary
  prompt-provided commands, generic repository exemptions for owner policy,
  and project relaxation of owner budgets have no assigned release.
  Consider ranked split candidates only if the replay shows that basic
  guidance and existing relationship facts still leave users guessing.
  Start with one demonstrated language and show actual relationship
  evidence where available. A filename match cannot establish test coverage.

## Policy discovery and Git adoption

The proposed v0.5.0 workflow is to configure policy once and then run ordinary
Assura commands. Keep one effective-policy resolver for the CLI, explain and
prepared checks so a later hook can use the same behavior.

| Input | Proposed behavior |
| --- | --- |
| User policy | An optional user-owned file in the platform config directory can supply baseline budgets and behavior. An explicit owner baseline selection replaces that default location. A missing default file is normal. An explicit invalid selection is an error. |
| Repository group | A user-owned Git conditional include can assign a shared Assura policy file to a directory of repositories once. Use `includeIf "gitdir:…/"` to anchor membership to repository metadata so linked checkouts can live elsewhere. This supports a policy file in the parent directory when explicitly selected. |
| Private repository settings | Shared repository-local Git config can select a repository policy in place of the group assignment and optionally select an external project config. Both are shared by the clone's linked worktrees. Explain shows any deliberate project-layer replacement. The owner baseline stays independently active. |
| Project discovery | Keep the nearest `.assura/config.yml` within the target checkout unless the user explicitly selected another project file. A configured subdirectory can scope its own layout. Stop ordinary discovery at the Git boundary. With owner policy only, use the Git top level, or the invocation directory outside Git. |

Discover Git from the checked target. Use Git's common directory for local
repository identity and its reported checkout root for file resolution.
Let Git resolve each selector as a scalar by normal precedence, including
conditional includes and existing per-worktree config where enabled. Use
absolute or home-relative policy paths in the initial contract. Report the
winning origin and diagnose an unreadable or invalid selection.

Read in-tree policy from the selected snapshot for staged or commit-tree
checks, including a selected config's addition or deletion. Keep local owner
policy separate. For reproducible CI, explicitly select and pin the required
owner artifact and project policy selection or snapshot. A missing or
mismatched required input fails the run. Report the resolved inputs and
prove the same verdict in a clean checkout with no personal Git settings.

Rediscover policy selections as part of freshness checks. Cache identity
includes selector origin and value, source identity and content, and
absence where a source is optional. Scope mutable results to the worktree
and comparison snapshot. Resolve Git scope once per check context with no
sibling-repository enumeration or remote requests. Prove warm selector
switch, file removal and recreation, and same bytes at a different source in
check and explain and in prepared consumers that claim support. Prove
assignment after a linked checkout move, at a nested repo boundary, and in
a separate clone. Use the same negative control to demonstrate that a
project exclusion, lower severity, or alternate config cannot hide a blocking
owner finding. A selected owner exemption should change that verdict.
[Git worktree configuration](https://git-scm.com/docs/git-worktree#_configuration_file)
and [conditional includes](https://git-scm.com/docs/git-config#_conditional_includes)
describe these mechanisms.

| Workflow | What Assura should check |
| --- | --- |
| Greenfield or repository-wide audit | Full project policy. A new repository can enforce layout from day one. |
| Established repository, local edit | Working-tree changes and relevant non-ignored new files for quick feedback. Show existing debt separately as advisory context when requested. |
| Established repository, commit | Compare Git's active commit index against `HEAD` or the repository-format empty tree for the first commit. In the hook, honor Git's index environment. Read policy and file blobs from the same index. Whole-project layout failures belong to the full check. |
| Branch review and CI | Choose either a unique merge base of the requested target and head or a direct comparison of two explicit trees. Report the resolved objects. Missing, unrelated, or ambiguous history produces a diagnostic and an explicit way to choose the range. |
| Pre-push, after adapter proof | Use each pushed local and remote object from Git's hook input. Specify the baseline for a new remote ref, skip deletions, and handle multiple refs, non-HEAD refs, force pushes and unavailable objects explicitly. |

Git supplies snapshots rather than authorship boundaries. Stage only the work
for this commit or use a dedicated linked worktree to isolate mixed changes.
Keep the repository identity shared while each worktree has its own index,
working tree and comparison state. Validate both directions of staged and
unstaged content divergence, including config, a path-limited commit, and
independent linked indexes. New and previously compliant files must fit.
An oversized file can stay flat or shrink. Use a detected one-to-one rename
with the same applicable budget for inherited allowance. Copies, ambiguous
renames and moves into a different budget are treated as new for that budget.
Prove untouched legacy debt, an unchanged oversized rename, and a rename
that grows by one line using deterministic Git rename settings.
[Git diff snapshots](https://git-scm.com/docs/git-diff)
and [pre-push inputs](https://git-scm.com/docs/githooks#_pre_push) define the inputs.

## Outstanding work from the existing queue

| Existing lane | Release disposition |
| --- | --- |
| R06, NU11 and SP03 (`ae-we5`, `ae-4s5`, `ae-kps`) | R06 owns v0.4.2 release proof and currently depends on NU11. Before assembling the release candidate, refresh the hold with the release and benchmark owners, recording the current required gate, next discriminating proof and owner. Candidate install and repair proof can proceed when available. Existing release dependencies and authority still apply. SP03's comparable native baseline has separate host and protocol requirements. |
| Windows trajectory and delivery closure (`ae-hbx`, `ae-fg4`, `ae-mvu`) | Landed observability, delivery and targeted-check work belongs in the v0.4.2 candidate. The historical Windows cause and related acceptance remain open. Use the current required Windows suite and captured diagnostic when it reproduces. |
| NU03 native quality scopes (`ae-7cd`) | Separate from the file-budget changes. Qualify for the next compatible train after the public Rust API decision and accepted proof. Do not promise its pending patch in v0.4.2. |
| NU08/09, NU12, NU17/18 and PG01 | Reconcile landed support and workflow evidence for v0.4.2 as applicable. Preserve outstanding ownership and protected cleanup holds and the separate rich-consumer performance-policy decision (`ae-ago`). Cleanup and policy decisions get no CLI feature version. |
| A07 and the external adoption sequence | Preserve blinded evaluation (`ae-lxd`), interviews, pilot/retention, launch, publication and the v1 decision. For the adoption replay, bind the packaged candidate SHA and record whether the user or agent installed, understood and repaired one real violation, time and workarounds, and whether repeated feedback helped. A faithful fixture can prove mechanics when the originating project is unavailable. A07 and the pilot keep their own acceptance. |

## Acceptance

- Current source and the packaged baseline are identified independently.
- Beads has one version-labeled entry per proposed outcome with real
  prerequisite edges, owner-independent acceptance criteria and readback.
- Release order gives the existing adoption fixes an independent path to
  packaging, and uses the original workflow to validate later scope.
- Feature targets remain proposals pending evidence and release approval.

## Premortem decisions

Three independent GPT-6.1 Terra reviewers read `eaf7988b` from Git,
policy/cache, and adoption/release perspectives. Their findings changed these
release conditions.

| Failure if shipped as originally planned | Decision |
| --- | --- |
| The commit check sees ordinary staged or working files instead of the proposed commit. Rename inference or a silently chosen merge base changes the verdict. | v0.5.0 requires the active index, deterministic rename eligibility and explicit committed comparisons. |
| An existing pre-push hook looks successful while it checks the checkout. | Keep native push enforcement conditional on a later, separately verified ref-aware adapter. |
| A project exclusion hides an owner finding, a warm daemon keeps stale policy, or CI succeeds after using a different input. | The workspace increment requires independent owner eligibility and verdicts, resolver freshness across supported consumers, and explicit pinned CI inputs. |
| A fixture passes while adopters still get stuck, or workspace work holds the reliability patch. | Observe the packaged v0.5.0 repair workflow, keep the existing release-proof dependencies visible, and give the landed fixes their own packaging path. |
| Global hooks disrupt existing runners or add repeated latency. | Require coexistence and A06 measurements before advancing the conditional rollout. Keep added-line rules and wider host coverage tied to observed demand. |

The coordinator reproduced two Git cases in a disposable SHA-256 repository.
A path-limited commit exported a temporary `GIT_INDEX_FILE`; its hook saw
only the requested file while the original index retained a different staged
file. A history with crossed merges produced two merge bases under
`git merge-base --all`. These probes establish Git behavior. Assura's future
snapshot handling, performance and adoption outcomes still require candidate
proof.

## Planning evidence

The local Beads head matched a fresh clone of the remote before intake.
Beads 1.3.4 could not list this store because it queried a missing `leases`
table. The checksum-verified 1.2.2 client read and updated the existing store
through the known Dolt 2.5.0 server. Five entries and their dependency edges
were read back with no cycles. A fresh native backup restore contained all
five entries at local Dolt head `7dop11d98csoanr004eaa67jllagulp1`.
The existing uncommitted local `issue_prefix` setting was preserved.

An isolated Git 2.47.1 fixture confirmed that a `gitdir` group assignment
applied in the repository and in a linked checkout outside the group. A
repository-local selection overrode both and survived a worktree move.
The moved checkout kept independent staged bytes, and an unrelated repository
had no assignment. This verifies the Git mechanism. Assura policy resolution
remains planned. Beads scope and acceptance were refreshed in `ae-whr.8` and
`ae-whr.9`, read back and backed up at Dolt head
`buts2o2pprpf0r087tjkve6jkb9jphin`.

The premortem refined `ae-whr.7`, `ae-whr.8`, `ae-whr.9` and `ae-whr.11`.
Their targets and prerequisite edges were verified after readback. The
local store was backed up at Dolt head `7sdsn0svdt25h3d9j15c4cgle0rciaiv`.

The version review left `ae-whr.10` and `ae-whr.11` unassigned and recorded
the existing F05 decision in `ae-220`. A fresh restore verified all three
updates at local Dolt head `6g89an9vv706s5ss6a3pk91pd5sh4gb7`.

The workspace increment review grouped `ae-whr.7`, `ae-whr.8` and
`ae-whr.9` under v0.5.0. The existing prerequisite edges and conditional
follow-ups were verified at local Dolt head `lsfih1kv3fkblal7ot6qfm89q9kh7n2r`.
