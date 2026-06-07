#!/usr/bin/env python3
"""Deterministic Trellis workflow gate for agent turn starts.

This script reports facts that can be derived from repository state without
guessing user intent or conversation history.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from common.active_task import resolve_active_task, resolve_task_ref
from common.git import run_git
from common.paths import get_repo_root
from common.tasks import load_task
from common.trellis_config import read_trellis_config


def _git_output(args: list[str], repo_root: Path) -> str:
    code, stdout, stderr = run_git(args, cwd=repo_root)
    if code != 0:
        return stderr.strip()
    return stdout.strip()


def _git_changes(repo_root: Path) -> list[str]:
    output = _git_output(["status", "--porcelain"], repo_root)
    return [line for line in output.splitlines() if line.strip()]


def _jsonl_has_curated_file(path: Path) -> bool:
    if not path.is_file():
        return False
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError:
        return False

    for line in lines:
        stripped = line.strip()
        if not stripped:
            continue
        try:
            item = json.loads(stripped)
        except json.JSONDecodeError:
            continue
        if isinstance(item, dict) and item.get("file") and not item.get("_example"):
            return True
    return False


def _task_artifacts(task_dir: Path | None) -> dict[str, bool]:
    if task_dir is None:
        return {
            "prd": False,
            "implement_context": False,
            "check_context": False,
        }
    return {
        "prd": (task_dir / "prd.md").is_file(),
        "implement_context": _jsonl_has_curated_file(task_dir / "implement.jsonl"),
        "check_context": _jsonl_has_curated_file(task_dir / "check.jsonl"),
    }


def _active_task_state(
    repo_root: Path,
    explicit_task: str | None,
    platform: str | None,
) -> dict[str, Any]:
    if explicit_task:
        task_path = explicit_task
        source = "explicit"
        source_type = "explicit"
        context_key = None
        resolved = resolve_task_ref(task_path, repo_root)
        stale = resolved is None or not resolved.is_dir()
    else:
        active = resolve_active_task(repo_root, platform=platform)
        task_path = active.task_path
        source = active.source
        source_type = active.source_type
        context_key = active.context_key
        stale = active.stale
        resolved = resolve_task_ref(task_path, repo_root) if task_path else None

    resolved = resolve_task_ref(task_path, repo_root) if task_path else None
    task = load_task(resolved) if resolved and resolved.is_dir() else None
    artifacts = _task_artifacts(resolved if task else None)

    return {
        "path": task_path,
        "source": source,
        "source_type": source_type,
        "context_key": context_key,
        "stale": stale,
        "dir_name": task.dir_name if task else None,
        "title": task.title if task else None,
        "status": task.status if task else None,
        "branch": task.raw.get("branch") if task else None,
        "artifacts": artifacts,
    }


def _derive_verdict(state: dict[str, Any]) -> dict[str, Any]:
    changes = state["git"]["changes"]
    task = state["task"]
    needs: list[str] = []
    options: list[str] = []
    warnings: list[str] = []
    ready = True
    workflow_state = "unknown"
    next_action = "Continue with the user request."
    task_branch = task.get("branch")

    if task_branch and task_branch != state["git"]["branch"]:
        warnings.append(
            f"Active task declares branch `{task_branch}` but shell is on `{state['git']['branch']}`."
        )

    if changes:
        ready = False
        workflow_state = "blocked_dirty_worktree"
        needs.append("Resolve or isolate uncommitted changes before changing task scope.")
        options = [
            "Commit prior/current work now",
            "Park it on a branch",
            "Leave it untouched and continue in a fresh worktree/branch",
        ]
        next_action = "Classify dirty paths; resolve automatically only when ownership is clear."
        return {
            "ready": ready,
            "workflow_state": workflow_state,
            "needs": needs,
            "options": options,
            "warnings": warnings,
            "next_action": next_action,
        }

    if task["stale"]:
        ready = False
        workflow_state = f"stale_{task['source_type']}"
        needs.append("Repair or clear the stale active-task pointer.")
        next_action = "Run `python3 ./.trellis/scripts/task.py current --source`, then attach a valid task or clear the stale session."
    elif not task["path"]:
        workflow_state = "no_task"
        next_action = "For direct Q&A, answer. For implementation or cleanup work, create/attach a Trellis task before editing."
    elif task["status"] == "planning":
        workflow_state = "planning"
        if not task["artifacts"]["prd"]:
            ready = False
            needs.append("Finish task requirements in prd.md.")
            next_action = "Load `trellis-brainstorm` and write the PRD before implementation."
        elif state["platform"] == "codex" and state["dispatch_mode"] == "inline":
            ready = False
            needs.append("Activate the planned task before implementation.")
            next_action = "Run `python3 ./.trellis/scripts/task.py start <task>` to enter in_progress. Codex inline mode skips JSONL curation."
        elif not task["artifacts"]["implement_context"] or not task["artifacts"]["check_context"]:
            ready = False
            if not task["artifacts"]["implement_context"]:
                needs.append("Curate implement.jsonl with real spec/research context.")
            if not task["artifacts"]["check_context"]:
                needs.append("Curate check.jsonl with real spec/research context.")
            next_action = "Add curated JSONL entries, then run `task.py start <task>`."
        else:
            ready = False
            needs.append("Activate the planned task before implementation.")
            next_action = "Run `python3 ./.trellis/scripts/task.py start <task>` to enter in_progress."
    elif task["status"] == "in_progress":
        workflow_state = "in_progress"
        next_action = "Ready for work. Read detailed workflow docs only when changing phase, blocked, or unsure."
    elif task["status"] == "completed":
        ready = False
        workflow_state = "completed"
        needs.append("Archive or finish the completed active task.")
        next_action = "Run finish-work/archive handling before starting new implementation."
    else:
        ready = False
        workflow_state = f"unknown_status:{task['status']}"
        needs.append("Resolve the active task status before proceeding.")
        next_action = "Inspect task.json and workflow.md for the supported state."

    return {
        "ready": ready,
        "workflow_state": workflow_state,
        "needs": needs,
        "options": options,
        "warnings": warnings,
        "next_action": next_action,
    }


def _dispatch_mode(repo_root: Path, platform: str | None) -> str:
    if platform != "codex":
        return "sub-agent"
    config = read_trellis_config(repo_root)
    codex_config = config.get("codex") if isinstance(config, dict) else None
    if isinstance(codex_config, dict):
        configured = codex_config.get("dispatch_mode")
        if isinstance(configured, str) and configured.strip() in {"inline", "sub-agent"}:
            return configured.strip()
    return "inline"


def collect_state(
    explicit_task: str | None = None,
    platform: str | None = None,
) -> dict[str, Any]:
    repo_root = get_repo_root()
    branch = _git_output(["branch", "--show-current"], repo_root) or "unknown"
    changes = _git_changes(repo_root)
    normalized_platform = platform.strip() if platform else None
    state: dict[str, Any] = {
        "repo_root": str(repo_root),
        "platform": normalized_platform,
        "dispatch_mode": _dispatch_mode(repo_root, normalized_platform),
        "git": {
            "branch": branch,
            "clean": not changes,
            "changes": changes,
        },
        "task": _active_task_state(repo_root, explicit_task, normalized_platform),
    }
    state["verdict"] = _derive_verdict(state)
    return state


def _format_text(state: dict[str, Any]) -> str:
    task = state["task"]
    verdict = state["verdict"]
    lines = [
        "Trellis workflow gate",
        f"- Branch: {state['git']['branch']}",
        f"- Git: {'clean' if state['git']['clean'] else str(len(state['git']['changes'])) + ' dirty path(s)'}",
        f"- Platform: {state['platform'] or 'unspecified'} ({state['dispatch_mode']})",
        f"- Task: {task['dir_name'] or 'none'} ({task['status'] or 'none'})",
        f"- Source: {task['source']}",
        f"- State: {verdict['workflow_state']}",
        f"- Ready: {'yes' if verdict['ready'] else 'no'}",
        f"- Next: {verdict['next_action']}",
    ]
    if verdict["warnings"]:
        lines.append("- Warnings:")
        lines.extend(f"  - {warning}" for warning in verdict["warnings"])
    if verdict["needs"]:
        lines.append("- Needs:")
        lines.extend(f"  - {need}" for need in verdict["needs"])
    if verdict["options"]:
        lines.append("- Options:")
        lines.extend(f"  {index}. {option}" for index, option in enumerate(verdict["options"], 1))
    if state["git"]["changes"]:
        lines.append("- Dirty paths:")
        lines.extend(f"  - {change}" for change in state["git"]["changes"])
    return "\n".join(lines)


def main() -> None:
    parser = argparse.ArgumentParser(description="Report deterministic Trellis workflow state")
    parser.add_argument(
        "--task",
        help="Explicit active task path/ref from injected workflow-state when session identity is unavailable",
    )
    parser.add_argument(
        "--platform",
        help="Platform name used for platform-specific routing, e.g. codex",
    )
    parser.add_argument("--json", action="store_true", help="Output machine-readable JSON")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="Exit non-zero when the gate is not ready",
    )
    args = parser.parse_args()

    state = collect_state(explicit_task=args.task, platform=args.platform)
    if args.json:
        print(json.dumps(state, indent=2, ensure_ascii=False))
    else:
        print(_format_text(state))
    if args.strict and not state["verdict"]["ready"]:
        raise SystemExit(2)


if __name__ == "__main__":
    main()
