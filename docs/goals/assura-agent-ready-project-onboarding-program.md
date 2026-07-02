---
id: goal-assura-agent-ready-project-onboarding-program
type: goal
title: Assura agent-ready project onboarding program
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-performance-polish-program.md
  - ./assura-agent-integration-lifecycle.md
  - ./assura-supported-document-graph.md
  - ./assura-content-model-source-of-truth.md
  - ./assura-content-query-and-search-cli.md
  - ./assura-project-intelligence-usability-program.md
  - ./assura-beta-structure-severity-contract.md
  - ../../.trellis/spec/assura/roadmap.md
---

# Assura Agent-Ready Project Onboarding Program

## Objective

Make Assura the default scaffold, doctor, and feedback loop for agent-ready
repositories.

This goal captures the new-project dogfood backlog from July 2, 2026. The core
finding was not that Assura needs more abstract intelligence. The finding was
that a coding agent can make `assura check --format text .` return zero
violations while the project still lacks active models, searchable facts,
reference discovery, enforceable agent guidance, skill contracts, source
document custody, and a clear explanation of what was not checked.

The product should prevent that false-green failure mode, or at minimum make
it obvious.

## Product Milestone

The next broad adoption milestone is:

> Make Assura the default scaffold, doctor, and feedback loop for agent-ready
> repos.

At the end of this program, a coding agent should be able to enter a new
project and reliably answer:

- What rules apply here?
- What guidance should I follow?
- Which skills exist?
- What project facts are modeled?
- What references are broken?
- What is checked versus unchecked?
- What should I fix next?
- Will this be a nudge, warning, or merge gate?

## Current Gap

Today, "Assura passed" primarily means "the currently configured checks
passed." It does not necessarily mean:

- recommended agent-project checks are configured;
- content models are active;
- search chunks exist;
- reference graph edges are complete;
- frontmatter paths are treated as repository references;
- AGENTS.md has the sections an agent needs;
- project-local skills have enforceable shape;
- binary source documents are under custody;
- inheritance effects are understandable;
- the agent knows the next best fix;
- the feedback lifecycle is clear.

Agents optimize to the success signal. A green check that does not say what is
unchecked can train an agent to stop too early.

## Priority Relative To Other Backlog

This program is the highest general adoption backlog for agentic development
projects. It should outrank domain-specific proposal/SBIR work and lower-level
performance polish unless the active goal is explicitly performance-focused.

Keep the split clear:

- [Performance Polish](./assura-performance-polish-program.md) owns native
  performance evidence, no-slower gates, and CLI-floor work.
- This program owns new-project activation, scaffolding, doctor/explain,
  AGENTS.md and SKILL.md contracts, search/reference discovery, and nudge/warn
  versus gate workflows.

## P0 - Universal Agent-Project Foundation

### 1. Agent-Project Preset And Scaffold

Deliver:

- a future `init` preset named `agent-project`;
- a merge mode for applying that preset to an existing repository;
- default `.assura/config.yml`;
- `AGENTS.md`;
- `.agents/skills/`;
- `docs/process/`;
- `docs/learnings/`;
- default root hygiene, Markdown links, line limits, safe excludes, agent
  guidance, and skill folder checks.

Exit when a new coding-agent repo can run the preset and receive useful
structure, guidance, skill, Markdown, and content-readiness feedback without
hand-authoring a large config.

### 2. Reusable Dynamic Directory Contracts

Deliver dynamic repeated-structure contracts for cases like:

```yaml
structure:
  .agents/skills/*/:
    extra: false
    required:
      - SKILL.md
      - agents/openai.yaml
    optional_dirs:
      - references/
      - scripts/
      - assets/
```

This should support project-local skills, docs sections, components, packages,
services, examples, and test fixtures without explicitly listing every child.

### 3. Assura Doctor

Deliver:

- a future top-level `doctor` command with text output;
- JSON output for scripts and CI;
- agent-oriented output for coding-agent loops.

Doctor must report:

- configured checks;
- inactive models;
- model files that exist but are not wired into config;
- empty collections;
- zero search chunks;
- unresolved references;
- binary paths covered or excluded;
- inherited rules affecting target paths;
- recommended preset gaps;
- what passed, what was not configured, and what to do next.

### 4. Assura Explain Path

Deliver:

- a future path explanation command with text output;
- JSON output for scripts and CI;
- agent-oriented output for coding-agent loops.

Explain must show applied scopes, inherited rules, disabled inheritance,
skipped checks, binary/read behavior, suppressions, severity, and why each rule
does or does not apply.

## P1 - Agent Guidance And Skill Contracts

### 5. AGENTS.md Contract

Default required sections:

- Operating rules;
- Process docs vs skills;
- Skills;
- Anchors.

Validate stable heading anchors, maximum size, links to project-local skill
folders, and a clear separation between durable process docs and executable
skills.

### 6. SKILL.md Contract

Default required frontmatter:

- `name`;
- `description`;
- `applies_when`;
- optional `version`.

Default required sections:

- Workflow;
- Read as needed;
- Outputs;
- Guardrails.

Enforce concise progressive disclosure. Long workflows should route to
`references/` or `docs/process/` instead of bloating the skill entrypoint.

### 7. Skill Index Validation

Validate that `AGENTS.md` has a use-case-oriented skill index and that every
entry links to an existing `.agents/skills/<skill>/SKILL.md`.

### 8. Default Skill Folder Rules

Default contract:

- `SKILL.md` required;
- `agents/openai.yaml` required when the preset enables per-agent metadata;
- `references/` optional, Markdown-only, line-limited;
- `scripts/` optional, naming policy and executable policy configurable;
- `assets/` optional, binary/static allowed;
- unexpected folders forbidden by default.

## P2 - Search And Reference Discoverability

### 9. Raw Repo Search Fallback

Agents need discovery before a project is perfectly modeled.

Deliver:

- raw repository text search with JSON output;
- modeled content search with an explicit raw-search fallback mode.

Modeled search remains the higher-confidence path, but raw search should make
Assura useful on day one.

### 10. Frontmatter Repository-Reference Extraction

Make frontmatter path lists first-class repository references.

Example config:

```yaml
markdown:
  frontmatter_references:
    - source_documents
    - related
    - evidence
    - requirements
```

Assura should validate these paths and include them in reference graph output,
context packs, and affected-reference answers.

### 11. Reference Graph Discovery

Deliver:

- all-reference listing with JSON output;
- unresolved-reference listing with JSON output;
- agent-oriented reference summaries.

If a command reports unresolved reference counts, the next command must be able
to enumerate those edges directly.

### 12. Agent-Query Discoverability

Deliver:

- agent-query capability listing;
- gap-oriented agent query;
- next-action agent query;
- unresolved-reference agent query.

Keep deterministic capability names, but make them discoverable and include
suggested follow-up commands in JSON.

## P3 - Content Runtime Activation And Repo-Native Data

### 13. Content Init And Content Doctor

Deliver:

- content initialization from an `agent-project` template;
- content initialization from a `document-project` template;
- agent-oriented content doctor output.

Detect model files that exist but are not wired into config, configured empty
collections, schema-missing instances, zero search chunks, and relation
definitions with no edges.

### 14. Baseline Content Models

Provide common agent-project models:

- Decision;
- Task;
- Requirement;
- Evidence;
- Doc;
- SourceDocument;
- Finding;
- Skill;
- Process;
- Learning.

These are more universal than proposal scoring and support the top-down
project-intelligence story.

### 15. Source-Document Custody Model

Support a first-class binary custody pattern:

```text
source-documents/
  manifest.md
  files/
    *.pdf
    *.docx
```

Validate existence, naming, manifest links, optional checksum metadata, kind,
origin, related requirements, and MIME/extension expectations without reading
binary files as UTF-8.

## P4 - Nudge, Warn, And Gate Workflow

### 16. Advisory Versus Blocking Modes

Deliver either command modes:

- a nudge mode;
- a warn mode;
- a gate mode;

or equivalent config:

```yaml
modes:
  nudge:
    max_severity: info
    exit: 0
  warn:
    max_severity: warning
    exit: 0
  gate:
    min_blocking_severity: error
    exit: nonzero
```

### 17. Hook Profiles

Deliver:

- an agent-nudge hook profile;
- a pre-commit warning hook profile;
- a pre-push gate hook profile.

### 18. Next Best Fixes Output

Agent-facing output should include ranked next actions:

```json
{
  "next_actions": [
    {
      "priority": 1,
      "action": "Create missing specs/S-100.md or remove reference",
      "follow_up": "run the unresolved-reference discovery surface"
    }
  ]
}
```

## P5 - Domain Templates And Deeper Validation

### 19. Document-Project Preset

Deliver:

- a future `init` preset named `document-project`;
- baseline structure for `library/topics/`, `source-documents/`,
  `docs/process/`, `docs/learnings/`, `requirements/`, `evidence/`, and
  `decisions/`.

### 20. Requirements/Evidence Traceability

Support broadly useful checks:

- every high-priority requirement has coverage;
- every claim links to evidence;
- every evidence item has a source;
- every finding has owner/status.

### 21. Script-Backed Computed Checks

Start with script-backed computed checks for scoring, weighted criteria,
confidence adjustment, and rollups before building native computed fields.

### 22. Proposal/SBIR Domain Pack

Defer proposal/SBIR-specific behavior behind a domain pack. It may include
gates, weighted scores, confidence, review actions, portal submission
checklists, and final PDF/DOCX package checks via scripts, but it should not
drive the core product sequence.

## Definition Of Done

- A new agent project can be scaffolded with one command and gets useful
  default Assura checks.
- A clean Assura run distinguishes checks passed from checks not configured.
- Doctor and explain make active/inactive/inherited behavior visible to agents.
- AGENTS.md and project-local skills have enforceable default contracts.
- Raw search works before content models are active.
- Frontmatter references participate in the repository reference graph.
- Unresolved/all references can be listed without knowing the source or target
  first.
- Content models can be initialized and diagnosed intentionally.
- Binary source documents have a supported custody pattern.
- Nudge, warning, and merge-gate behavior is explicit.
- Agent-facing output includes next best fixes.
- Domain-specific proposal/SBIR behavior remains deferred behind templates or
  packs rather than bloating the core preset.

## Validation Commands

Planning-only updates to this backlog should run:

```bash
python3 ./.trellis/scripts/workflow_gate.py --platform codex
cargo run --quiet -- check --format json .
cargo xtask target-state
cargo xtask docs
cargo xtask evidence
git diff --check
```

Implementation goals should add focused CLI, fixture, and docs tests for each
surface before closure.

## Review Tasks

- R1: Confirm the backlog prevents a green check from implying unconfigured
  capabilities are complete.
- R2: Confirm the agent-project preset is broadly useful and not
  proposal-specific.
- R3: Confirm doctor and explain answer active, inactive, inherited, skipped,
  and next-action questions.
- R4: Confirm AGENTS.md and SKILL.md checks improve agent routing without
  turning every draft into a merge blocker.
- R5: Confirm raw search and reference discovery work before perfect modeling.
- R6: Confirm binary custody avoids UTF-8 reads while still validating
  existence and manifest/reference integrity.
- R7: Confirm nudge/warn/gate modes map to agent workflow, pre-commit, and
  merge lifecycle.

## Reviewer Blocking Criteria

Block if the program lets `assura check` imply operational completeness when
models, search, references, skills, or hooks are inactive; if the preset is too
domain-specific; if dynamic contracts require enumerating every skill or
package manually; if binary files are read as UTF-8; or if nudge/warn/gate
behavior remains implicit.

## Kickoff Text

Use this prompt to start the large goal-driven work:

Execute `docs/goals/assura-agent-ready-project-onboarding-program.md`.

Revalidate the goal against the current roadmap, existing init/content/query
surfaces, support-policy wording, agent integration lifecycle, document graph
support, and current self-check behavior before coding. Build the next product
milestone around making Assura the default scaffold, doctor, and feedback loop
for agent-ready repos: agent-project preset, dynamic directory contracts,
doctor and explain surfaces, AGENTS.md and SKILL.md contracts, skill index
validation, raw search fallback, frontmatter repository references, reference
graph discovery, content initialization and content doctor, baseline content
models, source-document custody, nudge/warn/gate modes, hook profiles, and
next-best-fix output. Keep proposal/SBIR scoring as a later domain pack.
