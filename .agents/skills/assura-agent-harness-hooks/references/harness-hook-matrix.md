# Harness Hook Matrix

Use this matrix as the first stop when adding or changing Assura agent harness
hook support. Update it when a harness API, install location, event name, or
payload shape is verified.

## Shared Assura Contract

| Assura event | Meaning | Required adapter behavior |
| --- | --- | --- |
| `session-start` | Agent session starts or resumes | Record baseline health; inject only compact recovery/daemon context. |
| `before-tool` | Harness can intercept a tool before execution | Pass intended paths when available; use for blocking only when the harness supports it and Assura policy explicitly asks. |
| `after-tool` | Tool finished and may have changed project state | Compute changed paths since the previous hook/message, call `assura agent nudge --event after-tool`, log JSONL, inject selectively. |
| `file-read` | Harness can observe high-value file reads | Pass read path as `--changed`; inject only path-specific structure/content/reference context. |
| `idle` | Prompt submit, idle review, or periodic check | Record baseline state; inject only when queued context is worth interrupting. |
| `recovery` | Failed tool, stale daemon, or context repair | Prefer compact daemon/project recovery instructions. |

## Distribution and Versioning Model

| Layer | Examples | Version owner | Update behavior |
| --- | --- | --- | --- |
| Core policy and validation | `assura agent nudge`, `assura check --format agent`, project signal aggregation | Assura binary semver | Ships with the CLI. Harness adapters must call into this instead of reimplementing policy. |
| Adapter contract | Event names, JSON payload fields, context-return shape, log schema | `adapter_contract_version` in each manifest | Change only with manifest/test updates. Backward-compatible additions are preferred; breaking changes require clear minimum Assura version. |
| Shared generated helpers | Shell launcher, shared Python helper, shared JSON normalization when runtime-compatible | `artifact_version` and content hash in generated files | Managed files may be refreshed by install/update when the Assura marker and manifest match. |
| Harness adapter code | Codex Python hook, Claude command hook, OpenCode TypeScript plugin, Pi/OpenClaw extension wrapper | Harness-specific `adapter_version`, runtime, and source hash | Keep thin. It translates harness payloads to the shared contract and delegates decisions to the Assura binary. |
| Harness-owned config | `.codex/hooks.json`, `.claude/settings.json`, `.opencode/plugins/*`, OpenClaw/Pi extension registration | Harness/project owner plus Assura managed marker when generated | Do not overwrite unmanaged config. Install/update should patch or warn only for Assura-managed sections. |

Prefer embedding small generated adapters in the Assura binary until adapter
distribution needs independent release cadence. Split into separately published
packages only when a harness requires a package registry, compiled extension, or
runtime dependency that should not ship inside the core CLI.

Each generated bundle manifest should include:

- `assura_version` and `minimum_assura_version`
- `adapter_contract_version`
- `adapter_version`
- `harness` and `harness_version_range` when known
- `runtime` such as `python3`, `node`, `shell`, or `native`
- `managed_files` with path, content hash, executable bit, and ownership marker
- `update_channel` such as `core-binary`, `harness-package`, or `manual`
- `generated_at` only when deterministic output does not matter; prefer stable
  content for tests and drift detection

Distribution rule of thumb:

- Put durable decisions, scoring, severity, and message shaping in Rust.
- Put cross-harness payload normalization in a shared contract or helper.
- Put only hook registration, payload extraction, subprocess invocation, and
  context-return formatting in harness-specific code.
- Avoid registry-distributed adapter packages until the host requires one;
  generated project-local bundles are easier to inspect, diff, and update.
- If a registry package becomes necessary, make the package a thin launcher
  pinned by manifest version and keep policy in the Assura binary.

## Harnesses

| Harness | Native mechanism | Strongest currently known events | Assura support target | Verification source |
| --- | --- | --- | --- | --- |
| Codex | `.codex/hooks.json` command hooks | `UserPromptSubmit`, `PostToolUse` | Native hook adapter plus managed wrapper under `.assura/integrations/codex/` | Official Codex hooks docs and repo-local proof tests. |
| OpenCode | TypeScript plugin hooks | Tool execute hooks and event hooks | Generate wrapper plus plugin instructions or adapter files; use plugin APIs when available. | Local `create-opencode-plugin` skill and generated SDK reference. |
| Claude | `.claude/settings*.json` command hooks | Prompt submit and tool-use hooks where enabled | Generate wrapper plus Claude settings snippet; keep user approval explicit. | Official Claude Code hooks docs or local Claude config docs. |
| Pi | Extension or command-wrapper integration | Depends on `pi_agent_rust` extension/runtime surface | Generate wrapper and extension guidance; mark unsupported native events explicitly. | Local `pi-agent-rust` skill and current Pi runtime source/docs. |
| OpenClaw | OpenClaw plugin/gateway/harness surface | Verify from current OpenClaw source before claiming native hooks | Generate wrapper and best available plugin/command integration; warn when native after-tool support is unknown. | Current OpenClaw checkout or official OpenClaw docs. |

## Research Checklist

- Record the exact hook names and whether they run before prompt, before tool,
  after tool, on idle/stop, or on recovery/error.
- Record payload fields needed for changed-path detection: cwd, session id,
  tool name, tool input, tool result, and turn/tool ids.
- Record how additional context is injected back into the harness, or state
  that the harness can only log/warn out-of-band.
- Record install/update path: project config file, global config file, plugin
  directory, extension directory, or wrapper-only.
- Record whether Assura may safely update managed files. If a file is not
  Assura-managed, lifecycle commands should warn or require `--force`.
- Add a test or proof command that exercises at least one hook payload and one
  stale/out-of-date bundle condition.

## Learning Record Format

When verifying a harness behavior, add or update a compact note with this shape:

```markdown
### <Harness> - <YYYY-MM-DD>

- Source: <official docs URL, local file path, or command output>
- Hook surface: <native hooks, plugin, extension, wrapper, or unavailable>
- Install/update path: <project path, global path, generated bundle path>
- Events verified: <event names mapped to Assura events>
- Payload fields: <cwd/session/tool/path/result fields that are available>
- Context return: <stdout injection, JSON response, log-only, or unavailable>
- Drift handling: <managed update behavior and stale-file detection>
- Proof: <test name, command, fixture, or manual repro>
- Gaps: <unsupported events or claims that still need verification>
```

Keep records factual and source-backed. If a behavior is inferred from source,
say so; do not promote it to supported until an adapter or proof test exists.

## Drift Policy

- Managed files contain the Assura managed marker and may be refreshed by
  `assura agent integration update`.
- `status` should report stale managed files as `outdated` or equivalent.
- `doctor` should fail stale/missing managed files when expected files are out
  of sync with the generator.
- Non-managed files must not be overwritten unless the user passes an explicit
  force option.
