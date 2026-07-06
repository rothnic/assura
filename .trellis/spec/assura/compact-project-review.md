# Compact Project Review Contract

## Scenario: Compact Project Review Command

### 1. Scope / Trigger

- Trigger: Assura exposes one first diagnostic command that composes existing
  structure, doctor, and content-query truth without replacing them.
- Applies to `assura review`, `src/cli/project_review.rs`, and
  `src/cli/project_review/**`.

### 2. Signatures

- `assura review [path] --format text|json|yaml|agent`
- Default path is the current directory.
- Default format is `text`.
- The command uses the existing global `--config <path>` option.

### 3. Contracts

- JSON output uses schema `assura.project-review.v1`.
- Agent output uses schema `assura.project-review.agent.v1`.
- The command reuses `doctor` output plus `content agent-query gaps` summary;
  it must not add a parallel validation engine.
- JSON includes `status`, `structure`, `summary`, `findings`,
  `content_gaps`, `omitted_noise`, `next_actions`, and
  `lower_level_commands`.
- Agent output includes bounded `findings`, `omitted_noise`, and
  `next_actions` arrays so wrappers do not need to scrape text.
- Finding severities are `blocking`, `advisory`, `inactive`, or
  `informational`.
- Finding action kinds are `fix-now`, `configure-intentionally`,
  `inspect-before-changing`, or `informational`.
- Raw unresolved repository-reference counts are informational unless an
  configured validation policy promotes specific findings through normal
  checks.
- Generated, archive, log, and benchmark reference noise must be filtered,
  classified, or explicitly listed as omitted from blocking review policy.
- Text output must stay compact and include the structure-fit boundary:
  inspect nearby shape before adding paths; change `.assura/config.yml` only
  when the path is intentional.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Structure check passes with only inactive/advisory guidance | Exit `0`; status `needs-review`; summary has no blocking findings. |
| Structure check has blocking violations | Exit `1`; status `fail`; findings include `fix-now` blocking items. |
| Content runtime has configured blocking diagnostics | Exit follows `check`; content gaps also point to content-query details. |
| Unresolved repository-reference candidates exist only as raw candidates | Exit `0` when no blocking checks fail; finding is informational. |
| No config is discoverable | Exit with the existing no-config error code. |
| Content-query loading fails | Exit with the existing content-query error code. |

### 5. Good/Base/Bad Cases

- Good: Before a PR, a user runs `assura review . --format text`, sees no
  blockers, sees advisory/inactive follow-up, and can jump to lower-level
  commands for evidence.
- Base: During onboarding, an agent runs `assura review . --format agent` and
  receives bounded JSON without scraping text.
- Bad: A review implementation reruns independent structure logic, treats raw
  generated/archive reference candidates as blockers, or hides whether the
  next step is "fix the file" versus "decide whether this path belongs".

### 6. Tests Required

- Clean repo test: pass structure, no blocking summary, inactive guidance.
- Structure mismatch test: nonzero exit and blocking `fix-now` finding.
- Unmodeled path-pressure test: unexpected path plus structure-fit guidance.
- Noisy reference test: unresolved candidate count is informational and
  omitted-noise policy includes generated/archive/log/benchmark categories.
- Actionable content-gap test: content runtime diagnostics surface as content
  findings and lower-level content-query commands.
- Agent format test: schema `assura.project-review.agent.v1` and bounded
  finding/action arrays.

### 7. Wrong vs Correct

#### Wrong

```bash
assura review . --format json
```

Implemented by copying structure traversal or content diagnostics into a new
review-only validator.

#### Correct

```bash
assura review . --format json
```

Implemented as a read-only orchestration layer over `assura check`,
`assura doctor`, `assura content agent-query gaps`, and
`assura explain <path>` follow-up guidance.
