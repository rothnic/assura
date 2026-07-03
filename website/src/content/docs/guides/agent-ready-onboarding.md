---
title: Agent-Ready Onboarding
description: Bootstrap a broad Assura baseline, inspect what is checked, and hand specialization back to the user.
---

Use this path when a coding agent is entering a new or existing repository and
needs a broad, truthful baseline before it starts adding project-specific
rules.

The current experimental local command is:

```bash
assura agent onboard . --agent auto --format json
```

That command creates or preserves a broad baseline, writes an onboarding
packet, runs local verification, and tells the agent which questions still need
human answers. It does not silently wire global host-agent configuration, guess
domain rules, or turn a green check into a claim that the project is fully
specialized.

## First-Run Phases

1. Install Assura with the release installer from the
   [Installation guide](/guides/installation/).
2. Run `assura agent onboard . --agent auto --format json` from the project
   root.
3. Read the generated report sections: `installed`, `detected`, `verified`,
   `inactive`, `lifecycle_profiles`, and `next_actions`.
4. Open `.assura/onboarding/agent-next.md`.
5. Ask the user only the remaining specialization questions.
6. Specialize the config, content templates, or hooks only after those answers
   are recorded.

Roadmap note: a future remote bootstrap wrapper may install Assura and delegate
to the installed CLI. Today, use the installer plus `assura agent onboard`; do
not treat any remote bootstrap command as a current quickstart command.

## Current Command

```bash
assura agent onboard . --agent auto --format json
```

Use a concrete host-agent label only when you want Assura to generate an
experimental, reviewable local integration bundle:

```bash
assura agent onboard . --agent codex --format json
assura agent integration doctor codex .
```

The bundle lives under `.assura/integrations/<agent>/`. Assura leaves host
configuration as manual opt-in so the generated files can be reviewed and
removed.

## Agent Prompt

Give the coding agent a short instruction like this when it enters a repository:

```text
Run Assura onboarding before changing project structure.

1. Run: assura agent onboard . --agent auto --format json
2. Read the report sections: installed, detected, verified, inactive,
   lifecycle_profiles, and next_actions.
3. Open .assura/onboarding/agent-next.md.
4. Treat inactive entries as unchecked, not as passing.
5. Ask the user the remaining questions before adding language, layout,
   naming, source-document, hook, or content-model rules.
6. Use warn mode while drafting and gate mode before push or CI.
```

The flow is intentionally simple:

| Step | Agent action | Output |
| --- | --- | --- |
| 1 | Run onboarding | `.assura/config.yml` and `.assura/onboarding/` |
| 2 | Read checked state | `verified`, `inactive`, and `doctor.json` |
| 3 | Read handoff | `.assura/onboarding/agent-next.md` |
| 4 | Ask before specializing | User-backed answers, not invented conventions |
| 5 | Validate again | `assura check` and `assura doctor` |

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

## Generated Packet

`assura agent onboard` writes a small packet under `.assura/onboarding/`:

| File | Purpose |
| --- | --- |
| `summary.md` | What Assura detected and installed. |
| `questions.md` | The specialization questions the agent should ask. |
| `agent-next.md` | The next handoff for coding agents. |
| `lifecycle.md` | When to use nudge, warn, and gate feedback. |
| `doctor.json` | A project doctor snapshot showing checked and unchecked state. |

The agent should read `agent-next.md` before it changes language, layout,
naming, traceability, source-document, hook, or domain conventions.

## Agent-Next Questions

The generated `agent-next.md` asks for the missing choices that Assura should
not invent:

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
3. Add host-agent integration bundles only for supported adapters the user
   wants to wire manually.
4. Rerun:

   ```bash
   assura check --format json .
   assura doctor . --format json
   ```

The goal is a repository that tells the agent what is checked, what is still
unchecked, and what the next user-backed specialization step should be.
