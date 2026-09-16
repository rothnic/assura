# Delivery lifecycle

Assura delivery is a provenance contract layered on the existing Trellis task
record. A task is not a delivered change merely because it is marked
`completed`, moved to `archive/`, has a branch, or has a green process check.

## Records

New tasks contain `meta.delivery` with `schema_version: 1`, a stable candidate
ID, kind, owner, repository, base ref, and acceptance reference. The candidate
ID is safe for use as a receipt filename and remains stable across rebases and
session changes. Release intents additionally require a validated package
version and unique safe asset names. Historical tasks without this metadata are
`legacy_unclassified` until an owner explicitly classifies them; malformed
typed intents are invalid and remain held.

Mutable observations live outside the task worktree at the Git common
directory's `assura/delivery-v1/` store. Receipts are atomic, generation-aware,
and protected by a kernel-held per-candidate lock. A stale writer fails rather
than overwriting newer evidence. Audits never write receipts, task JSON, refs,
or worktrees.

An active receipt also records its `attached_branch_ref`, which is an observed
worktree attachment rather than a replacement for the stable task intent. When
available, the live checkout branch is authoritative for that observation. A
detached checkout may use the declared branch only when its full `HEAD` OID
exactly matches that branch tip; the task declaration alone cannot prove the
attachment. A new or resumed registration must use the same attached branch.
Evidence recording and terminal closure validate the same binding before
mutation. They may run from
the canonical base checkout after integration, but that checkout cannot create
or repair a missing attachment. A non-terminal receipt with no attachment or
a different non-base branch is held as a binding conflict and requires
explicit recovery; it is never silently rebound by a second linked worktree.
This prevents two branches from advancing one candidate and sharing one
receipt. Pausing preserves the attachment and does not imply delivery.

For a legacy non-terminal receipt that predates `attached_branch_ref`, the
explicit recovery is to rerun `delivery register` from a live checkout of the
intent's declared candidate branch with the same candidate and owner. The
command validates the task and receipt identity, then attaches that branch with
a generation-checked update. Repeating the command is idempotent. A base,
detached, or different-branch checkout is rejected without changing the task
or receipt; callers must not edit the receipt store directly.

When registration explicitly fills a missing task `authority_ref`, the receipt
must carry the same provenance reference. A legacy receipt missing that field
is backfilled in the same compare-and-swap update as any missing branch
attachment; a conflicting existing value is rejected without changing either
record. If the task write fails after the receipt update, registration is
resumable: retrying with the same authority reference completes the task write
without advancing the receipt generation again. Git's moving `HEAD` and `@`
aliases are not valid base or candidate branch refs.

Registration validates the raw base argument as an existing local or remote
branch before normalization or persistence; tags, `HEAD`, missing refs, and
revision expressions are rejected without changing task or receipt state.
Recovery of an existing intent first verifies the current checkout's exact
repository identity. Registrations for one repository-relative task path share
a stable task lock across candidate IDs, so concurrent requests cannot create
an orphan receipt before one task intent wins. Fresh registration writes the
typed task intent before creating its receipt; a failed receipt step therefore
leaves a recoverable intent with no orphan receipt, and retrying the same
registration reconciles the missing receipt.

Starting a new task checks receipt binding before treating a terminal receipt
as complete. A terminal receipt misbound to another task, owner, repository,
authority, or branch remains a `RECEIPT_BINDING_CONFLICT` and blocks start;
terminal status never hides the binding error.

## Scenario: Validate bindings at every receipt consumer

### 1. Scope / Trigger

Any command or projection that reads a receipt must validate its task binding
before classification or closure. This includes `delivery inspect`, `audit`,
`next`, `checkpoint`, parent/child progress, `record`, `close`, `archive`, and
archive-closure recovery. The invariant prevents a copied or partially written
receipt from appearing delivered under another task, owner, authority, or
branch.

### 2. Signatures

The public command surface is unchanged:

```text
task.py delivery inspect <task> --format json|text
task.py delivery audit --format json|text [--refresh]
task.py delivery next [--owner <owner>] [--format json|text]
task.py delivery checkpoint [--owner <owner>]
task.py delivery record <task> --evidence-file <file> --expected-generation <n>
task.py delivery close <task> --outcome delivered|superseded|rejected|cancelled
task.py archive <task>
```

All consumers use the shared receipt-binding validator; projections pass the
task's declared binding rather than treating the coordinator's current branch
as the candidate branch.

### 3. Contracts

- Compare receipt `repository`, `owner`, `task_path`, `attached_branch_ref`, and
  `authority_ref` with the corresponding typed delivery intent and task record.
- Preserve branch-ref identity. Distinct remote names such as
  `refs/remotes/origin/feature` and `refs/remotes/upstream/feature` are not
  interchangeable by short branch name; accept alternate spellings only when
  alias/OID equivalence is proven.
- `audit`, `inspect`, `next`, `checkpoint`, and parent-progress reads never
  repair or rebind receipts. An invalid receipt is held/unknown, never a
  terminal child outcome.
- Only explicit `register` recovery may backfill missing authority/branch
  fields, using the same task/owner and generation-checked receipt update. A
  retry that omits an authority already present in the receipt conflicts; it
  must not claim success while task JSON remains stale.
- Binding rejection is state-preserving: task JSON, receipt bytes/generation,
  refs, worktrees, and archive paths remain unchanged.

### 4. Validation & Error Matrix

| Condition | Required result |
| --- | --- |
| Repository, owner, task path, branch, or authority mismatch | `RECEIPT_BINDING_CONFLICT` before classification/closure |
| Missing receipt on `inspect` | Preserve the existing non-mutating inspect behavior |
| Missing receipt on `archive` | Fail with `RECEIPT_MISSING`; do not move or rewrite task data |
| Binding conflict in `audit` / `next` | Return a held/unresolved candidate carrying the binding issue |
| Binding conflict in `checkpoint` | Do not count the candidate as terminal |
| Misbound child receipt | Do not invoke terminal child classification; progress remains `unknown` |
| Binding conflict during record/close/archive recovery | Fail before mutation; preserve task and receipt state |

Command-specific nonzero exit codes remain those of the existing CLI; structured
projections may return successfully while carrying an explicit held/unknown
issue. The semantic contract is the issue and unchanged state, not a single
global exit code.

### 5. Good / Base / Bad Cases

- **Good:** a valid receipt is visible from an unrelated coordinator branch
  when it matches the task's declared candidate branch; a verified child
  outcome contributes to parent progress.
- **Base:** no receipt exists. Inspection remains observational; archive still
  requires an explicit receipt and reports the missing-receipt condition.
- **Bad:** a receipt has another task path, owner, authority, repository, or
  branch. Every consumer holds it, parent progress does not count it, and
  reads/mutations preserve task and receipt bytes and generation.

### 6. Tests Required

- Use real temporary repositories and real `task.py` command boundaries for
  inspect, record, close, archive, and archive recovery.
- Assert mismatch issue/exit behavior and byte snapshots for task JSON and
  receipt, receipt generation, branch refs, worktree registrations, and archive
  location.
- Exercise audit/next/checkpoint from a different coordinator branch, plus
  `get_all_statuses`/`children_progress` and aggregate-child classification.
- Include valid and missing controls, interrupted registration with same-
  authority recovery, omitted-authority retry conflict, and distinct remote
  branch names without proven alias equivalence.
- Every regression has a nonzero test count and captures RED on the behavior
  being corrected, then GREEN on the corrected implementation.

### 7. Wrong vs Correct

**Wrong:** classify a receipt first, or use its terminal `outcome` to override
an identity mismatch; accept `origin/feature` and `upstream/feature` because
their shortened names match; repair bindings while rendering a read-only view.

**Correct:** validate all identity fields against the task's declared binding
first; preserve remote-ref identity unless alias/OID equivalence is verified;
surface conflicts as held/unknown; perform repair only in explicit, CAS-guarded
registration recovery.

Each recorded evidence section is a typed `assura.delivery-evidence.v1`
object. It names its source, exact repository, full candidate `head_oid`,
bounded `evidence_ref`, and successful result. Review sections additionally
name a distinct reviewer, role, review identity, and finding dispositions;
check sections name a run and job/check identity; post-merge sections name the
full integration base and merge OID; acceptance sections bind the intent's
authority and a distinct approver to the named criterion and a digest or
measurement. GitHub-sourced reviews must match the current PR number/head and
the live provider's exact review ID, author, and approved state; a matching URL
cannot substitute for provider identity. A self-asserted local audit fallback
is not independent review evidence. GitHub-sourced checks must also match the
current PR number/head and the live provider payload. A caller-supplied
boolean or stale URL is not terminal evidence.

Before accepting an integration's `delivered` outcome or archiving one,
`close` and `archive` read the advertised origin heads without fetching or
updating local refs. The advertised OID for the configured base branch must
exactly match the intent's resolved base OID. Local-tracking-only coverage,
an unavailable branch, repository-identity mismatch, or differing full OIDs
remain unknown/held and block delivery. Other explicit outcomes do not imply
delivery and retain their separate decision-evidence contract.
Task/child progress projections likewise do not count a delivered integration
from tracking-only inventory; callers can explicitly request an advertised
remote refresh when current terminal progress is needed.

Audit worktree names are not proof of base identity. A base-named checkout may
be `retained_base` only when its full worktree OID exactly equals the configured
base OID (or an alias has been proven to resolve to that same OID). A mismatch
is an unresolved `stale_base` candidate carrying both full OIDs and a next
action; audit never cleans up its branch or worktree.

## State and outcomes

`task.py finish` means pause: it records `session_state: paused` and then
clears the session pointer. It does not imply delivery or abandonment.

The computed candidate phase is based on current Git/GitHub facts and explicit
evidence. Terminal outcomes are `delivered`, `superseded`, `rejected`, and
`cancelled`; missing or conflicting evidence remains unresolved/held.

An integration candidate requires an exact tip, verified integration (including
squash/rebase PR mapping), resolved review, required checks, post-merge proof,
acceptance proof, complete required coverage, and no owned uncommitted changes.
Release candidates additionally require a matching `assura.release-receipt.v1`
record containing the exact source commit, tag object, workflow run, required
asset hashes, checksum verification, and installed version proof. The receipt
is retained as an Actions evidence artifact; retrying publication verifies
matching assets, uploads only missing assets, and stops on conflicts or missing
remote digests. Publication evidence remains distinct from explicit
owner-authorized acceptance, so a release receipt alone cannot close a release
candidate.
Superseded, rejected, and cancelled outcomes require a decision bound to the
exact tip; superseded additionally requires a replacement and remaining-diff
disposition. These outcomes remain distinct from delivered for parent
progress.

Physical archiving is optional storage maintenance after a verified outcome.
The archive command validates the outcome before changing status, session
state, or filesystem paths. After the move, archive closure reloads the typed
intent from the archived task and compares candidate, repository, branch, and
base identity before changing the receipt. Its auto-commit stages only the
selected move and related selected task paths; unrelated staged or dirty work
remains untouched.

## Commands

```text
python3 .trellis/scripts/task.py delivery register <task> --candidate <id> --owner <owner>
python3 .trellis/scripts/task.py delivery inspect <task> --format json
python3 .trellis/scripts/task.py delivery record <task> --evidence-file <file> --expected-generation <n>
python3 .trellis/scripts/task.py delivery close <task> --outcome delivered|superseded|rejected|cancelled
python3 .trellis/scripts/task.py delivery audit --format json [--refresh] [--strict --owner <owner>]
python3 .trellis/scripts/task.py delivery next [--owner <owner>]
python3 .trellis/scripts/task.py delivery checkpoint [--owner <owner>]
python3 .agents/skills/assura-goal-execution/scripts/audit-delivery.py --format json
```

`audit`, `next`, and `checkpoint` are read-only. `audit --refresh` performs a
read-only advertised-head/tag query; it does not fetch or mutate local refs.
Without it, remote coverage is explicitly `local-tracking-only` and strict
fleet claims cannot pass. These commands report unavailable GitHub/host
coverage as unknown rather than as an empty successful inventory.
The checkpoint is deterministic for unchanged inputs and is bounded to 1,024
UTF-8 bytes. No delivery command merges, publishes, deploys, deletes branches,
or executes shell text from task metadata.
