---
title: Website roadmap live revalidation
status: current
date: 2026-07-10
goal: ../goals/assura-website-landing-seo-roadmap.md
pull_request: https://github.com/rothnic/assura/pull/140
---

# Website Roadmap Live Revalidation

## Decision

Result: `valid`.

The roadmap remains necessary and is not superseded. Pull request 140 provides
the current standalone Astro landing foundation, compact review implementation,
agent-hook work, and initial responsive design, but it does not satisfy the
roadmap's P0 through P3 proof gates.

## Current Evidence

- The landing page is a standalone static Astro route and the technical docs
  remain on Starlight.
- The current page has the intended signal palette, light and dark modes,
  responsive layout, product-output styling, author links, and agent-first
  positioning.
- The CLI currently supports the public Onboard, Review, Explain, and Check
  journey described by the roadmap.
- The checked performance artifact reports Assura version 0.3.0, eight accepted
  cold comparisons, and eight accepted warm-session comparisons.
- Pull request 140 has a Workers preview and all prior checks passed except the
  performance job, whose failure is an artifact-upload contract: the native
  report files were not present at the paths required by the workflow.

## Unmet P0 Evidence

| Requirement | Live state | Evidence needed to close |
| --- | --- | --- |
| Public command truth | Two hero/review examples use unsupported review flags | Website command smoke test and rendered-page scan |
| Generated product evidence | Review examples are hand-authored in the page | Deterministic fixture, generated JSON, and clean regeneration check |
| Hero hierarchy | The full setup prompt is expanded above the primary review artifact | Accessible setup dialog or sheet and six-width screenshots |
| Review output truth | The page shows opaque scores and finding-history behavior not yet supported | Current deterministic review output with transparent measurements |
| Command journey | The page discusses lifecycle stages but does not present Onboard, Review, Explain, and Check as one clear journey | Rendered journey component and content audit |
| Performance proof | The homepage does not render the current cold/warm evidence | Generated proof strip linked to methodology |
| Reusable marketing system | The landing page is a 2,000-line single file | Shared layout, header, footer, section, terminal, setup, and SEO components |
| Visual regression | Prior spot screenshots do not cover all six roadmap widths | Automated overflow and screenshot report at all required widths in both themes |
| Accessibility | No dialog behavior or automated accessibility proof exists | Keyboard/focus test and accessibility audit |
| README truth | README still presents an older, narrower release story | Version and command-surface audit plus refreshed README |
| Performance CI | Native report artifact upload fails when files are not generated | Green performance job with explicit report generation or conditional upload |

## Later-Phase Gaps

- P1 marketing routes, sitemap, robots policy, preview `noindex`, reusable SEO
  metadata, structured data, social images, and CTA measurement are absent.
- P2 conversion routes, examples index, personal case-study link, benchmark
  article, and dogfood case study are absent.
- P3 monthly search and performance scorecard has no baseline or owner record.
- Product finding memory, explicit comparison-base control, shared worktree
  cache, and the complete latency-budget matrix remain planned work and must not
  be advertised as current until implemented and tested.

## Execution Order

1. Correct command and output truth and generate website demo evidence.
2. Extract the reusable marketing system while preserving the rendered design.
3. Replace the expanded setup prompt with an accessible agent-first dialog.
4. Add current cold/warm performance proof and repair the performance CI job.
5. Complete six-width visual, accessibility, docs-route, and command-smoke
   evidence.
6. Continue into P1 and P2 routes and SEO infrastructure.
7. Implement only the product gaps required for claims that remain on the site.

## Completion Rule

The roadmap is not complete until every item in the canonical goal's milestone
scorecard has direct evidence. Passing builds or a polished homepage alone do
not prove completion.
