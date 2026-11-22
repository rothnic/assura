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
| v0.4.2 | P1 | Package the landed adoption and reliability fixes. Give size violations an executable repair path through existing commands (`ae-whr.7`). Reuse R06 (`ae-we5`) for release proof. | Packaged clean install and upgrade, nested invocation, a size failure with runnable inspect and recheck instructions, a successful repair, and one real hook event. Preserve advisory behavior and existing schemas. |
| v0.5.0 | P1 | Let one repository gate a change set against its existing file budgets (`ae-whr.8`). Show current size, cap, base size and additions, with exact scoped follow-up commands. | Replay the reported workflow. New and previously compliant files fit. Oversized legacy files can stay flat or shrink. The commit check uses the staged snapshot and ignores untouched debt. |
| v0.6.0 | P1/P2 | Resolve persistent user, group and per-repository policy assignments, including a private project config (`ae-whr.9`, P1). Add narrowly configured added-line and addition-budget rules where the replay finds gaps (`ae-whr.10`, P2). | The same short command works in two repositories and an out-of-tree linked worktree. Prove precedence, private config selection, cache invalidation and explicit CI reproduction. Added-line rules have separate negative controls. |
| v0.7.0 | P2, conditional | Reduce repeated setup through user-installed runtime-resolving hooks (`ae-whr.11)) if the portable-policy replay still shows setup friction. Start with one native agent and one Git runner. | Real edit and Git events select the right inputs, leave unrelated projects quiet, preserve other hooks and survive update and removal. Extend coverage host by host after native proof. |

## Architecture decisions for the slices

- **Guide the repair from the finding.** Start with effective limit, actual
  count, amount to cut, an exact path-scoped explain or recheck, and brief
  owner escalation when a legitimate edit cannot fit. Reuse corrective
  context in the patch. In the change-budget release, add structured actions
  to finding-bearing formats and render the same policy result in hooks.
  Use the same comparison context for the rerun and for any optional review
  link. Preserve the summary contract of status and existing hook output
  caps. Offer only commands supported by the installed binary.
- **Compare the content being checked.** An explicit base resolves to a
  commit and comparison point. Local repair needs working edits and new
  non-ignored paths. Commit checks need the index, and CI and push checks
  need the relevant committed trees. Share the resolver with review,
  report the resolved ref and handle absent or ambiguous bases with a
  readable correction. A branch comparison is shared work and carries no
  author attribution. Keep full checks for repository-wide layout and
  relationship contracts. Probe rename, root or worktree selection, shallow
  history, binary and unusual filenames, and test staged versus unstaged
  divergence. Git documents the distinct
  [tree, index and working-tree comparisons](https://git-scm.com/docs/git-diff).
- **Compose policy by authority.** Keep project layout with the existing
  loader. Add a deliberate source of owner budget constraints that are
  independently enforced, so project exceptions, exclusions, severity,
  explicit project config, or local guidance cannot silently cancel them.
  Start with owner-side exemptions and opt-in portable policy selection.
  Test precedence for both sources and keep paths relative to the checked
  repo or selected scope. Expose source, rule and active exception in
  explain. A discovered malformed config is a diagnostic. Key cached
  results on all effective inputs and test replacement and removal while
  warm. Use built-in action IDs for commands and allow owner-authored
  explanatory text with bounded output. On a shared OS account this is a
  policy authority convention. Enforcement against
  changes to user-owned inputs belongs in independently configured CI.
- **Keep hooks thin.** Reuse the existing install, verify, update and remove
  lifecycle and one policy engine. Resolve context at event time. With no
  policy, automatic hooks take a fast quiet path. Measure single-path edit
  feedback independently of branch validation. Prove the actual indexed or
  committed snapshot before using a Git event as an enforcement point.
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
| User policy | An optional user-owned file in the platform config directory can supply baseline budgets and behavior. Explicit selection has higher priority than the default location. A missing optional file is normal. An explicit invalid file is an error. |
| Repository group | A user-owned Git conditional include can assign a shared Assura policy file to a directory of repositories once. Use `includeIf "gitdir:…/"` to anchor membership to repository metadata so linked checkouts can live elsewhere. This supports a policy file in the parent directory when explicitly selected. |
| Private repository settings | Shared repository-local Git config can remember a per-repository policy assignment and optionally select an external project config. It travels to linked worktrees without adding tracked files. Show a deliberate project-layer replacement in explain when both files exist. An owner-selected baseline still applies. |
| Project discovery | Keep the nearest `.assura/config.yml` within the target checkout unless the user explicitly selected another project file. A configured subdirectory can scope its own layout. Stop ordinary discovery at the Git boundary. With owner policy only, use the Git top level, or the invocation directory outside Git. |

Pin automatic group membership to the repository context, using Git's common
directory for identity and its reported checkout root for file resolution.
Read the selected snapshot of in-tree project policy for staged or
commit-tree checks. An unstaged config edit must not silently change a
commit verdict. Local owner input is separate from tracked project input.
Resolve selector paths independently of the invocation directory and cache
resolved policy with selector and file identities and contents. Keep mutable
results scoped to the worktree and comparison snapshot. Resolve Git scope once
per check context, with no sibling-repository enumeration or remote requests.
Prove that moving a
linked checkout preserves its assignment and a second clone or unrelated repo
cannot inherit a private assignment accidentally. Existing Git per-worktree
settings can provide a deliberate exception where enabled.
[Git worktree configuration](https://git-scm.com/docs/git-worktree#_configuration_file)
and [conditional includes](https://git-scm.com/docs/git-config#_conditional_includes)
describe these mechanisms.

| Workflow | What Assura should check |
| --- | --- |
| Greenfield or repository-wide audit | Full project policy. A new repository can enforce layout from day one. |
| Established repository, local edit | Working-tree changes and relevant non-ignored new files for quick feedback. Show existing debt separately as advisory context when requested. |
| Established repository, commit | Compare the Git index against `HEAD` or an empty tree for the first commit. Inspect staged content, including partial staging. New or previously compliant files must fit. Existing oversized files may stay flat or shrink. Whole-project layout failures belong to the full check. |
| Branch review and push | Compare committed snapshots. In CI use the intended target's merge base or a pinned comparison range. For pre-push use the refs Git supplies and a defined baseline for a new remote ref. Report missing history and an explicit way to choose the base. |

Git supplies snapshots rather than authorship boundaries. Stage only the work
for this commit or use a dedicated linked worktree to isolate mixed changes.
Keep the repository identity shared while each worktree has its own index,
working tree and comparison state. Validate staged and unstaged changes to the
same file, independent linked indexes, an untouched legacy violation and an
oversized file that grows by one line.
[Git diff snapshots](https://git-scm.com/docs/git-diff)
and [pre-push inputs](https://git-scm.com/docs/githooks#_pre_push) define the inputs.

## Outstanding work from the existing queue

| Existing lane | Release disposition |
| --- | --- |
| R06, NU11 and SP03 (`ae-we5`, `ae-4s5`, `ae-kps`) | R06 owns v0.4.2 release proof and currently depends on NU11. Historical performance attribution is still held. SP03's comparable native baseline is supporting evidence, with its own host and protocol requirements. Refresh readiness without waiving the release edge or claiming the old failure is resolved. |
| Windows trajectory and delivery closure (`ae-hbx`, `ae-fg4`, `ae-mvu`) | Landed observability, delivery and targeted-check work belongs in the v0.4.2 candidate. The historical Windows cause and related acceptance remain open. Use the current required Windows suite and captured diagnostic when it reproduces. |
| NU03 native quality scopes (`ae-7cd`) | Separate from the file-budget changes. Qualify for the next compatible train after the public Rust API decision and accepted proof. Do not promise its pending patch in v0.4.2. |
| NU08/09, NU12, NU17/18 and PG01 | Reconcile landed support and workflow evidence for v0.4.2 as applicable. Preserve outstanding ownership and protected cleanup holds and the separate rich-consumer performance-policy decision (`ae-ago`). Cleanup and policy decisions get no CLI feature version. |
| A07 and the external adoption sequence | Preserve blinded evaluation (`ae-lxd`), interviews, pilot/retention, launch, publication and the v1 decision. Replay the reported project workflow alongside A07. Use release and trial proofs for promotion and keep the external participation and public launch gates. Portfolio and discovery work follows its existing dependencies. |

## Acceptance

- Current source and the packaged baseline are identified independently.
- Beads has one version-labeled entry per proposed outcome with real
  prerequisite edges, owner-independent acceptance criteria and readback.
- Release order gives the existing adoption fixes an independent path to
  packaging, and uses the original workflow to validate later scope.
- Feature targets remain proposals pending evidence and release approval.

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
