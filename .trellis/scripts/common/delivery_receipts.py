"""Receipt registration and binding helpers for the delivery workflow."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .delivery import (
    DeliveryIntent,
    DeliveryValidationError,
    _TERMINAL_OUTCOMES,
    _branch_name,
    _git_oid,
    _normalise_ref,
    delivery_store_root,
    load_delivery_intent,
)
from .delivery_store import DeliveryStore, ReceiptExists
from .git import run_git
from .io import read_json


def _current_branch(repo_root: Path) -> str | None:
    code, stdout, _ = run_git(["branch", "--show-current"], cwd=repo_root)
    return stdout.strip() if code == 0 and stdout.strip() else None


def _intent_task_data(task_json: Path) -> tuple[dict[str, Any], DeliveryIntent | None]:
    data = read_json(task_json)
    if not isinstance(data, dict):
        raise DeliveryValidationError(f"cannot read task JSON: {task_json}")
    meta = data.get("meta")
    if not isinstance(meta, dict) or "delivery" not in meta:
        return data, None
    return data, load_delivery_intent(task_json)


def _receipt_store(repo_root: Path) -> DeliveryStore:
    return DeliveryStore(delivery_store_root(repo_root))


def _read_receipt(store: DeliveryStore, candidate_id: str) -> dict[str, Any] | None:
    path = store.path_for(candidate_id)
    if not path.exists():
        return None
    return store.read(candidate_id)


def _tip_for_intent(repo_root: Path, intent: DeliveryIntent, data: dict[str, Any]) -> str | None:
    branch = intent.branch_ref or data.get("branch")
    return _git_oid(repo_root, branch)


def _attached_branch_ref(
    repo_root: Path,
    intent: DeliveryIntent,
    data: dict[str, Any] | None = None,
) -> str | None:
    """Resolve the branch currently attaching a receipt to one worktree."""
    current = _normalise_ref(_current_branch(repo_root))
    if current is not None:
        return current
    branch: Any = intent.branch_ref
    if branch is None and isinstance(data, dict):
        branch = data.get("branch")
    return _normalise_ref(branch) if isinstance(branch, str) and branch.strip() else None


def _initial_receipt(
    repo_root: Path,
    intent: DeliveryIntent,
    data: dict[str, Any],
    phase: str = "registered",
) -> dict[str, Any]:
    task_path: str | None = None
    if intent.task_json is not None:
        try:
            task_path = intent.task_json.resolve().relative_to(repo_root.resolve()).as_posix()
        except ValueError:
            task_path = intent.task_json.resolve().as_posix()
    return {
        "repository": intent.repository,
        "task_path": task_path,
        "owner": intent.owner,
        "phase": phase,
        "session_state": "attached",
        "outcome": None,
        "closure": "open",
        "next_action": "Implement the candidate and record exact review/check/acceptance evidence.",
        "attached_branch_ref": _attached_branch_ref(repo_root, intent, data),
        "observed_tip": _tip_for_intent(repo_root, intent, data),
        "registered_tip": _tip_for_intent(repo_root, intent, data),
        "registered_base_oid": _git_oid(repo_root, intent.base_ref),
        "evidence_kind": "lifecycle",
        "evidence": {},
        "recorded_by": intent.owner,
        "authority_ref": intent.authority_ref,
    }


def _validate_receipt_binding(
    repo_root: Path,
    intent: DeliveryIntent,
    receipt: dict[str, Any],
    data: dict[str, Any] | None = None,
    allow_base_checkout: bool = False,
    allow_missing_attachment: bool = False,
) -> None:
    """Reject an existing receipt bound to another task, owner, or branch.

    Evidence recording and terminal closure may run from the canonical base
    checkout after integration. That checkout is allowed to observe the
    candidate's existing attachment, but it cannot create or repair a missing
    attachment. A non-base checkout must match the live branch identity.
    """
    if receipt.get("repository") != intent.repository:
        raise DeliveryValidationError(
            f"RECEIPT_BINDING_CONFLICT: receipt repository does not match {intent.repository}"
        )
    if receipt.get("owner") != intent.owner:
        raise DeliveryValidationError(
            f"RECEIPT_BINDING_CONFLICT: receipt owner does not match {intent.owner}"
        )
    expected_path: str | None = None
    if intent.task_json is not None:
        try:
            expected_path = intent.task_json.resolve().relative_to(repo_root.resolve()).as_posix()
        except ValueError:
            expected_path = intent.task_json.resolve().as_posix()
    if receipt.get("task_path") != expected_path:
        raise DeliveryValidationError(
            "RECEIPT_BINDING_CONFLICT: receipt task path does not match the delivery intent"
        )
    expected_branch = _attached_branch_ref(repo_root, intent, data)
    attached_branch = receipt.get("attached_branch_ref")
    terminal_receipt = receipt.get("outcome") in _TERMINAL_OUTCOMES and receipt.get("closure") in {"verified", "closed"}
    base_checkout = (
        allow_base_checkout
        and expected_branch
        and intent.base_ref
        and _branch_name(expected_branch) == _branch_name(intent.base_ref)
    )
    has_attachment = isinstance(attached_branch, str) and bool(attached_branch.strip())
    if expected_branch and not has_attachment and not terminal_receipt and not allow_missing_attachment:
        raise DeliveryValidationError(
            "RECEIPT_BINDING_CONFLICT: receipt has no attached branch binding; "
            "recover the candidate explicitly before reusing it"
        )
    if (
        expected_branch
        and has_attachment
        and _branch_name(attached_branch) != _branch_name(expected_branch)
        and not base_checkout
    ):
        raise DeliveryValidationError(
            "RECEIPT_BINDING_CONFLICT: receipt branch does not match the delivery intent"
        )


def _repair_missing_receipt_binding(
    repo_root: Path,
    intent: DeliveryIntent,
    data: dict[str, Any],
    receipt: dict[str, Any],
) -> dict[str, Any]:
    """Recover legacy authority and branch bindings from intent using CAS."""
    attached_branch = receipt.get("attached_branch_ref")
    has_attachment = isinstance(attached_branch, str) and bool(attached_branch.strip())
    _validate_receipt_binding(
        repo_root, intent, receipt, data, allow_missing_attachment=True
    )
    receipt_authority = receipt.get("authority_ref")
    if (
        intent.authority_ref is not None
        and receipt_authority not in (None, "", intent.authority_ref)
    ):
        raise DeliveryValidationError(
            "RECEIPT_BINDING_CONFLICT: receipt authority_ref does not match the delivery intent"
        )
    changes: dict[str, Any] = {}
    if intent.authority_ref is not None and not receipt_authority:
        changes["authority_ref"] = intent.authority_ref
    terminal_receipt = (
        receipt.get("outcome") in _TERMINAL_OUTCOMES
        and receipt.get("closure") in {"verified", "closed"}
    )
    if has_attachment or terminal_receipt:
        if not changes:
            return receipt
        return _receipt_store(repo_root).update(
            intent.candidate_id, int(receipt["generation"]), changes
        )

    current_ref = _normalise_ref(_current_branch(repo_root))
    declared_ref = _normalise_ref(intent.branch_ref) or _normalise_ref(data.get("branch"))
    if current_ref is None or declared_ref is None:
        raise DeliveryValidationError("RECEIPT_BINDING_CONFLICT: explicit recovery requires a live checkout of the declared candidate branch")
    if _branch_name(current_ref) == _branch_name(intent.base_ref):
        raise DeliveryValidationError("RECEIPT_BINDING_CONFLICT: explicit recovery cannot use the integration base branch")
    if _branch_name(current_ref) != _branch_name(declared_ref):
        raise DeliveryValidationError(f"current branch {_branch_name(current_ref)!r} does not match declared candidate branch {_branch_name(declared_ref)!r}")

    changes["attached_branch_ref"] = current_ref
    return _receipt_store(repo_root).update(
        intent.candidate_id, int(receipt["generation"]), changes
    )


def ensure_receipt(
    repo_root: Path,
    intent: DeliveryIntent,
    data: dict[str, Any],
    phase: str = "registered",
) -> dict[str, Any]:
    """Create a receipt exactly once for a validated intent."""
    store = _receipt_store(repo_root)
    existing = _read_receipt(store, intent.candidate_id)
    if existing is not None:
        _validate_receipt_binding(repo_root, intent, existing, data)
        return existing
    try:
        return store.create(intent.candidate_id, _initial_receipt(repo_root, intent, data, phase))
    except ReceiptExists:
        # Another linked worktree won the create race. Re-read its complete
        # receipt instead of turning an idempotent start/resume into failure.
        existing = store.read(intent.candidate_id)
        _validate_receipt_binding(repo_root, intent, existing, data)
        return existing
