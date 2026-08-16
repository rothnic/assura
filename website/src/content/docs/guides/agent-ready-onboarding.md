---
title: Agent-Ready Onboarding
description: Bootstrap Assura, inspect project evidence, and define a project-owned repository shape.
---

Use this path when a coding agent is entering a new or existing repository and
needs a broad, truthful baseline before it starts adding project-specific
rules.

The local command is:

```bash
assura agent onboard .
```

That command creates or preserves a broad baseline, writes an onboarding
packet, runs local verification, and tells the agent how to inspect the detected
stack and intentional layout before defining project-owned rules. It does not
silently wire global host-agent configuration, treat every observed path as
intentional, or turn a green check into a claim that the project is fully
specialized.

The generated packet is Assura-owned operational state. `assura check` validates
the user-authored project contract, while onboarding verification and
`assura doctor` report whether the core handoff file is present.

## First-Run Phases

1. Install Assura with the release installer from the
   [Installation guide](/guides/installation/).
2. Run `assura agent onboard .` from the project root.
3. Read the generated report sections: `installed`, `detected`,
   `rule_recommendations`, `verified`, `inactive`, `lifecycle_profiles`, and
   `next_actions`.
4. Open `.assura/onboarding/agent-next.md`.
5. Inspect manifests, tooling, generated outputs, project docs, and established
   paths; define the expected stack and intended shape from converging evidence.
6. Close stable scopes against unexpected files and ask only where the evidence
   is ambiguous or a choice would reject existing content.

Roadmap note: a future remote bootstrap wrapper may install Assura and delegate
to the installed CLI. Today, use the installer plus `assura agent onboard`; do
not treat any remote bootstrap command as a current quickstart command.

## Current Command

```bash
assura agent onboard .
```

Use a concrete host-agent label plus `--activate` when you want Assura to
generate the reviewable bundle and explicitly activate its project-local host
integration:

```bash
assura agent onboard . --agent codex --activate --format json
assura agent integration doctor codex .
```

The bundle lives under `.assura/integrations/<agent>/`. Explicit activation
patches only Assura-owned project-local host entries, verifies the result, and
preserves unmanaged configuration. Use `assura agent integration deactivate`
to remove the host entries while retaining the bundle.

## Agent Prompt

Give the coding agent a short instruction like this when it enters a repository:

```text
Set up Assura for this project.

1. Run: assura agent onboard .
2. Read the report sections: installed, detected, rule_recommendations,
   verified, inactive, lifecycle_profiles, and next_actions.
3. Open .assura/onboarding/agent-next.md.
4. Treat inactive entries as unchecked, not as passing.
5. Inspect manifests, tooling, generated output, and established paths. Define
   project-owned rules for the expected stack and intentional layout.
6. Close stable scopes so unexpected files fail. Preserve legitimate paths and
   ask only where evidence is ambiguous.
7. Verify with assura review and assura check --format agent.
```

The flow is intentionally simple:

| Step | Agent action | Output |
| --- | --- | --- |
| 1 | Run onboarding | `.assura/config.yml` and `.assura/onboarding/` |
| 2 | Read checked state | `verified`, `inactive`, and `doctor.json` |
| 3 | Read handoff | `.assura/onboarding/agent-next.md` |
| 4 | Specialize from evidence | Project-owned expected shape and stable closed scopes |
| 5 | Ask focused questions | Only unresolved or potentially destructive choices |
| 6 | Validate again | `assura review`, `assura check`, and `assura doctor` |

## Report Shape

The onboarding JSON is meant for both humans and agents. The important sections
look like this:

```json
{
  "installed": {
    "config": ".assura/config.yml",
    "onboarding_packet": ".assura/onboarding/"
  },
  "detected": {
    "project_type": "rust",
    "agent_harness": "codex"
  },
  "rule_recommendations": [
    {
      "preset": "agentic-core + structure-health",
      "local_rule": "$agent-entrypoint",
      "status": "applied",
      "reason": "rust project detected; editable agentic-core and structure-health policy is active",
      "includes": [
        "$agent-entrypoint",
        "$skill-entrypoint",
        "$skill",
        "$folder-health",
        "$closed"
      ]
    }
  ],
  "content": {
    "template": "none",
    "status": "inactive"
  },
  "lifecycle_profiles": [
    {
      "name": "agent-working-loop",
      "mode": "nudge",
      "blocking": false,
      "command": "assura agent nudge --event before-tool --changed <path> --format json ."
    },
    {
      "name": "pre-commit-warning",
      "mode": "warn",
      "blocking": false,
      "command": "assura check --format agent --warn --min-severity low --max-issues 10 ."
    },
    {
      "name": "pre-push-or-ci-gate",
      "mode": "gate",
      "blocking": true,
      "command": "assura check --format agent --min-severity medium --max-issues 20 ."
    }
  ],
  "verified": [
    { "name": "structure_config", "status": "pass" },
    { "name": "onboarding_packet", "status": "pass" }
  ],
  "inactive": [
    { "name": "project_specialization", "status": "inactive" },
    { "name": "content_models", "status": "inactive" }
  ],
  "next_actions": [
    {
      "priority": 1,
      "action": "Read the onboarding handoff",
      "follow_up": ".assura/onboarding/agent-next.md"
    },
    {
      "priority": 2,
      "action": "Ask remaining specialization questions",
      "affected_paths": [".assura/onboarding/questions.md"],
      "follow_up": ".assura/onboarding/questions.md"
    }
  ]
}
```

`verified` means Assura checked the configured baseline. `inactive` means the
capability is deliberately not configured yet. A clean `assura check` result is
not the same thing as a fully onboarded repository.

Assura materializes the `agentic-core` and `structure-health` recipes as normal
YAML in `.assura/config.yml`. The project owns every generated rule and can
edit or remove it without relying on hidden runtime presets. Recommendation
status is `applied` when the generated entrypoint rule is active, `available`
when recipe rules exist without replacing the selected root policy,
`not-applied` when the selected config lacks them, and `conflict` when an
existing project value was preserved for review. Assura does not recommend
language, framework, naming, or domain rules until the project provides enough
evidence or a user confirms those choices.

## Generated Packet

`assura agent onboard` writes a small packet under `.assura/onboarding/`:

| File | Purpose |
| --- | --- |
| `summary.md` | What Assura detected and installed. |
| `rules.md` | Materialized recipes, active project rules, and how to customize them. |
| `questions.md` | The specialization questions the agent should ask. |
| `agent-next.md` | The next handoff for coding agents. |
| `lifecycle.md` | When to use nudge, warn, and gate feedback. |
| `doctor.json` | A project doctor snapshot showing checked and unchecked state. |

The agent should read `agent-next.md`, inspect repository evidence, and
materialize supported language, layout, and naming decisions into the
project-owned config. It should ask before making ambiguous choices or closing
a scope that rejects existing content.

## Project-Local Skills

Onboarding installs Assura guidance as project-local skills under
`.agents/skills/`. Agents should load these local skills from `AGENTS.md`
routing; Assura does not silently mutate host-agent or global skill
configuration.

The generated `assura-structure-fit` skill defines `STRUCTURE_FIT_CHECK`, a
compact anchor for structure mismatch feedback. When a new file or directory
does not fit `.assura/config.yml`, agents should apply that check before
editing config: inspect existing structure, prefer reuse or rename, and add a
new config rule only for a durable non-duplicative project role.

## Agent-Next Questions

The generated `agent-next.md` tells the agent to answer these from repository
evidence first, then ask only for unresolved choices:

- primary language or stack;
- project type;
- file naming convention;
- source and test layout;
- docs strictness;
- hook lifecycle preference;
- required project-specific files or folders;
- source-document custody needs;
- whether typed content models should be activated.

Record the answers in project notes or `.assura/onboarding/answers.yml` before
specializing the broad baseline.

## Checked Versus Unchecked

Use `assura doctor` when the agent needs to explain what is active, inactive,
or incomplete:

```bash
assura doctor . --format json
assura explain AGENTS.md --format json
```

Doctor output reports configured checks, inactive capabilities, generated
packet state, content-model state, binary custody state, gaps, and ranked next
actions. Explain output shows the effective checks and next actions for one
path.

## Content And Project Packs

The default `--content-template none` keeps content models inactive until the
user chooses a template.

Use the broad agent-project template when the repository should model common
project facts:

```bash
assura agent onboard . --content-template agent-project --format json
```

Use the document-project template for research-authoring projects: academic
research and content authoring work such as literature reviews, papers, theses,
reports, and knowledge bases.

```bash
assura agent onboard . --content-template document-project --format json
```

Document projects add `source-documents/`, `library/topics/`, `docs/drafts/`,
and `docs/final/` on top of the broad project records. They help an agent keep
source material, research notes, drafts, evidence, and final outputs linked
without requiring binary files to be read as text.

## Lifecycle Profiles

The onboarding report and `.assura/onboarding/lifecycle.md` use the same three
modes:

| Mode | Use | Command |
| --- | --- | --- |
| `nudge` | Agent working loop and path-aware tool events | `assura agent nudge --event before-tool --changed <path> --format json .` |
| `warn` | Draft work or pre-commit feedback | `assura check --format agent --warn --min-severity low --max-issues 10 .` |
| `gate` | Pre-push, merge, or CI checks | `assura check --format agent --min-severity medium --max-issues 20 .` |

`warn` reports without blocking. `gate` preserves the configured severity
contract and exits nonzero for blocking findings.

## Specialization Flow

After the user answers the generated questions:

1. Update `.assura/config.yml` with the chosen language, layout, naming, and
   strictness rules.
2. Activate `agent-project` or `document-project` when the user wants modeled
   facts.
3. Activate a supported project-local host integration only when the user wants
   Assura feedback in that harness.
4. Rerun:

   ```bash
   assura check --format json .
   assura doctor . --format json
   ```

The goal is a repository that tells the agent what is checked, what is still
unchecked, and what the next user-backed specialization step should be.
