"""Archive guards and recoverable receipt closure helpers."""

from __future__ import annotations

from dataclasses import replace
from pathlib import Path
from typing import Any

from .delivery import (
    CandidateStatus,
    DeliveryIntent,
    DeliveryValidationError,
    _TERMINAL_OUTCOMES,
    _refs_match_or_proven_alias,
    classify_candidate,
    collect_inventory,
    load_delivery_intent,
    validate_transition,
)
from .delivery_receipts import (
    _intent_task_data,
    _read_receipt,
    _receipt_store,
    _validate_receipt_binding,
)
from .delivery_store import DeliveryStoreError
from .io import read_json
from .paths import FILE_TASK_JSON, get_tasks_dir


def archive_guard(
    repo_root: Path,
    task_dir: Path,
    github: Any = None,
) -> tuple[bool, str, DeliveryIntent | None, CandidateStatus | None]:
    """Check archive eligibility before any task/status/session mutation."""
    task_json = task_dir / FILE_TASK_JSON
    if not task_json.is_file():
        return False, "TASK_RECORD_INVALID: task.json is missing", None, None
    try:
        data, intent = _intent_task_data(task_json)
        if intent is None:
            return False, "LEGACY_UNCLASSIFIED: archive requires explicit delivery classification", None, None
        store = _receipt_store(repo_root)
        receipt = _read_receipt(store, intent.candidate_id)
        if receipt is None:
            return False, "RECEIPT_MISSING: archive requires an explicit delivery receipt", intent, None
        _validate_receipt_binding(
            repo_root,
            intent,
            receipt,
            data,
            allow_base_checkout=True,
        )
        inventory = collect_inventory(
            repo_root,
            github=github,
            base_ref=intent.base_ref,
            refresh_remote=True,
        )
        status = classify_candidate(intent, inventory, receipt)
        issues = [*status.issues, *validate_transition(intent, status, "archive")]
        seen_codes: set[str] = set()
        issues = [issue for issue in issues if not (issue.code in seen_codes or seen_codes.add(issue.code))]
        if issues:
            return False, "\n".join(f"{issue.code}: {issue.message}" for issue in issues), intent, status
        return True, "", intent, status
    except (DeliveryValidationError, DeliveryStoreError, ValueError) as error:
        return False, str(error), None, None


def _validate_archived_intent(
    repo_root: Path,
    requested: DeliveryIntent,
    archived: DeliveryIntent,
) -> None:
    """Require the moved typed intent to retain the requested identity."""
    if archived.candidate_id != requested.candidate_id:
        raise DeliveryValidationError(
            "ARCHIVE_INTENT_MISMATCH: archived candidate_id does not match the requested delivery intent"
        )
    if archived.repository != requested.repository:
        raise DeliveryValidationError(
            "ARCHIVE_INTENT_MISMATCH: archived repository does not match the requested delivery intent"
        )
    for field in ("branch_ref", "base_ref"):
        requested_ref = getattr(requested, field)
        archived_ref = getattr(archived, field)
        if requested_ref == archived_ref:
            continue
        if _refs_match_or_proven_alias(repo_root, requested_ref, archived_ref):
            continue
        raise DeliveryValidationError(
            f"ARCHIVE_INTENT_MISMATCH: archived {field} does not match the requested delivery intent"
        )


def record_archive_closure(
    repo_root: Path,
    intent: DeliveryIntent,
    archive_path: Path,
) -> tuple[bool, str]:
    """Record physical archive completion after the task move succeeds."""
    try:
        store = _receipt_store(repo_root)
        receipt = store.read(intent.candidate_id)
        archive_task_json = archive_path / FILE_TASK_JSON
        task_data = read_json(archive_task_json)
        if not isinstance(task_data, dict):
            raise DeliveryValidationError(
                f"cannot read archived task JSON: {archive_task_json}"
            )
        archived_intent = load_delivery_intent(archive_task_json)
        _validate_archived_intent(repo_root, intent, archived_intent)
        _validate_receipt_binding(
            repo_root,
            intent,
            receipt,
            task_data,
            allow_base_checkout=True,
            use_declared_branch=True,
        )
        store.update(
            intent.candidate_id,
            int(receipt["generation"]),
            {
                "closure": "closed",
                "archive_path": archive_path.relative_to(repo_root).as_posix(),
                "next_action": "No delivery action remains; retain the receipt and historical refs.",
            },
        )
        return True, ""
    except (DeliveryStoreError, DeliveryValidationError, KeyError, ValueError) as error:
        return False, str(error)


def recover_archive_closure(
    repo_root: Path,
    archived_task_dir: Path,
) -> tuple[bool, str]:
    """Finish a receipt after a prior archive move was interrupted."""
    task_json = archived_task_dir / FILE_TASK_JSON
    try:
        task_data, archived_intent = _intent_task_data(task_json)
        if archived_intent is None:
            raise DeliveryValidationError(
                "ARCHIVE_RECEIPT_BINDING_CONFLICT: archived task has no delivery intent"
            )
        intent = replace(
            archived_intent,
            task_json=get_tasks_dir(repo_root) / archived_task_dir.name / FILE_TASK_JSON,
        )
        store = _receipt_store(repo_root)
        receipt = store.read(intent.candidate_id)
        _validate_receipt_binding(
            repo_root,
            intent,
            receipt,
            task_data,
            allow_base_checkout=True,
            use_declared_branch=True,
        )
        if receipt.get("closure") == "closed":
            return True, ""
        if receipt.get("outcome") not in _TERMINAL_OUTCOMES or receipt.get("closure") != "verified":
            return False, "ARCHIVE_RECEIPT_NOT_READY: archived task does not have a verified terminal receipt"
        return record_archive_closure(repo_root, intent, archived_task_dir)
    except (DeliveryValidationError, DeliveryStoreError, KeyError, ValueError) as error:
        return False, str(error)
