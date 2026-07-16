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
  tree rules such as `web/: $web-app`.
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
- `pnpm --dir website test:marketing` passed 52 browser checks across landing,
  performance, canonical docs, themes, accessibility, metadata, and links.
- The compact project contract now groups TypeScript extensions with
  `"**/*.{ts,tsx}"` relative to the root scope, groups generated-output
  exclusions with one brace glob,
  and places `.dir` under `apps/` so its child-directory scope is visible.
  The executable fixture, 360px/390px overflow checks, and mobile render all
  pass with the shorter configuration.
- `target/performance/landing-config-alignment.json` measured all eight accepted
  cold LS-Lint comparisons and all eight warm session comparisons; the strict
  no-slower gate passed with 1.2463x aggregate cold and 16.0526x aggregate warm
  speedups on this machine.
- `cargo xtask pr` passed on the completed implementation, including 488 library
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

## Notation Ergonomics Review Loop

The follow-up notation objective uses
`.agents/skills/assura-notation-review/SKILL.md` as its durable review contract.
Each syntax range must preserve the full project-shaped policy model, support
single and multiple reusable rules, keep the detailed nested form available,
and produce no material performance or configurability regression.

The current visible costs are the YAML-required quotes around `@rule`
references and the inline JSON-like mapping used for compact multi-attribute
rules. Reviewers must inspect both canonical YAML examples and rendered mobile
and desktop screenshots. Each retained range receives focused tests, regression
tests, before/after performance evidence, screenshot verification, and its own
commit before the next independent review.

### Iteration Log

- Iteration 0: current committed baseline is `ee0e28f`. The mobile performance
  screenshot shows quoted `@rule` definitions/references and the project
  contract uses an inline `{ naming: ..., max_lines: ... }` rule definition.
  The review guide is being established before selecting a replacement syntax.
- Iteration 1: independent review retained plain rule definitions, unquoted
  `$rule` references, and block mappings for public examples. Unsigiled scalar
  references remain deferred because they collide with ordinary directive
  values; rewriting invalid unquoted `@rule` YAML before parsing was rejected
  because it would require a second lexer. Runtime normalization, onboarding,
  canonical website fixtures, docs, and specs now use the retained notation.
  Focused verification passed 34 structure-notation unit tests, 17 public
  notation integration tests, 57 executable docs YAML examples, 51 marketing
  browser checks, and the repository self-check over 1,644 files and 376
  directories. The paired same-host VPS comparison kept every accepted Assura
  fixture no slower than LS-Lint. Although one aggregate high-scope fixture
  varied by +17.9%, exact repeated command probes improved by 3.2% for `assura`
  and 4.0% for `assura-check`, so the controlled tie-breaker found no repeatable
  command-level regression.
- Iteration 2: the stop-condition review retained hardening only. The marketing
  renderer now preserves scalar YAML sequence rows exactly, and a browser test
  reconstructs the displayed Assura policy byte-for-byte against its executable
  fixture. Loader notation routing now detects the semantic top-level `rules`
  key across plain, spaced, quoted, and root-indented YAML forms; each removed
  `@` definition returns targeted migration guidance. Existing ordered `use`
  composition is documented as executable YAML, and the final active
  `use: @group-name` example was migrated. The loader split keeps the production
  file at 487 lines under its 500-line contract. On the paired VPS, every
  accepted fixture remained no slower than LS-Lint, aggregate Assura runtime
  improved 1.1%, and config loading improved 23.0%. A 50-run alternating exact
  tie-breaker measured +1.27% for `assura` and -0.05% for `assura-check`, within
  normal runtime variance and with no material regression.
- Iteration 3: Linux CI repeatedly exposed the known tight cold-command row on
  the 800-scope regression fixture. A canonical-first root-deserializer
  experiment improved measured config-load, checker-init, and validation phases
  but lost 2.18% in a 100-run exact public-command tie-breaker, so it was
  rejected. An optimized marker scan passed a local 15-sample gate but failed
  Linux CI and did not meet the performance keep bar, so it was also removed.
  Report inspection identified the benchmark defect: all Assura samples ran as
  one block before all LS-Lint samples, allowing shared-runner load drift to
  favor the second tool. Headline samples now alternate tool order on every
  iteration, and CI measures 16 balanced samples rather than 5 without relaxing
  the strict no-slower threshold. Each paired row records that ordering method
  in JSON/JSONL evidence. The corrected local fixture measured 44.86 ms for
  Assura and 46.15 ms for LS-Lint with all 8 accepted rows passing. Focused
  ordering, failure-path, and notation tests, all 492 library tests, every integration suite,
  self-check over 1,645 files and 376 directories, 57 documentation examples,
  Clippy, evidence/target-state checks, and the 48-page Astro build passed.
  Independent review found no notation correctness or configurability issue;
  GitHub CI remains the final Linux confirmation of the corrected measurement.
- Final closure review found no remaining actionable public-notation issue and
  declared the stop condition met. Scalar `$rule` references, ordered `use`
  sequences, and expanded mappings cover the retained concise and complex
  forms. Unsigiled references and unquoted-`@` preprocessing remain rejected
  because they introduce scalar collisions or a second YAML parsing layer.

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
