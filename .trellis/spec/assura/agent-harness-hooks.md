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
