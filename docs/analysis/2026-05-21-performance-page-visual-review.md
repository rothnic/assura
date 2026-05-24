---
title: Performance Page Visual Review
date: 2026-05-21
---

# Performance Page Visual Review - 2026-05-21

## Scope

Reviewed the built website performance page after updating the PR #11
performance claim language and evidence component.

Page reviewed:

- `/reference/performance/`

## Evidence

Build command:

```bash
ASTRO_TELEMETRY_DISABLED=1 ./node_modules/.bin/astro build
```

Local static review:

```bash
python3 -m http.server 4173 --bind 127.0.0.1
```

Screenshots captured with headless Chrome:

- `/private/tmp/assura-performance-desktop-http.png`
- `/private/tmp/assura-performance-mobile-http-v3.png`

## Findings

- Desktop layout is aligned: the summary card, run-context rail, and test-case
  table line up inside the Starlight content column.
- Mobile layout initially clipped long prose at the right edge. The performance
  wrappers now use viewport-bounded inline sizing, and prose in the summary and
  section intro is constrained to a readable line length on mobile.
- The page headline and summary now scope the 2x claim to the Linux static-CRT
  release artifact and keep warm/editor-session evidence separate.
- The evidence component now prefers `assura-check-cli` rows for the headline
  comparison and displays build profile plus platform context.

## Residual Risk

The screenshot evidence is local and transient under `/private/tmp`. Re-run the
commands above before publishing if another CSS or Starlight dependency change
lands before PR merge.
