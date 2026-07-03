# Website Agent Onboarding Experience

## Goal

Give users and coding agents a dedicated website path for the current
agent-ready first-run onboarding flow: install, run the broad local onboarding
command, inspect installed/detected/verified/inactive output, read the
onboarding packet, ask the remaining specialization questions, and then
specialize without inventing conventions.

## What I Already Know

- Parent program: `docs/goals/assura-agent-ready-project-onboarding-program.md`
  is the active P0 adoption lane.
- Child goal: `docs/goals/assura-website-agent-onboarding-experience.md`
  remains planned and targets a dedicated website onboarding path.
- Existing website Getting Started and Quick Start pages still focus on
  `assura init` and `assura check`; they mention agent feedback but do not
  walk through `assura agent onboard`.
- `website/src/content/docs/reference/api.md` lists `assura agent onboard` and
  the content templates, but it is a reference page, not a first-run journey.
- `website/src/content/docs/reference/agent-feedback.md` explains feedback
  delivery and lifecycle profiles, but it is not an onboarding page.
- `website/astro.config.mjs` has no Agent-Ready Onboarding sidebar item.
- Current support docs classify `assura agent onboard` as an experimental local
  surface. Remote bootstrap wrapper and later specialization flows remain
  planned or roadmap behavior and must not be shown as current commands.
- Current command-surface truth includes `assura agent onboard`,
  `--content-template agent-project`, `--content-template document-project`,
  `assura doctor`, `assura explain`, `assura agent nudge`, and
  `assura agent integration`.

## Revalidation Result

Status: valid.

Live evidence shows the website is behind the implemented onboarding surface:
the CLI now emits generated packet files, checked/unchecked state, lifecycle
profiles, content templates, and ranked next actions, but the public docs lack
a dedicated guide that teaches that journey end to end.

## Requirements

- Add a dedicated Agent-Ready Onboarding guide reachable from Getting Started
  navigation and the homepage or getting-started page.
- Explain the current local first-run journey using supported commands only.
- Explain remote install-and-delegate wrapper as a roadmap/convenience concept,
  not as a current quickstart command.
- Show generated onboarding packet files and the purpose of
  `.assura/onboarding/agent-next.md`.
- Teach checked versus unchecked capabilities and avoid implying a green check
  means fully onboarded.
- Show first-run output snippets for installed, detected, verified, inactive,
  lifecycle profiles, and user choices needed.
- Explain agent-project, document-project, and optional domain packs without
  adding proposal/SBIR behavior to the core preset.
- Explain nudge, warn, and gate lifecycle modes using existing commands.
- Add target-state checks that fail if the dedicated guide, sidebar link, or
  key command-surface truth markers drift.
- Capture rendered desktop and mobile proof for the revised page.

## Acceptance Criteria

- [ ] `website/src/content/docs/guides/agent-ready-onboarding.md` exists.
- [ ] The sidebar includes Agent-Ready Onboarding under Getting Started.
- [ ] The homepage or Getting Started page links to the dedicated guide.
- [ ] The page includes sections for first-run phases, generated packet,
  checked versus unchecked capabilities, `agent-next.md` questions,
  project-type/content packs, adapter behavior, hook lifecycle, and
  specialization.
- [ ] Unsupported planned surfaces are labeled roadmap-only and are not shown
  as quickstart commands.
- [ ] Target-state checks guard the dedicated guide and planned-surface labels.
- [ ] Website build passes.
- [ ] Desktop and mobile rendered evidence is captured under the Trellis task.

## Definition Of Done

- Website docs and navigation updated.
- Target-state drift checks added.
- Rendered desktop and mobile screenshots captured.
- Independent review confirms the page does not overclaim unsupported commands
  or hide unchecked capabilities.
- Validation commands from the goal pass.

## Technical Approach

Create one dedicated Starlight Markdown page rather than reshaping the whole
website. Link it from the sidebar and lightweight entry points. Keep examples
limited to currently supported local commands and use roadmap callouts for the
remote wrapper and later specialization flows. Add narrow `cargo xtask
target-state` checks around the page path, sidebar slug, core section markers,
supported commands, and forbidden quickstart examples.

## Out Of Scope

- No new marketing landing page.
- No changes to performance pages or claims.
- No new CLI command surfaces.
- No browser automation beyond rendered proof for this page.

## Technical Notes

- Current website docs live in `website/src/content/docs/`.
- Navigation lives in `website/astro.config.mjs`.
- Target-state checks live in `xtask/src/main.rs`.
- Website build can run through `cargo xtask docs` or `pnpm --dir website build`.
