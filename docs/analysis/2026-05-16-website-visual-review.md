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

The performance page now presents the realistic equivalent fixtures directly on
the page instead of requiring reviewers to open raw JSON to validate the claim.
The rows are rendered as compact Markdown list items so the numbers remain
visible on narrow mobile viewports.

The realistic equivalent rows show Assura multiple times faster than warm
LS-Lint 2.3 on the same generated project shapes:

| Fixture | Assura ms | LS-Lint ms | Speedup |
| --- | ---: | ---: | ---: |
| `simple_library` | 0.843 | 104.185 | 123.5x |
| `web_app` | 0.801 | 101.060 | 126.2x |
| `monorepo_packages` | 1.669 | 104.973 | 62.9x |
| `rule_heavy_repo` | 26.145 | 99.177 | 3.8x |
| `ignored_generated_heavy_repo` | 0.542 | 108.457 | 200.3x |

The page also separates synthetic stress fixtures from realistic equivalent
fixtures so `rule_heavy` does not get used as evidence for the faster-than
LS-Lint product claim. `rule_heavy` remains documented as a synthetic stress
case where LS-Lint is faster.

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
```

Post-fix checks:

- the performance page renders the realistic fixture speedup rows;
- no visible `@astrojs/starlight/components` import text appears in built HTML;
- representative docs pages no longer expose component imports or JSX tags as
  visible prose;
- desktop and mobile screenshots show readable content without obvious overlap
  in the reviewed first-pass routes.
