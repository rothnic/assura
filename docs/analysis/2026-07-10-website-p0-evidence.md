---
title: Website P0 verification evidence
status: current
date: 2026-07-10
goal: ../goals/assura-website-landing-seo-roadmap.md
pull_request: https://github.com/rothnic/assura/pull/140
---

# Website P0 Verification Evidence

## Result

Website P0 passes its local and hosted proof gates. Pull request 140 completed
every required check, including the strict Performance Report and marketing
browser verification, and Cloudflare published the refreshed preview.

- Branch preview: https://codex-assura-landing-experience-assura.nlr06886.workers.dev
- Commit preview: https://926791e0-assura.nlr06886.workers.dev

## Product Truth

- `cargo xtask website-demo-data` builds the full CLI, creates a deterministic
  Git fixture, runs actual Onboard, Review, and Check commands, and writes the
  sanitized website JSON.
- `website/src/data/claims.yml` smoke-tests the exact supported and
  experimental public commands and their expected exit codes.
- `cargo xtask website-demo-data --check` proves the checked-in examples and
  promoted-release performance summary are current.
- Website source validation rejects unsupported explicit-base and path-scoped
  Review examples.

## Browser Matrix

`pnpm --dir website test:marketing` passed 19 tests in Chromium.

- Light and dark screenshots were captured at 360, 390, 430, 768, 1024, and
  1440 pixels.
- Every target width reported zero horizontal overflow.
- A 360 by 640 viewport retained the primary project-review artifact.
- The setup dialog passed keyboard dismissal, initial focus, focus restoration,
  and no-JavaScript destination checks.
- Axe reported zero violations for the landing page and open setup dialog.
- Reduced-motion mode disables smooth scrolling.
- The technical docs route remained reachable.

CI uploads the Playwright report and screenshot matrix as the
`marketing-visual-verification` artifact.

## Production Lighthouse

The built static site was audited from a production preview at
`http://127.0.0.1:4323/`.

| Category | Score |
| --- | ---: |
| Performance | 100 |
| Accessibility | 100 |
| Best Practices | 100 |
| SEO | 100 |

The same run measured 0.9 seconds for first and largest contentful paint, zero
cumulative layout shift, and zero milliseconds total blocking time. The local
JSON report is `/tmp/assura-p0-lighthouse-production.json`.

## Repository Gates

The following commands passed after the final browser fixes:

```bash
cargo fmt --all
cargo xtask website-demo-data
cargo test -p xtask website_demo --quiet
scripts/check-ci-scope.sh
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask evidence
cargo xtask docs
pnpm --dir website test:marketing
git diff --check
```

## Performance Evidence

The homepage renders version 0.3.0 evidence from the checked performance
report. It keeps cold and warm-session results separate: all eight accepted
cold fixtures are faster in the checked report, and all eight warm fixtures
meet the 2x target.

A fresh local release run at
`target/performance/website-roadmap-local.json` also passed the strict
no-slower gate without weakening its threshold. The prior pull-request failure
was a 0.17 millisecond cold-run miss on one fixture. The fresh hosted strict
Performance Report passed without weakening its threshold.

## Independent Review

The read-only review identified command-smoke, generated-output provenance,
experimental-status, accessibility, short-mobile, no-JavaScript, dark-contrast,
CI-scope, and nested-card gaps. The implementation now covers each finding with
generated evidence or browser regression tests. The remaining review note is
that screenshot evidence is artifact-based rather than pixel-diff-baseline
based; that is acceptable for P0 but remains available as later hardening.
