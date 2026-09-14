"""
Task data access layer.

Single source of truth for loading and iterating task directories.
Replaces scattered task.json parsing across 9+ files.

Provides:
    load_task          — Load a single task by directory path
    iter_active_tasks  — Iterate all non-archived tasks (sorted)
    get_all_statuses   — Get {dir_name: status} map for children progress
"""

from __future__ import annotations

import json
from collections.abc import Iterator
from pathlib import Path

from .io import read_json
from .paths import FILE_TASK_JSON
from .types import TaskInfo


def load_task(task_dir: Path) -> TaskInfo | None:
    """Load task from a directory containing task.json.

    Args:
        task_dir: Absolute path to the task directory.

    Returns:
        TaskInfo if task.json exists and is valid, None otherwise.
    """
    task_json = task_dir / FILE_TASK_JSON
    if not task_json.is_file():
        return None

    data = read_json(task_json)
    if not data:
        return None

    return TaskInfo(
        dir_name=task_dir.name,
        directory=task_dir,
        title=data.get("title") or data.get("name") or "unknown",
        status=data.get("status", "unknown"),
        assignee=data.get("assignee", ""),
        priority=data.get("priority", "P2"),
        children=tuple(data.get("children", [])),
        parent=data.get("parent"),
        package=data.get("package"),
        raw=data,
    )


def iter_active_tasks(tasks_dir: Path) -> Iterator[TaskInfo]:
    """Iterate all active (non-archived) tasks, sorted by directory name.

    Skips the "archive" directory and directories without valid task.json.

    Args:
        tasks_dir: Path to the tasks directory.

    Yields:
        TaskInfo for each valid task.
    """
    if not tasks_dir.is_dir():
        return

    for d in sorted(tasks_dir.iterdir()):
        if not d.is_dir() or d.name == "archive":
            continue
        info = load_task(d)
        if info is not None:
            yield info


def get_all_statuses(tasks_dir: Path) -> dict[str, str]:
    """Get a {dir_name: status} mapping for all active tasks.

    Useful for computing children progress without loading full TaskInfo.

    Args:
        tasks_dir: Path to the tasks directory.

    Returns:
        Dict mapping directory names to status strings.
    """
    statuses = {t.dir_name: t.status for t in iter_active_tasks(tasks_dir)}
    archive_dir = tasks_dir / "archive"
    if not archive_dir.is_dir():
        return statuses

    # Archived task folders remain useful provenance. Resolve their explicit
    # delivery receipt when available; an old completed flag without a receipt
    # is deliberately represented as unknown rather than as success.
    try:
        from .delivery import delivery_store_root, load_delivery_intent
        from .delivery_store import DeliveryStore, DeliveryStoreError

        store = DeliveryStore(delivery_store_root(tasks_dir.parent.parent))
    except Exception:
        store = None

    for task_json in sorted(archive_dir.rglob("task.json")):
        task_name = task_json.parent.name
        state = "unknown:archived"
        try:
            raw = json.loads(task_json.read_text(encoding="utf-8"))
            if store is not None:
                intent = load_delivery_intent(task_json)
                receipt = store.read(intent.candidate_id)
                outcome = receipt.get("outcome")
                if outcome in {"delivered", "superseded", "rejected", "cancelled"} and receipt.get("closure") in {"verified", "closed"}:
                    state = f"outcome:{outcome}"
            elif isinstance(raw, dict) and raw.get("status") not in {"completed", "done"}:
                state = str(raw.get("status", "unknown"))
        except Exception:
            pass
        statuses[task_name] = state
    return statuses


def children_progress(
    children: tuple[str, ...] | list[str],
    all_statuses: dict[str, str],
) -> str:
    """Format children progress string like " [2/3 done]".

    Args:
        children: List of child directory names.
        all_statuses: Status map from get_all_statuses().

    Returns:
        Formatted string, or "" if no children.
    """
    if not children:
        return ""
    delivered = sum(
        1 for c in children
        if all_statuses.get(c) in ("outcome:delivered", "delivered")
    )
    dispositioned = sum(
        1 for c in children
        if all_statuses.get(c) in {"outcome:superseded", "outcome:rejected", "outcome:cancelled"}
    )
    known_outcomes = {
        "outcome:delivered",
        "outcome:superseded",
        "outcome:rejected",
        "outcome:cancelled",
        "delivered",
    }
    unknown = sum(1 for c in children if all_statuses.get(c) not in known_outcomes)
    if unknown or dispositioned:
        return f" [{delivered}/{len(children)} delivered; dispositioned={dispositioned}; unknown={unknown}]"
    return f" [{delivered}/{len(children)} delivered]"
