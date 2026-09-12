//! Generated thin adapters for supported agent hosts.

use crate::cli::AgentIntegrationTarget;
use std::path::Path;

use super::bundle::{quote_path, MANAGED_MARKER};

pub(super) fn python_hook(agent: AgentIntegrationTarget) -> String {
    let agent = agent.as_str();
    format!(
        r#"#!/usr/bin/env python3
# {MANAGED_MARKER}
"""Translate one {agent} hook payload into the shared Assura nudge contract."""

import json
import os
import shutil
import subprocess
import sys
from pathlib import Path

TARGET = "{agent}"
EVENTS = {{
    "SessionStart": "session-start",
    "UserPromptSubmit": "idle",
    "PreToolUse": "before-tool",
    "PostToolUse": "after-tool",
    "PostToolUseFailure": "recovery",
    "Stop": "idle",
}}
PATH_KEYS = {{"file_path", "filePath", "path"}}
MAX_CONTEXT_BYTES = 256


def project_root(payload):
    current = Path(payload.get("cwd") or os.getcwd()).resolve()
    for candidate in (current, *current.parents):
        if (candidate / ".assura" / "config.yml").is_file():
            return candidate
    return current


def collect_paths(value, found):
    if isinstance(value, dict):
        for key, nested in value.items():
            if key in PATH_KEYS and isinstance(nested, str):
                found.add(nested)
            elif key == "paths" and isinstance(nested, list):
                found.update(item for item in nested if isinstance(item, str))
            else:
                collect_paths(nested, found)
    elif isinstance(value, list):
        for nested in value:
            collect_paths(nested, found)


def assura_command(root):
    configured = os.environ.get("ASSURA_BIN")
    if configured:
        return [configured]
    installed = shutil.which("assura")
    if installed:
        return [installed]
    local = root / "target" / "debug" / "assura"
    return [str(local)] if local.is_file() else ["assura"]


def compact_context(payload):
    summary = payload.get("summary") or {{}}
    if not summary.get("should_inject"):
        return ""
    items = payload.get("nudges") or []
    item = next((item for item in items if item.get("severity") == "critical"), None)
    item = item or next((item for item in items if item.get("severity") == "high"), None)
    item = item or (items[0] if items else None)
    line = f"assura: event={{payload.get('event', 'event')}}"
    if item:
        path = item.get("path") or "project"
        rule = item.get("rule") or item.get("category") or "signal"
        line += f"; [{{rule}}] {{path}}: {{item.get('message', '')}}"
    context = "\n".join(["<assura-feedback>", line, "</assura-feedback>"])
    feedback = payload.get("feedback") or {{}}
    effective = feedback.get("effective") or {{}}
    limit = effective.get("max_bytes", MAX_CONTEXT_BYTES)
    if not isinstance(limit, int) or not 64 <= limit <= MAX_CONTEXT_BYTES:
        limit = MAX_CONTEXT_BYTES
    if len(context.encode("utf-8")) <= limit:
        return context
    suffix = "\n...\n</assura-feedback>"
    remaining = limit - len(suffix.encode("utf-8"))
    prefix = context.encode("utf-8")[:remaining].decode("utf-8", errors="ignore")
    return prefix + suffix


def main():
    try:
        payload = json.load(sys.stdin)
    except (json.JSONDecodeError, OSError):
        return 0
    hook_event = payload.get("hook_event_name") or payload.get("hookEventName")
    event = EVENTS.get(hook_event)
    if not event:
        return 0
    root = project_root(payload)
    changed = set()
    collect_paths(payload.get("tool_input"), changed)
    command = assura_command(root) + [
        "agent", "nudge", str(root), "--agent", TARGET,
        "--event", event, "--delivery", "automatic", "--format", "json",
    ]
    for path in sorted(changed)[:20]:
        command.extend(["--changed", path])
    env = os.environ.copy()
    env.setdefault("ASSURA_AGENT_LOG", "1")
    env.setdefault("ASSURA_AGENT_SESSION_ID", str(payload.get("session_id") or f"{{TARGET}}-hook"))
    try:
        result = subprocess.run(
            command, cwd=root, env=env, capture_output=True, text=True, timeout=8
        )
        nudge = json.loads(result.stdout) if result.returncode == 0 else {{}}
    except (OSError, subprocess.TimeoutExpired, json.JSONDecodeError):
        return 0
    context = compact_context(nudge)
    if context:
        print(json.dumps({{
            "hookSpecificOutput": {{
                "hookEventName": hook_event,
                "additionalContext": context,
            }}
        }}, separators=(",", ":")))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
"#
    )
}

pub(super) fn opencode_plugin() -> String {
    format!(
        r#"// {MANAGED_MARKER}
import {{ execFile }} from "node:child_process"
import {{ promisify }} from "node:util"

const runFile = promisify(execFile)

function pathsFrom(value, found = new Set()) {{
  if (Array.isArray(value)) for (const item of value) pathsFrom(item, found)
  else if (value && typeof value === "object") for (const [key, nested] of Object.entries(value)) {{
    if (["file_path", "filePath", "path"].includes(key) && typeof nested === "string") found.add(nested)
    else if (key === "paths" && Array.isArray(nested)) nested.filter((item) => typeof item === "string").forEach((item) => found.add(item))
    else pathsFrom(nested, found)
  }}
  return [...found].sort().slice(0, 20)
}}

function compact(payload) {{
  if (!payload?.summary?.should_inject) return ""
  const items = payload.nudges ?? []
  const item = items.find((entry) => entry.severity === "critical") ?? items.find((entry) => entry.severity === "high") ?? items[0]
  let line = `assura: event=${{payload.event ?? "event"}}`
  if (item) line += `; [${{item.rule ?? item.category ?? "signal"}}] ${{item.path ?? "project"}}: ${{item.message ?? ""}}`
  const limit = payload?.feedback?.effective?.max_bytes ?? 256
  return bounded(["<assura-feedback>", line, "</assura-feedback>"].join("\n"), limit)
}}

function bounded(value, limit) {{
  const encoder = new TextEncoder()
  const maxBytes = Number.isInteger(limit) && limit >= 64 && limit <= 256 ? limit : 256
  if (encoder.encode(value).length <= maxBytes) return value
  const suffix = "\n...\n</assura-feedback>"
  let body = ""
  for (const character of value) {{
    if (encoder.encode(body + character + suffix).length > maxBytes) break
    body += character
  }}
  return body + suffix
}}

async function nudge(directory, event, sessionID, args) {{
  const command = process.env.ASSURA_BIN || "assura"
  const changed = pathsFrom(args).flatMap((path) => ["--changed", path])
  try {{
    const {{ stdout }} = await runFile(command, ["agent", "nudge", directory, "--agent", "opencode", "--event", event, "--delivery", "automatic", "--format", "json", ...changed], {{
      cwd: directory,
      timeout: 8000,
      env: {{ ...process.env, ASSURA_AGENT_LOG: process.env.ASSURA_AGENT_LOG ?? "1", ASSURA_AGENT_SESSION_ID: sessionID }},
    }})
    return compact(JSON.parse(stdout))
  }} catch {{
    return ""
  }}
}}

export const AssuraPlugin = async ({{ directory }}) => ({{
  event: async ({{ event }}) => {{
    const mapped = event.type === "session.created" ? "session-start" : event.type === "session.error" ? "recovery" : event.type === "session.idle" ? "idle" : null
    if (mapped) await nudge(directory, mapped, event.properties?.sessionID ?? "opencode-event", event.properties)
  }},
  "tool.execute.after": async (input, output) => {{
    const context = await nudge(directory, "after-tool", input.sessionID, input.args)
    if (context) output.output = `${{output.output}}\n\n${{context}}`
  }},
}})
"#
    )
}

pub(super) fn pi_extension() -> String {
    format!(
        r#"// {MANAGED_MARKER}
function pathsFrom(value, found = new Set()) {{
  if (Array.isArray(value)) for (const item of value) pathsFrom(item, found)
  else if (value && typeof value === "object") for (const [key, nested] of Object.entries(value)) {{
    if (["file_path", "filePath", "path"].includes(key) && typeof nested === "string") found.add(nested)
    else if (key === "paths" && Array.isArray(nested)) nested.filter((item) => typeof item === "string").forEach((item) => found.add(item))
    else pathsFrom(nested, found)
  }}
  return [...found].sort().slice(0, 20)
}}

function compact(payload) {{
  if (!payload?.summary?.should_inject) return ""
  const items = payload.nudges ?? []
  const item = items.find((entry) => entry.severity === "critical") ?? items.find((entry) => entry.severity === "high") ?? items[0]
  let line = `assura: event=${{payload.event ?? "event"}}`
  if (item) line += `; [${{item.rule ?? item.category ?? "signal"}}] ${{item.path ?? "project"}}: ${{item.message ?? ""}}`
  const limit = payload?.feedback?.effective?.max_bytes ?? 256
  return bounded(["<assura-feedback>", line, "</assura-feedback>"].join("\n"), limit)
}}

function bounded(value, limit) {{
  const encoder = new TextEncoder()
  const maxBytes = Number.isInteger(limit) && limit >= 64 && limit <= 256 ? limit : 256
  if (encoder.encode(value).length <= maxBytes) return value
  const suffix = "\n...\n</assura-feedback>"
  let body = ""
  for (const character of value) {{
    if (encoder.encode(body + character + suffix).length > maxBytes) break
    body += character
  }}
  return body + suffix
}}

export default function (pi) {{
  async function nudge(event, ctx, input = {{}}) {{
    const changed = pathsFrom(input).flatMap((path) => ["--changed", path])
    const result = await pi.exec(process.env.ASSURA_BIN || "assura", ["agent", "nudge", ctx.cwd, "--agent", "pi", "--event", event, "--delivery", "automatic", "--format", "json", ...changed], {{ signal: ctx.signal, timeout: 8000 }})
    if (result.code !== 0) return ""
    try {{ return compact(JSON.parse(result.stdout)) }} catch {{ return "" }}
  }}

  pi.on("session_start", async (_event, ctx) => {{ await nudge("session-start", ctx) }})
  pi.on("before_agent_start", async (_event, ctx) => {{
    const context = await nudge("idle", ctx)
    if (context) return {{ message: {{ customType: "assura-feedback", content: context, display: true }} }}
  }})
  pi.on("tool_result", async (event, ctx) => {{
    const context = await nudge("after-tool", ctx, event.input)
    if (context) return {{ content: [...event.content, {{ type: "text", text: context }}] }}
  }})
}}
"#
    )
}

pub(super) fn wrapper_script(agent: AgentIntegrationTarget, project_root: &Path) -> String {
    let agent = agent.as_str();
    let project_root = quote_path(project_root);
    let check_extra = if agent == "codex" {
        " --agent codex"
    } else {
        ""
    };
    format!(
        r#"#!/bin/sh
# {MANAGED_MARKER}
set -eu

PROJECT_ROOT="${{ASSURA_PROJECT_ROOT:-}}"
if [ -z "$PROJECT_ROOT" ]; then
  PROJECT_ROOT={project_root}
fi
MODE="${{ASSURA_AGENT_MODE:-nudge}}"
EVENT="${{ASSURA_AGENT_EVENT:-session-start}}"
ASSURA_BIN="${{ASSURA_BIN:-assura}}"
export ASSURA_AGENT_LOG="${{ASSURA_AGENT_LOG:-1}}"
export ASSURA_AGENT_LOG_DIR="${{ASSURA_AGENT_LOG_DIR:-$PROJECT_ROOT/.assura/agent-sessions}}"
export ASSURA_AGENT_SESSION_ID="${{ASSURA_AGENT_SESSION_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$$}}"

case "$MODE" in
  nudge) "$ASSURA_BIN" agent nudge "$PROJECT_ROOT" --agent {agent} --event "$EVENT" --delivery automatic --format json "$@" ;;
  check) "$ASSURA_BIN" check "$PROJECT_ROOT" --format agent{check_extra} --warn "$@" ;;
  daemon-status) "$ASSURA_BIN" daemon status "$PROJECT_ROOT" --format json "$@" ;;
  daemon-doctor) "$ASSURA_BIN" daemon doctor "$PROJECT_ROOT" --format json "$@" ;;
  *) echo "Unsupported ASSURA_AGENT_MODE: $MODE" >&2; exit 2 ;;
esac
"#
    )
}
