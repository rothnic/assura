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


def get_all_statuses(tasks_dir: Path, github: object | None = None) -> dict[str, str]:
    """Get current {dir_name: status} values for active and archived tasks.

    Useful for computing children progress without loading full TaskInfo.

    Args:
        tasks_dir: Path to the tasks directory.
        github: Optional injectable GitHub reader for current-status checks.

    Returns:
        Dict mapping directory names to status strings.
    """
    statuses = {t.dir_name: t.status for t in iter_active_tasks(tasks_dir)}
    archive_dir = tasks_dir / "archive"

    # Archived task folders remain useful provenance. An old completed flag
    # without a receipt is deliberately represented as unknown rather than as
    # success.
    try:
        from .delivery import (
            classify_candidate,
            collect_inventory,
            delivery_store_root,
            load_delivery_intent,
        )
        from .delivery_store import DeliveryStore, DeliveryStoreError

        repo_root = tasks_dir.parent.parent
        store = DeliveryStore(delivery_store_root(repo_root))
        inventory = collect_inventory(repo_root, github=github)
    except Exception:
        store = None
        inventory = None

    # Recompute each typed task against one current inventory. A receipt is
    # evidence input, not a status override: stale tips, moved refs, invalid
    # evidence, duplicate identities, and unavailable coverage must remain
    # visible as unresolved instead of inflating parent progress.
    if store is not None and inventory is not None:
        candidate_task_paths: dict[str, list[str]] = {}
        for task in inventory.tasks:
            intent_data = task.get("intent")
            if isinstance(intent_data, dict):
                candidate_id = str(intent_data.get("candidate_id") or "")
                if candidate_id:
                    candidate_task_paths.setdefault(candidate_id, []).append(
                        str(task["task_path"])
                    )
        duplicate_candidates = {
            candidate_id
            for candidate_id, paths in candidate_task_paths.items()
            if len(paths) > 1
        }
        for task in inventory.tasks:
            if not isinstance(task.get("intent"), dict):
                continue
            task_json = repo_root / str(task["task_path"]) / FILE_TASK_JSON
            try:
                intent = load_delivery_intent(task_json)
                if intent.candidate_id in duplicate_candidates:
                    continue
                receipt = store.read(intent.candidate_id)
                status = classify_candidate(intent, inventory, receipt)
            except (DeliveryStoreError, OSError, ValueError):
                continue
            if (
                status.outcome in {"delivered", "superseded", "rejected", "cancelled"}
                and status.closure in {"verified", "closed"}
                and not status.issues
            ):
                task_name = Path(str(task["task_path"])).name
                statuses[task_name] = f"outcome:{status.outcome}"

    if not archive_dir.is_dir():
        return statuses

    for task_json in sorted(archive_dir.rglob("task.json")):
        task_name = task_json.parent.name
        state = "unknown:archived"
        try:
            raw = json.loads(task_json.read_text(encoding="utf-8"))
            if (
                store is None
                and isinstance(raw, dict)
                and raw.get("status") not in {"completed", "done"}
            ):
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
    delivered = sum(1 for c in children if all_statuses.get(c) == "outcome:delivered")
    dispositioned = sum(
        1 for c in children
        if all_statuses.get(c) in {"outcome:superseded", "outcome:rejected", "outcome:cancelled"}
    )
    known_outcomes = {
        "outcome:delivered",
        "outcome:superseded",
        "outcome:rejected",
        "outcome:cancelled",
    }
    unknown = sum(1 for c in children if all_statuses.get(c) not in known_outcomes)
    if unknown or dispositioned:
        return f" [{delivered}/{len(children)} delivered; dispositioned={dispositioned}; unknown={unknown}]"
    return f" [{delivered}/{len(children)} delivered]"
