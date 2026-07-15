# Astro landing experience

## Goal

Ship a polished standalone Assura landing experience whose configuration,
command output, capability status, and performance claims are executable
contracts with the Rust CLI. Keep the marketing page independent from
Starlight, but align the public docs, canonical notation, and release-surface
metadata so the site never demonstrates syntax or behavior Assura does not
support.

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
- Make the onboarding claim executable: detect the project, apply the broad
  built-in agent baseline through a project-owned wrapper, and report which
  project-specific policy remains undecided.
- Replace public `required` structure notation with one `exists` cardinality
  model while keeping literal hierarchy concise through implicit
  `exists:1` defaults.
- Support nested directory cardinality plus child policy and scalar reusable
  tree rules such as `web/: "@web-app"`.
- Make `assura explain` expose effective patterns, cardinality, inheritance,
  and the winning normalized policy needed to understand shorthand scope.
- Render landing examples and command-output visuals from canonical fixtures
  that the real CLI checks during docs and website evidence builds.
- Make one release-surface manifest authoritative for support maturity and
  evidence provenance; website claims must reference those surface IDs.
- Replace stale or duplicate docs pages that teach unsupported rule names or
  internal expanded config as the normal authored format.
- Keep equivalent LS-Lint performance measurements separate from untimed
  Assura-native capability examples.

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
- [x] Mobile performance proof uses two equal metric columns and a single-row
      evidence footer at 390px.
- [x] Mobile execution layers use aligned markers with no row over 150px.
- [x] Footer navigation precedes a compact creator byline with no empty trailing
      block at 390px.
- [x] Onboarding report and generated config expose an applied project-owned
      rule wrapper while leaving unsupported specialization inactive.
- [x] Exact literal files/directories share an implicit `exists:1` default;
      optional, forbidden, and bounded cases use explicit `exists`.
- [x] Direct-child file-glob and directory-capture cardinality works;
      capture-based counterpart files retain per-match relationship semantics,
      and multi-segment cardinality is rejected with hierarchy guidance.
- [x] Public authored `required` receives a migration diagnostic and is absent
      from all current examples, generated configs, and canonical docs.
- [x] Scalar directory rule references normalize equivalently to `use:` and
      reject node/tree type mismatches, cycles, and unknown rules.
- [x] The compact project and agentic monorepo examples each have passing and
      failing executable fixtures that prove naming, line limits, cardinality,
      closed-world policy, inheritance, and local overrides.
- [x] Every public YAML fence is classified; Assura config examples parse and
      execute, expected-invalid examples assert their diagnostic, and
      unclassified Assura-looking YAML fails CI.
- [x] Every marketing capability claim maps to one or more specific
      release-surface IDs with separate support and evidence status.
- [x] `review`, `check`, `explain`, onboarding, branch/worktree signals, stable
      agent feedback, configured docs/references, and measured performance have
      support-grade tests and truthful public status.
- [x] There are zero unsupported list-form `rules:` examples and zero public
      structure-node `required:` examples.
- [x] Canonical marketing and docs routes pass responsive light/dark checks at
      360, 390, 768, 1024, and 1440 widths with no horizontal overflow and no
      serious or critical accessibility findings.

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
- `cargo xtask website-demo-data --check` validates both executable marketing
  configs, 12 bidirectional release-surface mappings, and 57 classified YAML
  fences.
- `cargo test --test structure_config_notation_tests` passed 17 public CLI
  notation cases, including optional subtrees, captured directory counts, and
  unmatched reusable capture scopes.
- `cargo test --test ls_lint_parity_regression_tests` passed 11 parity cases
  with one manual performance audit ignored by design.
- `pnpm --dir website test:marketing` passed 51 browser checks across landing,
  performance, canonical docs, themes, accessibility, metadata, and links.
- `target/performance/landing-config-alignment.json` measured all eight accepted
  cold LS-Lint comparisons and all eight warm session comparisons; the strict
  no-slower gate passed with 1.2463x aggregate cold and 16.0526x aggregate warm
  speedups on this machine.
- `cargo xtask pr` passed on the completed implementation, including 485 library
  tests, all workspace integration suites, self-check, target-state, docs
  evidence, website fixture generation, and the 48-page production build.
- Independent review identified optional subtree, capture count, unmatched
  reusable capture, rule-reference scan, and support-status gaps; each accepted
  finding has a regression or source-of-truth check in this change.
- Follow-up review identified stale support-policy rows, one semantically loose
  page marker, one-way claim validation, and overstated packet-health wording;
  these now use explicit next-release status, capability-specific page markers,
  bidirectional claim checks, and core-handoff presence language.
- Tool-owned `.assura/onboarding/` state intentionally remains outside the user
  structure contract; onboarding verification and `assura doctor` report core
  handoff presence, and the canonical guide now states that boundary explicitly.

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

- Replacing Starlight or redesigning the full docs information architecture.
- Adding a marketing CMS, analytics, forms, backend routes, or hosted service.
- Code-symbol/dependency intelligence, autonomous semantic repair, MCP, or a
  public plugin API beyond currently evidenced local surfaces.
- Introducing a large frontend framework dependency for this static page.

## Technical Notes

- Existing site dependencies: Astro, Starlight, Catppuccin plugin, Sharp.
- Existing palette currently centers cyan/teal on a dark Catppuccin docs theme.
- `website/node_modules` was not installed in the fresh worktree at task start;
  dependency installation is required before build verification.
