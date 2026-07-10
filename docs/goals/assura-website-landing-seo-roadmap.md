---
id: goal-assura-website-landing-seo-roadmap
type: goal
title: Assura website, landing-page, performance, and SEO roadmap
status: planned
created: 2026-07-10
owners:
  - assura-maintainers
source_drive_file_id: 1WjFntl5Bitormyb_PLjpNo7ANyfG81kC
source_drive_url: https://drive.google.com/file/d/1WjFntl5Bitormyb_PLjpNo7ANyfG81kC/view
---

# Assura Website, Landing-Page, Performance, and SEO Roadmap

## Objective

Preserve the current Assura design system while simplifying the first-screen
hierarchy, making the command model truthful, elevating performance as proof,
and reusing the marketing system across focused second-level pages.

The product story is:

> **Assura catches project drift while the fix is still small.**

The workflow is:

> **Onboard once. Review while working. Explain when needed. Check before
> merge.**

The command distinction is:

> **Review is the radar. Check is the gate.**

## Imported Plan

The complete Drive source is preserved in three chapters so each file stays
within Assura's 1,000-line document limit. Parts 2 and 3 add only storage
frontmatter required by repository policy; removing those two frontmatter
blocks and concatenating the files in order reproduces the imported Markdown
byte for byte:

1. [Product truth, performance, design, and landing layout](../archive/assura-website-landing-seo-roadmap/01-product-design-and-landing.md)
2. [Install UX, marketing pages, documentation boundaries, and SEO](../archive/assura-website-landing-seo-roadmap/02-install-pages-and-seo.md)
3. [Creator strategy, claim evidence, backlogs, acceptance, and implementation prompt](../archive/assura-website-landing-seo-roadmap/03-entity-backlogs-and-acceptance.md)

Source SHA-256:
`463a686202e1e6c48e06c272594555c55603ab35e909c1b53b2022c816e684f8`.

## Scope

- Preserve the current Astro marketing identity and Starlight documentation
  split.
- Correct public command, integration, and performance claims.
- Establish the Onboard, Review, Explain, Check journey.
- Make current product output and generated performance evidence the primary
  proof.
- Extract reusable marketing components without introducing a frontend
  framework.
- Add focused marketing pages, SEO infrastructure, author attribution, and
  conversion measurement.
- Advance the product capabilities needed to support the marketing truth.

## Milestone Scorecard

These checks make progress observable without changing the imported roadmap's
direction. Record evidence links, measured values, and completion dates in the
progress log as each phase closes.

| Milestone | Expected measurable outcome | Required evidence |
| --- | --- | --- |
| Baseline before P0 | Capture production at 360, 390, 430, 768, 1024, and 1440 pixels; record Lighthouse scores; inventory every public command and benchmark claim | Dated screenshots, Lighthouse report, and claim inventory |
| Website P0 complete | Zero unsupported commands in rendered pages; zero horizontal overflow at all six widths; all copied commands pass smoke tests; homepage benchmark version matches the promoted release; setup dialog passes keyboard and screen-reader checks | CI output, six-width visual report, command-smoke results, current performance JSON, and accessibility report |
| Website P1 complete | Every P1 route has a unique title, description, H1, canonical, and social image; sitemap and robots endpoints succeed; previews emit `noindex`; JSON-LD validates; every high-value CTA emits a named event | Route metadata matrix, endpoint checks, schema validation, preview evidence, and analytics event tests |
| Website P2 complete | Four conversion pages and the examples index are published and mutually linked; the Assura case study and benchmark-methodology article are live; GitHub metadata is aligned; submitted URLs are discoverable | Production URL inventory, crawl report, GitHub metadata snapshot, and Search Console submission evidence |
| Website P3 cadence | Search and performance data is reviewed monthly; each review logs indexed pages, impressions, clicks, CTR, query mix, page-two queries, backlinks, Core Web Vitals, and crawl errors; every accepted action has an owner and due date | Monthly dated scorecard and action log, with the first month treated as baseline |
| Product P0/P1 complete | Public commands have automated coverage; review JSON is versioned; review output is bounded; transparent values and thresholds replace opaque scores; finding-state behavior has regression coverage | CLI smoke suite, schema fixture, output-bound test, deterministic review fixture, and finding-state tests |
| Product P2 complete | Latency budgets exist for no-change, one-file, directory-change, config-change, and agent-feedback paths; every path has a checked benchmark row; cache and fallback mode are observable | Versioned latency budget, benchmark history, diagnostic fixture, and CI regression gate |

Progress is measured against the baseline captured at the start of each phase.
Search ranking and field Core Web Vitals are monitored outcomes, not guaranteed
delivery claims.

## Ordered Delivery

1. **Website P0:** preserve design, correct truth, simplify the hero, refresh
   current-release performance evidence, and establish visual regression
   coverage.
2. **Website P1:** add reusable marketing infrastructure, SEO foundations, and
   the first focused second-level pages.
3. **Website P2:** add conversion and category-ownership pages, examples, case
   studies, and aligned public profiles.
4. **Website P3:** operate an evidence-led publishing and search-measurement
   cadence.
5. **Product P0/P1:** clarify command semantics and complete the compact review
   and onboarding experiences required by the marketing.
6. **Product P2/P3:** harden warm-loop behavior, caching, lifecycle feedback,
   and broader language-agnostic project signals.

Website P0 is the first implementation slice. Product work that is required to
make a claim truthful blocks publishing that claim, but does not require the
entire product backlog to precede website work.

## Definition Of Done

- The imported roadmap is preserved and retrievable from the chapter links;
  only required chapter-storage frontmatter differs from the Drive source.
- Every public exact command, benchmark number, and terminal example is
  generated from or tested against the promoted release.
- A cold visitor can identify the problem, product output, differentiation,
  start path, and Review-versus-Check distinction within ten seconds.
- The site has no horizontal overflow at the six target widths.
- Current, experimental, and planned capabilities are distinguishable.
- Cold and warm performance claims are separate and link to methodology.
- All marketing routes meet the metadata, canonical, structured-data,
  accessibility, and production-build gates defined in the source plan.
- The monthly measurement cadence has an owner and a first baseline report.

## Validation Commands

Run the checks appropriate to each delivered slice, including:

```bash
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
pnpm --dir website build
git diff --check
```

Website P0 additionally requires browser and accessibility checks at 360, 390,
430, 768, 1024, and 1440 pixel widths. Performance claims additionally require
the current performance no-slower and native no-regression gates named in the
imported plan.

## Reviewer Blocking Criteria

Block a milestone if it publishes unsupported commands or integrations,
hardcodes stale benchmark values, conflates Review with Check, hides inactive
capabilities behind a passing result, introduces unbounded output, regresses
mobile overflow or accessibility, or reports search rankings as guaranteed
delivery outcomes.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-10 | Imported the Drive roadmap verbatim, split it at natural section boundaries to satisfy document-size policy, and added measurable phase outcomes. | Drive file `1WjFntl5Bitormyb_PLjpNo7ANyfG81kC`; source SHA-256 `463a686202e1e6c48e06c272594555c55603ab35e909c1b53b2022c816e684f8` |
| 2026-07-10 | Iteration 1: revalidated the roadmap against pull request 140 and the live website, CLI, performance evidence, and CI state. Result: valid. P0 remains open because command examples, generated demo evidence, hero hierarchy, reusable components, performance proof, full visual/accessibility evidence, README truth, and the performance artifact job are incomplete. Context level: not exposed. | `docs/analysis/2026-07-10-website-roadmap-revalidation.md`; pull request 140; failed Performance Report job 86264104581 |
| 2026-07-10 | Iteration 2: completed the local Website P0 implementation and proof matrix. Product examples now come from deterministic CLI runs, the hero prioritizes Review with an accessible agent-setup dialog, shared marketing components replace the monolithic page, current cold and warm evidence is rendered, and the six-width light/dark browser matrix passes. Production Lighthouse scores 100 in all four categories. Hosted CI and the refreshed Workers preview remain the final P0 release proof. | `docs/analysis/2026-07-10-website-p0-evidence.md`; 19 Playwright tests; `/tmp/assura-p0-lighthouse-production.json`; `target/performance/website-roadmap-local.json` |
| 2026-07-10 | Iteration 3: implemented the local Website P1 foundation. Astro now emits production canonicals, robots and sitemap discovery; five focused marketing routes have unique metadata, structured data, social images, accessible responsive layouts, internal links, and named CTA events. The milestone remains open for hosted preview noindex proof and Search Console/Bing submission. | `docs/analysis/2026-07-10-website-p1-evidence.md`; 22 Playwright tests; 44-page static build |
| 2026-07-10 | Hosted Website P0 proof completed. All pull-request checks passed, including Performance Report and Verify Marketing Website, and Cloudflare published commit and branch previews. | Pull request 140 at commit `dc253df`; `https://926791e0-assura.nlr06886.workers.dev`; `https://codex-assura-landing-experience-assura.nlr06886.workers.dev` |
