"""Current delivery audit projection and strict-scope reporting."""

from __future__ import annotations

import re
from dataclasses import replace
from pathlib import Path
from typing import Any

from .delivery import (
    CandidateStatus,
    DeliveryIssue,
    DeliveryValidationError,
    _branch_name,
    _worktree_matches_candidate,
    classify_candidate,
    collect_inventory,
    delivery_store_root,
    load_delivery_intent,
)
from .delivery_store import DeliveryStore, DeliveryStoreError
from .paths import FILE_TASK_JSON


def _receipt_store(repo_root: Path) -> DeliveryStore:
    return DeliveryStore(delivery_store_root(repo_root))


def _read_receipt(store: DeliveryStore, candidate_id: str) -> dict[str, Any] | None:
    path = store.path_for(candidate_id)
    if not path.exists():
        return None
    return store.read(candidate_id)


def _legacy_status(task: dict[str, Any]) -> CandidateStatus:
    task_path = task["task_path"]
    candidate_id = "legacy-" + re.sub(r"[^A-Za-z0-9._-]+", "-", task_path).strip("-")
    issue = DeliveryIssue(
        "LEGACY_UNCLASSIFIED",
        "task has no versioned meta.delivery intent",
        candidate_id,
        task_path,
        "Classify this historical task explicitly or create a new owned recovery task.",
    )
    return CandidateStatus(
        candidate_id=candidate_id,
        owner=str(task.get("assignee") or "unknown"),
        kind="legacy",
        task_path=task_path,
        tip=None,
        phase="held",
        outcome=None,
        integration="unknown",
        dirty=False,
        closure="open",
        next_action=issue.next_action,
        issues=(issue,),
    )


def _invalid_intent_status(task: dict[str, Any]) -> CandidateStatus:
    """Represent a malformed typed intent without treating it as legacy."""
    task_path = task["task_path"]
    candidate_id = "invalid-" + re.sub(r"[^A-Za-z0-9._-]+", "-", task_path).strip("-")
    issue = DeliveryIssue(
        "DELIVERY_INTENT_INVALID",
        str(task.get("intent_error") or "task has an invalid meta.delivery intent"),
        candidate_id,
        task_path,
        "Preserve the task and repair its versioned delivery intent before continuing.",
    )
    return CandidateStatus(
        candidate_id=candidate_id,
        owner=str(task.get("assignee") or "unknown"),
        kind="unknown",
        task_path=task_path,
        tip=None,
        phase="held",
        outcome=None,
        integration="unknown",
        dirty=False,
        closure="open",
        next_action=issue.next_action,
        issues=(issue,),
    )


def project_audit(
    repo_root: Path,
    owner: str | None = None,
    github: Any = None,
    refresh_remote: bool = False,
) -> tuple[dict[str, Any], list[CandidateStatus]]:
    """Build the current audit projection without writing anything."""
    inventory = collect_inventory(repo_root, github=github, refresh_remote=refresh_remote)
    store = _receipt_store(repo_root)
    statuses: list[CandidateStatus] = []
    legacy: list[dict[str, Any]] = []
    invalid_intents: list[dict[str, Any]] = []
    bound_branches: set[str] = set()
    owned_worktree_paths: set[str] = set()
    execution_bindings: list[dict[str, Any]] = []
    execution_worktree_paths: set[str] = set()
    known_candidate_ids: set[str] = set()
    candidate_paths: dict[str, list[str]] = {}
    for task in inventory.tasks:
        intent_data = task.get("intent")
        if not isinstance(intent_data, dict):
            if task.get("intent_error"):
                invalid_status = _invalid_intent_status(task)
                invalid_intents.append(invalid_status.as_dict())
                if owner is None or invalid_status.owner == owner:
                    statuses.append(invalid_status)
                continue
            legacy_status = _legacy_status(task)
            legacy.append(legacy_status.as_dict())
            if owner is None or legacy_status.owner == owner:
                statuses.append(legacy_status)
            continue
        task_json = repo_root / task["task_path"] / FILE_TASK_JSON
        candidate_id = str(intent_data.get("candidate_id") or task.get("id") or task["task_path"])
        try:
            intent = load_delivery_intent(task_json)
            known_candidate_ids.add(intent.candidate_id)
            candidate_paths.setdefault(intent.candidate_id, []).append(task["task_path"])
            if intent.branch_ref:
                bound_branches.add(_branch_name(intent.branch_ref) or intent.branch_ref)
            elif task.get("branch"):
                bound_branches.add(_branch_name(str(task["branch"])) or str(task["branch"]))
            receipt = _read_receipt(store, intent.candidate_id)
            status = classify_candidate(intent, inventory, receipt)
            for worktree in inventory.worktrees:
                if _worktree_matches_candidate(worktree, intent, task, status.tip):
                    path = worktree.get("path")
                    if isinstance(path, str) and path:
                        owned_worktree_paths.add(path)

            execution_branches = _execution_branches(task)
            dirty_execution_paths: list[str] = []
            for branch in execution_branches:
                bound_branches.add(branch)
                binding = {
                    "branch": branch,
                    "branch_ref": f"refs/heads/{branch}",
                    "candidate_id": intent.candidate_id,
                    "owner": status.owner,
                    "task_path": task["task_path"],
                    "worktrees": [],
                }
                for worktree in inventory.worktrees:
                    if _branch_name(worktree.get("branch_ref")) != branch:
                        continue
                    path = worktree.get("path")
                    worktree_row = {
                        **worktree,
                        "scope": "candidate",
                        "owner": status.owner,
                        "candidate_id": intent.candidate_id,
                        "evidence_ref": path,
                        "disposition": "held" if worktree.get("dirty") is True else "bound",
                        "next_action": (
                            "Preserve and resolve the dirty execution worktree before closure."
                            if worktree.get("dirty") is True
                            else "Retain this clean execution worktree as provenance for the candidate."
                        ),
                    }
                    binding["worktrees"].append(worktree_row)
                    if isinstance(path, str) and path:
                        execution_worktree_paths.add(path)
                        if worktree.get("dirty") is True:
                            dirty_execution_paths.append(path)
                        else:
                            owned_worktree_paths.add(path)
                execution_bindings.append(binding)
            if dirty_execution_paths:
                issue = DeliveryIssue(
                    "OWNED_EXECUTION_WORKTREE_DIRTY",
                    "execution worktree has uncommitted changes: "
                    + ", ".join(sorted(dirty_execution_paths)),
                    intent.candidate_id,
                    dirty_execution_paths[0],
                    "Preserve and resolve the dirty execution worktree before closure.",
                )
                status = replace(
                    status,
                    dirty=True,
                    next_action=issue.next_action,
                    issues=(*status.issues, issue),
                )
            if owner is None or status.owner == owner:
                statuses.append(status)
        except (DeliveryValidationError, DeliveryStoreError) as error:
            issue = DeliveryIssue("CANDIDATE_UNRESOLVED", str(error), candidate_id, task["task_path"], "Preserve the task and resolve its schema or receipt before closure.")
            status = CandidateStatus(candidate_id, str(task.get("assignee") or "unknown"), "unknown", task["task_path"], None, "held", None, "unknown", False, "open", issue.next_action, issues=(issue,))
            if owner is None or status.owner == owner:
                statuses.append(status)

    duplicate_candidates = {
        candidate_id: paths
        for candidate_id, paths in candidate_paths.items()
        if len(paths) > 1
    }
    if duplicate_candidates:
        corrected: list[CandidateStatus] = []
        for status in statuses:
            paths = duplicate_candidates.get(status.candidate_id)
            if not paths:
                corrected.append(status)
                continue
            shown_paths = ", ".join(paths[:4])
            if len(paths) > 4:
                shown_paths += f", ... (+{len(paths) - 4} more)"
            issue = DeliveryIssue(
                "DUPLICATE_CANDIDATE_ID",
                f"candidate identity is bound to multiple task records: {shown_paths}",
                status.candidate_id,
                shown_paths,
                "Assign one unique candidate identity to each task and preserve the existing receipt before closure.",
            )
            corrected.append(
                replace(
                    status,
                    phase="held",
                    outcome=None,
                    next_action=issue.next_action,
                    issues=(*status.issues, issue),
                )
            )
        statuses = corrected

    ignored_ref_names = {"master", "main", "HEAD"}
    base_branch = _branch_name(inventory.base_ref)
    if base_branch:
        ignored_ref_names.add(base_branch)
    unowned_refs = []
    all_refs = [*inventory.refs, *inventory.remote_refs]
    seen_ref_keys: set[tuple[str, str]] = set()
    for ref in all_refs:
        ref_key = (str(ref.get("name") or ""), str(ref.get("oid") or ""))
        if ref_key in seen_ref_keys:
            continue
        seen_ref_keys.add(ref_key)
        if ref.get("kind") not in {"heads", "remotes", "tags"}:
            continue
        branch = _branch_name(ref.get("name"))
        if not branch or branch in ignored_ref_names or branch in bound_branches:
            continue
        is_tag = ref.get("kind") == "tags"
        unowned_refs.append(
            {
                "name": ref["name"],
                "oid": ref["oid"],
                "kind": ref.get("kind"),
                "owner": "unknown",
                "candidate_id": None,
                "evidence_ref": ref["name"],
                "disposition": "retained_archive" if is_tag else "unresolved",
                "next_action": (
                    "Retain this historical tag; bind it to release evidence if it is part of a release audit."
                    if is_tag
                    else "Bind to an owner/task or record a preserved historical disposition."
                ),
            }
        )

    unowned_worktrees = []
    base_worktrees = []
    for item in inventory.worktrees:
        if item.get("path") in owned_worktree_paths:
            continue
        if item.get("path") in execution_worktree_paths:
            continue
        branch = _branch_name(item.get("branch_ref"))
        if branch and branch in bound_branches:
            continue
        if branch and branch in ignored_ref_names:
            base_worktrees.append(
                {
                    **item,
                    "scope": "base",
                    "owner": "unknown",
                    "candidate_id": None,
                    "evidence_ref": item.get("path"),
                    "disposition": "held" if item.get("dirty") is True else "retained_base",
                    "next_action": (
                        "Preserve dirty base-checkout content and resolve it before changing scope."
                        if item.get("dirty") is True
                        else "Retain the clean integration checkout as the base topology."
                    ),
                }
            )
            continue
        unowned_worktrees.append(
            {
                **item,
                "scope": "candidate",
                "owner": "unknown",
                "candidate_id": None,
                "evidence_ref": item.get("path"),
                "disposition": "held" if item.get("dirty") is True else "unresolved",
                "next_action": (
                    "Preserve dirty content and resolve ownership before any move or integration."
                    if item.get("dirty") is True
                    else "Bind the worktree to an owner/task or record a preserved historical disposition."
                ),
            }
        )

    orphan_receipts = []
    if store.root.is_dir():
        for receipt_path in sorted(store.root.glob("*.json")):
            candidate_id = receipt_path.stem
            if candidate_id in known_candidate_ids:
                continue
            orphan = {
                "candidate_id": candidate_id,
                "path": receipt_path.as_posix(),
                "owner": "unknown",
                "disposition": "unresolved",
                "evidence_ref": receipt_path.as_posix(),
                "next_action": "Rebind the receipt to an immutable task intent or preserve it as orphaned evidence.",
            }
            try:
                receipt = store.read(candidate_id)
                orphan.update(
                    {
                        "owner": receipt.get("owner") or "unknown",
                        "generation": receipt.get("generation"),
                        "outcome": receipt.get("outcome"),
                        "closure": receipt.get("closure"),
                    }
                )
            except (DeliveryStoreError, ValueError) as error:
                orphan["error"] = str(error)
            orphan_receipts.append(orphan)
    report = {
        "schema_version": 1,
        "repository": inventory.repository,
        "base_ref": inventory.base_ref,
        "base_oid": inventory.base_oid,
        "coverage": inventory.coverage,
        "inventory": inventory.as_dict(),
        "candidates": [status.as_dict() for status in sorted(statuses, key=lambda item: item.candidate_id)],
        "execution_bindings": sorted(
            execution_bindings,
            key=lambda item: (str(item.get("candidate_id") or ""), str(item.get("branch") or "")),
        ),
        "legacy": legacy,
        "invalid_intents": sorted(invalid_intents, key=lambda item: item["candidate_id"]),
        "unowned_refs": sorted(unowned_refs, key=lambda item: item["name"]),
        "unowned_worktrees": sorted(unowned_worktrees, key=lambda item: str(item.get("path", ""))),
        "base_worktrees": sorted(base_worktrees, key=lambda item: str(item.get("path", ""))),
        "orphan_receipts": sorted(orphan_receipts, key=lambda item: item["candidate_id"]),
        "issues": [issue.as_dict() for issue in inventory.issues],
    }
    return report, sorted(statuses, key=lambda item: item.candidate_id)


def _execution_branches(task: dict[str, Any]) -> tuple[str, ...]:
    """Return valid task-level execution branches in stable order."""
    meta = task.get("meta")
    values = meta.get("execution_branches") if isinstance(meta, dict) else None
    if not isinstance(values, (list, tuple)):
        return ()
    branches: list[str] = []
    for value in values:
        if not isinstance(value, str) or not value.strip():
            continue
        branch = _branch_name(value.strip())
        if branch and branch not in branches:
            branches.append(branch)
    return tuple(branches)


def strict_audit_pass(
    report: dict[str, Any],
    statuses: list[CandidateStatus],
    owner: str | None = None,
) -> bool:
    """Return whether a scoped audit has complete, dispositioned evidence."""
    if any(status.outcome is None or status.dirty or status.issues for status in statuses):
        return False
    if not report.get("coverage", {}).get("complete"):
        return False
    if owner is None:
        unresolved_refs = [
            item for item in report.get("unowned_refs", [])
            if item.get("disposition") in {"unresolved", "held"}
        ]
        unresolved_worktrees = [
            item
            for item in [*report.get("unowned_worktrees", []), *report.get("base_worktrees", [])]
            if item.get("disposition") in {"unresolved", "held"}
        ]
        return not (
            report.get("legacy")
            or unresolved_refs
            or unresolved_worktrees
            or report.get("orphan_receipts")
            or report.get("invalid_intents")
            or report.get("issues")
        )
    return not any(
        item.get("owner") in {owner, "unknown"}
        for item in report.get("orphan_receipts", [])
    )
