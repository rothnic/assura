# Assura agent entrypoint

Assura is a pre-1.0, structure-first repository-policy CLI. Keep one configured
policy engine, thin host adapters, project-owned conventions, and honest
evidence. Broad automatic repair, hosted orchestration, semantic inference as
policy, and public third-party plugin APIs are outside the supported contract.

## Start and ownership

1. Read this file, then run `python3 .trellis/scripts/workflow_gate.py --platform
   codex` on each new request; use the actual platform outside Codex. Steering
   within a turn does not require restarting. If task resolution is missing,
   rerun with `--task <task-path>`. Whenever `Ready: no`, follow `Next`/`Needs`
   before starting work; resolve or isolate the stated prerequisite.
2. Inspect cwd, branch, remote, HEAD and `git status --short`. Preserve unknown
   work. Use a clean owned checkout when the current one has unrelated dirt.
3. Resolve task files at the refreshed integration revision. A canonical task
   path in an old checkout is historical; it is not the latest ledger.
4. Read only the selected task/card and the relevant spec/skill routes below.
   Do not create a second task for an already active canonical task.

## Rules every agent needs

- One owner per candidate; keep file ownership explicit when delegating.
  Commit coherent owned work before review, handoff, or changing cards.
- Test observable behavior, including meaningful negative controls. Preserve
  user configuration, errors, policy thresholds, and unfavorable evidence.
- A passing command is insufficient when expected tests were skipped or zero,
  policy was empty, or generated hooks were never observed executing.
- Independently review complex changes before a PR. Resolve concrete findings,
  rerun affected checks, and obtain scoped rereview of the final changes.
- Merge only within existing authorization, with current-base source, resolved
  review, required hosted/local checks and the slice's acceptance evidence.
  Deployment, release, protections, invitations and publication need their own
  authority. Inspect actual trigger coupling before an affected push or merge.
- Do not lower gates, hide failures, or infer that earlier failures excuse a
  current one. A policy change needs an explicit separate decision.
- Preserve source SHA, cwd, binary/toolchain, commands, exits and review proof.
  Verify merged reachability and clean ownership before removing branches.
- Use conventional commits. Document public Rust APIs; preserve contextual
  errors and bounded, thread-safe behavior. Breaking public API changes require
  approval; new modules need spec review and dependencies need justification.

## Skills

Read the selected `SKILL.md` before applying it. Paths below are repo-relative.

| Work | Entry point |
| --- | --- |
| Resume or orchestrate a goal/Trellis backlog | [.agents/skills/assura-goal-execution/SKILL.md](.agents/skills/assura-goal-execution/SKILL.md) |
| Revalidate an older goal or scope | [.agents/skills/assura-goal-validation/SKILL.md](.agents/skills/assura-goal-validation/SKILL.md) |
| Session setup without injected context | [.agents/skills/trellis-start/SKILL.md](.agents/skills/trellis-start/SKILL.md) |
| New requirements / implementation / verification | `.agents/skills/trellis-{brainstorm,before-dev,check}/SKILL.md` |
| Rust changes | [.agents/skills/assura-rust-quality/SKILL.md](.agents/skills/assura-rust-quality/SKILL.md) |
| Build environment or VPS validation | [.agents/skills/assura-local-build/SKILL.md](.agents/skills/assura-local-build/SKILL.md) |
| Performance evidence | [.agents/skills/assura-performance-reporting/SKILL.md](.agents/skills/assura-performance-reporting/SKILL.md) |
| Host hooks/lifecycle | [.agents/skills/assura-agent-harness-hooks/SKILL.md](.agents/skills/assura-agent-harness-hooks/SKILL.md) |
| Config notation / rejected structure | `.agents/skills/assura-{notation-review,structure-fit}/SKILL.md` |

Specs start at [.trellis/spec/assura/index.md](.trellis/spec/assura/index.md).
Rust/toolchain truth is `Cargo.toml`, toolchain files and current CI, not a
copied version here. Use the validation matrix linked from goal execution;
docs/process edits do not automatically require the full Rust tier.
Keep operational procedures in skills, current state in task evidence, and
AGENTS as this shared router. Compatibility policy lives in the scoped specs;
do not introduce internal pre-1.0 shims without a demonstrated consumer.
For layered context or A07 screening-manifest routing, load the linked
`assura-goal-execution` references only when that phase requires them; never
inline private evaluator details here.

<!-- TRELLIS:START -->
# Trellis Instructions

These instructions are for AI assistants working in this project.

Trellis is the canonical workflow, task, and spec system for Assura.

When starting work:
- Read `.trellis/workflow.md` for the active development process.
- Check `.trellis/tasks/` for active and archived work.
- Check `.trellis/spec/` for durable project specs and constraints.
- Check `.trellis/workspace/` for session/developer continuity.
- Use `.assura/config.yml` and `assura check .` for project structure validation.

For Codex, Trellis context injection depends on user-level
`features.hooks = true` and one-time `/hooks` approval. If hooks are not active,
read `.agents/skills/trellis-start/SKILL.md` manually before starting Trellis
workflow work.

OpenSpec and `specs-bak/` are historical unless a newer ADR says otherwise.
See `docs/analysis/2026-05-09-trellis-governance-adr.md` and
`docs/analysis/2026-05-09-documentation-cleanup-register.md`.

Keep this managed block so `trellis update` can refresh the instructions.

<!-- TRELLIS:END -->
