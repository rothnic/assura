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
- `assura daemon start [path] --format json`
- `assura daemon stop [path] --format json`
- `assura daemon restart [path] --format json`
- `assura daemon doctor [path] --format json`
- `assura daemon logs [path] --tail <n> --format json`
- `assura daemon health [path] --format json`
- `assura daemon check-path [path] --changed <path> --format json`
- `assura daemon references [path] (--source <path> | --target <path> |
  --moved-target <path> --new-target <path>) --format json`

### 3. Contracts

- `status` emits schema `assura.daemon.status.v1` with
  `protocol_version`, `health`, `process`, and `management` fields.
- `doctor` emits schema `assura.daemon.doctor.v1` with
  `protocol_version`, `health`, and `checks`.
- `start`, `stop`, and `restart` emit schema
  `assura.daemon.lifecycle.v1` with `action`, `changed`, `health`,
  `runtime`, and optional `error`.
- `logs` emits schema `assura.daemon.logs.v1` with `health`, `log_file`,
  requested `tail`, line counts, truncation metadata, and returned lines.
- `health` must come from `LocalDaemonCore` when the project loads, or from
  `DaemonHealth::unavailable` when loading fails.
- Runtime paths must stay under the explicit daemon runtime area exposed by
  `DaemonRuntimePaths`; do not write status, lock, or log files directly into
  `.assura/` root.
- Lifecycle commands in this preview manage project-local runtime metadata and
  logs. Do not claim a long-running socket/process daemon until that server is
  implemented and tested.

### 4. Validation & Error Matrix

| Condition | Behavior |
| --- | --- |
| Project loads | `status` and `doctor` return JSON with `health.state = "running"`. |
| Project cannot load | `status` returns JSON health with `state = "unavailable"`; `doctor` returns JSON diagnostics and exits with runtime error. |
| `start` runs twice | First call returns `changed = true`; repeated call returns `changed = false` with runtime `state = "started"`. |
| `stop` runs twice | First call after start returns `changed = true`; repeated call returns `changed = false` with runtime `state = "stopped"`. |
| `restart` runs after start | Return `action = "restart"`, `changed = true`, runtime `state = "started"`, and append a runtime log entry. |
| `logs --tail <n>` runs | Return at most the last `<n>` lines with line counts and truncation metadata. |
| References mode omits all selectors or selects more than one | Return configuration error before loading daemon state. |
| Daemon state is stale | Return schema `assura.daemon.error.v1` with structured stale health in JSON/YAML modes. |
| Long-running socket/process daemon is not implemented | Keep support docs experimental/roadmap and keep process `running = false`. |

### 5. Good/Base/Bad Cases

- Good: `daemon doctor --format json` on an uninitialized directory returns a
  machine-readable `project_state` error with a remediation command.
- Good: repeated `daemon start --format json` and
  `daemon stop --format json` calls are idempotent.
- Base: `daemon status --format json` on a valid project reports protocol,
  process placeholder metadata, management hints, and health.
- Bad: `daemon start --format json` claims `process.running = true` before a
  real long-running process or socket server exists.

### 6. Tests Required

- CLI tests for status schema, protocol version, health, process metadata, and
  lifecycle command hints.
- CLI tests for idempotent start/stop, restart, runtime status files, and
  bounded logs.
- CLI tests for doctor success and unavailable-project remediation.
- Parity tests showing `daemon references --source` and `--target` match the
  corresponding `content references` graph output.
- Target-state checks proving `.assura/command-surface.yml`, support policy,
  compatibility docs, source variants, and tests stay synchronized.

### 7. Wrong vs Correct

#### Wrong

```json
{
  "process": {
    "running": true
  }
}
```

#### Correct

```json
{
  "process": {
    "running": false,
    "mode": "managed_runtime_metadata"
  }
}
```

Keep process status honest until a long-running process or socket server exists,
even when runtime metadata lifecycle commands are available.
