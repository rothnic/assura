---
title: Post-beta follow-up roadmap goals
status: active
---

# Post-Beta Follow-Up Roadmap Goals

## Goal

Create repo-native goal files and roadmap routing for the next large Assura
iteration after `v0.2.0`, covering the unresolved follow-up areas from the beta
completion summary.

## What I Already Know

- The beta master goal is complete and released as `v0.2.0`.
- The requested follow-ups are:
  - Assura self-config hardening and documentation/structure variance cleanup;
  - fully supported document graph support for content validation, searching,
    querying, graph expansion, relation checks, affected references, and bounded
    agent context;
  - true daemon mode beyond the runtime-metadata preview;
  - performance hardening beyond the no-slower LS-Lint gate;
  - installed agent integration lifecycle for Codex, OpenCode, Claude, and Pi;
  - markdownlint-consistent, highly performant Rust Markdown linting/fixing;
  - supported VS Code extension path beyond the experimental beta package;
  - clarification of what extension APIs mean before deciding whether to plan
    them;
  - final LS-Lint performance reassessment with row-by-row fixture gates;
  - final support and release hardening.
- Existing beta goals and docs should remain completed; this task should add
  next-iteration planning artifacts, not reopen the beta release.

## Requirements

- Add one parent post-beta program goal.
- Add child goals for each requested follow-up area.
- Add a detailed north-star use case to the parent goal and make it final
  verification criteria for the program.
- For Markdown linting/fixing, incorporate local research and current ecosystem
  research before setting the proof gates.
- Preserve Assura's layered validation model: structure and coarse file-level
  policy first, then Markdown, content models, repository references, and
  language-specific checks.
- Update the internal roadmap and public roadmap artifact so the next large
  iteration is discoverable.
- Validate docs/Trellis/self-check state and open a GitHub PR.

## Acceptance Criteria

- [x] Parent goal exists under `docs/goals/` and references all child goals.
- [x] Parent goal includes a concrete end-to-end use case that can verify the
      final program outcome.
- [x] Child goal files exist for self-config/doc variance hardening, supported
      document graph, daemon mode, performance hardening, agent integration
      lifecycle, markdownlint-compatible Rust lint/fix, VS Code support,
      extension API clarification, LS-Lint performance reassessment, and
      release hardening.
- [x] Markdown docs no longer imply Markdown linting sits above structure
      validation.
- [x] Roadmap points to the parent goal as the current recommended post-beta
      work.
- [x] Public roadmap has a concise current/next representation for the new
      iteration.
- [x] `cargo run --quiet -- check --format json .`, `cargo xtask docs`,
      `cargo xtask evidence`, `cargo xtask target-state`, and `git diff --check`
      pass.
- [x] A PR is opened against `master`: <https://github.com/rothnic/assura/pull/113>.

## Review Evidence

- Independent review agent Bohr found no blocker or high-risk findings after
  checking the parent program, child goals, internal roadmap, public roadmap,
  Markdown hierarchy wording, and PRD.
- Local validation passed:
  `python3 ./.trellis/scripts/task.py validate 07-01-post-beta-followup-roadmap-goals`,
  `cargo run --quiet -- check --format json .`, `cargo xtask docs`,
  `cargo xtask evidence`, `cargo xtask target-state`, `python3 -m json.tool
  docs/data/public-roadmap.json`, and `git diff --check`.

## Out Of Scope

- Implementing the daemon, linter, extension APIs, agent installers, or
  performance fixes in this planning PR.
- Changing `v0.2.0` release artifacts.

## Technical Notes

- Use `docs/goals/assura-beta-code-agnostic-capabilities-program.md` as the
  parent-goal model.
- Use `.agents/skills/assura-goal-validation/SKILL.md` for goal proof gates.
- Use `.trellis/spec/assura/roadmap.md` and `docs/data/public-roadmap.json` for
  roadmap routing.
