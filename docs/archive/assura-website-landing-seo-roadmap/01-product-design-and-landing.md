---
title: Assura website, landing-page, performance, and SEO roadmap
status: proposed
audience:
  - implementation agents
  - Assura maintainers
owner: Nick Roth
product_site: https://assura.dev/
personal_site: https://www.nickroth.com/
github_profile: https://github.com/rothnic
github_repository: https://github.com/rothnic/assura
---

# Assura website, landing-page, performance, and SEO roadmap

## Executive decision

Do **not** redesign the site from scratch.

The current landing page already has a distinctive and appropriate visual language:

- dark, technical, high-contrast presentation;
- monospaced labels and product-output framing;
- a restrained teal, amber, red, and blue signal palette;
- strong mobile behavior;
- clean bordered surfaces;
- terminal and diagnostic motifs;
- clear light/dark theme foundations;
- an overall feel that fits a fast Rust developer tool.

The path forward is to **preserve that design system, simplify the first-screen hierarchy, make the command model truthful, elevate performance as proof, and reuse the same marketing components across a focused set of second-level pages**.

The central product story remains:

> **Assura catches project drift while the fix is still small.**

The supporting workflow is:

> **Onboard once. Review while working. Explain when needed. Check before merge.**

The command distinction is:

> **Review is the radar. Check is the gate.**

The performance distinction is:

> **Fast from a cold start. Much faster in a persistent agent or editor session.**

The landing page may describe the target product experience, but every exact command, option, benchmark value, and terminal output presented as current must be generated from or tested against the release being promoted.

---

# 1. Product truth the website must preserve

## 1.1 Current command model

Assura currently has both `check` and `review`. They serve different purposes and should not compete in the copy.

| User need | Command | Intended role | Enforcement |
|---|---|---|---|
| Set up a project for agents | `assura agent onboard . --agent auto --format json` | Create or preserve the broad baseline, install project-local guidance, verify setup, and identify questions still needing human answers | Mutating but reviewable and idempotent |
| Understand current project health | `assura review` | Compact first diagnostic over validation, doctor state, content gaps, branch/worktree state, and hot directories | Advisory product surface |
| Investigate one path | `assura explain <path>` | Explain scopes, inherited rules, skipped checks, binary handling, suppressions, and next actions | Read-only |
| Enforce configured policy | `assura check` | Deterministic validation used locally, in hooks, and in CI | Blocking by default; advisory with supported warning modes |
| Diagnose setup completeness | `assura doctor` | Report configured, inactive, and recommended capabilities | Read-only |
| Deliver bounded agent feedback | `assura agent nudge` and `assura check --format agent` | Give agent harnesses concise feedback without inventing host-specific validation logic | Event and lifecycle dependent |

### Website language

Use these four verbs consistently:

1. **Onboard** — establish the baseline.
2. **Review** — show what deserves attention.
3. **Explain** — drill into one area.
4. **Check** — decide whether configured policy passes.

Do not use `review` and `check` as synonyms.

### Current CLI syntax that can appear publicly

```bash
assura agent onboard . --agent auto --format json
assura review
assura explain src/cli
assura check
assura check --format json .
assura check --format agent --agent codex .
```

### Syntax that must not appear until implemented and released

```bash
assura review --base auto
assura review --path src/cli --explain
```

An explicit review base is a sensible roadmap feature, but current automatic base detection is internal. Path explanation is a separate `assura explain` command.

## 1.2 Current integration surface

Current host-agent integration targets include:

- Codex;
- OpenCode;
- Claude Code;
- Pi;
- a generic onboarding path when no supported host is detected.

Do not advertise Cursor, GitHub Copilot, or another harness as directly integrated until the connector exists and passes the integration lifecycle tests. Generic shell and `AGENTS.md` guidance may still be described as vendor-neutral.

## 1.3 Checked is not complete

A passing `assura check` means the **configured policy** passed. It does not mean every optional model, content collection, query surface, integration, search index, or domain-specific rule has been activated.

This is a key trust differentiator, not a caveat to bury.

Recommended language:

> **Assura tells the agent what passed, what is inactive, and what still needs a human decision.**

---

# 2. Performance story and supporting evidence

Performance is not a side feature. It is part of why Assura can move feedback earlier in the agent loop rather than waiting for pre-commit or CI.

## 2.1 What the repository currently proves

The checked performance system separates two claims.

### Cold one-shot path

The public `assura` launcher routes supported one-shot checks through a lightweight checker rather than loading every full-product surface.

The current checked comparison report, captured on 2026-07-02, reports:

- 8 accepted LS-Lint-equivalent fixtures;
- Assura faster than native LS-Lint on 8 of 8 accepted cold rows;
- no accepted Assura cold row slower than its native LS-Lint counterpart;
- approximately **1.32x aggregate cold speedup** across those accepted rows;
- the universal cold 2x target **not complete** in that macOS dynamic-build report.

That is a strong, supportable claim:

> **Assura is no slower than native LS-Lint on every accepted cold comparison in the current checked cohort.**

It is not support for:

> “Assura is always 2x faster than LS-Lint from a cold start.”

Historical Linux static-CRT evidence did clear a 2x cold gate, but it must remain separately labeled by platform and build profile unless refreshed as the current release claim.

### Warm persistent session

The same checked report separately measures a persistent `assura-check-session` process connected to `assura-checkd`.

The current warm summary reports:

- Assura faster than native LS-Lint on 8 of 8 accepted cases;
- the 2x target met on 8 of 8;
- approximately **19.97x aggregate warm-session speedup** over native LS-Lint in that cohort.

This is the differentiated loop story:

> **Persistent agent and editor sessions avoid paying the full startup and setup cost on every request.**

Do not describe this merely as “the second `assura check` is faster.” The measured warm path is a persistent session and daemon contract, not two unrelated shell invocations.

## 2.2 Why warm sessions are faster

The current architecture contains real foundations for this story:

- `PreparedStructureCheck` keeps validated compiled configuration in memory.
- It reloads only when configuration bytes change.
- Changed-path checking can validate a touched path and affected parent aggregate rules without walking the entire project.
- `assura-checkd` tracks dirty-path information.
- A previously clean project can use conservative incremental checking for safe file-level changes.
- Configuration changes, directory events, ambiguous watcher events, too many dirty paths, or a previously failing project fall back to a full check.
- A persistent session can reuse a daemon connection instead of launching a new client process each time.
- Config fingerprint probing protects the warm path from silently using changed configuration.
- An opt-in unchanged-tree cache exists for LS-Lint-compatible checks with explicit root, version, schema, config, path, and directory fingerprint validation.

This is worth illustrating visually:

```text
Agent or editor
      ↓
Persistent Assura session
      ↓
Prepared configuration + local daemon state
      ↓
Changed paths and affected parents
      ↓
Conservative full-check fallback when certainty is insufficient
```

The important product promise is not “cache everything.” It is:

> **Reuse work when correctness can be proven; fall back when it cannot.**

## 2.3 What remains incomplete

Do not imply all repeated standalone checks are already fully incremental.

Current design notes still identify future work around:

- broad file-local result reuse outside the daemon;
- a persistent file-state index;
- robust directory content fingerprints;
- git-assisted dirty-path discovery for the standalone cache path;
- benchmark rows for warm no-change, small-change, delete/rename, and config-change scenarios;
- shared immutable cache reuse across worktrees;
- persistent finding history and repeated-noise suppression.

## 2.4 Performance tooling already in the repository

The site should make use of the existing evidence system rather than hand-writing claims.

Current tooling includes:

- `assura performance-report`;
- machine-readable current JSON;
- capped JSONL performance history;
- phase attribution for process floor, Rust CLI floor, config load, checker initialization, traversal/validation, and report sorting;
- `cargo xtask performance-no-slower`, which fails when any accepted Assura cold row is slower than the corresponding native LS-Lint row;
- `cargo xtask native-performance-no-regression`, which protects Assura-native surfaces that LS-Lint does not provide;
- Criterion benchmarks for structure checking;
- content-runtime benchmarks;
- release-size gates;
- release smoke and live-install gates;
- an existing website performance component that reads repository-tracked JSON evidence.

## 2.5 Immediate performance evidence issue

`Cargo.toml` is currently version `0.3.0`, while the checked public comparison JSON identifies Assura `0.2.0`.

Before placing benchmark numbers prominently on the homepage:

1. refresh the performance report against the current release candidate;
2. run the no-slower gate;
3. review all accepted rows;
4. regenerate website data;
5. confirm the performance page shows the same version promoted by the site.

## 2.6 Homepage performance treatment

Add a compact proof strip below the primary product visual, generated from current JSON:

```text
COLD CHECKS
Faster in 8 / 8 checked LS-Lint comparisons
1.32x aggregate in the current recorded cohort

WARM SESSION
19.97x aggregate speedup
2x target met in 8 / 8 checked cases

Local · persistent · correctness-first fallback
[See benchmark methodology]
```

The exact figures must be populated from the report rather than hardcoded.

Round only for marketing display while retaining the exact machine-readable report.

## 2.7 Performance page treatment

The clean marketing performance page should contain:

1. **Cold start** — comparable one-shot CLI execution.
2. **Warm loop** — persistent session and daemon-backed execution.
3. **Why both matter** — CI needs cold correctness; agents and editors need warm responsiveness.
4. **Current result cards** — generated from JSON.
5. **Per-fixture comparison chart**.
6. **Warm architecture diagram**.
7. **Methodology summary**.
8. **Known limits and environment**.
9. **Links to technical performance docs and raw evidence**.

Recommended headline:

> **Fast enough to stay in the agent loop.**

Recommended subhead:

> **Assura protects the cold CLI path and separately measures a persistent warm session that reuses prepared project state.**

---

# 3. Design-preservation contract

## 3.1 Preserve these visual foundations

Do not replace:

- the current dark-first technical aesthetic;
- the existing brand mark and lowercase `assura` wordmark;
- the signal color system;
- monospaced eyebrows and labels;
- large editorial headings;
- bordered diagnostic surfaces;
- terminal-style product evidence;
- the subtle grid/background treatment;
- sticky header behavior;
- current responsive layout philosophy;
- the custom Astro landing page plus Starlight technical docs split.

## 3.2 Improve hierarchy without changing identity

The current visual issue is not the design language. It is that too many elements have equal emphasis.

Reduce competing weight by:

- keeping only one dominant product-output card per viewport;
- replacing secondary bordered cards with rows, timelines, accordions, or grouped lists;
- moving the full setup prompt out of the hero;
- reducing mobile H2 size by roughly 10–15%;
- making section copy shorter and more scannable;
- using generous whitespace between narrative sections;
- reserving glow and strong border colors for actual product signals and primary CTAs.

## 3.3 Do not migrate frameworks

Continue using:

- Astro;
- the existing custom `src/pages/index.astro` marketing surface;
- Starlight for technical documentation;
- static rendering;
- minimal client-side JavaScript.

Do not introduce React, a general component framework, or a new design system merely to revise the page.

## 3.4 Extract, do not rebuild

Refactor the current page incrementally into reusable components while preserving rendered appearance.

Suggested structure:

```text
website/src/
├─ components/
│  └─ marketing/
│     ├─ MarketingHeader.astro
│     ├─ MarketingFooter.astro
│     ├─ MarketingSection.astro
│     ├─ SectionHeading.astro
│     ├─ SignalTree.astro
│     ├─ TerminalCard.astro
│     ├─ CommandJourney.astro
│     ├─ LifecycleRail.astro
│     ├─ ComparisonAccordion.astro
│     ├─ PerformanceProof.astro
│     ├─ AgentSetupDialog.astro
│     ├─ AuthorAttribution.astro
│     └─ SeoHead.astro
├─ layouts/
│  └─ MarketingLayout.astro
├─ styles/
│  └─ marketing.css
└─ pages/
   ├─ index.astro
   ├─ ai-coding-agent-guardrails.astro
   ├─ project-review.astro
   ├─ agent-onboarding.astro
   ├─ repository-validation.astro
   ├─ project-intelligence.astro
   ├─ performance.astro
   ├─ compare/
   │  └─ ls-lint.astro
   └─ about.astro
```

First extract tokens and shared layout with no intentional visual change. Then revise page hierarchy.

---

# 4. Landing-page target layout

## 4.1 Header

Desktop:

```text
assura    How it works    Compare    Performance    Docs    GitHub    Start
```

Mobile:

```text
assura                                      Docs    Start
```

Put GitHub in the mobile menu or footer to avoid crowding.

`Start` opens the setup dialog or mobile bottom sheet.

## 4.2 Hero

### Eyebrow

> Early warning signals for AI-assisted work

### H1

> Catch project drift before review.

### Subhead

> Assura shows what changed, where cleanup risk is growing, and what an AI agent should fix next—before hooks, CI, or review turn a small issue into a broad refactor.

### Supporting line

> Small fixes while the work is still small.

### CTAs

Primary:

> See a project review

Secondary:

> Start with your agent

### Hero product visual

Use a compact project tree rather than a large installation prompt:

```text
PROJECT REVIEW
Compared with origin/main

Needs attention

project/
├─ src/cli/            WARN
│  ├─ file length      282 / 300
│  └─ branch churn     +412 / -96
├─ website/src/        WATCH
│  └─ references       1 stale link
└─ .agents/skills/     WATCH
   └─ guidance         missing required section

Next: explain src/cli
```

Use transparent measurements rather than opaque heat scores. A visitor can understand `282 / 300 lines`; `82 / 70` requires an undocumented scoring system.

Until finding memory is implemented, do not show:

```text
3 new · 1 worsened · 12 unchanged hidden
```

Use only data currently emitted by the release, or label the visual as a conceptual preview.

## 4.3 Performance proof strip

Place directly under the hero visual.

Purpose:

- establish that this is not another slow checker stack;
- introduce cold versus warm behavior;
- provide the immediate LS-Lint differentiation;
- link to the full methodology.

Keep it visually compact.

## 4.4 Problem section

### Heading

> The current workflow catches problems too late.

### Copy

> Teams already have checks. The problem is that each tool sees only a slice, and many of them run after the agent has completed a large batch of work.

Use a simple list instead of three equally heavy cards:

```text
Language linters inspect code inside files.
Hooks run after a batch.
CI runs after push.
Convention docs explain rules but cannot enforce them.
Custom scripts become glue.
```

Then show the cost curve:

```text
edit
  → small drift
  → drift grows
  → hook blocks
  → rushed refactor
  → review still finds cleanup
```

## 4.5 Product journey section

### Heading

> Review first. Check last.

Use a four-step strip:

```text
ONBOARD
Create the broad agent-ready baseline.

REVIEW
See what changed and where attention is needed.

EXPLAIN
Understand one path, inherited rule, or signal.

CHECK
Enforce configured policy before merge.
```

This section resolves the `review` versus `check` confusion before technical visitors reach command examples.

## 4.6 Full review section

### Heading

> A compact project map agents can act on.

Use real output generated from a deterministic fixture.

Plain-language intro:

> Assura starts from the current branch or worktree, rolls signals up by directory, and points the agent to the next useful command.

Show the actual current CLI syntax:

```bash
assura review
```

Link to:

```bash
assura explain src/cli
```

Do not hand-author output in `index.astro`; generate it from a fixture or committed JSON artifact.

## 4.7 Where Assura fits

### Heading

> One signal layer above your existing tools.

### Lead

> Assura does not replace your linters, hooks, CI, or documentation. It sits above them and tells agents where the project itself is getting messy.

Desktop: concise comparison table.

Mobile: accordion.

| Tool | Good at | Assura adds |
|---|---|---|
| Language linters | Details inside source files | Language-agnostic project structure and change signals |
| LS-Lint | Fast filesystem naming | Broader checks, reusable policy, agent feedback, and warm sessions |
| Pre-commit | Running local gates | Earlier threshold and review signals |
| CI | Authoritative post-push gates | Local branch/worktree context |
| Convention docs | Explaining intent | Repairable findings |
| Custom scripts | One-off policies | Shared configuration, reporting, lifecycle, and performance discipline |

## 4.8 Watch, warn, gate

### Heading

> Small fixes while the work is still small.

Use a compact lifecycle rail rather than three giant cards:

```text
WHILE WORKING      WATCH
Approaching thresholds and emerging hot spots.

BEFORE COMMIT      WARN
New and changed-file findings without old background noise.

BEFORE MERGE       GATE
Only configured policy violations that must not land.
```

Be precise:

- “Watch” requires a supported agent/editor integration or an active session.
- “Warn” maps to advisory output and local hook profiles.
- “Gate” maps to `assura check`.

## 4.9 One-command agent onboarding

### Heading

> Start with one agent instruction.

The hero should not display the full prompt. Use a button that opens a dialog or bottom sheet.

Visible copy:

> The setup prompt installs Assura, creates a broad baseline, verifies what is active, and tells the agent what it must ask before guessing project-specific conventions.

Buttons:

```text
Copy agent setup prompt
Show manual install
```

Dialog contents:

```text
Agent setup

[copy full prompt]

Manual install

curl -fsSL https://assura.dev/install.sh | sh

What happens
✓ creates or preserves the broad baseline
✓ detects a supported agent harness
✓ creates project-local Assura skills
✓ verifies setup
✓ lists inactive capabilities
✓ writes the questions that still need human answers
```

## 4.10 Progressive intelligence

### Heading

> Start with structure. Build toward project understanding.

Show the top-down progression:

```text
Repository structure
→ universal file and folder hygiene
→ Markdown and agent guidance
→ typed project records
→ relationships and references
→ queryable local project context
```

Keep this visual, but do not let future layers dominate the immediate value proposition.

## 4.11 Final CTA

> Catch drift while the fix is still small.

Buttons:

```text
Start with your agent
Read the quick start
View on GitHub
```

## 4.12 Footer and author attribution

Include:

> Assura is open-source software created by [Nick Roth](https://www.nickroth.com/), a product manager and AI engineer focused on AI-assisted software development.

Also link visibly to:

- [Nick Roth’s GitHub profile](https://github.com/rothnic)
- [the Assura repository](https://github.com/rothnic/assura)
- documentation;
- releases;
- support policy;
- security;
- license.

Use normal followed links for these trusted first-party identities.

---

