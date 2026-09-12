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
| Codex | `.codex/hooks.json` command hooks | `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse` | Managed Python adapter plus exact Assura matcher groups; project trust and `/hooks` approval remain host-owned. | Official Codex hooks docs verified 2026-08-14; `codex_post_tool_event_injects_bounded_assura_context`. |
| OpenCode | `.opencode/plugins/*.js` project plugin | `session.created`, `session.idle`, `session.error`, `tool.execute.after` | Managed JavaScript plugin; session events log and after-tool appends bounded context. | Official OpenCode plugin docs verified 2026-08-14; `opencode_after_tool_event_appends_bounded_assura_context`. |
| Claude | `.claude/settings.json` command hooks | `SessionStart`, `UserPromptSubmit`, `PreToolUse`, `PostToolUse`, `PostToolUseFailure` | Managed Python adapter plus exact Assura matcher groups; preserve unrelated settings and hooks. | Official Claude Code hooks docs verified 2026-08-14; `claude_pre_tool_event_injects_bounded_assura_context`. |
| Pi | `.pi/extensions/*.ts` project extension | `session_start`, `before_agent_start`, `tool_result` | Managed TypeScript-compatible extension; project trust remains host-owned. | Current Pi extension docs/source verified 2026-08-14; `pi_tool_result_event_appends_bounded_assura_context`. |
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

## Verified Records

### Codex - 2026-08-14

- Source: `https://learn.chatgpt.com/docs/hooks`
- Hook surface: project-local `.codex/hooks.json` command hooks
- Install/update path: `.assura/integrations/codex/` plus exact Assura groups in `.codex/hooks.json`
- Events verified: `PostToolUse` to `after-tool`; manifest also maps the other installed groups
- Payload fields: `cwd`, `session_id`, `hook_event_name`, `tool_name`, and nested path fields in `tool_input`
- Context return: `hookSpecificOutput.additionalContext`
- Drift handling: exact-group patch/unpatch; reject edited Assura groups; preserve unrelated JSON
- Proof: `codex_post_tool_event_injects_bounded_assura_context`
- Gaps: project trust and exact hook approval remain host-owned; hosted tools can bypass local tool hooks

### Claude Code - 2026-08-14

- Source: `https://code.claude.com/docs/en/hooks`
- Hook surface: project-local `.claude/settings.json` command hooks
- Install/update path: `.assura/integrations/claude/` plus exact Assura groups in `.claude/settings.json`
- Events verified: `PreToolUse` to `before-tool`; manifest maps prompt, session, post-tool, and failure groups
- Payload fields: common hook fields plus nested `tool_input` path fields
- Context return: `hookSpecificOutput.additionalContext`
- Drift handling: exact-group patch/unpatch; reject edited Assura groups; preserve unrelated settings
- Proof: `claude_pre_tool_event_injects_bounded_assura_context`
- Gaps: no separate `file-read` lifecycle mapping; host trust remains outside Assura verification

### OpenCode - 2026-08-14

- Source: `https://opencode.ai/docs/plugins/`
- Hook surface: project-local JavaScript plugin
- Install/update path: `.opencode/plugins/assura.js`
- Events verified: `tool.execute.after` to `after-tool`
- Payload fields: plugin directory, session ID, and recursive path fields from tool args
- Context return: bounded context appended to tool output; session event mappings are log-only
- Drift handling: managed-marker update; reject unmanaged plugin replacement
- Proof: `opencode_after_tool_event_appends_bounded_assura_context`
- Gaps: no model-context delivery is claimed for session event callbacks

### Pi - 2026-08-14

- Source: `https://raw.githubusercontent.com/badlogic/pi-mono/main/packages/coding-agent/docs/extensions.md`
- Hook surface: project-local TypeScript extension
- Install/update path: `.pi/extensions/assura.ts`
- Events verified: `tool_result` to `after-tool`
- Payload fields: extension context cwd plus recursive path fields from tool input
- Context return: bounded text item appended to tool-result content; `before_agent_start` returns a custom message
- Drift handling: managed-marker update; reject unmanaged extension replacement
- Proof: `pi_tool_result_event_appends_bounded_assura_context`
- Gaps: project trust remains host-owned; no dedicated recovery event is claimed

## Drift Policy

- Managed files contain the Assura managed marker and may be refreshed by
  `assura agent integration update`.
- `status` should report stale managed files as `outdated` or equivalent.
- `doctor` should fail stale/missing managed files when expected files are out
  of sync with the generator.
- Non-managed files must not be overwritten unless the user passes an explicit
  force option.
