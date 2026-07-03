---
id: goal-assura-agent-ready-project-onboarding-program
type: goal
title: Assura agent-ready project onboarding program
status: planned
created: 2026-07-02
owners:
  - assura-maintainers
related:
  - ./assura-agent-onboarding-bootstrap-command.md
  - ./assura-agent-project-preset-dynamic-contracts.md
  - ./assura-agent-doctor-explain-feedback.md
  - ./assura-agent-guidance-skill-contracts.md
  - ./assura-agent-search-reference-discovery.md
  - ./assura-agent-content-activation-source-docs.md
  - ./assura-agent-lifecycle-hooks-next-actions.md
  - ./assura-website-agent-onboarding-experience.md
  - ./assura-agent-document-project-preset.md
  - ./assura-agent-requirements-evidence-traceability.md
  - ./assura-agent-script-backed-computed-checks.md
  - ./assura-agent-proposal-sbir-domain-pack.md
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

## First-Run Product Surface

The desired product surface is one bootstrap action that does four jobs:

1. install or update Assura if it is missing;
2. create a safe broad agent-ready baseline;
3. install the right hooks or adapters for the detected agent harness;
4. verify setup and tell the agent exactly what to ask the user next.

The remote bootstrap wrapper should be a convenience script hosted by Assura.
Its only durable responsibility is to install or update Assura, verify the
binary is available, and delegate to the real CLI. The installed CLI owns the
actual onboarding workflow through a future agent-onboarding subcommand with
project-type detection, agent-harness detection, apply, and verify options.

The first-run rule is: start broad, verify, then ask. Do not ask the user 20
questions before creating a useful baseline. Apply low-risk defaults first,
then produce the short specialization questions an agent should ask before it
adds project-specific conventions.

## Bootstrap Phases

### Phase 1: Install

- Install or update Assura.
- Verify the binary is on `PATH`.
- Pin and report the installed version.
- Avoid hidden global side effects beyond the documented install location.

### Phase 2: Inspect

- Detect whether the directory is empty, existing, a git repository, a
  monorepo, docs-heavy, Rust, Node, Python, web app, or unknown.
- Detect the agent harness: Codex, Claude, Cursor, GitHub Actions, generic
  shell, or unknown.
- Detect existing `AGENTS.md`, `.agents/`, `README`, package files,
  `Cargo.toml`, `pyproject.toml`, docs, source directories, and test
  directories.
- Classify confidence. If detection is uncertain, apply low-risk defaults and
  defer specialization.

### Phase 3: Apply Broad Agent Baseline

- Create `.assura/config.yml` if missing.
- Add broad safe defaults if config already exists.
- Create or merge `AGENTS.md`.
- Create `.agents/skills/` baseline structure.
- Create `docs/process/` and `docs/learnings/`.
- Add binary and source-document custody defaults.
- Add Markdown, link, line-limit, and safe-exclude defaults.
- Add reusable skill-directory rules.

### Phase 4: Install Harness Integration

- Install advisory Assura hooks for the detected or requested agent harness.
- Add Codex, Claude, Cursor, GitHub Actions, or generic shell adapter config
  only when support is known.
- If adapter support is uncertain, install generic hooks and print exact
  manual steps rather than pretending unsupported integrations are active.
- Default lifecycle behavior to nudge while working, warn before commit, and
  gate before merge.

### Phase 5: Apply Project-Type Pack

- If project type is explicit, add best-practice examples and checks for that
  pack.
- If project type is auto-detected with high confidence, apply low-risk pack
  defaults.
- If uncertain, do not guess. Write the specialization questions and leave the
  project in a valid broad baseline.

### Phase 6: Verify

- Run the configured check.
- Run the future doctor surface.
- Explain key generated files and scopes.
- Report checked versus unchecked capabilities.
- Do not treat "no violations" as equivalent to "fully onboarded."

### Phase 7: Tell The Agent What To Ask Next

- Write a concise question list.
- Tell the agent not to invent project conventions.
- Point the agent at the generated next-step file.
- Include the next specialization action after the user answers.

## Successful Bootstrap Output Contract

A successful first run should not primarily say "no violations found." It
should say that the project has a known baseline, the agent knows what is
checked and unchecked, which hooks are active, and which user choices remain.

The output should include:

- installed Assura version;
- installed or merged `.assura/config.yml`;
- installed or merged `AGENTS.md` baseline;
- `.agents/skills/` contract status;
- `docs/process/` and `docs/learnings/` status;
- agent nudge hook status;
- pre-commit warning hook status;
- pre-push or CI gate profile status;
- detected project type;
- detected agent harness;
- git repository status;
- existing source-file status;
- verified active checks;
- inactive capabilities;
- recommended next action for the agent.

## Generated Onboarding Packet

The bootstrap should create a small predictable packet:

```text
.assura/
  config.yml
  presets.lock.yml
  onboarding/
    summary.md
    questions.md
    agent-next.md
    doctor.json
  examples/
    agent-project/
      AGENTS.example.md
      SKILL.example.md

AGENTS.md

.agents/
  skills/
    assura-project-maintenance/
      SKILL.md
      agents/
        openai.yaml
      references/
        assura-onboarding.md

docs/
  process/
    agent-workflow.md
  learnings/
    README.md
```

The most important generated file is `.assura/onboarding/agent-next.md`.

## Agent-Next File Contract

`agent-next.md` is written for coding agents. It should say that Assura is
installed and the broad agent-ready baseline is active, then instruct the
agent not to invent project conventions.

It should ask the user these specialization questions:

1. What primary language or stack should this project use?
2. What project type is this: library, app, docs site, proposal, research
   repo, data project, monorepo, or other?
3. What file naming convention should apply: kebab-case, snake_case,
   PascalCase, or mixed by folder?
4. What source layout should the project use?
5. What test layout should the project use?
6. Should docs be strict from day one or advisory until the first milestone?
7. Should agent hooks be advisory only while working and blocking only in CI?
8. Are there required files or folders specific to this project?
9. Are there binary or source documents that should be tracked by manifest
   instead of read as text?
10. Should Assura create typed content models for tasks, decisions,
    requirements, evidence, or source documents?

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

## Execution Sequence

Execute these child goals in order unless revalidation proves a different
dependency order is necessary:

1. [Agent Onboarding Bootstrap Command](./assura-agent-onboarding-bootstrap-command.md)
   creates the first-run bootstrap/onboard flow, install-and-delegate wrapper
   contract, generated onboarding packet, and specialization handoff.
2. [Agent Project Preset And Dynamic Contracts](./assura-agent-project-preset-dynamic-contracts.md)
   implements the broad agent-project baseline and reusable repeated-directory
   contracts for skills and other project-local structures.
3. [Agent Doctor Explain Feedback](./assura-agent-doctor-explain-feedback.md)
   adds checked-versus-unchecked doctor output and path-level explanation for
   scopes, inheritance, skips, suppressions, and next actions.
4. [Agent Guidance And Skill Contracts](./assura-agent-guidance-skill-contracts.md)
   makes `AGENTS.md`, `SKILL.md`, skill indexes, and skill folders enforceable
   without overfitting a single project.
5. [Agent Search And Reference Discovery](./assura-agent-search-reference-discovery.md)
   adds raw search fallback, frontmatter repository references, all/unresolved
   reference listing, and agent-query discoverability.
6. [Agent Content Activation And Source Docs](./assura-agent-content-activation-source-docs.md)
   adds content initialization, content doctor, baseline content models, and
   source-document custody.
7. [Agent Lifecycle Hooks And Next Actions](./assura-agent-lifecycle-hooks-next-actions.md)
   formalizes nudge/warn/gate lifecycle modes, hook profiles, and ranked
   next-best-fix output.
8. [Website Agent Onboarding Experience](./assura-website-agent-onboarding-experience.md)
   rewrites the website onboarding path around the actual first-run agent
   journey and the checked-versus-unchecked mental model.
9. [Agent Document Project Preset](./assura-agent-document-project-preset.md)
   layers a reusable document-project pack on top of the broad agent baseline.
10. [Agent Requirements Evidence Traceability](./assura-agent-requirements-evidence-traceability.md)
    adds reusable requirements, claims, evidence, findings, and coverage checks.
11. [Agent Script Backed Computed Checks](./assura-agent-script-backed-computed-checks.md)
    creates a controlled extension point for scores, rollups, and derived
    validations before native computed fields exist.
12. [Agent Proposal SBIR Domain Pack](./assura-agent-proposal-sbir-domain-pack.md)
    packages proposal-specific gates, scoring, review, and final package checks
    as an optional domain pack.

## P0 - Universal Agent-Project Foundation

### 1. Agent-Project Preset And Scaffold

Deliver:

- the future agent onboarding flow as the user-facing first-run entrypoint;
- the remote bootstrap wrapper as install-and-delegate convenience, not as the
  source of product behavior;
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

The default baseline should be broad and safe. It should require `AGENTS.md`,
`.assura/config.yml`, `.agents/skills/`, `docs/process/`, and
`docs/learnings/`; recommend `README.md` and `.gitignore`; and check root
clutter, unexpected binary reads, Markdown links, `AGENTS.md` size and
sections, reusable skill contracts, unexpected skill folders, skill
references/scripts/assets conventions, and advisory Markdown heading behavior.

### 2. Reusable Dynamic Directory Contracts

Deliver dynamic repeated-structure contracts for cases like:

```yaml
structure:
  ".agents/skills/*/":
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

## Project-Type Packs

Project-type packs layer examples and checks after the broad baseline:

- `agent-project`: `AGENTS.md`, skills, hooks, `docs/process/`, and
  `docs/learnings/`.
- `document-project`: source-document manifest, `library/topics/`,
  `docs/drafts/`, `docs/final/`, and evidence.
- `rust`: `Cargo.toml`, `src/`, `tests/`, `benches/`, `examples/`, and Rust
  naming conventions.
- `node`: `package.json`, `src/`, `test/`, docs, and package-manager
  detection.
- `python`: `pyproject.toml`, `src` package layout, `tests/`, and docs.
- `web-app`: `src/`, routes or pages, components, public assets, and tests.
- `monorepo`: `packages/*`, `apps/*`, shared docs, and workspace config.
- `proposal-project`: requirements, evidence, source documents, scoring, and
  review findings.

Packs should be composable. Domain packs remain later layers, not requirements
for the broad agent-project baseline.

## Agent Harness Adapters

The onboarding flow should support auto-detected and explicit harness adapters:

- auto;
- Codex;
- Claude;
- Cursor;
- GitHub Copilot or GitHub Actions where appropriate;
- generic shell.

Each adapter should install only supported behavior. For Codex-like adapters,
the expected shape is project hook guidance, a project-local
`assura-project-maintenance` skill, supported hook or wrapper metadata when
available, and delegation to shared Assura agent/check/daemon contracts. The
generic adapter should create shell hooks, write `AGENTS.md` instructions, and
provide check commands without claiming host-agent integration.

If adapter support is uncertain, the onboarding flow must install generic
hooks and print exact manual steps. It must not pretend unsupported
integrations are active.

## Hook Profile Semantics

The default hook profile should match the product philosophy:

- nudge while working;
- warn before commit;
- gate before merge.

The future default profile should map working-tree or agent events to
advisory output with exit 0, pre-commit to warning behavior, and pre-push or
CI to blocking behavior on configured errors.

## Specialization Flow

After the user answers the generated questions, the agent should run a second
specialization flow. It should accept answers from
`.assura/onboarding/answers.yml` or an interactive equivalent, then turn those
answers into project-specific rules.

The first bootstrap command must still be enough to create a valid broad
baseline. Specialization is for adding language, layout, naming, typed content
models, source-document custody, traceability, and domain-specific behavior.

## Ideal UX

The intended user flow is:

1. The user asks an agent to set up the new repository with Assura.
2. The agent runs one bootstrap action from the project root.
3. Assura installs itself if needed, creates a broad agent-ready config,
   installs advisory hooks, verifies setup, and writes next questions.
4. The agent reports that the broad baseline is active.
5. The agent asks the user only the remaining specialization questions instead
   of inventing project conventions.

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
- The onboarding packet includes `summary.md`, `questions.md`,
  `agent-next.md`, and `doctor.json`.
- The generated `agent-next.md` tells agents not to invent project conventions
  and provides the required specialization questions.
- The remote bootstrap wrapper installs and delegates; the installed CLI owns
  the onboarding behavior.
- The document-project preset, requirements/evidence traceability, computed
  checks, and proposal/SBIR pack are captured as separately executable goals.
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
- R8: Confirm the bootstrap flow starts broad, verifies, and then asks only
  specialization questions.
- R9: Confirm generated onboarding files give agents enough instruction to stop
  and ask instead of guessing conventions.

## Reviewer Blocking Criteria

Block if the program lets `assura check` imply operational completeness when
models, search, references, skills, or hooks are inactive; if the preset is too
domain-specific; if dynamic contracts require enumerating every skill or
package manually; if binary files are read as UTF-8; or if nudge/warn/gate
behavior remains implicit. Also block if the remote bootstrap script owns
product behavior instead of delegating to the installed CLI, or if first-run
onboarding asks excessive questions before establishing a safe baseline.

## Progress Log

| Date | Update | Evidence |
| --- | --- | --- |
| 2026-07-02 | Completed child goal 4, `assura-agent-guidance-skill-contracts`. The agent-project baseline now has opt-in `extensions.agent_guidance` checks for `AGENTS.md`, project-local `SKILL.md` entrypoints, skill index links, required routing sections, concise progressive disclosure, and compiled config artifacts. The next executable child goal is `assura-agent-search-reference-discovery`. | `docs/goals/assura-agent-guidance-skill-contracts.md`; `tests/agents_md.rs`; `tests/skill_contract.rs`; `crates/assura-check-cli/tests/compiled_agent_guidance_cli.rs`; independent review `McClintock`; `cargo fmt --check`; `cargo test agents_md --quiet`; `cargo test skill_contract --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`; `git diff --check`. |
| 2026-07-02 | Completed child goal 2, `assura-agent-project-preset-dynamic-contracts`. The agent-project onboarding baseline now uses a reusable skill-directory contract, dynamic contracts work for packages, docs sections, examples, and fixtures, and review/CI evidence is green; the next executable child goal is `assura-agent-doctor-explain-feedback`. | `docs/goals/assura-agent-project-preset-dynamic-contracts.md`; PR #139 `mergeStateStatus=CLEAN`; `cargo test --test project_intelligence_onboarding --quiet`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`. |
| 2026-07-02 | Completed child goal 3, `assura-agent-doctor-explain-feedback`. `assura doctor` and `assura explain` now make checked versus unchecked state explicit, agent output carries next actions plus inherited/skipped/severity context, and the next executable child goal is `assura-agent-guidance-skill-contracts`. | `docs/goals/assura-agent-doctor-explain-feedback.md`; PR #139 `mergeStateStatus=CLEAN` at `68981993fcb4dac5e889076281d8485628e60ee7`; all PR checks successful; `cargo test --test doctor_explain_cli --quiet`; `cargo xtask target-state`; `cargo xtask docs`; `cargo xtask evidence`. |
| 2026-07-02 | Started child goal 2 by converting the generated onboarding baseline from a one-off skill directory shape into a reusable dynamic skill-directory contract. Captured directory scopes now apply exact child requirements correctly, and generated onboarding configs validate multiple skills without enumerating every skill name. | `docs/goals/assura-agent-project-preset-dynamic-contracts.md`; `src/config/config/structure_notation.rs`; `src/cli/check/rule_plan.rs`; `src/cli/agent_onboarding_templates.rs`; `agent_onboard_generated_config_validates_dynamic_directory_skill_contracts`. |
| 2026-07-02 | Implemented the installed local CLI portion of child goal 1: `assura agent onboard` now generates the broad baseline and onboarding packet, preserves existing user files, reports installed/detected/verified/inactive/next-action sections, and leaves remote bootstrap plus later specialization as future surfaces. | `docs/goals/assura-agent-onboarding-bootstrap-command.md`; `cargo test --test project_intelligence_onboarding --quiet`; `cargo test --test cli_command_surface_tests --quiet`; `cargo test --test agent_surface_cli --quiet`; `cargo run --quiet -- check --format json .`; `cargo xtask docs`; `cargo xtask evidence`; `cargo check --workspace --all-targets --all-features --quiet`. |
| 2026-07-02 | Started implementation of the first child goal, `assura-agent-onboarding-bootstrap-command`, after revalidating the live roadmap, support policy, agent feedback direction lock, agent integration lifecycle, website onboarding gap, and current self-check behavior. The implementation slice is scoped to the installed local CLI onboarding surface; the remote wrapper and later specialization flow remain planned contracts until implemented. | `.trellis/tasks/07-02-agent-onboarding-bootstrap-command-implementation/prd.md`; `.trellis/spec/assura/roadmap.md`; `docs/support-policy.md`; `.trellis/spec/assura/codex-agent-feedback.md`; `docs/goals/assura-agent-integration-lifecycle.md`; `cargo run --quiet -- check --format json .` reported 0 violations across 1403 files. |

## Kickoff Text

Use this prompt to start the large goal-driven work:

Execute `docs/goals/assura-agent-ready-project-onboarding-program.md`.

This increment excludes the performance backlog. Revalidate the goal against the
current roadmap, existing init/content/query surfaces, support-policy wording,
agent integration lifecycle, document graph support, website onboarding pages,
and current self-check behavior before coding. Build the next product milestone
around making Assura the default scaffold, doctor, and feedback loop for
agent-ready repos: one-command bootstrap, install-and-delegate remote wrapper,
agent onboarding flow, generated onboarding packet, `agent-next.md`,
agent-project preset, dynamic directory contracts, doctor and explain surfaces,
AGENTS.md and SKILL.md contracts, skill index validation, raw search fallback,
frontmatter repository references, reference graph discovery, content
initialization and content doctor, baseline content models, source-document
custody, nudge/warn/gate modes, hook profiles, specialization flow,
next-best-fix output, a much clearer website onboarding experience,
document-project preset, requirements/evidence traceability, controlled
script-backed computed checks, and an optional proposal/SBIR domain pack. Work
through the linked child goals in order and keep domain-specific behavior out of
the core presets.
