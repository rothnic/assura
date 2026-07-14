# Agent-Ready Onboarding

## 1. Scope / Trigger

This contract applies when `assura agent onboard` detects a repository,
recommends built-in policy, writes the project baseline, or changes the
machine-readable onboarding report.

## 2. Signatures

```text
assura agent onboard [PATH] --agent <auto|codex|opencode|claude|pi|generic> --format <text|json|yaml>
```

Implementation surfaces:

- `src/cli/agent_onboarding.rs` detects the project and selects recommendations.
- `src/cli/agent_onboarding_templates.rs` writes project-owned config and handoff files.
- `src/cli/agent_onboarding_report.rs` defines serialized report fields.

## 3. Contracts

The report includes `rule_recommendations[]` with:

| Field | Contract |
| --- | --- |
| `preset` | Versioned Assura built-in rule selected as the starting point. |
| `local_rule` | Project-owned wrapper written under `rules:` in `.assura/config.yml`. |
| `status` | `applied` when the root uses the wrapper, `available` when existing root policy is preserved, `not-applied` when the selected config lacks the wrapper, or `conflict` when an existing wrapper needs review. |
| `reason` | Concise explanation tied to detected project evidence. |
| `includes` | Built-in rule layers composed by the selected preset. |

The broad baseline maps `@project-agentic-baseline` to `@agentic-project` and
uses the local wrapper at the project root. The wrapper is the customization
point; onboarding must not expand and duplicate the full built-in preset.

Onboarding writes `.assura/onboarding/rules.md` so an agent can inspect the
recommendation and learn where project-specific overrides belong. Language,
framework, naming, layout, and domain policy remain undecided until supported
evidence or a user decision exists.

## 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Empty or unknown project | Apply only the broad agent-ready baseline. |
| Recognized project type | Report the detected type; do not invent unsupported language policy. |
| New config | Write the local wrapper and use it at `structure.\"./\"`. |
| Existing config | Preserve user policy and merge the missing local wrapper. |
| Existing root `use` | Preserve it and report the new wrapper as `available`. |
| Existing local-wrapper name | Preserve its body and report `conflict` unless it already composes the recommended preset. |
| Scalar or list `use` | Detect the recommended preset and local wrapper in either supported form. |
| Alternate `--config` | Derive status from the selected config; do not claim the generated default is active there. |
| Existing generated `rules.md` | Refresh it when recommendation status changes. |
| Generated baseline fails verification | Return validation failure instead of reporting successful onboarding. |

## 5. Good / Base / Bad Cases

- Good: a Rust repository receives the broad local wrapper and a report that
  identifies Rust detection while leaving Rust-specific rules undecided.
- Base: an empty repository receives the same broad wrapper without a false
  claim that language or framework policy was inferred.
- Bad: onboarding references `@agentic-project` directly everywhere, leaving no
  project-owned customization point.
- Bad: onboarding copies a large built-in rule body into every project.

## 6. Tests Required

- JSON report coverage for every recommendation field.
- Generated packet coverage for `rules.md` and the local wrapper.
- Existing-config merge coverage proving user config is preserved.
- Generated config verification through the normal Assura check path.
- Landing-page browser coverage separating applied recommendations from
  undecided policy.

## 7. Wrong vs Correct

Wrong:

```yaml
structure:
  ./:
    use: "@agentic-project"
```

Correct:

```yaml
rules:
  "@project-agentic-baseline":
    use: "@agentic-project"

structure:
  ./:
    use: "@project-agentic-baseline"
```
