# Post-Beta Primary Use-Case Verification Story

## Goal

Strengthen the post-beta parent goal so the remaining release-hardening work is
judged against one concrete maintainer workflow, not a loose list of feature
tasks.

## Requirements

- Add a user-specific, end-to-end scenario to the parent goal that names the
  repo state, branch changes, operator decisions, required proof artifacts, and
  failure cases.
- Keep the story aligned with staged validation: structure and coarse file
  policy first, then Markdown, frontmatter/content models, references, daemon,
  agent/editor surfaces, and performance gates.
- Make Support Hardening inherit this scenario as a blocking release criterion.
- Update roadmap routing if the active task pointer would otherwise be stale.

## Acceptance Criteria

- [x] The parent goal can be used by a fresh agent to verify the final beta
      increment from a single detailed use case.
- [x] The final use case makes clear what the maintainer should be able to do
      that they could not safely do before this program.
- [x] The use case includes concrete success and blocker conditions for CLI,
      daemon, content graph, Markdown fixes, agents, VS Code, support docs, and
      LS-Lint performance.
- [x] Planning-only validation passes.

## Definition Of Done

- Parent goal and release-hardening goal are updated.
- Trellis task is archived or otherwise cleanly routed.
- Validation commands pass on a clean worktree.

## Technical Approach

Patch documentation only. Do not add new feature scope or alter performance
evidence in this task.

## Out Of Scope

- New runtime behavior.
- New benchmark data.
- Changing child-goal completion status.

## Technical Notes

- Parent goal: `docs/goals/assura-post-beta-capabilities-program.md`
- Next child: `docs/goals/assura-post-beta-support-release-hardening.md`
- Roadmap: `.trellis/spec/assura/roadmap.md`
