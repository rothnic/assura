# Execution contract

Only explicit user invocation or a later implementation request activates
CF01–CF04 in [backlog](backlog.md). The [feedback contract](feedback-contract.md)
is the acceptance source. This contract does not depend on a task framework.

## Start

In an Assura Codex conversation:

```text
$assura-feedback-execution
```

Or from a terminal in the checkout containing this package:

```sh
codex '$assura-feedback-execution'
```

Single quotes preserve the literal skill name. No model, permission, hook-trust
or automation flags are added. Positional prompt syntax was checked against
the installed CLI; a real execution session is deliberately left to the user.

1. Inspect cwd, remote, HEAD, branch, status and existing candidate/PR ownership.
   Refresh `origin/master`; this repository uses `master`.
2. If the planning package is not integrated, preserve its branch. Bring its
   owned planning commit into a clean candidate on the refreshed base, or reuse
   the current candidate when suitable. Do not discard the only copy or invent
   that it was merged. Integrate the package with CF01, not a pointer-only PR.
3. Read this contract, the backlog index and selected card. Find card boundaries
   with `rg -n '^## CF'` rather than loading all card detail on every turn.
   Keep any current reviewer/test handle. Read only the card's named specs.
4. If the host requires Trellis, attach the existing planning record:

   ```sh
   python3 .trellis/scripts/workflow_gate.py --platform codex --task .trellis/tasks/09-12-compact-feedback-plan
   python3 .trellis/scripts/task.py start .trellis/tasks/09-12-compact-feedback-plan
   ```

   Resolve actual `Ready: no` prerequisites. Do not create a task per card,
   load the entire maturity train or repeat settled requirements interviews.
5. Capture the approved product initiative in Hindsight immediately before
   implementation as required by its tool contract. Planning did not start it.

## Delivery

- One owner/candidate per active slice, explicit outcome and acceptance. Reuse
  existing A06/R01/R02/R03 and compact-review code; their state is not reset by
  this plan. Completed adjacent work should reduce the remaining scope.
- Apply scoped instruction corrections with CF01. Do not begin a separate
  framework migration or status-cleanup campaign.
- A passing command, document and PR are intermediate evidence. Pursue
  authorized integration; hold only the exact action missing proof/authority.
  When held, preserve the candidate and prepare useful independent work rather
  than accumulate dependent unmerged branches.
- Use focused negative tests, minimal changes and affected positive checks.
  Batch related fixes into a reviewable outcome; there is no one-line-per-PR
  requirement. Retain thresholds and unfavorable evidence.
- Bind evidence to actual code/config/toolchain/fixture/harness and consumed
  instruction inputs. Keep immutable as-of records. A record cannot be required
  to contain its own resulting commit hash. Derive present PR/integration state
  when needed; never commit merely to chase a new SHA.
- After two repetitions of the same failed method on unchanged inputs, change
  the hypothesis or run one discriminating probe. Predeclared samples, flake
  investigations and waits follow bounded protocols. No instruction-rewriting
  or status-artifact requirement is created by an advisory signal.
- Resolve concrete correctness/acceptance review findings; defer unrelated
  suggestions. Later edits receive delta review. Keep required final-candidate
  CI/merge controls. Reuse unchanged valid proof and avoid duplicate nested suites.
- Commit coherent owned work before review/handoff. Use the Assura orchestration
  independent review brief; reviewers report findings without expanding scope.
  Do not dispatch agents just to restate the plan. Reviews and independent
  validation can overlap; sustained builds/benchmarks use the approved dev host.
- Resume live processes after observation timeouts. Poll at sensible intervals;
  an unchanged wait is not a reason to recreate the job or commit another note.

## State and finish

The backlog table is the only new card-status ledger. Update a row on a real
transition with candidate/PR proof or its exact hold. Routine observations stay
in ignored runtime state. Do not duplicate the queue in recovery plans/prompts.

After merge, prove its PR merge SHA reachable from the integration ref, handling
squash correctly. Remove only clean owned worktrees with proved integration or
deliberate preservation. CF01–CF04 acceptance completes this scope; do not add
new cards to keep execution alive. The user can pause/narrow it at any point.

Invocation grants no new merge, deployment, release, publication or protection
authority. Inspect existing authority and actual trigger coupling; keep a precise
held action when needed. For an obsolete command/path, correct that route once
against current source instead of rebuilding the plan or restarting finished cards.
