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
| v0.5.0 | P1 | Let one repository gate a change set against its existing file budgets (`ae-whr.8`). Show current size, cap, base size and additions, with exact scoped follow-up commands. | Replay the original workflow or a faithful fixture. New and previously compliant files fit. Oversized legacy files can stay flat or shrink. An untouched violation stays outside the file-budget comparison. |
| v0.6.0 | P1/P2 | Apply opt-in owner-managed budgets across repositories (`ae-whr.9`, P1). Add narrowly configured added-line and addition-budget rules where the replay finds gaps (`ae-whr.10`, P2). | One policy works across two distinct layouts, including a repository without a project config. The same selected policy reproduces in CI. Explain shows the active sources and caches react to both layers changing. Added-line rules have separate negative controls. |
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
  explanatory text with bounded output. Evaluate default home discovery
  only once explicit selection proves the authority contract. On a shared
  OS account this is a policy authority convention; enforcement against
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
