# Daemon Management CLI Contract

This spec applies when changing `assura daemon` management commands used by
humans, editors, hooks, and agents.

## Scenario: Management Preview Commands

### 1. Scope / Trigger

- Trigger: new or changed `assura daemon` command signatures or JSON response
  contracts.
- The daemon CLI must reuse the shared daemon/client state contract instead of
  adding editor-only or agent-specific lifecycle behavior.
- Experimental preview commands may expose lifecycle placeholders, but they
  must not advertise unimplemented `start`, `stop`, `restart`, or `logs`
  behavior as available.

### 2. Signatures

- `assura daemon status [path] --format json`
- `assura daemon doctor [path] --format json`
- `assura daemon health [path] --format json`
- `assura daemon check-path [path] --changed <path> --format json`
- `assura daemon references [path] (--source <path> | --target <path> |
  --moved-target <path> --new-target <path>) --format json`

### 3. Contracts

- `status` emits schema `assura.daemon.status.v1` with
  `protocol_version`, `health`, `process`, and `management` fields.
- `doctor` emits schema `assura.daemon.doctor.v1` with
  `protocol_version`, `health`, and `checks`.
- `health` must come from `LocalDaemonCore` when the project loads, or from
  `DaemonHealth::unavailable` when loading fails.
- Runtime paths must stay under the explicit daemon runtime area exposed by
  `DaemonRuntimePaths`; do not write status, lock, or log files directly into
  `.assura/` root.
- Preview lifecycle command hints for unimplemented commands must serialize as
  `null`, not as shell commands that imply support exists.

### 4. Validation & Error Matrix

| Condition | Behavior |
| --- | --- |
| Project loads | `status` and `doctor` return JSON with `health.state = "running"`. |
| Project cannot load | `status` returns JSON health with `state = "unavailable"`; `doctor` returns JSON diagnostics and exits with runtime error. |
| References mode omits all selectors or selects more than one | Return configuration error before loading daemon state. |
| Daemon state is stale | Return schema `assura.daemon.error.v1` with structured stale health in JSON/YAML modes. |
| Full lifecycle command is not implemented | Keep support docs experimental/roadmap and keep command hints absent or `null`. |

### 5. Good/Base/Bad Cases

- Good: `daemon doctor --format json` on an uninitialized directory returns a
  machine-readable `project_state` error with a remediation command.
- Base: `daemon status --format json` on a valid project reports protocol,
  process placeholder metadata, management hints, and health.
- Bad: `daemon status` returns `start: "assura daemon start ..."` before
  `daemon start` exists and has idempotence tests.

### 6. Tests Required

- CLI tests for status schema, protocol version, health, process metadata, and
  lifecycle-placeholder nulls.
- CLI tests for doctor success and unavailable-project remediation.
- Parity tests showing `daemon references --source` and `--target` match the
  corresponding `content references` graph output.
- Target-state checks proving `.assura/command-surface.yml`, support policy,
  compatibility docs, source variants, and tests stay synchronized.

### 7. Wrong vs Correct

#### Wrong

```json
{
  "management": {
    "start": "assura daemon start --format json ."
  }
}
```

#### Correct

```json
{
  "management": {
    "start": null
  }
}
```

Use `null` until the command exists, is idempotent, and is covered by lifecycle
tests.
