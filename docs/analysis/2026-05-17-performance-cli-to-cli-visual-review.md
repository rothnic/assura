---
title: Performance CLI-to-CLI Visual Review
date: 2026-05-17
status: current
---

# Performance CLI-to-CLI Visual Review

## Target

- URL: `http://127.0.0.1:4321/reference/performance/`
- Build command: `cd website && pnpm build`
- Preview command: `cd website && pnpm preview --host 127.0.0.1 --port 4321`
- Data source: `website/public/data/performance/current.json`
- Capture command: headless Google Chrome screenshot against the local preview

## Screenshots

- Desktop viewport, 1440px wide:
  `website/public/data/performance/2026-05-17-performance-cli-to-cli-desktop.png`
- Mobile viewport, 390px wide:
  `website/public/data/performance/2026-05-17-performance-cli-to-cli-mobile.png`

## Review Notes

- Desktop and mobile pages render the generated performance summary without
  broken MDX syntax or raw import text.
- The headline uses `assura-cli` and `ls-lint-cli` rows from the current
  machine-readable report, not manually transcribed in-process Assura rows.
- The rendered headline shows 84.8% lower total CLI runtime, 6.58x total
  runtime speedup, and `rule_heavy_repo` as the weakest realistic row at 82.7%
  lower runtime.
- The realistic comparison table shows fixture id, scale, rule surface, Assura
  CLI median runtime, LS-Lint CLI median runtime, percent lower runtime,
  speedup, config references, and row status.
- The realistic and synthetic tables fit in the content column on desktop and
  transform into stacked rows on mobile, so core columns are not hidden by
  horizontal clipping.
- A mobile overflow issue in the generated evidence section was found during
  the final screenshot pass and fixed by clamping the evidence component width
  and stacking table cell labels/values on narrow viewports.
- The page communicates the Assura-versus-LS-Lint benefit from the rendered
  summary and table without requiring a reader to inspect raw JSON.
