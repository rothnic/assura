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
  `content_gaps`, `heatmap`, `omitted_noise`, `next_actions`, and
  `lower_level_commands`.
- Agent output includes bounded `findings`, `heatmap.hot_dirs`,
  `omitted_noise`, and `next_actions` arrays so wrappers do not need to scrape
  text.
- `heatmap` is an advisory rolled-up signal packet, not an additional
  validation engine. It reuses the structure check report, content-gap summary,
  and best-effort local Git state.
- `heatmap.git_available=false` is non-fatal. Review must still complete
  outside Git repositories.
- `heatmap.hot_dirs` is capped at five entries and ranks directory-level
  pressure from validation violations, naming violations, line-limit
  violations, tracked worktree changes, untracked files, deleted/conflicted
  files, branch-changed files, and line churn.
- `heatmap.totals` includes branch-level signals such as files changed since
  the detected base branch and commits on the current branch when Git can
  provide them.
- Branch heat should prefer the repository default/base branch such as
  `origin/main`, `origin/master`, `main`, or `master`. A feature branch's
  tracking upstream is only the fallback base and is used for ahead/behind
  counters when present, so pushed feature branches do not hide their own
  branch delta.
- Finding severities are `blocking`, `advisory`, `inactive`, or
  `informational`.
- Finding action kinds are `fix-now`, `configure-intentionally`,
  `inspect-before-changing`, or `informational`.
- Raw unresolved repository-reference counts are informational unless an
  configured validation policy promotes specific findings through normal
  checks.
- Generated, archive, log, and benchmark reference noise must be filtered,
  classified, or explicitly listed as omitted from blocking review policy.
- Text output must stay compact, row-aligned, and scan-first for humans:
  header/status, check, heat, hot dirs, content, finding counts, action
  buckets, policy, next command, and detail commands.
- Text output may use ANSI color only when stdout is a terminal or
  `ASSURA_FORCE_COLOR=1`/`CLICOLOR_FORCE` is set. Piped/captured output must
  remain plain text, and `NO_COLOR`/`CLICOLOR=0` must disable automatic color.
- Text output must include the structure-fit boundary: inspect nearby shape
  before adding paths; change `.assura/config.yml` only when the path is
  intentional.

### 4. Validation & Error Matrix

| Condition | Expected behavior |
| --- | --- |
| Structure check passes with only inactive/advisory guidance | Exit `0`; status `needs-review`; summary has no blocking findings. |
| Structure check has blocking violations | Exit `1`; status `fail`; findings include `fix-now` blocking items. |
| Content runtime has configured blocking diagnostics | Exit follows `check`; content gaps also point to content-query details. |
| Unresolved repository-reference candidates exist only as raw candidates | Exit `0` when no blocking checks fail; finding is informational. |
| Project is not a Git checkout | Exit follows normal review; `heatmap.git_available=false`; Git counters stay zero/unknown. |
| Git checkout has branch/worktree pressure | Review includes compact aligned heat/hot-dir rows plus JSON `heatmap` totals and directory rollups. |
| Text output is captured or piped | Output contains no ANSI escapes by default. |
| Text output is forced with `ASSURA_FORCE_COLOR=1` | Output contains ANSI styling without changing text content or JSON/agent output. |
| No config is discoverable | Exit with the existing no-config error code. |
| Content-query loading fails | Exit with the existing content-query error code. |

### 5. Good/Base/Bad Cases

- Good: Before a PR, a user runs `assura review . --format text`, sees aligned
  status/heat/action rows with terminal color when interactive, sees no
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
- Heat-map test: real Git branch/worktree state plus a validation violation
  rolls up to `heatmap.totals` and the expected hot directory.
- Noisy reference test: unresolved candidate count is informational and
  omitted-noise policy includes generated/archive/log/benchmark categories.
- Actionable content-gap test: content runtime diagnostics surface as content
  findings and lower-level content-query commands.
- Agent format test: schema `assura.project-review.agent.v1` and bounded
  finding/action arrays.
- Text format test: captured output stays plain, aligned rows preserve the
  structure-fit policy, and forced color emits ANSI styling.

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
