---
title: Assura website roadmap - install, pages, and SEO
source_drive_file_id: 1WjFntl5Bitormyb_PLjpNo7ANyfG81kC
source_part: 2
---

# 5. Fast-install UX without hero clutter

## 5.1 The problem with the current implementation

The expanded prompt and manual command consume more than a mobile viewport before the visitor has seen the product signal.

This makes the page feel like setup documentation rather than a product explanation.

## 5.2 Target interaction

### Desktop

- `Start with your agent` opens a centered dialog.
- The dialog has tabs:
  - Agent prompt
  - Manual install
- Copy actions provide visible success feedback.
- Escape closes the dialog.
- Focus is trapped correctly.
- The underlying page does not jump.

### Mobile

- Open a bottom sheet.
- Keep the title and copy actions above the fold.
- Allow the full prompt to scroll inside the sheet.
- Keep a sticky copy button at the bottom.
- Use a close control large enough for touch.

### No-JavaScript fallback

The CTA should link to `#onboard` if JavaScript is unavailable, where the full prompt remains visible.

## 5.3 Prompt content

Keep the default prompt short:

```text
Install and onboard Assura for this repository.

Use the supported local installer if Assura is missing, then run:

assura agent onboard . --agent auto --format json
assura review

Read .assura/onboarding/agent-next.md and summarize:
- what is active,
- what is inactive,
- which project-specific choices need human answers.

Do not invent language, structure, naming, or domain conventions.
```

The manual shell command can remain a separate copy target.

---

# 6. Second-level marketing pages

Technical Starlight documentation should support the marketing pages, not substitute for them.

Each marketing page should reuse the current landing-page visual language, answer one search intent, show a product artifact, and link into the relevant technical docs.

## Priority 0 pages

### 6.1 `/compare/ls-lint/`

**Purpose**

Capture the clearest existing alternative and migration intent.

**Primary search themes**

- LS-Lint alternative
- LS-Lint replacement
- repository naming linter
- file and folder naming validation
- faster repository structure checks

**Page structure**

1. What LS-Lint does well.
2. Where Assura starts with parity.
3. Where Assura expands beyond naming.
4. Cold comparison.
5. Warm persistent-session comparison.
6. Config migration example.
7. Support and compatibility boundaries.
8. CTA to migrate.

**Recommended title**

> Assura vs LS-Lint: Repository Validation Beyond Naming

**Recommended H1**

> A faster path from LS-Lint-style checks to agent-ready project validation.

Do not claim a universal cold 2x win. Generate current numbers from evidence.

### 6.2 `/performance/`

**Purpose**

Turn extensive benchmark work into a clean, understandable proof page.

**Primary search themes**

- Assura performance
- fast repository linter
- Rust repository validator
- LS-Lint benchmark
- incremental repository validation
- warm daemon validation

**Page structure**

Use the performance treatment defined earlier in this document.

Link to:

- `/reference/performance/`
- `/reference/performance-test-cases/`
- `/reference/performance-implementation/`
- raw current JSON.

### 6.3 `/ai-coding-agent-guardrails/`

**Purpose**

Own the emerging category-level problem rather than only the product name.

**Primary search themes**

- AI coding agent guardrails
- coding agent quality checks
- AI code quality workflow
- agent hooks
- Codex project guardrails
- Claude Code project rules
- AI-assisted development quality

**Page structure**

1. Why late gates cause bad repairs.
2. Watch / Warn / Gate.
3. Bounded nudges.
4. Branch and worktree context.
5. Supported harnesses.
6. Human decisions versus automated rules.
7. Product example.
8. CTA to onboard.

### 6.4 `/about/`

**Purpose**

Establish product provenance, open-source trust, and Nick Roth’s authorship.

**Primary search themes**

- Assura open source
- Nick Roth Assura
- Nick Roth AI engineer
- open-source AI developer tools

**Page structure**

1. Why Assura exists.
2. Product principles.
3. Performance and correctness discipline.
4. Roadmap philosophy.
5. Created by Nick Roth.
6. Links to personal site, GitHub profile, repository, and relevant case study.

Use visible authorship, not footer-only attribution.

## Priority 1 pages

### 6.5 `/project-review/`

**Purpose**

Explain the flagship review experience.

**Search themes**

- repository health check
- project health CLI
- branch-aware code quality
- repository heat map
- codebase hot spots
- AI project review

**Product artifact**

A full real review output, plus `review → explain → check`.

### 6.6 `/agent-onboarding/`

**Purpose**

Convert users who want the one-command setup path.

**Search themes**

- agent-ready repository setup
- AGENTS.md setup
- coding agent project template
- Codex project setup
- Claude Code repository setup
- local coding agent hooks

**Page structure**

1. Copyable prompt.
2. What it creates.
3. What it preserves.
4. What it refuses to guess.
5. Harness integration choices.
6. Active versus inactive.
7. Technical guide link.

### 6.7 `/repository-validation/`

**Purpose**

Own the broad, stable category that begins the product stack.

**Search themes**

- repository structure validation
- folder structure linter
- file naming convention checker
- repository policy as code
- monorepo structure validation
- required files checker

### 6.8 `/project-intelligence/`

**Purpose**

Explain the longer-term queryable context vision without confusing it with the current first-run value.

**Search themes**

- project intelligence CLI
- local repository knowledge graph
- codebase context for AI agents
- repo-native project data
- local semantic project search

Clearly distinguish supported, experimental, and planned layers.

## Priority 2 pages and content

### 6.9 `/examples/`

Create a designed example index linking to:

- agent-ready software project;
- document/research project;
- LS-Lint migration;
- monorepo;
- Markdown-heavy repository;
- requirements and evidence traceability.

### 6.10 Engineering articles or case studies

High-value topics:

- Why AI coding agents need early project-level feedback.
- Watch, warn, and gate: a better lifecycle for agent quality.
- Assura versus LS-Lint benchmark methodology.
- Building a persistent Rust validation session.
- Correct incremental validation without stale passes.
- Quality signals for Git worktrees.
- Validating `AGENTS.md` and project-local skills.
- Repository structure linting for monorepos.
- Why a clean check is not complete onboarding.
- Dogfooding Assura on a document-generation project.

These pages should contain first-hand evidence and implementation details, not generic AI-generated summaries.

---

# 7. Marketing pages versus documentation

## Marketing pages

Use custom Astro pages for:

- category education;
- problem/solution narratives;
- visual product explanation;
- comparisons;
- performance summaries;
- use cases;
- author/project trust;
- conversion CTAs.

## Documentation pages

Keep Starlight for:

- exact installation steps;
- command reference;
- configuration schema;
- rule reference;
- support status;
- integration lifecycle;
- benchmark methodology;
- raw evidence;
- troubleshooting;
- migration details;
- current limitations.

## Avoid duplicate-content overlap

Each marketing page should summarize and link to documentation.

Do not copy an entire technical guide into a marketing page.

Example:

```text
/agent-onboarding/
  Explains why, outcome, and flow.

/guides/agent-ready-onboarding/
  Explains every command, generated file, option, and failure mode.
```

Use self-referential canonical URLs on both because the pages serve distinct intents.

---

# 8. SEO architecture

No technical change guarantees a first-place Google ranking. The goal is to make the site easy to crawl, clearly organized, genuinely useful, internally connected, and authoritative around a focused topic.

## 8.1 Keyword and page map

| Route | Primary intent | Supporting terms |
|---|---|---|
| `/` | Early warning signals for AI coding agents | project drift, repository validation CLI, AI-assisted development quality |
| `/compare/ls-lint/` | LS-Lint alternative | LS-Lint replacement, file naming linter, folder naming checker |
| `/performance/` | Fast and incremental repository validation | Rust linter performance, LS-Lint benchmark, warm daemon |
| `/ai-coding-agent-guardrails/` | Coding-agent guardrails | agent hooks, AI code quality, watch warn gate |
| `/project-review/` | Project health review | repository heat map, branch-aware review, codebase hot spots |
| `/agent-onboarding/` | Agent-ready project setup | AGENTS.md setup, Codex setup, Claude Code setup |
| `/repository-validation/` | Repository structure validation | naming conventions, required files, monorepo policy |
| `/project-intelligence/` | Local project intelligence | repository knowledge graph, codebase context, project facts |
| `/about/` | Product and creator provenance | Nick Roth Assura, open-source developer tooling |

Do not force every keyword into the homepage. Give each meaningful intent one strong page.

## 8.2 Titles and descriptions

Every page must have:

- one unique, concise `<title>`;
- one clear H1;
- a human-readable meta description;
- matching Open Graph and Twitter metadata;
- a crawlable canonical URL;
- a purpose-specific social image.

Example homepage metadata:

```html
<title>Assura – Early Warning Signals for AI Coding Agents</title>
<meta
  name="description"
  content="Assura is a fast local repository validation CLI that shows AI coding agents project drift before hooks, CI, or review force a large cleanup."
/>
```

The visible H1 can remain:

> Catch project drift before review.

The title and H1 do not need to be identical, but they must describe the same page.

## 8.3 Internal linking

Build a deliberate hub-and-spoke structure.

The homepage should link to:

- Project Review
- AI Coding Agent Guardrails
- LS-Lint Comparison
- Performance
- Agent Onboarding
- Documentation
- About

Each second-level page should link to:

- two adjacent marketing pages;
- one or more relevant technical docs;
- GitHub only where source or issue participation is relevant;
- the primary onboarding CTA.

Use descriptive anchor text, not repeated “learn more.”

## 8.4 Sitemap and crawl infrastructure

The current Astro site does not show `@astrojs/sitemap` in `website/package.json` and does not define the deployed `site` URL in `astro.config.mjs`.

Implement:

```js
import sitemap from '@astrojs/sitemap';

export default defineConfig({
  site: 'https://assura.dev',
  integrations: [
    sitemap(),
    starlight(/* ... */),
  ],
});
```

Add:

```text
website/src/pages/robots.txt.ts
```

with:

```text
User-agent: *
Allow: /
Sitemap: https://assura.dev/sitemap-index.xml
```

Also add:

```html
<link rel="sitemap" href="/sitemap-index.xml" />
```

Submit the sitemap in Google Search Console and Bing Webmaster Tools.

## 8.5 Preview environment indexing

Preview deployments such as `*.workers.dev` must not compete with `assura.dev`.

For previews:

- send `X-Robots-Tag: noindex, nofollow`;
- or render a `noindex` robots meta tag based on deployment environment;
- retain canonical URLs pointing at production;
- avoid including previews in sitemaps.

## 8.6 Canonicalization

Ensure:

- `https://assura.dev/` is the one preferred homepage;
- HTTP redirects to HTTPS;
- any `www` host redirects consistently to the chosen host;
- trailing-slash behavior is consistent;
- every indexable page has a self-referential canonical;
- query-string preview URLs do not become canonical variants.

## 8.7 Structured data

Add JSON-LD through a reusable component.

### Homepage

Use:

- `WebSite`;
- `SoftwareApplication` and/or `SoftwareSourceCode`.

Suggested properties:

```json
{
  "@context": "https://schema.org",
  "@type": ["SoftwareApplication", "SoftwareSourceCode"],
  "name": "Assura",
  "applicationCategory": "DeveloperApplication",
  "operatingSystem": "Linux, macOS, Windows",
  "description": "Fast local repository validation and early warning signals for AI coding agents.",
  "url": "https://assura.dev/",
  "codeRepository": "https://github.com/rothnic/assura",
  "downloadUrl": "https://github.com/rothnic/assura/releases/latest",
  "license": [
    "https://opensource.org/license/mit",
    "https://www.apache.org/licenses/LICENSE-2.0"
  ],
  "author": {
    "@type": "Person",
    "name": "Nick Roth",
    "url": "https://www.nickroth.com/"
  },
  "offers": {
    "@type": "Offer",
    "price": 0,
    "priceCurrency": "USD"
  }
}
```

Do not fabricate ratings or reviews. Google’s Software App rich-result rules require a genuine rating or review in addition to the app name and offer price. Schema can still help describe the entity without guaranteeing a rich result.

### About page

Use `ProfilePage` with `Person`:

```json
{
  "@context": "https://schema.org",
  "@type": "ProfilePage",
  "mainEntity": {
    "@type": "Person",
    "name": "Nick Roth",
    "alternateName": "rothnic",
    "url": "https://www.nickroth.com/",
    "sameAs": [
      "https://github.com/rothnic",
      "https://www.linkedin.com/in/nicholasleeroth/"
    ]
  }
}
```

### Second-level pages

Use:

- `BreadcrumbList`;
- `TechArticle` for benchmark or engineering articles;
- visible author byline linked to the author page or personal site.

## 8.8 Page experience

Continue the current static Astro approach.

Protect:

- good mobile layout;
- low JavaScript cost;
- no intrusive install interstitial;
- stable image dimensions;
- optimized WebP/AVIF assets;
- accessible dialogs;
- visible focus states;
- reduced-motion support;
- no layout shift when fonts or images load.

Track Core Web Vitals, but do not optimize solely for a perfect score at the expense of usefulness.

## 8.9 Search measurement

Configure:

- Google Search Console for `assura.dev`;
- Google Search Console for `www.nickroth.com`;
- Bing Webmaster Tools;
- a privacy-conscious analytics product;
- event tracking for:
  - setup-prompt copy;
  - install-command copy;
  - GitHub click;
  - docs click;
  - performance methodology click;
  - onboarding completion.

Review monthly:

- indexed pages;
- search queries;
- page impressions;
- click-through rate;
- branded versus non-branded queries;
- pages with high impressions and low CTR;
- queries where Assura appears on page two;
- backlinks;
- Core Web Vitals;
- crawl and canonical errors.

---
