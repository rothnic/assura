# Agent-Ready Onboarding

## 1. Scope / Trigger

This contract applies when `assura agent onboard` detects a repository,
materializes project policy, writes the onboarding packet, or changes the
machine-readable onboarding report.

## 2. Signature

```text
assura agent onboard [PATH] --agent <auto|codex|opencode|claude|pi|generic> --format <text|json|yaml>
```

Implementation surfaces:

- `src/cli/agent_onboarding.rs` detects the project and merges generated files.
- `src/cli/agent_onboarding_templates.rs` owns project-policy templates.
- `src/cli/agent_onboarding_report.rs` defines serialized report fields.

## 3. Contracts

Onboarding materializes ordinary, editable `agentic-core` and
`structure-health` YAML into `.assura/config.yml`. Runtime validation depends
only on that project-owned file; it must not require hidden recipe lookup.

The report includes `rule_recommendations[]` with:

| Field | Contract |
| --- | --- |
| `preset` | Materialized recipe names, currently `agentic-core + structure-health`. |
| `local_rule` | Primary editable rule reference, currently `$agent-entrypoint`. |
| `status` | `applied`, `available`, `not-applied`, or `conflict`. |
| `reason` | Concise explanation tied to detected project evidence. |
| `includes` | Project-owned reusable rules installed by onboarding. |

Onboarding writes `.assura/onboarding/rules.md` so an agent can inspect the
active policy and its customization point. Language, framework, naming,
layout, and domain policy remain undecided until supported evidence or a user
decision exists.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Empty or unknown project | Apply only language-agnostic agentic recipes. |
| Recognized project type | Report the type; do not invent language policy. |
| New config | Write project-owned rules and implicit-root Option A structure. |
| Existing config | Preserve user values and recursively merge missing recipe values. |
| Legacy `structure: ./` | Normalize the wrapper before merging current notation. |
| Existing recipe collision | Preserve user policy and report `conflict`. |
| Alternate `--config` | Derive status from the selected config. |
| Generated baseline fails verification | Return validation failure. |

## 5. Good / Base / Bad Cases

- Good: the project receives editable `agent-entrypoint`, `skill`,
  `folder-health`, and closed-directory rules plus explicit structure uses.
- Base: an empty repository receives the same broad rules without invented
  language, naming, or framework policy.
- Bad: current onboarding references hidden `$agentic-project` or
  `$project-agentic-baseline` aliases.
- Bad: merging replaces a project-authored value without `--force`.

## 6. Tests Required

- JSON report coverage for every recommendation field.
- Existing-config merge and conflict coverage.
- Dynamic skill-directory contract coverage.
- Generated config verification through `assura check`.
- Binary source-document custody coverage.
- Landing-page coverage separating applied policy from undecided policy.

## 7. Canonical Shape

```yaml
rules:
  agent-entrypoint:
    max_lines: 160
    severity: low
    message: See docs/process/agent-workflow.md.

structure:
  AGENTS.md: exists:1 | $agent-entrypoint
  README.md: exists:1
```
