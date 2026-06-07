---
name: trellis-continue
description: "Resume work on the current task. Loads the workflow Phase Index, figures out which phase/step to pick up at, then pulls the step-level detail via get_context.py --mode phase. Use when coming back to an in-progress task and you need to know what to do next."
---

# Continue Current Task

Resume work on the current task — pick up at the right phase/step in `.trellis/workflow.md`.

---

## Step 1: Load Current Context

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
# If an injected <workflow-state> Task path is available but the gate cannot
# resolve it from session state, rerun with:
# python3 ./.trellis/scripts/workflow_gate.py --platform codex --task <task-path>
```

Run `python3 ./.trellis/scripts/get_context.py` only when the gate says
`Ready: no`, when no task is active and you need task listings, or when you need
full session detail.

Before routing phases, enforce the workflow gate:

- Run `workflow_gate.py` on each new user request before changing files or
  scope. Steering/correction messages inside the same turn count as part of the
  original request.
- If it prints `Ready: yes`, continue without reading the full workflow doc
  unless you are changing phase, blocked, or unsure.
- If it prints `Ready: no`, follow its `Next` and `Needs` output before
  continuing.
- If dirty ownership is unclear, offer exactly these options:
  1. Commit prior work now
  2. Park it on a branch
  3. Leave it untouched and continue in a fresh worktree/branch

Never carry unclassified uncommitted changes into a new task or phase.

## Step 2: Load the Phase Index

```bash
python3 ./.trellis/scripts/get_context.py --mode phase
```

Skip this when the gate says `Ready: yes` and you are not changing phase,
blocked, or unsure. It shows the Phase Index (Plan / Execute / Finish) with
routing + skill mapping.

## Step 3: Decide Where You Are

`get_context.py` shows the active task's `status` field. Route by `status` + artifact presence:

- `status=planning` + no `prd.md` → **1.1** (load `trellis-brainstorm`)
- `status=planning` + Codex inline mode + `prd.md` exists → **1.4** (run `task.py start`; JSONL curation is skipped)
- `status=planning` + `prd.md` exists + `implement.jsonl` or `check.jsonl` not curated (only the seed `_example` row) → **1.3**
- `status=planning` + `prd.md` + curated `implement.jsonl` + curated `check.jsonl` → **1.4** (run `task.py start` to enter Phase 2)
- `status=in_progress` + implementation not started → **2.1**
- `status=in_progress` + implementation done, not yet checked → **2.2**
- `status=in_progress` + check passed → **3.1**
- `status=completed` (rare; usually archived immediately) → archive flow

Phase rules (full detail in `.trellis/workflow.md`):

1. Run steps **in order** within a phase — `[required]` steps must not be skipped
2. `[once]` steps are already done if the output exists (e.g., `prd.md` for 1.1; `implement.jsonl` with curated entries for 1.3) — skip them
3. You may go back to an earlier phase if discoveries require it

## Step 4: Load the Specific Step

Once you know which step to resume at:

```bash
python3 ./.trellis/scripts/get_context.py --mode phase --step <X.X> --platform codex
```

Follow the loaded instructions. After each `[required]` step completes, move to the next.

---

## Reference

Full workflow, skill routing table, and the DO-NOT-skip table live in `.trellis/workflow.md`. This command is only an entry point — the canonical guidance is there.
