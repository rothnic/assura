# Agent-Ready Onboarding

## 1. Scope / Trigger

This contract applies when `assura agent onboard` detects a repository,
materializes project policy, writes the onboarding packet, or changes the
machine-readable onboarding report.

## 2. Signature

```text
assura agent onboard [PATH] [--recipe-file <PATH>] --agent <auto|codex|opencode|claude|pi|generic> --format <text|json|yaml>
```

The default first-run form is `assura agent onboard .`; `--agent auto` and
`--format json` are implicit. Host activation remains explicit because an
unknown or ambiguous host must not be guessed.

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
active policy and its customization point. The generated handoff tells the
agent to inspect manifests, tooling, documentation, generated outputs, and
established paths before specializing the contract. Evidence-backed language,
framework, naming, and layout policy should be materialized as project-owned
rules; only ambiguous or potentially destructive choices require a user
decision.

### Independent initialization-evaluation guidance evidence

`scripts/evaluate-agent-init.py` accepts the fixture-owned optional contract
field `guidance_assertions[]`:

```json
{"id":"evidence-first-handoff","path":".assura/onboarding/agent-next.md","contains":"Inspect explicit repository instructions"}
```

Each record requires a nonempty id, repository-relative nonempty path, and
nonempty required text fragment. Evaluation reads it only from a disposable
copy. Missing files/fragments fail the requested `guidance` dimension; omitting
the field leaves that dimension `unavailable`, never passing. Symlinked files
or path components are refused so external content cannot satisfy an assertion.
Public evaluator output exposes only aggregate state, not paths, ids, or text.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Empty or unknown project | Apply only language-agnostic agentic recipes. |
| Recognized project type | Report the type; do not invent language policy. |
| New config | Write project-owned rules and implicit-root Option A structure. |
| Existing config | Preserve user values and recursively merge missing recipe values. |
| Legacy `structure: ./` | Normalize the wrapper before merging current notation. |
| Existing recipe collision | Preserve user policy and report `conflict`. |
| Explicit `--recipe-file` | Read only the supplied local YAML, validate and atomically merge it, record source path/SHA-256, and preserve conflicts without a partial config write. |
| Alternate `--config` | Derive status from the selected config. |
| Generated baseline fails verification | Return validation failure. |
| Project evidence identifies stable paths | Tell the agent to model those paths and close the stable scope against unexpected entries. |
| Current tree conflicts with expected tech | Keep the conflict visible; do not treat every observed path as intended policy. |
| Evidence is ambiguous | Leave the scope open and ask a focused question instead of guessing. |
| Guidance assertion has an empty/unsafe path | Reject the evaluator contract. |
| Guidance assertion path traverses a symlink | Fail the assertion without reading external content. |
| Guidance assertion file or fragment is absent | Fail the requested guidance dimension. |

## 5. Good / Base / Bad Cases

- Good: the project receives editable `agent-entrypoint`, `skill`,
  `folder-health`, and closed-directory rules plus explicit structure uses.
- Good: the handoff directs the agent to turn detected stack evidence into an
  explicit expected shape and verify it with `review` and `check`.
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
- Generated handoff coverage for evidence inspection, stable-scope closure,
  focused questions, and final `review` plus `check` verification.
- Evaluator coverage for matching, missing-file, missing-fragment, malformed,
  and symlinked guidance assertions; omitted assertions remain unavailable.

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

## 7. Wrong vs Correct

Wrong: ask every specialization question before inspecting the repository, or
copy every observed path into policy as though existing means intentional.

Correct: inspect manifests, framework and workspace configuration, generated
output settings, documentation, and established paths; model the expected
stack and intentional shape; close stable scopes; ask only where evidence is
ambiguous or a proposed rule would reject existing content.
