# Agent Harness Hooks

This spec applies when adding or changing Assura hook integrations for Codex,
OpenCode, Claude, Pi, OpenClaw, or another coding-agent harness.

## Stable Surface

- The stable feedback API remains `assura check --format agent` and
  `assura agent nudge`.
- Do not add one public binary, one check format, or one durable command family
  per harness.
- Harness integrations are delivery adapters. They translate harness events
  into Assura events, call the Assura binary, and return or log the resulting
  compact context.
- Routine automatic context is default-silent or one deterministic selected
  line, <=256 UTF-8 bytes including its wrapper. Detailed payloads, log/state
  paths, omitted counts and provenance remain available through explicit
  reports rather than repeated host context.

## Distribution Layers

| Layer | Owner | Distribution | Rule |
| --- | --- | --- | --- |
| Core policy and scoring | Rust CLI | Assura binary semver | Keep validation, severity, heatmap, and nudge decisions here. |
| Adapter contract | Assura manifest/schema | Generated bundle manifest | Version event names, payload fields, context-return shape, and logs separately from binary semver. |
| Shared helpers | Assura generator | Embedded/generated files | Share only runtime-compatible launchers or JSON helpers. |
| Harness adapter | Harness-specific source | Generated project bundle or host package only when required | Keep Python/TypeScript/shell/native code thin and replaceable. |
| Harness config | User project or harness config | Explicit install/update action | Patch only Assura-managed sections; warn on unmanaged drift. |

Default to generated project-local bundles under
`.assura/integrations/<harness>/`. Use a separately published harness package
only when the harness requires registry installation, compiled extension
delivery, or dependencies that should not ship inside the core CLI.

## Manifest Contract

Each managed bundle must declare:

- `assura_version` and `minimum_assura_version`
- `adapter_contract_version`
- `adapter_version`
- the target harness contract and any known version boundary
- `runtime` such as `python3`, `node`, `shell`, or `native`
- `managed_files` with path, content hash, executable bit, and ownership marker
- host-event-to-Assura-event mappings, delivery mode, and unsupported events
- whether the host still requires project trust or hook approval
- the exact `assura agent integration update <host>` update channel

Manifests should be deterministic so `status`, `doctor`, and tests can compare
expected files without timestamp noise.

## Lifecycle

- `install` writes missing managed files and refuses to overwrite unmanaged
  files unless the command explicitly supports and receives a force option.
- `activate` explicitly writes or patches only Assura-owned project-local host
  configuration. Bundle generation alone never implies activation.
- `update` refreshes stale managed files when the marker and manifest ownership
  match, including an already-active host adapter.
- `deactivate` removes only Assura-owned host configuration while retaining the
  reviewable bundle; `remove` deactivates first and then removes the bundle.
- `status` reports missing, present, unmanaged, and outdated files in a compact
  machine-readable shape.
- `doctor` fails missing or outdated managed files that would make hook behavior
  differ from the current generator.
- Runtime hook logs belong under ignored `.assura/agent-sessions/*.jsonl`.

## Scenario: Exact Git hook ownership

### 1. Scope / Trigger

- Trigger: Git hook install, force refresh, status, or removal changes.

### 2. Signatures

- `assura hooks install [PATH] [--force]`
- `assura hooks uninstall [PATH]`

### 3. Contracts

- A Git hook pair is Assura-managed only when the complete entrypoint equals a
  deterministic Assura delegator for that hook and project-local sidecar path,
  and the complete sidecar equals the embedded hook script. The previous exact
  double-quoted delegator remains ownership evidence for refresh/removal, but
  it is legacy rather than current or ready and default installation upgrades
  it. Newly generated delegators single-quote literal paths while preserving
  their exact Unix path bytes.
- A marker substring, expected filename, or one matching artifact is never
  ownership proof for the other artifact. `--force` refreshes only a pair with
  no unowned content. Removal classifies the wrapper and sidecar together before
  deleting either; an exact managed orphan may be removed, while arbitrary
  orphan content is preserved.
- Static symbolic links at either hook file or in the project-local Assura hook
  directory are unowned and preserved, including dangling links. Assura does
  not follow them for lifecycle reads or writes.
- One hook pair is installed transactionally: if publishing the entrypoint
  fails after changing its sidecar, Assura restores the sidecar and reports any
  rollback failure. `install_all` remains a sequential series of these per-hook
  transactions, not one transaction across all hook types. This guarantee does
  not claim protection against arbitrary concurrent hostile filesystem changes.

### 4. Validation & Error Matrix

| Condition | Required behavior |
| --- | --- |
| Existing exact current wrapper | report unchanged, or refreshed under force; removal is allowed |
| Existing exact legacy wrapper | report owned but not ready; default install upgrades it; removal is allowed |
| Existing custom wrapper, including marker text | preserve and report it; never overwrite or delete |
| Custom wrapper with an Assura-named sidecar | preserve both files |
| Exact wrapper with modified sidecar | preserve both files and report drift |
| Missing half of an otherwise exact managed pair | repair the missing artifact |
| Arbitrary orphan sidecar | preserve and report it |
| Exact managed orphan sidecar | removal is allowed |
| Hook file or Assura hook directory is a symbolic link | preserve it and do not touch its target |
| Entrypoint publication fails after sidecar publication | restore the prior sidecar or report rollback failure |

### 5. Good / Base / Bad Cases

- Good: a force refresh replaces an exact stale Assura wrapper and exact
  sidecar without weakening advisory/default or opt-in blocking behavior.
- Good: default install upgrades an exact legacy wrapper before it can execute
  shell expansion from an otherwise literal project path.
- Base: a repeated install reports an exact current wrapper as unchanged.
- Bad: treating `Git hook managed by Assura`, an Assura filename, or a symlink
  target as ownership.

### 6. Tests Required

- Cover plain custom and marker-collision custom wrappers for install, force,
  direct uninstall, and bulk uninstall; cover a custom wrapper plus sidecar.
- Cover modified and orphan sidecars through direct and bulk mutation paths,
  missing managed artifacts, hook-file and directory symlinks, per-hook rollback,
  and real invocation through paths containing spaces and shell metacharacters.

### 7. Wrong vs Correct

Wrong: use `content.contains(marker)`, path existence, or symlink-following file
reads for lifecycle ownership.

Correct: classify both non-symlink artifacts by complete deterministic content
before any write or deletion, and publish a pair with rollback of the first
artifact when the second publication fails.

## Harness Rules

- Codex and Claude command hooks may use Python or shell adapters, but policy
  must remain in the Assura binary.
- OpenCode JavaScript or TypeScript plugins may be generated or packaged, but should only
  translate plugin events and call Assura.
- Pi and OpenClaw support must be based on current runtime source or official
  docs before claiming native hook coverage.
- Unsupported events are valid support states. Document fallback behavior rather
  than claiming parity.

## Quality Bar

- Golden tests for generated manifests and adapter files.
- Stale managed-file detection and update tests.
- Unmanaged-file protection tests.
- At least one real payload fixture per supported host. Every claimed mapping
  must be listed in the deterministic manifest, including whether it injects
  model context, appends tool context, or records a session log only.
- Site and docs examples must be generated from these command/report contracts;
  visual styling may adapt layout but cannot invent states, labels, or behavior.
- Matrix entry in
  `.agents/skills/assura-agent-harness-hooks/references/harness-hook-matrix.md`
  with source, proof, and gaps.
