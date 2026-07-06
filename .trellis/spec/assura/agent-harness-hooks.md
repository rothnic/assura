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
- `harness` and known `harness_version_range`
- `runtime` such as `python3`, `node`, `shell`, or `native`
- `managed_files` with path, content hash, executable bit, and ownership marker
- `update_channel`: `core-binary`, `harness-package`, or `manual`

Manifests should be deterministic so `status`, `doctor`, and tests can compare
expected files without timestamp noise.

## Lifecycle

- `install` writes missing managed files and refuses to overwrite unmanaged
  files unless the command explicitly supports and receives a force option.
- `update` refreshes stale managed files when the marker and manifest ownership
  match.
- `status` reports missing, present, unmanaged, and outdated files in a compact
  machine-readable shape.
- `doctor` fails missing or outdated managed files that would make hook behavior
  differ from the current generator.
- Runtime hook logs belong under ignored `.assura/agent-sessions/*.jsonl`.

## Harness Rules

- Codex and Claude command hooks may use Python or shell adapters, but policy
  must remain in the Assura binary.
- OpenCode TypeScript plugins may be generated or packaged, but should only
  translate plugin events and call Assura.
- Pi and OpenClaw support must be based on current runtime source or official
  docs before claiming native hook coverage.
- Unsupported events are valid support states. Document fallback behavior rather
  than claiming parity.

## Quality Bar

- Golden tests for generated manifests and adapter files.
- Stale managed-file detection and update tests.
- Unmanaged-file protection tests.
- At least one payload fixture per supported native hook event.
- Matrix entry in
  `.agents/skills/assura-agent-harness-hooks/references/harness-hook-matrix.md`
  with source, proof, and gaps.
