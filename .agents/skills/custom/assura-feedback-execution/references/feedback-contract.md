# Compact feedback contract

Status: active execution; CF01 is implemented on the current candidate and
CF02-CF04 remain pending. This replaces the earlier 2 KiB routine-nudge
proposal. Explicit reports remain available on demand.
Product semantics are independent of Trellis and any agent host.

Index: [output](#output), [controls](#controls), [facts](#facts),
[collection](#collection), [boundaries](#boundaries).

## Output

Default to silence. Emit one selected statistics line, normally 100–200 UTF-8
bytes, hard ceiling 256 bytes including the Assura text wrapper. Do not prepend
a paragraph, repeated legend, path inventory, tutorial, log location or full
provenance. Those belong in the explicit query.

Illustrative example, not current repository measurements:

```text
assura: base=origin/master(local); 30m commits=0 src=+0/-0; pending=42m commits=5 lines=+90/-4 dirty=3
```

`base(local)` means the locally known integration ref, without implying a fresh
remote fetch. The window applies to integrated activity; pending fields describe
this worktree's unintegrated changes. Explain labels once in generated guidance.
Users may select fewer metrics. Omit unknown metrics; never substitute zero.
If a selected statistic cannot be stated honestly within the limit, emit nothing
and retain the reason in inspect/status output.

The byte limit is deterministic; tokens vary by tokenizer. Prefer ASCII labels
and numbers. No model call, intent classification, productivity score, mandatory
advice, acknowledgment or automatic repair is involved.

## Controls

Proposed project config under `.assura/config.yml`; these are new CF03 keys,
not currently accepted configuration:

```yaml
agent_feedback:
  mode: threshold                 # off | threshold | periodic
  max_bytes: 256                  # 64..256 for routine automatic text
  min_interval_seconds: 600
  max_messages_per_hour: 4
  max_bytes_per_hour: 1024
  reminder_seconds: 1800          # 0 disables reminders
  max_reminders_per_episode: 1
  periodic_seconds: 900
  collection:
    debounce_ms: 1000
    min_refresh_seconds: 30
    stale_after_seconds: 120
    timeout_ms: 2000
    max_commits: 500
  trajectory:
    integration_ref: origin/master
    window: {minutes: 30}         # alternative: {commits: 20}; exactly one
    metrics: [integrated_commits, source_lines, pending_age,
              pending_commits, pending_lines, dirty_files]
    source_paths: [src/**, xtask/src/**, website/src/**]
    test_paths: [tests/**]
    coordination_paths: [.trellis/**, .agents/**, .codex/**, docs/goals/**]
    generated_paths: [target/**, dist/**]
    signals:
      unintegrated:
        enabled: true
        pending_minutes: 30
        step_minutes: 30
        clear_after_clean_seconds: 60
      coordination:
        enabled: false
        commits: 20
        min_only_coordination_commits: 19
        clear_below_only_coordination_commits: 16
      patch_size:
        enabled: false
        changed_lines: 1000
        step_lines: 500
        clear_below_changed_lines: 800
```

Defaults apply when this opt-in section is enabled. Do not silently enable
trajectory monitoring for existing installations. CF04 enables it explicitly
in Assura for the trial. Existing explicit validation keeps its semantics.

Selection is deterministic:

1. `off` disables new trajectory collection/injection. Explicit `inspect`
   requests remain available outside automatic cadence with bounded detailed
   JSON. Silence is not a claim of validation success.
2. A pending episode starts when changes outside the integration ref are first
   observed: branch/worktree differences or untracked nonexcluded files. Entry
   requires observed pending age >=30 minutes AND work still pending. A clean
   snapshot sustained for 60 seconds clears it. Inactivity alone never starts
   an episode. Commit author age is not task duration.
3. Coordination entry requires the complete configured commit window and
   >=19/20 commits touching only configured coordination paths. Unknown paths
   prevent that classification. Patch size uses pending tracked additions plus
   deletions; binaries/untracked content do not invent line counts. Both are
   opt-in, with the distinct clear thresholds shown above.
4. Threshold entry is eligible once. Further sends require a material step
   (pending age +30 minutes or patch +500 lines) or the one bounded reminder.
   A new SHA, clock tick, raw count within a step, or session restart does not
   itself create a new episode. Coordination has only its bounded reminder.
   Clearing/reentry follows the clear thresholds, not every ref movement.
5. A reminder needs a later actual foreground event while the condition persists.
   Never wake an idle agent just to repeat statistics. Cooldown expiry alone
   cannot cause another send. Count material-step sends separately from reminders.
6. Periodic mode makes one fresh stats line eligible at the configured cadence,
   without requiring a threshold. No catch-up queue or burst after idle.
   All routine sends share spacing and rolling hourly message/byte caps per
   canonical worktree. Restarts, new/concurrent host sessions, ref movement and
   config edits do not replenish that shared rolling budget. Cap exhaustion
   does not create later automatic catch-up.
7. Select at most one line, stable priority: unintegrated, coordination, patch.
   Coalesce useful counters only within the cap. Required stale/incomplete
   facts mean quiet plus refresh request. Keep suppression reasons outside
   injected text. On new sessions, preserve episode state to avoid repeat spam.
8. Existing newly critical policy notifications take precedence over stats
   and retain required delivery semantics. Render them concisely too; their
   exception to routine cadence must be explicit and tested. The routine quota
   does not claim to cap all explicit errors or critical notifications. Never
   weaken hard checks to meet a notification budget.

Validation: reject unknown fields, invalid ref syntax, negative durations,
zero windows, both window selectors, >500 requested commits, unbounded limits,
clear counts >= entry counts and impossible byte/interval combinations. A valid
but missing ref yields unavailable facts, not invented counts. `0` has only
its documented disable meaning. Expose effective configuration and suppression
counts on demand. Precedence: defaults, project config, explicit CLI overrides;
retain documented existing adapter overrides only for their existing fields.

## Facts

- Integrated commits: first-parent commits at the configured locally known ref
  in the selected window. Use committer history for minute windows; document
  timestamp limitations. Never infer active work time or user turns from Git.
- Source/test/coordination lines: per-first-parent diff categories, additions
  and deletions separate. Offer endpoint net change on demand. Classification
  precedence: generated, tests, coordination, source, other; no double-counting.
- Pending commits: commits unique to HEAD versus the integration ref. Pending
  lines/files: final branch/worktree diff relative to merge base. Keep gross
  churn separate; do not add staged and unstaged stats and call that a unique
  final patch. Count untracked files without reading their contents for LOC.
- Pending age: elapsed time since the collector first observed the current
  sustained episode. It is observed age, not time spent working.
- Dirty files: unique tracked/untracked paths after exclusions.

First-parent diffs avoid counting merge commits plus their branch history twice.
Squash integration must reconcile patch/tree evidence before interpreting old
candidate ancestry as pending work; uncertain equivalence suppresses that
automatic signal and is reported on inspect. Handle rename, deletion, binary,
generated files, detached HEAD, shallow history, missing refs, rebase/force-push
and concurrent worktrees explicitly. Discontinuity resets affected baselines
with a recorded reason, not a report of zero progress.

The MVP has no task-turn count, review-ready claim, accepted-goal count, agent
attribution or repeated-tool detector. Those require observed host/task events.
A docs/research task can use periodic stats, disable a signal or configure its
own categories. Line counts never become productivity or acceptance targets.

## Collection

Separate collection from delivery. The foreground hook reads a small validated
snapshot, applies scheduling and emits at most one line. It never waits for a
query graph, repository scan, history walk, remote fetch or model.

Use existing resident/snapshot machinery where its lifecycle works. One bounded
refresh is in flight per repository/worktree generation; later events coalesce
into at most the newest queued generation. No hosted service or database.
On cold/missing/stale state, request refresh and return no routine text.
Prove the worker survives the target host's hook lifecycle. If it cannot,
automatic stats stay quiet with explicit query fallback; never substitute a
synchronous full scan. Do not assume an orphaned process survives tool exit.

Cache history by repository, integration SHA, window bucket and classification
version; cache worktree facts by identity and observed generation. Event gaps,
config/ignore changes, deletion/rename, clock rollback, corrupt state and watcher
overflow require bounded refresh or unknown coverage. Snapshot freshness and
remote-ref freshness are distinct and both visible on inspect.

Use ignored Git-local/cache state, atomic replace and a single-writer lease.
Current snapshots target <=16 KiB; history has bounded retention. Never read the
growing audit JSONL to recover current state. Limit background CPU/I/O as well
as foreground time. Cancel/discard obsolete generations; kill timed-out worker
trees, release leases, retain old evidence as stale and retry no sooner than
the next allowed refresh. Never spawn per tool call or loop on failures.

Acceptance on an identified host: unchanged automatic event p95 <=25 ms end to
end; warm structural single-file feedback p95 <=100 ms when complete coverage
fits; <=1 full fallback per edit batch. Foreground must return within 50 ms
under an injected slow collector. Rich work exceeding its budget stays pending
with honest coverage. These are proposed targets, not current product claims
or replacements for existing gates.

## Boundaries

Keep one policy engine and thin adapters. Add facts and compact rendered text
to the existing nudge envelope; reuse `assura agent nudge` and
`assura check --format agent`. Do not duplicate thresholds in Python. New
collector internals need scoped spec review; public breaking changes and new
dependencies retain their existing justification and authority requirements.

Assura reports configured conditions; it does not decide productivity, merge
merit or goal completion. Actual host delivery must be observed before claiming
support. Generated hook files alone are insufficient proof.
