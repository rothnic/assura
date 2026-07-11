---
title: Website P1 verification evidence
status: current
date: 2026-07-10
goal: ../goals/assura-website-landing-seo-roadmap.md
pull_request: https://github.com/rothnic/assura/pull/140
---

# Website P1 Verification Evidence

## Result

The reusable SEO and focused-page implementation is complete. The hosted
branch preview returns `x-robots-tag: noindex`; Google Search Console/Bing
Webmaster Tools submission remains an account-owner publication step.

## Route Matrix

| Route | Search intent | Structured data | Social image |
| --- | --- | --- | --- |
| `/` | Early warning signals for AI coding agents | `SoftwareApplication`, `SoftwareSourceCode` | `social/home.webp` |
| `/compare/ls-lint/` | LS-Lint alternative | `TechArticle`, `BreadcrumbList` | `social/ls-lint-comparison.webp` |
| `/performance/` | Fast repository validation | `TechArticle` | `social/performance.webp` |
| `/ai-coding-agent-guardrails/` | Coding-agent guardrails | `TechArticle` | `social/agent-guardrails.webp` |
| `/about/` | Product and creator provenance | `ProfilePage`, `Person` | `social/about.webp` |

Every route has a unique title, description, H1, self-referential production
canonical, Open Graph/Twitter image, and parseable JSON-LD payload. The five
social images are 1200 by 630 WebP assets derived from the generated Assura
visual system.

## Crawl And Measurement

- Astro now has the canonical `https://assura.dev` site value and emits
  `sitemap-index.xml`.
- `/robots.txt` allows production crawling and references the production
  sitemap.
- Every marketing page advertises the sitemap and retains a production
  canonical.
- The browser applies `noindex, nofollow` on `*.workers.dev` preview hosts.
- High-value CTA elements emit stable `assura:cta` events and call Cloudflare
  Zaraz when it is available, without introducing a required analytics runtime.

## Browser And Build Evidence

`pnpm --dir website test:marketing` passed 22 tests. In addition to the P0
matrix, the suite checks all P1 metadata, structured data, mobile overflow, axe
accessibility, robots discovery, sitemap discovery, and named CTA events.

`pnpm --dir website build` generated 44 static pages, `robots.txt`, and
`sitemap-index.xml` without the prior missing-site warning. Assura self-check
also passed with 1,599 files and 363 directories checked.

## Open External Proof

- Verified on 2026-07-10:
  `https://codex-assura-landing-experience-assura.nlr06886.workers.dev/project-review/`
  returned HTTP 200 with `x-robots-tag: noindex`.
- Submit `https://assura.dev/sitemap-index.xml` to Google Search Console and
  Bing Webmaster Tools using the site-owner accounts.
- Record the accepted submission states or validation errors in this document.
