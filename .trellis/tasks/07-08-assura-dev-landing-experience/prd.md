# Astro landing experience

## Goal

Create a polished standalone Assura landing page for `assura.dev` inside the
existing Astro website, independent of the Starlight docs experience. The page
should quickly explain that Assura makes repository structure and agent
onboarding rules explicit, local, and checkable, while preserving existing docs
routes and only aligning their color palette.

## What I already know

- The site lives in `website/` and uses Astro 6 with Starlight.
- The current `/` route is `website/src/content/docs/index.mdx` with Starlight's
  splash template.
- Existing docs routes such as `/guides/quickstart/`, `/reference/performance/`,
  and `/roadmap/` should keep working.
- The user asked for generated visual direction, auto light/dark mode, polished
  product feel, responsive breakpoint verification, and an explicit expected
  change checklist.
- The user provided a Google Drive ZIP mockup,
  `review-and-redesign-assura-dev-pages.zip`, with a more polished
  Next/shadcn-style direction. The Astro implementation should use it as
  inspiration rather than porting it directly.
- The implementation is in a dedicated worktree:
  `/Users/nroth/workspace/assura-landing-experience` on branch
  `codex/assura-landing-experience`.

## Assumptions

- A custom `website/src/pages/index.astro` is the right boundary for a landing
  page independent of the docs content collection.
- Existing docs content should not be rewritten except where route separation or
  palette alignment requires a small change.
- A generated raster hero visual can be used as design direction or a project
  asset, but the page should remain robust without JavaScript-heavy animation.

## Requirements

- Add a modern, techy, minimalist landing experience at `/`.
- Keep the hook concise: Assura validates repository shape, documentation, and
  agent handoffs from repo-local contracts.
- Provide primary CTAs for quick start/install and GitHub.
- Surface the practical product story: local checks, structure contracts,
  agent-ready onboarding, CI/report outputs, and performance evidence.
- Support automatic light and dark mode via `prefers-color-scheme`.
- Align docs palette with the landing page without redesigning docs content.
- Use accessible semantic HTML, stable responsive dimensions, and no overlapping
  or overflowing text at common breakpoints.
- Keep docs site behavior independent from the landing page.

## Verification Checklist

- [x] `/` renders the new landing page, not the Starlight splash page.
- [x] Existing docs routes still build and remain reachable.
- [x] The landing hero includes a concise Assura product hook.
- [x] The page includes install/quick-start and GitHub CTAs.
- [x] The page includes a polished visual system informed by generated imagery
      and the provided shadcn-style mockup.
- [x] Light mode and dark mode both have intentional palettes.
- [x] Docs palette variables are aligned to the landing palette.
- [x] Desktop, tablet, and mobile screenshots show no text wrapping overflow,
      horizontal scrolling, or incoherent overlap.
- [x] Mobile hero proportions leave the next section visible in the first
      viewport, including short 320x568 and 360x640 phones.
- [x] Desktop hero proportions leave a visible hint of the next content band.
- [x] Production build passes.
- [x] Assura structure validation passes for changed files or any new paths are
      intentionally added to `.assura/config.yml`.

## Verification Evidence

- `pnpm build` passed in `website/`.
- `cargo xtask docs` passed.
- `cargo xtask evidence` passed.
- `cargo xtask target-state` passed.
- `cargo run --quiet -- check --format json .` passed.
- `git diff --check` passed.
- Browser screenshots and overflow checks were captured under
  `/tmp/assura-landing-verification/`.
- Revised mockup-inspired screenshots and full-page captures were captured under
  `/tmp/assura-landing-redesign-verification/`.
- Browser overflow verification was saved to
  `/tmp/assura-landing-redesign-verification/overflow-results.json`.
- Browser overflow verification covered 1440x1000 desktop light/dark, 768x1024
  tablet, 390x844 mobile light/dark, 360x640 small-phone light, and 320x568
  iPhone SE light mode with no horizontal overflow, no viewport-wide offenders,
  and visible next-section pixels in every viewport.
- Current CLI command-output shapes were checked against `cargo run --quiet --
  check` and `cargo run --quiet -- check --format agent --agent codex` before
  updating the terminal examples.

## Definition of Done

- Generated design direction has been inspected and either integrated as an
  asset or translated into the final Astro/CSS system.
- `pnpm build` succeeds in `website/`.
- Browser verification covers key widths including mobile and desktop.
- `cargo run --quiet -- check --format json .` passes, or any failures are
  explained and unrelated.
- A review agent has reviewed the implementation before PR creation or final
  readiness.

## Out of Scope

- Rewriting docs content or restructuring the full Starlight docs tree.
- Adding a marketing CMS, analytics, forms, backend routes, or hosted service.
- Changing core Rust CLI behavior.
- Introducing a large frontend framework dependency for this static page.

## Technical Notes

- Existing site dependencies: Astro, Starlight, Catppuccin plugin, Sharp.
- Existing palette currently centers cyan/teal on a dark Catppuccin docs theme.
- `website/node_modules` was not installed in the fresh worktree at task start;
  dependency installation is required before build verification.
