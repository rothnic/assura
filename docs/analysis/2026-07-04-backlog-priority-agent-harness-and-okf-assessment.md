---
title: Backlog Priority, Agent Harness, And OKF Assessment
date: 2026-07-04
status: current
---

# Backlog Priority, Agent Harness, And OKF Assessment

## Purpose

This assessment refreshes the Assura backlog after the agent-ready onboarding
lane and folds in three product questions:

- what the most common and painful user issues are likely to be;
- which agent harnesses need first-class install or routing paths;
- whether Open Knowledge Format (OKF) should be a supported content structure.

## Live Snapshot

Checked on 2026-07-04 from branch
`codex/agent-ready-onboarding-backlog`.

| Surface | Current Signal | Backlog Implication |
| --- | --- | --- |
| PR #139 | Open, mergeable, and checks green on head `2922ea9f6e4556be12e4c847e617ec1724a5d450`. | Finish the PR handoff before opening new implementation lanes. |
| `assura doctor . --format json` | Passes with no violations; reports inactive optional repository reference and onboarding packet gaps. | Users need one compact "where am I and what matters next?" review flow. |
| `assura content agent-query gaps . --format json` | Reports 1,089 unresolved repository references. Manual inspection shows many low-confidence hits from generated JSON, archived task logs, and benchmark history. | Improve reference-graph signal quality before making unresolved references more prominent or blocking. |
| Roadmap | Agent-ready onboarding is complete locally; current ordered lane is performance polish. | Keep performance as the next core delivery lane. |
| Planned goals | Several older `status: planned` goals are superseded by completed project-intelligence work. | Add a cleanup pass that marks stale goals completed, superseded, or merged into current goals. |

## Priority Scoring

Score = `reach + pain + differentiation + confidence - effort`.

| Score | Item | Why It Matters | Done Signal |
| ---: | --- | --- | --- |
| 16 | Close PR #139 and preserve handoff truth | The current branch is the delivery vehicle for completed onboarding work. Starting new implementation before PR truth is stable creates review drift. | PR #139 includes the latest docs, checks are green, and local branch is clean. |
| 15 | Performance polish and native report gates | Every user hits CLI latency, install confidence, and validation runtime before advanced content modeling matters. | LS-Lint parity and Assura-native suites have checked history, website data, and no-regression gates. |
| 14 | Compact project review surface | The common user question is not "what command exists?" but "is this repo healthy, and what should I fix next?" | A single documented flow combines doctor, self-check, next actions, and high-signal content gaps. |
| 13 | Reference graph signal-quality cleanup | Current unresolved-reference counts are useful discovery but too noisy to drive agent behavior without filtering. | Generated artifacts, archives, logs, and low-confidence tokens are classified or filtered; fixtures prove fewer false positives. |
| 12 | Agent harness install matrix | Users will bring Assura to their existing coding agent, editor, or terminal workflow. | Supported and experimental harnesses have install recipes that delegate to shared Assura commands. |
| 11 | OKF supported content starter | OKF matches Assura's strengths: files, markdown, YAML frontmatter, links, progressive disclosure, and permissive validation. | An OKF starter validates conformant and broken bundles, has query examples, and is included in native performance fixtures. |
| 10 | LLM-wiki and research-authoring starter pack | This is a compelling differentiator once core review and performance are stable. | Starter variants cover minimal wiki, Obsidian, research authoring, agent-skill wiki, and OKF. |
| 8 | Release hardening | Needed for broader adoption after the product path is coherent. | Release checklist, support matrix, install smoke, and public docs agree. |
| 7 | Backlog status cleanup | Important for agent continuity, but mostly governance once the top lanes are explicit. | Stale goals are revalidated and marked completed, superseded, or active with exact next steps. |

## Recommended Roadmap

### P0: Merge-Ready Onboarding Closure

Keep the current branch focused. Do not reopen agent-ready onboarding unless PR
review, CI, or a current product regression proves it is necessary.

Outputs:

- PR #139 green on latest head;
- local workspace clean;
- handoff comment or docs pointing to the next executable goal.

### P1: Performance And Common-Issue Review

Execute `docs/goals/assura-performance-polish-program.md`, but include one
adjacent product proof: a compact project review flow that combines self-check,
doctor, next-actions, and high-signal content gaps.

Why this comes before new content-model work:

- it protects every user and every integration path;
- it gives agents a reliable first diagnostic before changing structure;
- it prevents noisy reference findings from becoming unhelpful nudges.

### P2: Agent Harness Install Matrix

Assura should avoid per-agent validation engines. Each harness should call the
same shared surfaces:

- `assura check --format agent`;
- `assura agent nudge`;
- `assura daemon status|doctor`;
- `assura content agent-query`;
- generated `.assura/integrations/<agent>/` bundles where useful.

| Tier | Harness | Current/Target Support |
| --- | --- | --- |
| 1 | Codex | Supported delivery wrapper; keep as the reference path. |
| 1 | Claude Code / Agent Skills | Add skill-bundle install docs and hook/wrapper examples over shared commands. |
| 1 | VS Code / GitHub Copilot | Promote the existing beta package path and align with VS Code Agent Skills. |
| 1 | Cursor | Add rules/skills/MCP-oriented install recipe; do not require a Cursor-only engine. |
| 1 | Gemini CLI | Add command-wrapper recipe and agent JSON examples. |
| 1 | OpenCode | Replace the historical prototype with a thin hook/plugin wrapper. |
| 1 | Cline / Roo Code | Add extension-agent wrapper docs around shared nudge/check commands. |
| 2 | Zed / ACP | Track ACP as the editor-agent bridge and add a recipe once the shared adapter shape is stable. |
| 2 | JetBrains | Prefer MCP/ACP recipes over a custom IDE plugin until demand proves otherwise. |
| 2 | Aider | Add a lightweight terminal recipe for pre-edit and post-edit checks. |
| 2 | Pi | Keep existing experimental bundle support and validate against current package conventions. |
| 2 | Devin / PR review agents | Document CI/PR review usage; do not treat hosted review as local agent feedback. |

Source notes:

- OpenAI Codex exposes local CLI, editor, desktop, and web surfaces:
  <https://github.com/openai/codex>.
- Anthropic Agent Skills package instructions, scripts, and resources:
  <https://platform.claude.com/docs/en/agents-and-tools/agent-skills/overview>.
- VS Code Agent Skills are folders of instructions, scripts, and resources:
  <https://code.visualstudio.com/docs/agent-customization/agent-skills>.
- Cursor documents Agent, Rules, MCP, Skills, and CLI surfaces:
  <https://cursor.com/docs>.
- Gemini CLI is an open-source agent with local and remote MCP support:
  <https://developers.google.com/gemini-code-assist/docs/gemini-cli>.
- OpenCode, Cline, Roo Code, Aider, Pi, and ACP were checked from their
  official docs or repositories on 2026-07-04.

Popularity should inform but not decide the order. Current GitHub star checks
put Claude Code, Gemini CLI, Codex, Zed, Pi, Cline, Aider, Continue, Roo Code,
OpenCode, and ACP in the broad priority set, but Assura should prioritize
where users can install a thin wrapper over shared commands with low support
risk.

### P3: Supported Content Structures

Make content starters a second-order product layer on top of the stable review
and performance path.

Recommended starter order:

1. Research authoring, because it builds on the completed `document-project`
   preset and requirements/evidence traceability.
2. LLM wiki minimal, because it is plain markdown plus source custody.
3. OKF bundle, because it is a specified interoperable LLM-wiki shape.
4. Obsidian vault, because it is popular but adds optional app conventions.
5. Agent skill wiki, because it helps Assura dogfood progressive disclosure.

## OKF Support Assessment

Google introduced Open Knowledge Format on 2026-06-12 as a vendor-neutral,
agent- and human-friendly format for curated knowledge. The OKF v0.1 spec is a
directory of UTF-8 markdown files with YAML frontmatter. Every concept document
requires a non-empty `type` field. Recommended fields include `title`,
`description`, `resource`, `tags`, and `timestamp`. Reserved filenames include
`index.md` for progressive-disclosure directory listings and `log.md` for
update history. Concepts use standard markdown links for graph relationships,
and consumers are expected to tolerate unknown types, unknown fields, missing
optional fields, and broken links.

Primary sources:

- Google Cloud announcement:
  <https://cloud.google.com/blog/products/data-analytics/how-the-open-knowledge-format-can-improve-data-sharing>.
- OKF v0.1 spec:
  <https://raw.githubusercontent.com/GoogleCloudPlatform/knowledge-catalog/main/okf/SPEC.md>.
- OKF reference repository:
  <https://github.com/GoogleCloudPlatform/knowledge-catalog/tree/main/okf>.

Assura fit:

- Structure rules can validate bundle shape, reserved filenames, and allowed
  root or directory patterns.
- Existing markdown/frontmatter parsing can validate required `type` fields and
  recommended query fields.
- Repository-reference discovery can power cross-link and citation diagnostics.
- Content runtime collections can model Concept, Citation, Resource, Tag, and
  LogEntry records without a new schema language.
- Agent-query output can give agents bounded next actions without loading the
  full bundle into context.

Performance approach:

- Treat OKF as markdown/frontmatter files, not as JSON graph data or a vector
  database.
- Validate with one file walk, one markdown/frontmatter parse pass, and an
  optional link-index pass.
- Reuse the content runtime index for concept IDs, frontmatter keys, tags,
  resources, outbound links, backlinks, and citation targets.
- Keep broken links advisory by default to match OKF's permissive consumption
  model; allow stricter project configs for teams that want blocking checks.
- Add native performance fixtures for small, medium, large, and
  reference-heavy OKF bundles.
- Record cold CLI, warm/session, link-index, query, context-pack, and report
  serialization rows before claiming support.
- Avoid a required persistent store until measured native fixtures show the
  in-memory path cannot meet the target.

Supported starter done criteria:

- `examples/okf-basic` and `examples/okf-broken` or equivalent fixtures exist.
- The valid fixture passes `assura check --format json`.
- The broken fixture reports missing `type`, malformed frontmatter, reserved
  filename misuse, invalid `log.md` date headings, and configured link issues.
- Query examples list concepts by type, resources, tags, outbound links,
  backlinks, missing recommended fields, citations, and next actions.
- The starter docs state that OKF is format support, not a BigQuery exporter,
  visualizer, hosted catalog, LLM crawler, or generation engine.

## Backlog Cleanup Rules

When cleaning goal status, apply these rules:

- Keep active goals only when they map to a current roadmap lane or a concrete
  user-requested follow-up.
- Mark completed implementation goals completed if their progress log and
  current code/docs prove the surface exists.
- Mark older project-intelligence goals superseded when their scope landed in
  the post-beta or agent-ready program.
- Merge duplicate research/content starter work into the LLM-wiki and OKF
  starter goal rather than maintaining parallel goals.
- Do not delete historical goals; update frontmatter and add a short
  revalidation note.

## Proposed Next Goal Prompt

```text
Execute a backlog cleanup and roadmap prioritization pass for Assura.

Start from live state, not stale goal frontmatter: run the Trellis workflow
gate, inspect git status, verify PR #139, inspect .trellis/spec/assura/roadmap.md,
docs/data/public-roadmap.json, docs/support-policy.md, and the planned goal
list under docs/goals.

Produce a reviewed cleanup commit that:
1. marks stale or superseded goals completed/superseded with short evidence;
2. keeps performance polish as the next core implementation lane;
3. adds a compact project review/common-issues goal if one does not already
   exist;
4. records an agent-harness install matrix for Codex, Claude Code, VS Code /
   Copilot, Cursor, Gemini CLI, OpenCode, Cline, Roo Code, Zed/ACP,
   JetBrains, Aider, Pi, and PR-review agents;
5. folds Open Knowledge Format into the LLM-wiki/content-structure backlog
   with validation, query, and native performance proof gates.

Do not implement new adapters, OKF validators, or performance code in the
cleanup pass. The output is a truth-maintained backlog with measurable goals
and reviewer blocking criteria.
```
