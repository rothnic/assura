# Daemon Management CLI Contract

This spec applies when changing `assura daemon` management commands used by
humans, editors, hooks, and agents.

## Scenario: Managed Local Daemon Commands

### 1. Scope / Trigger

- Trigger: new or changed `assura daemon` command signatures or JSON response
  contracts.
- The daemon CLI must reuse the shared daemon/client state contract instead of
  adding editor-only or agent-specific lifecycle behavior.
- Experimental daemon commands may expose only the process and IPC behavior
  that is implemented and tested. Keep editor, agent, reference, and hosted
  daemon claims out of this contract until their separate goals prove them.

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
  `protocol_version`, `health`, `project`, `process`, and `management`
  fields.
- `status.process` includes `state`, `running`, `pid`, `socket_path`,
  `listen_addr`, `mode`, `message`, and `updated_at_unix`. A started daemon
  must be probed over IPC before `running = true` is reported.
- `status.project` includes `project_root`, `config_path`,
  `config_fingerprint`, and git `dirty_paths`. Non-git projects return an
  empty dirty path list.
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
- Lifecycle commands manage a project-local daemon process and log/status
  files. `start` launches the hidden `daemon serve` process; `stop` and
  `restart` terminate or replace it idempotently.
- The IPC protocol is versioned as `assura.daemon.v1` and supports health
  probes, changed-path structure checks, and bounded repository-reference
  context for `--source`, `--target`, and moved-target queries.

### 4. Validation & Error Matrix

| Condition | Behavior |
| --- | --- |
| Project loads | `status` and `doctor` return JSON with `health.state = "running"`. |
| `status` runs in a git repository with local changes | Return changed or untracked paths in `project.dirty_paths`. |
| `status` can read config | Return a stable hex `project.config_fingerprint`. |
| Project cannot load | `status` returns JSON health with `state = "unavailable"`; `doctor` returns JSON diagnostics and exits with runtime error. |
| `start` runs twice | First call returns `changed = true`; repeated call returns `changed = false` with runtime `state = "started"` and `running = true`. |
| `stop` runs twice | First call after start returns `changed = true`; repeated call returns `changed = false` with runtime `state = "stopped"`. |
| `restart` runs after start | Return `action = "restart"`, `changed = true`, runtime `state = "started"`, and append a runtime log entry. |
| `logs --tail <n>` runs | Return at most the last `<n>` lines with line counts and truncation metadata. |
| Managed daemon process exits unexpectedly | `status` returns `process.state = "crashed"` and `process.running = false`. |
| `check-path` runs while the daemon is running | Return schema `assura.daemon.check_path.v1` with `protocol_version = "assura.daemon.v1"` from daemon IPC. |
| `references` runs while the daemon is running | Return schema `assura.daemon.references.v1` with `protocol_version = "assura.daemon.v1"` from daemon IPC. |
| References mode omits all selectors or selects more than one | Return configuration error before loading daemon state. |
| Daemon state is stale | Return schema `assura.daemon.error.v1` with structured stale health in JSON/YAML modes. |

### 5. Good/Base/Bad Cases

- Good: `daemon doctor --format json` on an uninitialized directory returns a
  machine-readable `project_state` error with a remediation command.
- Good: repeated `daemon start --format json` and
  `daemon stop --format json` calls are idempotent.
- Good: `daemon status --format json` after `daemon start` reports
  `process.running = true`, a PID, and a local IPC address only after a
  successful protocol probe.
- Base: `daemon status --format json` on a valid project reports protocol,
  project metadata, config fingerprint, dirty paths, process metadata,
  management hints, and health.
- Bad: `daemon status --format json` reports `process.running = true` from a
  stale runtime file without probing the daemon process.

### 6. Tests Required

- CLI tests for status schema, protocol version, health, process metadata, and
  lifecycle command hints.
- CLI tests for status config fingerprint and git dirty paths.
- CLI tests for idempotent start/stop, restart, runtime status files, and
  bounded logs.
- CLI tests for IPC-backed changed-path checks, reference queries, and
  crashed-process status.
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

#### Correct When Stopped Or Not Started

```json
{
  "process": {
    "running": false,
    "mode": "managed_runtime_metadata"
  }
}
```

#### Correct When Started And Probed

```json
{
  "process": {
    "running": true,
    "mode": "managed_process",
    "pid": 12345,
    "listen_addr": "127.0.0.1:58461"
  }
}
```

Keep process status honest by probing the managed daemon before reporting it as
fresh and running.
