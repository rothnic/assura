---
title: Website Visual Review
date: 2026-05-16
status: current
---

# Website Visual Review

This note records the PR #11 website review after the performance evidence and
interpretation updates were considered complete.

## Scope

The review focused on the website routes affected by the performance work and
on representative pages that previously rendered Starlight component imports as
visible prose.

Reviewed routes:

- `/reference/performance/`
- `/introduction/`
- `/why-assura/`
- `/guides/quickstart/`
- `/examples/ci-cd-integration/`

## Findings

The initial local preview showed visible import text such as:

```text
import { Card, CardGrid, Aside, LinkButton } from '@astrojs/starlight/components';
```

Root cause: docs pages used Starlight component imports and JSX in files that
were rendered as Markdown prose, so the imports were displayed as paragraphs.

Fix: convert those pages to plain Markdown equivalents for this PR. This keeps
the rendered documentation correct without changing the site's MDX integration
surface.

## Performance Page Review

The first pass still made the benefit difficult to parse because the page was
mostly prose. A second pass made the headline more visible but used separated
comparison bars and a history progression graphic that was still too hard to
defend. The final follow-up makes the page evidence-table-first:

- headline metric: Assura completes the comparable realistic bundle with 94.2%
  lower total runtime than warm LS-Lint;
- summary cards: bundle reduction, bundle speedup, weakest realistic win, and
  the synthetic caveat;
- evidence table: each fixture includes what it models, fixture scale, rule
  surface, generator/manifest links, Assura timing, LS-Lint timing, runtime
  reduction, and speedup;
- fairness contract: explains same materialized tree, converted config source,
  warm LS-Lint CLI timing, and Assura top-level timing scope;
- synthetic section: stress fixtures that should not support the product claim;
- history section: explicitly reframed as an audit log, not a cross-machine
  progression chart.

The realistic equivalent rows show Assura multiple times faster than warm
LS-Lint 2.3 on the same generated project shapes:

| Fixture | Assura ms | LS-Lint ms | Runtime reduction | Speedup |
| --- | ---: | ---: | ---: | ---: |
| `simple_library` | 0.843 | 104.185 | 99.2% | 123.5x |
| `web_app` | 0.801 | 101.060 | 99.2% | 126.2x |
| `monorepo_packages` | 1.669 | 104.973 | 98.4% | 62.9x |
| `rule_heavy_repo` | 26.145 | 99.177 | 73.6% | 3.8x |
| `ignored_generated_heavy_repo` | 0.542 | 108.457 | 99.5% | 200.3x |

The page also separates synthetic stress fixtures from realistic equivalent
fixtures so `rule_heavy` does not get used as evidence for the faster-than
LS-Lint product claim. `rule_heavy` remains documented as a synthetic stress
case where LS-Lint is faster.

The history section no longer visualizes cross-machine speedup progression.
It states that JSONL history is an audit log until report shape, environment,
and LS-Lint setup are normalized enough for a true trend chart.

## Verification

Commands:

```bash
cd website && pnpm build
cd website && pnpm preview --host 127.0.0.1 --port 4321
agent-browser --session assura-pr11-visual set viewport 1440 1000
agent-browser --session assura-pr11-visual open http://127.0.0.1:4321/reference/performance/
agent-browser --session assura-pr11-visual screenshot target/website-visual-review/performance-desktop.png --full
agent-browser --session assura-pr11-visual set viewport 390 844
agent-browser --session assura-pr11-visual open http://127.0.0.1:4321/reference/performance/
agent-browser --session assura-pr11-visual screenshot target/website-visual-review/performance-mobile.png --full
agent-browser --session assura-perf-page-redesign set viewport 1440 1000
agent-browser --session assura-perf-page-redesign open http://127.0.0.1:4321/reference/performance/
agent-browser --session assura-perf-page-redesign screenshot target/website-visual-review/performance-redesign-desktop.png --full
agent-browser --session assura-perf-page-redesign set viewport 390 844
agent-browser --session assura-perf-page-redesign open http://127.0.0.1:4321/reference/performance/
agent-browser --session assura-perf-page-redesign screenshot target/website-visual-review/performance-redesign-mobile.png --full
agent-browser --session assura-perf-evidence-table set viewport 1440 1000
agent-browser --session assura-perf-evidence-table open http://127.0.0.1:4321/reference/performance/
agent-browser --session assura-perf-evidence-table screenshot target/website-visual-review/performance-evidence-table-desktop.png --full
agent-browser --session assura-perf-evidence-table set viewport 390 844
agent-browser --session assura-perf-evidence-table open http://127.0.0.1:4321/reference/performance/
agent-browser --session assura-perf-evidence-table screenshot target/website-visual-review/performance-evidence-table-mobile.png --full
```

Post-fix checks:

- the performance page renders the headline reduction, summary cards,
  apples-to-apples evidence table, fairness contract, synthetic caveat section,
  and audit-log history section;
- no visible `@astrojs/starlight/components` import text appears in built HTML;
- representative docs pages no longer expose component imports or JSX tags as
  visible prose;
- desktop and mobile screenshots show readable content without obvious overlap
  in the reviewed first-pass routes.
