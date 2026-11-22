# Assura adoption release plan

## Outcome

A developer can adopt Assura on an existing repository, make a change without
growing inherited policy debt, and get the information needed to repair a
failure. A user who repeats this across repositories can eventually apply the
same budgets while each project keeps ownership of its layout.

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
for CLI artifacts. [The release train](../../../docs/release-train.md) governs
candidate validation and publication.

| Target | Priority | Outcome and work | Exit evidence |
| --- | --- | --- | --- |
| v0.4.2 | P1 | Prepare the landed adoption and reliability fixes for R06 release proof (`ae-we5`). Target actionable size guidance through existing commands (`ae-whr.7`), with an independent packaging path for the fixes if the guidance needs more work. | Verify clean install and upgrade from the same candidate on supported platforms. For the guidance, observe install or upgrade, nested invocation, a size failure, repair and successful recheck on an established repo, including one real supported hook event. |
| v0.5.0 | P1 | Gate a change set against existing file budgets (`ae-whr.8`). Start with explicit working, staged and committed comparisons and one verified existing pre-commit route. Show current size, cap, base size and additions with exact scoped follow-up commands. | Replay the established-repo workflow. New and previously compliant files fit. Oversized legacy files can stay flat or shrink. Real Git events prove the selected index, including a path-limited commit. |
| v0.6.0 | P1/P2 | Resolve persistent user, group and per-repository policy assignments, including a private project config (`ae-whr.9`, P1). Add narrowly configured added-line and addition-budget rules only where the replay finds gaps (`ae-whr.10`, P2). | A user can set up, inspect and remove private policy with no recurring flags. Prove two repositories, an out-of-tree linked worktree, isolation of owner rules, warm replacement and explicit CI reproduction. |
| v0.7.0 | P2, conditional | Reduce repeated setup through user-installed runtime-resolving hooks (`ae-whr.11`) if portable-policy replay still shows setup friction. Start with one native agent and one Git runner. Add pre-push enforcement only after its ref contract is proved. | Real edit and Git events select the right inputs, leave unrelated projects quiet, preserve other hooks and survive update and removal. Extend coverage host by host after native proof. |

## Architecture decisions for the slices

- **Guide the repair from the finding.** Start with effective limit, actual
  count, amount to cut, an exact path-scoped explain or recheck, and brief
  owner escalation when a legitimate edit cannot fit. Reuse corrective
  context in the patch. In the change-budget release, add structured actions
  to finding-bearing formats and render the same policy result in hooks.
  Use the same comparison context for the rerun and for any optional review
  link. Preserve the summary contract of status, existing schemas in the
  patch, and hook output caps. Prove that emitted commands run from the
  stated directory, including a path with spaces, using the packaged CLI.
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

The proposed v0.6.0 workflow is to configure policy once and then run ordinary
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
| A project exclusion hides an owner finding, a warm daemon keeps stale policy, or CI succeeds after using a different input. | v0.6.0 requires independent owner eligibility and verdicts, resolver freshness across supported consumers, and explicit pinned CI inputs. |
| A fixture passes while adopters still get stuck, or optional guidance holds the reliability patch. | Observe the packaged repair workflow, keep the existing release-proof dependencies visible, and give the landed fixes their own packaging path. |
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
