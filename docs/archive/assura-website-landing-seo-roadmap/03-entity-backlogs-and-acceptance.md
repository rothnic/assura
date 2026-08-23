---
title: Assura website roadmap - entity, backlogs, and acceptance
source_drive_file_id: 1WjFntl5Bitormyb_PLjpNo7ANyfG81kC
source_part: 3
---


# 9. Nick Roth entity and career-value strategy

The product site should help establish Nick Roth as the identifiable creator, but the personal site should remain the canonical personal entity.

## 9.1 Site-wide attribution

Use visible text:

> Created by Nick Roth

Link “Nick Roth” to:

- `https://www.nickroth.com/`

Also link to:

- `https://github.com/rothnic`
- `https://github.com/rothnic/assura`

## 9.2 Dedicated About page

The About page should establish:

- Nick’s role as product manager and AI engineer;
- the problem he identified;
- product and architecture decisions;
- performance discipline;
- open-source ownership;
- links to his personal site, GitHub profile, LinkedIn, and the repository.

## 9.3 Personal site work

Add a dedicated Assura case study to `www.nickroth.com`, such as:

```text
/work/assura/
```

Suggested title:

> Building Assura: Fast Repository Validation for AI Coding Agents | Nick Roth

The case study should cover:

- problem discovery;
- product positioning;
- benchmark methodology;
- progressive validation architecture;
- agent onboarding;
- dogfood lessons;
- landing-page iteration;
- screenshots;
- links back to Assura and GitHub.

Feature Assura in “Recent Work” on the personal homepage.

## 9.4 GitHub profile and repository

The public GitHub profile already links to `www.nickroth.com` and describes Nick as a product manager and engineer focused on AI-assisted building. Keep that wording aligned with the Assura About page.

Actions:

1. Pin `rothnic/assura` on the GitHub profile.
2. Update repository description to the current product message.
3. Set repository homepage to `https://assura.dev`.
4. Add topics:
   - `rust`
   - `cli`
   - `repository-validation`
   - `repository-linter`
   - `ai-agents`
   - `code-quality`
   - `developer-tools`
   - `ls-lint`
   - `markdown`
   - `project-intelligence`
5. Refresh the README.

## 9.5 Urgent README consistency fix

The current repository has a version-truth conflict:

- `Cargo.toml` says `0.3.0`;
- the support policy describes the `v0.3.0` beta surface;
- the README still says the current public release is `v0.1.0` and describes a much narrower product.

This must be fixed before a broader SEO or marketing push. Search engines, GitHub visitors, and AI tools frequently use README content as a primary source.

The README should use the same core model:

```text
Assura catches project drift while the fix is still small.

Onboard once. Review while working. Explain when needed. Check before merge.
```

Then distinguish supported and experimental commands truthfully.

---

# 10. Marketing truth and generated evidence

## 10.1 Do not hand-author terminal output

Create deterministic website fixtures:

```text
tests/fixtures/website_demo/
```

Generate marketing evidence with a repository task:

```bash
cargo xtask website-demo-data
```

Suggested outputs:

```text
website/src/data/review-demo.json
website/src/data/check-demo.json
website/src/data/onboarding-demo.json
website/src/data/performance-summary.json
```

The website renders these files.

CI reruns the generator and fails on a diff.

## 10.2 Add a claim manifest

Create:

```text
website/src/data/claims.yml
```

Example:

```yaml
claims:
  - id: cold-ls-lint-comparison
    copy: Faster on every accepted cold comparison in the current checked cohort.
    status: evidence-backed
    source: website/public/data/performance/current.json

  - id: warm-session-performance
    copy: Persistent warm sessions exceed the 2x target on every accepted case.
    status: evidence-backed
    source: website/public/data/performance/current.json

  - id: compact-review
    copy: Review project health with branch, worktree, validation, and content-gap context.
    status: experimental
    command: assura review

  - id: explicit-review-base
    copy: Select any comparison base.
    status: planned
    proposed_command: assura review --base origin/main

  - id: finding-history
    copy: See new, worsened, unchanged, and resolved findings.
    status: planned
```

The build should fail when public exact commands do not match the current CLI surface.

## 10.3 Product-status presentation

Use one compact status page or capability matrix:

```text
Supported
Experimental
Planned
```

Do not scatter “coming soon” badges across every marketing section. Keep the main narrative clean.

---

# 11. Prioritized website backlog

## P0 — Preserve design, correct truth, simplify the hero

1. Capture current desktop and mobile screenshots as visual regression references.
2. Extract current tokens into `marketing.css`.
3. Extract `MarketingLayout`, header, footer, section heading, terminal, and button components without intentional visual change.
4. Replace invalid `assura review --base auto` syntax.
5. Replace invalid `assura review --path ... --explain` syntax.
6. Remove unsupported output fields and opaque heat scores.
7. Move the full agent prompt into a dialog/bottom sheet.
8. Make the compact project review the hero visual.
9. Change hero CTAs to:
   - See a project review
   - Start with your agent
10. Add the four-command journey:
    - Onboard
    - Review
    - Explain
    - Check
11. Add visible author attribution and first-party profile links.
12. Refresh README and repository metadata.
13. Refresh performance evidence for the current release version.
14. Generate homepage benchmark facts from current JSON.
15. Add visual tests for 360, 390, 430, 768, 1024, and 1440 pixel widths.

## P1 — SEO and reusable marketing foundation

1. Add `site: 'https://assura.dev'` to Astro configuration.
2. Add `@astrojs/sitemap`.
3. Add dynamic `robots.txt`.
4. Add canonical metadata through `SeoHead`.
5. Add preview `noindex`.
6. Add JSON-LD.
7. Add unique Open Graph images.
8. Build:
   - `/compare/ls-lint/`
   - `/performance/`
   - `/ai-coding-agent-guardrails/`
   - `/about/`
9. Add marketing navigation and footer IA.
10. Connect marketing pages to technical docs.
11. Set up Search Console and Bing Webmaster Tools.
12. Add analytics events for high-value CTAs.

## P2 — Conversion and category ownership

1. Build:
   - `/project-review/`
   - `/agent-onboarding/`
   - `/repository-validation/`
   - `/project-intelligence/`
2. Add a designed examples index.
3. Publish the Assura case study on Nick’s personal website.
4. Pin Assura on GitHub.
5. Publish the first benchmark-methodology article.
6. Publish the dogfood case study.
7. Add changelog/release pages with unique title and canonical metadata.

## P3 — Ongoing content and evidence

1. Publish benchmark refreshes only when methodology and current release align.
2. Add real-project case studies.
3. Add worktree and incremental-validation engineering articles.
4. Add performance history visualization after the history is clean enough for release-over-release comparison.
5. Track rankings and revise pages based on Search Console rather than guessing.

---

# 12. Prioritized product backlog required to match the marketing

## P0 — Clarify command semantics

1. Make `review` advisory by default.
2. Keep `check` authoritative and blocking.
3. Document:
   - review is radar;
   - check is gate.
4. Stabilize review JSON for website, agents, and editors.
5. Add exact command smoke tests for every command copied by the website.

## P1 — Complete the review experience

1. Add explicit `--base <auto|ref>` while retaining automatic default behavior.
2. Add a compact tree renderer.
3. Use transparent per-signal values and thresholds.
4. Rank hot directories.
5. Add next-action routing to `explain`, `doctor`, `check`, and content commands.
6. Add stable finding fingerprints.
7. Add state:
   - new;
   - worsened;
   - unchanged;
   - resolved.
8. Separate:
   - repeated findings hidden;
   - generated/archive/log noise omitted.
9. Keep output bounded.

## P1 — Complete onboarding

1. Run review and verification inside `agent onboard`.
2. Generate a concise onboarding summary.
3. Detect supported harnesses.
4. Offer integration installation explicitly.
5. Ask questions rather than guessing.
6. Make reruns idempotent.
7. Add remote installer delegation only after local onboarding is stable.
8. Ensure generic onboarding remains useful without a host-specific integration.

## P2 — Warm loop and caching

1. Refresh warm-session benchmarks on the current release.
2. Define latency budgets:
   - no-change warm review;
   - one-file change;
   - directory create/delete;
   - config change;
   - agent nudge.
3. Persist finding fingerprints.
4. Add safe file-local result reuse.
5. Add directory fingerprints.
6. Add worktree-aware cache namespaces.
7. Share immutable cache data across worktrees where safe.
8. Add cache inspection and cleanup commands.
9. Report cache mode and fallback reason in diagnostic output.
10. Add no-change, small-change, rename/delete, and config-change benchmark rows.

## P2 — Lifecycle behavior

1. Map agent events to bounded nudges.
2. Add cooldowns for repeated messages.
3. Show approaching thresholds.
4. Keep advisory events exit-zero.
5. Route blocking decisions through `check`.
6. Add integration health checks.

## P3 — Broader universal signals

Prioritize:

1. naming and placement;
2. required and forbidden paths;
3. child and nesting pressure;
4. line and section thresholds;
5. generated-output boundaries;
6. branch/worktree churn;
7. Markdown and reference health;
8. agent-guidance contracts;
9. frontmatter references;
10. typed project records and missing relationships;
11. binary source-document custody;
12. requirements and evidence traceability;
13. bounded computed project-specific checks.

---

# 13. Landing-page acceptance criteria

## Visual

- Existing visual identity is recognizable.
- Hero no longer contains an expanded multi-screen setup prompt.
- One dominant project signal appears above the fold.
- No horizontal overflow at supported breakpoints.
- H1 and H2 remain editorial but do not consume an entire small-screen viewport.
- Secondary sections use fewer heavy cards.
- Setup dialog is accessible and polished.

## Content

A cold visitor can answer within ten seconds:

1. What problem does Assura solve?
2. What does it show?
3. Why is that better than current tools?
4. How do I start?
5. What is `review` versus `check`?

## Command truth

- Every displayed command runs against the promoted release.
- Review examples use `assura review`.
- Path investigation uses `assura explain <path>`.
- Merge validation uses `assura check`.
- Unsupported integrations are not named as supported.

## Performance truth

- Homepage values come from current JSON.
- Promoted release version matches benchmark version.
- Cold and warm claims are visibly separate.
- Environment and methodology are one click away.
- No universal cold 2x claim unless current evidence supports it.

## SEO

- All pages have unique titles, descriptions, H1s, canonicals, and social metadata.
- Sitemap exists and is submitted.
- Preview deployments are noindex.
- Structured data validates.
- Marketing pages link to docs.
- About page links visibly to Nick Roth’s personal site and GitHub profile.
- Personal site links back through an Assura case study.
- README and repository metadata match current product truth.

## Performance and accessibility

Suggested gates:

- Lighthouse Performance: 95 or better on a representative production build.
- Accessibility: 100 target.
- Best Practices: 95 or better.
- SEO: 100 target.
- Good Core Web Vitals in field data once traffic exists.
- Dialog keyboard and screen-reader behavior tested.
- Reduced-motion behavior tested.

These are engineering targets, not ranking guarantees.

---

# 14. Copy/paste implementation prompt for the agent

```text
Revise the Assura website according to
assura-website-landing-seo-roadmap.md.

Do not redesign the site from scratch.

Start by preserving the current rendered design:
- capture desktop and mobile screenshots;
- extract the current marketing tokens, layout, header, footer, section heading,
  terminal, and button patterns into reusable Astro components;
- do not add a frontend framework;
- keep Starlight for technical docs.

First complete P0 only.

Critical product truth:
- assura agent onboard establishes the agent-ready baseline;
- assura review is the compact project-health radar;
- assura explain investigates one path;
- assura check is the authoritative gate;
- review and check must not be described as synonyms.

Fix the landing page so it does not show unsupported syntax:
- do not show `assura review --base auto` until that option exists;
- do not show `assura review --path ... --explain`;
- use `assura review`, `assura explain <path>`, and `assura check`.

Preserve the current visual identity, but:
- remove the expanded agent prompt from the hero;
- put setup in an accessible modal or mobile bottom sheet;
- make a compact project-review tree the hero product artifact;
- add the Onboard → Review → Explain → Check journey;
- add a compact cold/warm performance proof strip generated from current
  performance JSON;
- separate cold CLI claims from persistent warm-session claims;
- remove opaque or unsupported heat scores and finding-history claims;
- add visible Nick Roth attribution and links to www.nickroth.com,
  github.com/rothnic, and github.com/rothnic/assura.

Before publishing benchmark values:
- confirm Cargo/release version matches the performance-report version;
- refresh the report if needed;
- run `cargo xtask performance-no-slower`;
- regenerate website performance data.

Add build-time or test-time verification that:
- copied commands are valid;
- marketing demo output is generated from deterministic fixtures;
- preview deployments are noindex;
- production canonicals use https://assura.dev;
- the site has a sitemap and robots.txt.

Run:
- Assura self-check;
- relevant Rust tests;
- website build;
- docs build;
- visual checks at 360, 390, 430, 768, 1024, and 1440 widths;
- accessibility checks;
- git diff --check.

Stop after P0 with:
1. screenshots;
2. a list of files changed;
3. commands run;
4. supported, experimental, and planned claims still visible on the page;
5. follow-up items for P1.
```

---

# 15. Repository evidence reviewed

Key repository sources:

- `src/cli/args.rs`
- `src/cli/agent_args.rs`
- `src/cli/project_review.rs`
- `src/cli/project_review/text.rs`
- `src/cli/project_review/heatmap/git.rs`
- `docs/support-policy.md`
- `docs/goals/assura-compact-project-review-common-issues.md`
- `docs/analysis/2026-07-02-ls-lint-performance-reassessment.md`
- `docs/analysis/2026-05-19-ls-lint-2x-completion-audit.md`
- `docs/analysis/2026-05-15-incremental-cache-aware-checking-strategy.md`
- `benches/README.md`
- `website/src/content/docs/reference/performance.mdx`
- `website/src/components/performance-evidence.astro`
- `website/public/data/performance/current.json`
- `website/src/pages/index.astro`
- `website/astro.config.mjs`
- `website/package.json`
- `README.md`
- `Cargo.toml`

External implementation and SEO references:

- [Google SEO Starter Guide](https://developers.google.com/search/docs/fundamentals/seo-starter-guide)
- [Google people-first content guidance](https://developers.google.com/search/docs/fundamentals/creating-helpful-content)
- [Google title-link guidance](https://developers.google.com/search/docs/appearance/title-link)
- [Google canonical URL guidance](https://developers.google.com/search/docs/crawling-indexing/consolidate-duplicate-urls)
- [Google sitemap guidance](https://developers.google.com/search/docs/crawling-indexing/sitemaps/overview)
- [Google page-experience guidance](https://developers.google.com/search/docs/appearance/page-experience)
- [Google ProfilePage structured data](https://developers.google.com/search/docs/appearance/structured-data/profile-page)
- [Google SoftwareApplication structured data](https://developers.google.com/search/docs/appearance/structured-data/software-app)
- [Astro sitemap integration](https://docs.astro.build/en/guides/integrations-guide/sitemap/)
