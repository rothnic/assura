# Agent Script Backed Computed Checks

## Objective

Add a narrow, explicit `extensions.computed_checks` policy that lets a project
run allowlisted local scripts to emit deterministic Assura findings for
rollups, scores, confidence adjustments, and other derived validations before a
native computed-field language exists.

## Current Gap

Assura now models requirements, claims, evidence, source documents, findings,
agent guidance, references, and next actions. It still has no controlled way to
compute derived validations from those modeled facts. The current alternatives
are either hand-written native Rust checks for every use case or out-of-band
scripts whose output does not enter normal Assura reports, doctor output,
agent-query, hooks, or merge gates.

## Scope

- Add a first-party `extensions.computed_checks` config family.
- Require explicit project-local script allowlisting; no implicit script
  discovery or remote execution.
- Execute scripts with bounded timeout, deterministic stdin JSON, and no shell
  interpolation.
- Accept only versioned JSON findings with stable rule/message/severity/path
  fields and reproducible metadata.
- Feed valid computed findings into normal `assura check` reports, corrective
  context, `doctor`, `content agent-query diagnostics/gaps/next-actions`,
  agent feedback, hooks, and merge-gate behavior through existing report paths.
- Provide fixtures for pass, findings, missing script, invalid output, timeout,
  and nonzero exit behavior.
- Document the boundary between first-party extension policies, computed
  checks, internal Rust APIs, and deferred public plugin APIs.

## Non-Goals

- No arbitrary unconfigured script execution.
- No remote plugin loading, marketplace plugin system, or shell-executed
  validator plugin API.
- No native formula language in this slice.
- No domain-specific domain-specific scoring in core presets.
- No hidden global installs or host-agent configuration mutation.

## Definition Of Done

- A project can configure a computed check and receive normal Assura findings
  with `computed_check:<policy-id>:<finding-code>` rule IDs.
- Missing, unsafe, timed-out, nonzero, or invalid-output scripts fail with
  clear diagnostics.
- `assura doctor` and `assura content agent-query` expose concise computed
  check gaps and next actions without dumping large script payloads.
- Website and support-policy docs describe computed checks as an advanced,
  explicit project-local feature and do not present them as a general plugin
  API.
- Generated agent/document project presets do not enable domain-specific
  computed checks by default.

## Proof Gates

- `cargo fmt --check`
- `cargo check --workspace --all-targets --all-features --quiet`
- `cargo test computed_checks --quiet`
- `cargo test --test agent_surface_cli --quiet`
- `cargo test --test content_query_cli --quiet`
- `cargo run --quiet -- check --format json .`
- `cargo xtask target-state`
- `cargo xtask docs`
- `cargo xtask evidence`
- `git diff --check`

## Review Criteria

Block if scripts run implicitly, paths can escape the project, execution uses a
shell string instead of argument vectors, findings bypass ordinary severity or
rule/message contracts, output is not deterministic enough for CI, computed
checks are documented as a public plugin API, or core onboarding presets gain
domain-specific scoring behavior.
