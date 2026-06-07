---
name: trellis-start
description: "Initializes an AI development session by reading workflow guides, developer identity, git status, active tasks, and project guidelines from .trellis/. Classifies incoming tasks and routes to brainstorm, direct edit, or task workflow. Use when beginning a new coding session, resuming work, starting a new task, or re-establishing project context."
---

# Start Session

Initialize a Trellis-managed development session. This platform has no session-start hook, so manually load the equivalent context by following these steps (each one mirrors a section the hook would otherwise inject).

---

## Step 1: Current state
Identity, git status, current task, active tasks, journal location.

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
# If an injected <workflow-state> Task path is available but the gate cannot
# resolve it from session state, rerun with:
# python3 ./.trellis/scripts/workflow_gate.py --platform codex --task <task-path>
```

Run `python3 ./.trellis/scripts/get_context.py` only when the gate says
`Ready: no`, when no task is active and you need task listings, or when you need
full session detail. If that output includes a line beginning `Trellis update
available:`, copy the full line verbatim when summarizing session context. Do
not shorten operational command hints.

Workflow gate:

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
  3. Leave it untouched and start in a fresh worktree/branch

Never start a new task while carrying unclassified uncommitted changes.

## Step 2: Workflow overview
Phase Index + skill routing table + DO-NOT-skip rules.

```bash
python3 ./.trellis/scripts/get_context.py --mode phase
```

Skip this when the gate says `Ready: yes` and you are not changing phase,
blocked, or unsure. Full guide in `.trellis/workflow.md` (read on demand).

## Step 3: Guideline indexes
Discover packages + spec layers, then read each relevant index file.

```bash
python3 ./.trellis/scripts/get_context.py --mode packages
cat .trellis/spec/guides/index.md
cat .trellis/spec/<package>/<layer>/index.md   # for each relevant layer
```

Index files list the specific guideline docs to read when you actually start coding.

## Step 4: Decide next action
Follow the gate's `Next` line first. Load specific phase detail only when the
gate says work is not ready, you are changing phase, blocked, or unsure:

```bash
python3 ./.trellis/scripts/get_context.py --mode phase --step <X.X> --platform codex
```

---

## Skill routing (quick reference)

| User intent | Skill |
|---|---|
| New feature / unclear requirements | `trellis-brainstorm` |
| About to write code | `trellis-before-dev` |
| Done coding / quality check | `trellis-check` |
| Stuck / fixed same bug multiple times | `trellis-break-loop` |
| Learned something worth capturing | `trellis-update-spec` |

Full rules + anti-rationalization table in `.trellis/workflow.md`.
