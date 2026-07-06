#!/usr/bin/env python3
"""Codex hook adapter for Assura agent nudges.

This adapter keeps Codex-specific protocol handling out of the Assura nudge
payload. The generated Assura wrapper still owns validation, daemon awareness,
and JSONL nudge logging; this script maps Codex lifecycle events to that shared
contract and adds a small amount of hook-local state for post-tool deltas.
"""

from __future__ import annotations

import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


STATE_SCHEMA = "assura.codex-hook-state.v1"
STATE_FILE = "codex-hook-state.jsonl"
MAX_CONTEXT_NUDGES = 5
MAX_CHANGED_PATHS = 20
DEFAULT_MIN_SEVERITY = "medium"
GIT_WRITE_INTENTS = {
    "git_commit",
    "git_merge",
    "git_rebase",
    "git_pull",
    "git_push",
    "git_checkout",
    "git_switch",
    "git_reset",
    "git_clean",
    "git_stash",
}


def find_project_root(start: Path) -> Path | None:
    current = start.resolve()
    while current != current.parent:
        if (current / ".assura" / "config.yml").is_file():
            return current
        current = current.parent
    return None


def assura_is_available(root: Path) -> bool:
    configured = os.environ.get("ASSURA_BIN")
    if configured and Path(configured).is_file():
        return True
    return shutil.which("assura") is not None or (root / "target/debug/assura").is_file()


def hook_event_name(hook_input: dict[str, Any]) -> str:
    event = hook_input.get("hook_event_name") or hook_input.get("hookEventName")
    if isinstance(event, str) and event:
        return event
    if hook_input.get("tool_name"):
        return "PostToolUse"
    return "UserPromptSubmit"


def session_id(hook_input: dict[str, Any]) -> str:
    configured = os.environ.get("ASSURA_AGENT_SESSION_ID")
    if configured:
        return configured
    value = hook_input.get("session_id")
    if isinstance(value, str) and value:
        return value
    return f"codex-{os.getpid()}"


def state_path(root: Path) -> Path:
    return root / ".assura" / "agent-sessions" / STATE_FILE


def run_git(root: Path, args: list[str], timeout: int = 3) -> str:
    try:
        result = subprocess.run(
            ["git", *args],
            cwd=str(root),
            capture_output=True,
            text=False,
            timeout=timeout,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return ""
    if result.returncode != 0:
        return ""
    return result.stdout.decode("utf-8", errors="replace")


def git_status_entries(root: Path) -> dict[str, str]:
    output = run_git(root, ["status", "--porcelain=v1", "-z", "--untracked-files=all"])
    if not output:
        return {}
    entries: dict[str, str] = {}
    parts = output.split("\0")
    index = 0
    while index < len(parts):
        record = parts[index]
        index += 1
        if not record:
            continue
        if len(record) < 4:
            continue
        status = record[:2]
        path = record[3:]
        entries[path] = status
        if status[0] in {"R", "C"} or status[1] in {"R", "C"}:
            if index < len(parts) and parts[index]:
                entries[parts[index]] = status
                index += 1
    return entries


def path_signature(root: Path, path: str, status: str) -> str:
    disk_path = root / path
    try:
        stat = disk_path.stat()
    except OSError:
        return f"{status}:missing"
    return f"{status}:{stat.st_size}:{stat.st_mtime_ns}"


def git_snapshot(root: Path) -> dict[str, str]:
    return {
        path: path_signature(root, path, status)
        for path, status in sorted(git_status_entries(root).items())
        if not path.startswith(".assura/agent-sessions/")
    }


def read_last_state(root: Path, current_session_id: str) -> dict[str, Any] | None:
    path = state_path(root)
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return None
    for line in reversed(lines):
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        if record.get("session_id") == current_session_id:
            return record
    return None


def changed_since_previous(
    current: dict[str, str],
    previous: dict[str, Any] | None,
    assume_current_changed: bool,
) -> list[str]:
    previous_snapshot = previous.get("git_snapshot") if isinstance(previous, dict) else None
    if not isinstance(previous_snapshot, dict):
        return sorted(current) if assume_current_changed else []
    changed = [
        path
        for path, signature in current.items()
        if previous_snapshot.get(path) != signature
    ]
    removed = [path for path in previous_snapshot if path not in current]
    return sorted(set(changed + removed))


def append_state(
    root: Path,
    hook_input: dict[str, Any],
    current_session_id: str,
    event_name: str,
    current: dict[str, str],
    changed_paths: list[str],
    intent: str,
) -> None:
    path = state_path(root)
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        tool_input = hook_input.get("tool_input")
        record = {
            "schema": STATE_SCHEMA,
            "session_id": current_session_id,
            "turn_id": hook_input.get("turn_id"),
            "tool_use_id": hook_input.get("tool_use_id"),
            "hook_event_name": event_name,
            "tool_name": hook_input.get("tool_name"),
            "tool_intent": intent,
            "timestamp_unix_seconds": int(time.time()),
            "dirty_path_count": len(current),
            "changed_since_previous_count": len(changed_paths),
            "changed_since_previous": changed_paths[:MAX_CHANGED_PATHS],
            "git_snapshot": current,
            "tool_input_sha256": stable_digest(tool_input),
        }
        with path.open("a", encoding="utf-8") as file:
            file.write(json.dumps(record, separators=(",", ":")) + "\n")
    except OSError:
        return


def stable_digest(value: Any) -> str | None:
    if value is None:
        return None
    encoded = json.dumps(value, sort_keys=True, default=str).encode("utf-8")
    return hashlib.sha256(encoded).hexdigest()


def command_text(hook_input: dict[str, Any]) -> str:
    tool_input = hook_input.get("tool_input")
    if isinstance(tool_input, dict):
        command = tool_input.get("command")
        if isinstance(command, str):
            return command
        return json.dumps(tool_input, sort_keys=True, default=str)
    if isinstance(tool_input, str):
        return tool_input
    return ""


def detect_intent(hook_input: dict[str, Any]) -> str:
    tool_name = str(hook_input.get("tool_name") or "")
    command = command_text(hook_input).lower()
    if tool_name == "apply_patch":
        return "edit"
    if re.search(r"\bgit\s+commit\b", command):
        return "git_commit"
    if re.search(r"\bgit\s+merge\b", command):
        return "git_merge"
    if re.search(r"\bgit\s+rebase\b", command):
        return "git_rebase"
    if re.search(r"\bgit\s+pull\b", command):
        return "git_pull"
    if re.search(r"\bgit\s+push\b", command):
        return "git_push"
    if re.search(r"\bgit\s+checkout\b", command):
        return "git_checkout"
    if re.search(r"\bgit\s+switch\b", command):
        return "git_switch"
    if re.search(r"\bgit\s+reset\b", command):
        return "git_reset"
    if re.search(r"\bgit\s+clean\b", command):
        return "git_clean"
    if re.search(r"\bgit\s+stash\b", command):
        return "git_stash"
    if re.search(r"\b(cargo\s+fmt|rustfmt|prettier|eslint\s+--fix|ruff\s+--fix)\b", command):
        return "format_or_fix"
    if re.search(r"\b(cargo\s+test|cargo\s+clippy|cargo\s+check|cargo\s+xtask|npm\s+test|bun\s+test)\b", command):
        return "validation"
    if re.search(r"(^|\s)(cat|sed|perl|python|node|ruby|awk).*(>|-i\b)", command):
        return "scripted_edit"
    return "tool"


def mutating_intent(intent: str) -> bool:
    return intent in {"edit", "format_or_fix", "scripted_edit"} or intent in GIT_WRITE_INTENTS


def run_wrapper(
    root: Path,
    wrapper: Path,
    event: str,
    changed_paths: list[str],
    current_session_id: str,
) -> dict[str, Any] | None:
    env = os.environ.copy()
    env.setdefault("ASSURA_PROJECT_ROOT", str(root))
    env["ASSURA_AGENT_EVENT"] = event
    env.setdefault("ASSURA_AGENT_MODE", "nudge")
    env.setdefault("ASSURA_AGENT_LOG", "1")
    env.setdefault("ASSURA_AGENT_LOG_DIR", str(root / ".assura" / "agent-sessions"))
    env["ASSURA_AGENT_SESSION_ID"] = current_session_id
    command = [str(wrapper)]
    min_severity = env.get("ASSURA_AGENT_MIN_SEVERITY", DEFAULT_MIN_SEVERITY)
    max_issues = env.get("ASSURA_AGENT_MAX_ISSUES")
    if min_severity:
        command.extend(["--min-severity", min_severity])
    if max_issues:
        command.extend(["--max-issues", max_issues])
    for changed_path in changed_paths[:MAX_CHANGED_PATHS]:
        command.extend(["--changed", changed_path])
    try:
        result = subprocess.run(
            command,
            cwd=str(root),
            env=env,
            capture_output=True,
            text=True,
            timeout=8,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return None
    if result.returncode != 0:
        return None
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return None


def compact_context(payload: dict[str, Any], meta: dict[str, Any]) -> str:
    summary = payload.get("summary", {})
    lines = [
        "<assura-nudge>",
        f"Hook: {meta.get('hook_event_name', 'unknown')}",
        f"Event: {payload.get('event', 'unknown')}",
        f"Tool: {meta.get('tool_name') or '-'}",
        f"Intent: {meta.get('intent') or 'tool'}",
        (
            "Git delta: "
            f"{meta.get('changed_since_previous_count', 0)} changed since previous "
            f"hook/message, {meta.get('dirty_path_count', 0)} dirty path(s)"
        ),
        (
            "Summary: "
            f"{summary.get('nudge_count', 0)} nudge(s), "
            f"{summary.get('changed_path_count', 0)} checked changed path(s)"
        ),
    ]
    if meta.get("git_intent"):
        lines.append(
            "Git intent: detected "
            f"{meta.get('intent')} while the workspace has "
            f"{meta.get('dirty_path_count', 0)} dirty path(s)."
        )
    for nudge in payload.get("nudges", [])[:MAX_CONTEXT_NUDGES]:
        severity = nudge.get("severity", "unknown")
        category = nudge.get("category", "unknown")
        path = nudge.get("path") or "-"
        rule = nudge.get("rule") or "-"
        message = nudge.get("message") or ""
        command = nudge.get("suggested_command") or summary.get("suggested_command") or ""
        lines.append(f"- {severity} {category} {path} {rule}: {message}")
        if command:
            lines.append(f"  next: {command}")
    changed_paths = meta.get("changed_since_previous") or []
    if changed_paths and not payload.get("nudges"):
        lines.append("Changed since previous hook/message:")
        for path in changed_paths[:MAX_CONTEXT_NUDGES]:
            lines.append(f"- {path}")
    lines.append("Log: .assura/agent-sessions/nudges.jsonl")
    lines.append("State: .assura/agent-sessions/codex-hook-state.jsonl")
    lines.append("</assura-nudge>")
    return "\n".join(lines)


def should_inject(payload: dict[str, Any], meta: dict[str, Any]) -> bool:
    summary = payload.get("summary", {})
    if summary.get("should_inject"):
        return True
    if meta.get("git_intent") and meta.get("dirty_path_count", 0) > 0:
        return True
    return meta.get("changed_since_previous_count", 0) >= 10


def codex_output(event_name: str, context: str) -> dict[str, Any]:
    return {
        "hookSpecificOutput": {
            "hookEventName": event_name,
            "additionalContext": context,
        }
    }


def main() -> int:
    try:
        hook_input = json.load(sys.stdin)
    except (json.JSONDecodeError, ValueError):
        hook_input = {}

    cwd = Path(hook_input.get("cwd") or os.getcwd())
    root = find_project_root(cwd)
    if root is None:
        return 0
    wrapper = root / ".assura/integrations/codex/assura-agent.sh"
    if not wrapper.is_file() or not assura_is_available(root):
        return 0

    current_session_id = session_id(hook_input)
    event_name = hook_event_name(hook_input)
    intent = detect_intent(hook_input)
    current_snapshot = git_snapshot(root)
    previous = read_last_state(root, current_session_id)
    changed_paths = changed_since_previous(
        current_snapshot,
        previous,
        assume_current_changed=event_name == "PostToolUse" and mutating_intent(intent),
    )
    append_state(
        root,
        hook_input,
        current_session_id,
        event_name,
        current_snapshot,
        changed_paths,
        intent,
    )

    if event_name == "PostToolUse":
        paths_for_nudge = changed_paths
        git_intent = intent in GIT_WRITE_INTENTS
        if git_intent and not paths_for_nudge:
            paths_for_nudge = sorted(current_snapshot)
        payload = run_wrapper(
            root,
            wrapper,
            "after-tool",
            paths_for_nudge,
            current_session_id,
        )
        output_event = "PostToolUse"
    else:
        payload = run_wrapper(root, wrapper, "idle", [], current_session_id)
        git_intent = False
        output_event = "UserPromptSubmit"

    if payload is None:
        return 0

    meta = {
        "hook_event_name": event_name,
        "tool_name": hook_input.get("tool_name"),
        "intent": intent,
        "git_intent": git_intent,
        "dirty_path_count": len(current_snapshot),
        "changed_since_previous_count": len(changed_paths),
        "changed_since_previous": changed_paths[:MAX_CHANGED_PATHS],
    }
    if not should_inject(payload, meta):
        return 0

    print(json.dumps(codex_output(output_event, compact_context(payload, meta))))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
