---
title: Assura website search and performance scorecard - July 2026
status: baseline
date: 2026-07-10
owner: assura-maintainers
goal: ../goals/assura-website-landing-seo-roadmap.md
---

# July 2026 Website Scorecard

## Baseline

This is the first monthly operating baseline. Search-console fields are marked
unavailable until site ownership and sitemap submission are completed; they are
not inferred from public search-result sampling.

| Signal | July baseline | Evidence |
| --- | ---: | --- |
| Production homepage | HTTP 200 | `curl -L https://assura.dev/` on 2026-07-10 |
| Production robots | HTTP 200, current Cloudflare policy | `curl -L https://assura.dev/robots.txt` |
| Production sitemap | HTTP 404 before roadmap merge | `curl -L https://assura.dev/sitemap-index.xml` |
| Public `site:assura.dev` sample | No Assura result returned | Public search sample on 2026-07-10; not an index-count substitute |
| Indexed pages | Unavailable | Google Search Console not yet connected |
| Impressions | Unavailable | Google Search Console not yet connected |
| Clicks | Unavailable | Google Search Console not yet connected |
| Click-through rate | Unavailable | Google Search Console not yet connected |
| Branded/non-branded query mix | Unavailable | Google Search Console not yet connected |
| Page-two queries | Unavailable | Google Search Console not yet connected |
| Backlinks | Unavailable | No backlink data source connected |
| Field Core Web Vitals | Unavailable | Insufficient field data/account access |
| Production lab Lighthouse | 100/100/100/100 on the roadmap build | `docs/analysis/2026-07-10-website-p0-evidence.md` |
| Crawl/canonical errors | Unavailable | Webmaster tools not yet connected |

## July Actions

| Action | Owner | Due | Completion evidence |
| --- | --- | --- | --- |
| Merge and deploy pull request 140 | assura-maintainers | 2026-07-13 | Production routes and sitemap return HTTP 200 |
| Verify `*.workers.dev` previews are noindex | assura-maintainers | 2026-07-13 | Browser metadata capture from branch preview |
| Add `assura.dev` to Google Search Console and submit sitemap | Nick Roth | 2026-07-17 | Accepted property and sitemap status capture |
| Add `assura.dev` to Bing Webmaster Tools and submit sitemap | Nick Roth | 2026-07-17 | Accepted site and sitemap status capture |
| Merge and deploy personal-site pull request 60 | Nick Roth | 2026-07-17 | Live personal case-study URL |
| Choose one current GitHub profile pin to replace with Assura | Nick Roth | 2026-07-17 | Assura visible in profile pinned repositories |
| Capture first account-backed search baseline | assura-maintainers | 2026-08-10 | August scorecard with index, query, CTR, and crawl fields |

## Next Review

The next review is due 2026-08-10. It must retain unavailable fields rather
than omitting them and assign an owner and due date to every accepted action.
