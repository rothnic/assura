# Create Assura Structure Fit Skill

## Goal

Create a repo-local Assura skill that teaches agents how to respond when a
new file or directory does not fit `.assura/config.yml`, using progressive
disclosure instead of long repeated violation messages.

## What I Already Know

- The core use case is a structure mismatch while adding a file or directory.
- The nudge should be concise and point to a stable reference such as
  `STRUCTURE_FIT_CHECK`.
- Agents should consider reuse, naming/style consistency, duplication risk, and
  whole-repo shape before changing config.
- Config edits should be last, not the default repair path.
- Repo-local skills live under `.agents/skills/`.
- Adding a new repo-local skill requires updating `.assura/config.yml` because
  the project structure policy whitelists skill directories.

## Requirements

- Add an Assura-focused skill under `.agents/skills/`.
- Keep `SKILL.md` concise and use a `references/` file for deeper guidance.
- Include a stable anchor/keyword for concise violation nudges.
- Include workflow guidance for path-first repair versus config changes.
- Include guidance for top-level directory additions that require a wider repo
  map.
- Define where the skill belongs when Assura onboarding installs or templates
  agent guidance: project-local `.agents/skills/<skill-name>/` first, with
  host/global skill installation treated as an explicit user/platform decision.
- Explain how onboarding should reference the skill without assuming every
  agent has it globally installed: generated `AGENTS.md`/onboarding packets
  should route to the project-local skill path or stable anchor.
- Extend `assura agent onboard` so new Assura-adopted repos receive the
  structure-fit skill under `.agents/skills/assura-structure-fit/`.
- Generated onboarding handoffs should mention `STRUCTURE_FIT_CHECK` as the
  concise anchor for structure mismatch violations.
- Document the install/update decision: copy or generate the skill into the
  target repo only when the repo adopts Assura agent guidance, preserve
  user-authored local skill edits, and do not silently mutate host-agent config.
- Register the skill in `AGENTS.md` without bloating the project guidelines.
- Update `.assura/config.yml` narrowly so the new skill and reference file are
  allowed.
- Audit existing Assura-focused repo-local skills for stale commands or
  outdated feature claims while this new skill is being added.
- Keep skill content aligned with current Assura features: structure-first
  checks, `assura check --format agent`, `assura agent onboard`,
  `assura doctor`, `assura explain`, local integration bundles, and current
  support boundaries.

## Acceptance Criteria

- [ ] The skill frontmatter description is concise and trigger-oriented.
- [ ] `SKILL.md` functions as an index and points to deeper reference content.
- [ ] The reference content provides the decision frame for structure
      mismatches and config changes.
- [ ] The skill describes how Assura onboarding should install/reference the
      skill in the right project-local location.
- [ ] `assura agent onboard` installs or preserves
      `.agents/skills/assura-structure-fit/` in generated baselines.
- [ ] Generated AGENTS/onboarding handoff routes structure mismatches to the
      local skill and `STRUCTURE_FIT_CHECK` anchor.
- [ ] Existing project skill routing mentions when to load the new skill.
- [ ] Existing Assura-focused skills do not advertise stale or unsupported
      commands.
- [ ] Assura self-check accepts the new skill directory and reference.

## Validation Commands

- `cargo fmt --check`
- `cargo run --quiet -- check --format json .`
- `cargo test --test project_intelligence_onboarding --quiet`
- `cargo xtask evidence`
- `cargo xtask target-state`
- `cargo xtask docs`
- `git diff --check`

## Out Of Scope

- Changing runtime violation output.
- Adding a new CLI command.
- Broadly relaxing `.assura/config.yml` for arbitrary skill files.
- Implementing global host-agent skill installation.
