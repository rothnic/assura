---
id: goal-assura-llm-wiki-personal-knowledge-base-starters
type: goal
title: Assura LLM wiki personal knowledge base starters
status: planned
created: 2026-07-04
owners:
  - assura-maintainers
related:
  - ./assura-agent-ready-project-onboarding-program.md
  - ./assura-agent-document-project-preset.md
  - ./assura-agent-requirements-evidence-traceability.md
  - ./assura-project-intelligence-content-model-validation-demo.md
  - ./assura-content-query-and-search-cli.md
  - ./assura-supported-document-graph.md
---

# Assura LLM Wiki Personal Knowledge Base Starters

## Objective

Make Assura a strong validation and querying layer for personal knowledge-base
and LLM-wiki projects without forcing users into one wiki application,
Obsidian layout, RAG stack, or agent skill convention.

Deliver a small set of starter configurations, fixtures, and docs that show
how Assura content models can validate source custody, generated wiki pages,
internal links, evidence references, and queryable relationships across common
LLM-wiki structures.

## Current Gap

The agent-ready onboarding program now has a generic `document-project`
preset, requirements/evidence traceability, source-document custody, raw search,
repository-reference discovery, and content-query commands. That is enough
foundation for knowledge-base projects, but the public product path does not
yet make the LLM-wiki use case obvious.

The current ecosystem is moving quickly around Karpathy-style LLM wikis,
Obsidian vaults, agent skills, local RAG, and self-maintaining second brains.
Most projects provide a prompt, skill, plugin, desktop app, or Python scripts.
Assura can be useful at a lower layer: portable repository structure,
content-model contracts, relation validation, link health, and deterministic
query examples that any agent or wiki tool can use.

## External Scan

Representative GitHub scan, refreshed on 2026-07-04:

| Project | What It Shows | Assura Opportunity |
| --- | --- | --- |
| [Karpathy LLM Wiki gist](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) | The core pattern: raw sources compiled into a persistent Markdown wiki that agents query and maintain. | Model the repository contract behind the pattern instead of depending on one prompt. |
| [nashsu/llm_wiki](https://github.com/nashsu/llm_wiki) | A desktop app with document ingest, scenario templates, source traceability, graph/search, MCP, and a bundled agent skill. | Provide lighter source-control-native starter configs for teams that do not want a full app. |
| [AgriciDaniel/claude-obsidian](https://github.com/AgriciDaniel/claude-obsidian) | A popular Obsidian plus Claude Code workflow with methodology modes, linting, hot cache, and skills. | Validate vault shape, skill shape, links, sources, and traceability independent of Claude-specific commands. |
| [Ar9av/obsidian-wiki](https://github.com/Ar9av/obsidian-wiki) | Agent-readable Markdown skills, multi-agent install routing, vault doctor/query/lint commands, and Obsidian-centric setup. | Reuse the multi-agent routing idea while keeping Assura's role to config, validation, and query facts. |
| [VectifyAI/OpenKB](https://github.com/VectifyAI/OpenKB) | CLI that compiles raw documents into an interlinked wiki and generates skills, decks, visualizations, query/chat outputs. | Treat generated outputs as modeled content with evidence and link contracts. |
| [swarmclawai/swarmvault](https://github.com/swarmclawai/swarmvault) | Local-first LLM wiki, graph, context-pack, search, review, and agent memory store with profiles and agent install options. | Use as a benchmark for the breadth of questions users expect: next action, query, graph, context pack, doctor. |
| [ussumant/llm-wiki-compiler](https://github.com/ussumant/llm-wiki-compiler) | Claude/Codex-compatible plugin that compiles Markdown or codebases into a topic wiki with search, lint, and context helper. | Show Assura can validate compiled wiki outputs and references without owning the compiler. |
| [NulightJens/ai-second-brain-skills](https://github.com/NulightJens/ai-second-brain-skills) | Two skills around setup and self-healing, with `raw/`, `wiki/`, index, log, ingest, query, and lint mental model. | Provide first-party Assura examples for skill-driven progressive disclosure plus link/reference checks. |
| [khoj-ai/khoj](https://github.com/khoj-ai/khoj), [Vasallo94/ObsidianRAG](https://github.com/Vasallo94/ObsidianRAG), [Zackriya-Solutions/MCP-Markdown-RAG](https://github.com/Zackriya-Solutions/MCP-Markdown-RAG) | RAG and semantic-search systems for notes and Markdown, often with Obsidian or MCP integration. | Keep semantic/vector search optional; make structural correctness and deterministic references valuable before embeddings. |

## Product Thesis

Assura should not become an LLM wiki app. It should make LLM wiki projects less
fragile by answering:

- Is the vault shape intentional?
- Are sources, wiki pages, claims, concepts, and analyses modeled?
- Are internal Markdown links and frontmatter references valid?
- Which pages lack source evidence?
- Which sources are not represented in the wiki?
- Which claims or analyses lack citations?
- Which generated pages are orphaned?
- What bounded context should an agent read before answering a question?

## User Certainty Bar

A user should be able to ask an agent:

> Set up an Assura-backed LLM wiki starter for academic research and content
> authoring. Keep it plain Markdown, validate links and source evidence, and
> show me query examples for finding gaps before I ask an LLM to ingest more
> documents.

The agent should have a documented Assura path that creates or points to a
starter config, explains the tradeoffs between starter shapes, runs validation,
and demonstrates query output without inventing a bespoke folder convention.

## Starter Configurations

Deliver these as examples or preset templates. They should be copyable and easy
to customize, not hidden behind a single rigid project-type guess.

### 1. LLM Wiki Minimal

Plain Markdown starter for the Karpathy-style folder-as-application pattern.

```text
raw/
wiki/
  index.md
  log.md
  sources/
  concepts/
  analyses/
```

Model Source, Concept, Analysis, Claim, and Term records. Validate source
custody, source-to-wiki coverage, page IDs, links, and required relations from
analyses or claims to source evidence.

### 2. Obsidian Vault

Obsidian-compatible Markdown and wikilink starter.

```text
raw/
wiki/
attachments/
.obsidian/
```

Keep `.obsidian/` optional and non-invasive. Validate Markdown links,
frontmatter references, attachment paths, source pages, index/log files, and
orphaned pages without requiring any Obsidian plugin.

### 3. Research Authoring

Academic research and long-form content authoring starter.

```text
source-documents/
library/
notes/
claims/
analyses/
drafts/
final/
```

Layer on the existing `document-project` and requirements/evidence
traceability work. Model SourceDocument, Topic, Note, Claim, Evidence,
Finding, Draft, and FinalDocument. Validate claims to evidence, evidence to
source documents, draft-to-claim coverage, unresolved references, and binary
source custody.

### 4. Agent Skill Wiki

Starter for projects that use agent skills as the primary progressive
disclosure surface.

```text
AGENTS.md
.agents/
  skills/
    <skill-name>/
      SKILL.md
      references/
      scripts/
      examples/
```

Reuse agent guidance and skill contracts. Add a modeled skill-reference table
showing when a task should load a skill or skill pattern, and validate that
`SKILL.md` files route to internal references rather than embedding long
examples in the entrypoint.

## Scope

- Add starter configs or preset templates for the four shapes above.
- Provide valid and invalid fixtures for each starter.
- Document when to choose each starter and how to customize it safely.
- Model collections and relations using the existing content runtime rather
  than adding a separate schema language.
- Validate Markdown links, frontmatter repository references, source custody,
  required relation coverage, and orphan or missing-reference conditions.
- Include agent-focused query examples using existing `assura content` and
  `assura agent` surfaces.
- Include docs that compare Assura's role with agent skills, Obsidian plugins,
  desktop apps, and RAG systems.
- Keep examples generic enough for personal research, academic writing,
  content authoring, documentation, and small-team knowledge bases.

## Effective Query Examples

The goal is not done until the examples prove concrete commands, not just
describe concepts. Include JSON-oriented examples for agents and concise text
examples for humans.

Required query examples:

- list modeled collections and counts;
- show one source and all generated pages that reference it;
- find wiki pages with no source evidence;
- find claims without evidence;
- find evidence records whose source document is missing;
- list unresolved Markdown links and frontmatter path references;
- search for a topic across sources, concepts, claims, and analyses;
- expand a concept to related sources, claims, analyses, and diagnostics;
- build a bounded context pack for answering a question about a topic;
- ask agent-query for next actions on a partially configured vault.

Example command shapes:

```bash
assura content collections examples/llm-wiki-minimal --format json
assura content agent-query unresolved-references examples/research-authoring --format json
assura content search retrieval examples/obsidian-vault --format json
assura content missing-relations examples/research-authoring --format json
assura content expand examples/llm-wiki-minimal concepts concept-retrieval --format json
assura content context-pack examples/research-authoring --text "source traceability" --limit 8 --format json
assura content agent-query gaps examples/agent-skill-wiki --format json
assura content agent-query next-actions examples/llm-wiki-minimal --format json
```

If current command names differ, the implementation should use the supported
current names and update this goal with the final command list.

## Non-Goals

- No full LLM wiki application.
- No Obsidian plugin.
- No document ingestion, PDF parsing, summarization, or LLM generation engine.
- No vector database or embeddings requirement.
- No hosted service, remote provider, or MCP server requirement.
- No single canonical folder structure that all knowledge bases must use.
- No automatic claim extraction from arbitrary prose.

## Definition Of Done

- Four starter configurations are available as checked examples, templates, or
  documented preset variants.
- Each starter has a passing fixture and an intentionally broken fixture.
- `assura check --format json` catches the broken fixture conditions with
  actionable diagnostics.
- Query examples are copy/pasteable and covered by tests or checked docs.
- Website or repo docs show the starter-selection decision tree and the first
  useful validation/query loop.
- The docs explicitly state that Assura validates and queries repository
  knowledge contracts; it does not replace the user's chosen agent, wiki app,
  or RAG system.
- Existing `document-project`, source-document custody, requirements/evidence
  traceability, repository-reference, and content-query surfaces are reused
  instead of duplicated.
- The starter docs link to the external scan above as comparative inspiration,
  not as hard dependencies.

## Validation Commands

```bash
cargo fmt --check
cargo test --test content_runtime_check_cli --quiet
cargo test --test content_query_cli --quiet
cargo test --test content_runtime_references --quiet
cargo test --test project_intelligence_onboarding --quiet
cargo test --test content_runtime_dx_docs --quiet
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

Add starter-specific fixture tests as the implementation lands. If no Rust code
changes are needed for the first slice, keep validation to the docs, fixture,
Assura self-check, evidence, and docs gates that cover the changed files.

## Review Tasks

- R1: Compare the implementation against the external scan and confirm Assura's
  role is validation/querying, not a copied wiki app.
- R2: Confirm starter configs are customizable and do not force one folder
  structure across all knowledge bases.
- R3: Confirm the research-authoring starter composes with the generic
  `document-project` preset and does not reintroduce domain-specific overfit.
- R4: Confirm each query example is backed by a fixture and has stable JSON for
  agents.
- R5: Confirm broken links, missing source evidence, orphan pages, and missing
  relation targets cannot pass silently.

## Reviewer Blocking Criteria

Block if the implementation:

- hardcodes a single LLM-wiki or Obsidian layout as the only supported path;
- duplicates content-runtime models or traceability checks instead of reusing
  existing surfaces;
- promises ingestion, summarization, semantic search, or app behavior that
  Assura does not implement;
- ships starter configs that cannot validate their own examples;
- provides query examples that are not executable from the repo;
- treats vector search, hosted models, or MCP as required for correctness;
- lets generated wiki pages, claims, analyses, or skill references pass without
  source or target validation when the starter says they are required.

## Copy/Paste Goal Prompt

```text
Execute docs/goals/assura-llm-wiki-personal-knowledge-base-starters.md.

Start by refreshing the external scan enough to verify the goal is still
positioned correctly. Then inspect the current Assura document-project,
source-document custody, requirements/evidence traceability, repository
reference, and content-query surfaces before designing any new config.

Implement the smallest slice that proves the product path:
1. add the four LLM-wiki starter configurations or clearly documented template
   variants;
2. add valid and broken fixtures for each;
3. prove assura check catches broken source/link/relation conditions;
4. add copy/paste query examples for collections, references, gaps, search,
   expansion, context packs, and agent next actions;
5. document when to choose each starter and how to customize it without being
   locked into one wiki structure.

Do not build an LLM wiki app, Obsidian plugin, ingestion engine, vector search
stack, or hosted service. Use Assura's existing content modeling and query
surfaces unless live code inspection proves a narrow missing capability.
Record validation commands, changed files, and any reviewer findings in this
goal before closing it.
```
