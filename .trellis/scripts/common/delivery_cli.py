"""Command handlers for the explicit Trellis delivery workflow."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from dataclasses import replace
from pathlib import Path
from typing import Any

from .active_task import clear_active_task, resolve_active_task
from .delivery_audit import project_audit, strict_audit_pass
from .delivery import (
    CandidateStatus,
    DeliveryIntent,
    DeliveryIssue,
    DeliveryValidationError,
    _TERMINAL_OUTCOMES,
    _branch_name,
    _choose_base_ref,
    _git_oid,
    _is_remote_head_alias,
    _is_symbolic_head_ref,
    _DELIVERY_EVIDENCE_SCHEMA,
    _github_checks_match_pr,
    _github_evidence_matches_pr,
    _github_review_matches_pr,
    _normalise_ref,
    _pr_for_candidate,
    _worktree_matches_candidate,
    classify_candidate,
    collect_inventory,
    load_delivery_intent,
    render_checkpoint,
    repository_identity,
    validate_evidence_mapping,
    validate_delivery_mapping,
    validate_transition,
)
from .delivery_store import (
    DeliveryStore,
    DeliveryStoreError,
    ReceiptCorrupt,
)
from .delivery_receipts import (
    _attached_branch_ref,
    _current_branch,
    _intent_task_data,
    _read_receipt,
    _receipt_store,
    _repair_missing_receipt_binding,
    _validate_receipt_binding,
    ensure_receipt,
)
from .git import run_git
from .io import read_json, write_json
from .paths import FILE_TASK_JSON, get_repo_root, get_tasks_dir
from .task_utils import find_task_by_name, resolve_task_dir


_EVIDENCE_SECTIONS = {
    "review",
    "checks",
    "postmerge",
    "acceptance",
    "durable",
    "tag",
    "version",
    "assets",
    "checksums",
    "install",
    "decision",
    "replacement",
    "children",
    "release",
}
_EVIDENCE_METADATA = {"observed_tip"}
_MAX_EVIDENCE_BYTES = 64 * 1024


def _validate_base_branch_ref(repo_root: Path, raw_ref: str | None) -> str:
    """Require a raw base value to identify an existing commit branch ref."""
    if not isinstance(raw_ref, str) or not raw_ref.strip():
        raise DeliveryValidationError("BASE_REF_INVALID: provide an explicit branch ref")
    value = raw_ref.strip()
    if _is_symbolic_head_ref(value):
        raise DeliveryValidationError(
            "BASE_REF_INVALID: base ref must name a branch, not HEAD or @"
        )
    normalized = _normalise_ref(value)
    if (
        normalized is None
        or not normalized.startswith(("refs/heads/", "refs/remotes/"))
        or _is_remote_head_alias(normalized)
    ):
        raise DeliveryValidationError(
            f"BASE_REF_INVALID: {value!r} is not an explicit local or remote branch ref"
        )
    check_code, _, _ = run_git(["check-ref-format", normalized], cwd=repo_root)
    show_code, _, _ = run_git(
        ["show-ref", "--verify", "--quiet", normalized], cwd=repo_root
    )
    if check_code != 0 or show_code != 0 or _git_oid(repo_root, normalized + "^{commit}") is None:
        raise DeliveryValidationError(
            f"BASE_REF_INVALID: {value!r} does not resolve to an existing branch commit"
        )
    return normalized


def _assert_current_repository_matches(
    repo_root: Path, intent: DeliveryIntent
) -> None:
    try:
        current_repository = repository_identity(repo_root)
    except DeliveryValidationError as error:
        raise DeliveryValidationError(
            f"REPOSITORY_IDENTITY_UNAVAILABLE: {error}"
        ) from error
    if current_repository != intent.repository:
        raise DeliveryValidationError(
            "REPOSITORY_IDENTITY_MISMATCH: current checkout origin "
            f"{current_repository!r} does not match intent {intent.repository!r}"
        )


def _task_registration_lock_id(repo_root: Path, task_json: Path) -> str:
    try:
        task_path = task_json.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError as error:
        raise DeliveryValidationError(
            "TASK_PATH_INVALID: delivery registration task must be inside this repository"
        ) from error
    return "task-" + hashlib.sha256(task_path.encode("utf-8")).hexdigest()


def _error(message: str) -> int:
    print(f"Error: {message}", file=sys.stderr)
    return 1


def _error_code(message: str, code: int) -> int:
    print(f"Error: {message}", file=sys.stderr)
    return code


def _task_dir(args: argparse.Namespace, repo_root: Path) -> Path:
    return resolve_task_dir(args.task, repo_root)


def _task_json(args: argparse.Namespace, repo_root: Path) -> Path:
    return _task_dir(args, repo_root) / FILE_TASK_JSON


def pending_owned_candidate(
    repo_root: Path, owner: str, candidate_id: str
) -> dict[str, Any] | None:
    """Return another unfinished candidate owned by the same coordinator.

    The receipt store is an operational cache, not the ownership ledger. Scan
    active task intents as well so a missing or corrupt receipt cannot make an
    unfinished candidate disappear from the start guard.
    """
    store = _receipt_store(repo_root)
    terminal = {"delivered", "superseded", "rejected", "cancelled"}
    if store.root.is_dir():
        for receipt_path in sorted(store.root.glob("*.json")):
            other_id = receipt_path.stem
            if other_id == candidate_id:
                continue
            try:
                receipt = store.read(other_id)
            except (DeliveryStoreError, ReceiptCorrupt):
                # A corrupt orphan cannot establish ownership by itself. If an
                # active task binds it, the task scan below reports the hold.
                continue
            if receipt.get("owner") != owner:
                continue
            if receipt.get("outcome") in terminal and receipt.get("closure") in {
                "verified",
                "closed",
            }:
                continue
            return {
                "candidate_id": other_id,
                "task_path": receipt.get("task_path"),
                "phase": receipt.get("phase") or "held",
                "next_action": receipt.get("next_action") or "resolve the candidate",
            }

    tasks_root = get_tasks_dir(repo_root)
    if tasks_root.is_dir():
        for other_json in sorted(tasks_root.rglob(FILE_TASK_JSON)):
            try:
                other_intent = load_delivery_intent(other_json)
            except DeliveryValidationError:
                continue
            if other_intent.candidate_id == candidate_id or other_intent.owner != owner:
                continue
            task_path = other_json.resolve().relative_to(repo_root.resolve()).as_posix()
            receipt_path = store.path_for(other_intent.candidate_id)
            if not receipt_path.is_file():
                return {
                    "candidate_id": other_intent.candidate_id,
                    "task_path": task_path,
                    "phase": "held",
                    "next_action": "receipt_missing: register or reconstruct the candidate receipt",
                }
            try:
                receipt = store.read(other_intent.candidate_id)
            except ReceiptCorrupt:
                return {
                    "candidate_id": other_intent.candidate_id,
                    "task_path": task_path,
                    "phase": "held",
                    "next_action": "receipt_corrupt: preserve and recover the candidate receipt",
                }
            except DeliveryStoreError:
                return {
                    "candidate_id": other_intent.candidate_id,
                    "task_path": task_path,
                    "phase": "held",
                    "next_action": "receipt_unavailable: inspect the delivery store",
                }
            try:
                other_data = read_json(other_json)
                if not isinstance(other_data, dict):
                    raise DeliveryValidationError(
                        "TASK_RECORD_INVALID: task.json is not an object"
                    )
                _validate_receipt_binding(
                    repo_root,
                    other_intent,
                    receipt,
                    other_data,
                    use_declared_branch=True,
                )
            except DeliveryValidationError as error:
                return {
                    "candidate_id": other_intent.candidate_id,
                    "task_path": task_path,
                    "phase": "held",
                    "next_action": str(error),
                }
            if receipt.get("owner") != owner:
                continue
            if receipt.get("outcome") in terminal and receipt.get("closure") in {
                "verified",
                "closed",
            }:
                continue
            return {
                "candidate_id": other_intent.candidate_id,
                "task_path": task_path,
                "phase": receipt.get("phase") or "held",
                "next_action": receipt.get("next_action") or "resolve the candidate",
            }
    return None


def _candidate_bound_elsewhere(repo_root: Path, task_json: Path, candidate_id: str) -> Path | None:
    """Find another task that already owns ``candidate_id``."""
    tasks_root = get_tasks_dir(repo_root)
    if not tasks_root.is_dir():
        return None
    requested = task_json.resolve()
    for other_json in sorted(tasks_root.rglob(FILE_TASK_JSON)):
        if other_json.resolve() == requested:
            continue
        try:
            other = load_delivery_intent(other_json)
        except DeliveryValidationError:
            continue
        if other.candidate_id == candidate_id:
            return other_json
    return None


def register_task(
    repo_root: Path,
    task_json: Path,
    candidate_id: str,
    owner: str,
    kind: str = "integration",
    base_ref: str | None = None,
    acceptance_ref: str | None = "prd.md#acceptance",
    authority_ref: str | None = None,
    version: str | None = None,
    required_assets: list[str] | None = None,
) -> DeliveryIntent:
    """Serialize task registration and persist its resumable receipt."""
    # Validate legacy recovery authority and raw CLI input before creating the
    # task lock file. The same checks run again under the task-wide lock so a
    # competing registration cannot change the intent between preflight and
    # mutation.
    data = read_json(task_json)
    if not isinstance(data, dict):
        raise DeliveryValidationError(f"cannot read task JSON: {task_json}")
    existing = data.get("meta", {}).get("delivery") if isinstance(data.get("meta"), dict) else None
    if isinstance(existing, dict):
        _assert_current_repository_matches(repo_root, load_delivery_intent(task_json))
    if base_ref is not None:
        _validate_base_branch_ref(repo_root, base_ref)

    store = _receipt_store(repo_root)
    task_lock_id = _task_registration_lock_id(repo_root, task_json)
    with store._locked(task_lock_id):
        return _register_task_locked(
            repo_root,
            task_json,
            candidate_id,
            owner,
            kind,
            base_ref,
            acceptance_ref,
            authority_ref,
            version,
            required_assets,
        )


def _register_task_locked(
    repo_root: Path,
    task_json: Path,
    candidate_id: str,
    owner: str,
    kind: str,
    base_ref: str | None,
    acceptance_ref: str | None,
    authority_ref: str | None,
    version: str | None,
    required_assets: list[str] | None,
) -> DeliveryIntent:
    """Persist registration while the task-wide lock is held."""
    if base_ref is not None:
        _validate_base_branch_ref(repo_root, base_ref)
    data = read_json(task_json)
    if not isinstance(data, dict):
        raise DeliveryValidationError(f"cannot read task JSON: {task_json}")
    collision = _candidate_bound_elsewhere(repo_root, task_json, candidate_id)
    if collision is not None:
        raise DeliveryValidationError(
            f"candidate {candidate_id!r} is already bound to {collision.relative_to(repo_root).as_posix()}"
        )
    existing = data.get("meta", {}).get("delivery") if isinstance(data.get("meta"), dict) else None
    if isinstance(existing, dict):
        _assert_current_repository_matches(repo_root, load_delivery_intent(task_json))
    current_ref = _normalise_ref(_current_branch(repo_root))
    declared_ref = _normalise_ref(data.get("branch"))
    if isinstance(existing, dict):
        declared_ref = _normalise_ref(existing.get("branch_ref")) or declared_ref
    if declared_ref and current_ref and _branch_name(declared_ref) != _branch_name(current_ref):
        raise DeliveryValidationError(
            f"current branch {_branch_name(current_ref)!r} does not match declared candidate branch {_branch_name(declared_ref)!r}"
        )
    if isinstance(existing, dict):
        if existing.get("candidate_id") != candidate_id or existing.get("owner") != owner:
            raise DeliveryValidationError("task already has a different delivery owner or candidate")
        intent = load_delivery_intent(task_json)
        updated_task = False
        if authority_ref is not None:
            if intent.authority_ref and intent.authority_ref != authority_ref:
                raise DeliveryValidationError(
                    "delivery intent already has a different authority_ref"
                )
            if not intent.authority_ref:
                updated_delivery = dict(existing)
                updated_delivery["authority_ref"] = authority_ref
                intent = validate_delivery_mapping(updated_delivery, task_json)
                meta = data.get("meta") if isinstance(data.get("meta"), dict) else {}
                meta["delivery"] = updated_delivery
                data["meta"] = meta
                updated_task = True
        existing_receipt = _read_receipt(_receipt_store(repo_root), intent.candidate_id)
        if existing_receipt is not None:
            _repair_missing_receipt_binding(repo_root, intent, data, existing_receipt)
        ensure_receipt(repo_root, intent, data)
        if updated_task and not write_json(task_json, data):
            raise DeliveryValidationError(f"cannot write delivery intent: {task_json}")
        return intent

    raw_base = base_ref if base_ref is not None else data.get("base_branch") or _choose_base_ref(repo_root)
    chosen_base = _validate_base_branch_ref(repo_root, raw_base)
    branch = _normalise_ref(data.get("branch")) or _normalise_ref(_current_branch(repo_root))
    delivery = {
        "schema_version": 1,
        "kind": kind,
        "candidate_id": candidate_id,
        "owner": owner,
        "repository": repository_identity(repo_root),
        "base_ref": chosen_base,
        "branch_ref": branch,
        "acceptance_ref": acceptance_ref,
        "authority_ref": authority_ref,
    }
    if version is not None:
        delivery["version"] = version
    if required_assets is not None:
        delivery["required_assets"] = required_assets
    intent = validate_delivery_mapping(delivery, task_json)
    if _current_branch(repo_root) is None and _attached_branch_ref(repo_root, intent, data) is None:
        head_oid = _git_oid(repo_root, "HEAD") or "unresolved"
        declared_oid = _git_oid(repo_root, intent.branch_ref or "") or "unresolved"
        raise DeliveryValidationError(
            "BRANCH_BINDING_UNPROVEN: detached HEAD "
            f"{head_oid} does not match declared branch {intent.branch_ref!r} at {declared_oid}"
        )
    existing_receipt = _read_receipt(_receipt_store(repo_root), intent.candidate_id)
    if existing_receipt is not None:
        _repair_missing_receipt_binding(repo_root, intent, data, existing_receipt)
    meta = data.get("meta") if isinstance(data.get("meta"), dict) else {}
    meta["delivery"] = delivery
    data["meta"] = meta
    ensure_receipt(repo_root, intent, data)
    if not write_json(task_json, data):
        raise DeliveryValidationError(f"cannot write delivery intent: {task_json}")
    return intent


def _status_for_task(
    repo_root: Path,
    task_json: Path,
    github: Any = None,
    receipt_override: dict[str, Any] | None = None,
    allow_base_checkout: bool = False,
    refresh_remote: bool = False,
) -> tuple[DeliveryIntent, CandidateStatus, Any, dict[str, Any]]:
    data = read_json(task_json)
    if not isinstance(data, dict):
        raise DeliveryValidationError(f"cannot read task JSON: {task_json}")
    intent = load_delivery_intent(task_json)
    store = _receipt_store(repo_root)
    receipt = receipt_override if receipt_override is not None else _read_receipt(store, intent.candidate_id)
    if receipt is not None:
        _validate_receipt_binding(
            repo_root,
            intent,
            receipt,
            data,
            allow_base_checkout=allow_base_checkout,
        )
    inventory = collect_inventory(
        repo_root,
        github=github,
        base_ref=intent.base_ref,
        refresh_remote=refresh_remote,
    )
    status = classify_candidate(intent, inventory, receipt)
    return intent, status, inventory, data


def _print_status(status: CandidateStatus, output_format: str) -> None:
    if output_format == "json":
        print(json.dumps(status.as_dict(), indent=2, sort_keys=True))
        return
    print(f"candidate: {status.candidate_id}")
    print(f"owner: {status.owner}")
    print(f"phase: {status.phase}")
    print(f"outcome: {status.outcome or 'unknown'}")
    print(f"integration: {status.integration}")
    print(f"tip: {status.tip or 'unknown'}")
    print(f"dirty: {'yes' if status.dirty else 'no'}")
    print(f"next: {status.next_action}")
    for issue in status.issues:
        print(f"issue[{issue.code}]: {issue.message}")


def cmd_delivery_register(args: argparse.Namespace) -> int:
    """Register a task's stable delivery identity and owner."""
    repo_root = get_repo_root()
    task_json = _task_json(args, repo_root)
    if not task_json.is_file():
        return _error(f"task.json not found at {task_json}")
    try:
        intent = register_task(
            repo_root,
            task_json,
            args.candidate,
            args.owner,
            kind=args.kind,
            base_ref=args.base_ref,
            acceptance_ref=args.acceptance_ref,
            authority_ref=args.authority_ref,
            version=getattr(args, "version", None),
            required_assets=getattr(args, "required_asset", None),
        )
    except (DeliveryValidationError, DeliveryStoreError, ValueError) as error:
        return _error_code(str(error), 2)
    print(f"registered: {intent.candidate_id} owner={intent.owner}")
    return 0


def cmd_delivery_inspect(args: argparse.Namespace) -> int:
    """Inspect one current candidate without writing state."""
    repo_root = get_repo_root()
    task_json = _task_json(args, repo_root)
    try:
        _, status, _, _ = _status_for_task(repo_root, task_json)
    except (DeliveryValidationError, DeliveryStoreError, ValueError) as error:
        return _error_code(str(error), 2)
    _print_status(status, args.format)
    return 0


def _load_evidence(path: Path) -> dict[str, Any]:
    raw = path.read_bytes()
    if len(raw) > _MAX_EVIDENCE_BYTES:
        raise DeliveryValidationError("evidence file exceeds the 64 KiB bound")
    try:
        data = json.loads(raw.decode("utf-8"))
    except (UnicodeError, json.JSONDecodeError) as error:
        raise DeliveryValidationError("evidence file must be UTF-8 JSON") from error
    if not isinstance(data, dict):
        raise DeliveryValidationError("evidence file must contain a JSON object")
    if data.get("schema_version") == "assura.release-receipt.v1":
        # The release helper emits its durable receipt as a top-level object;
        # store it under the lifecycle evidence envelope without rewriting or
        # weakening its schema.
        return {"release": data}
    unknown = sorted(set(data) - _EVIDENCE_SECTIONS - _EVIDENCE_METADATA)
    if unknown:
        raise DeliveryValidationError(f"unsupported evidence sections: {', '.join(unknown)}")
    if not data:
        raise DeliveryValidationError("evidence file must contain at least one evidence section")
    invalid_sections = sorted(
        section
        for section, value in data.items()
        if section not in _EVIDENCE_METADATA and not isinstance(value, dict)
    )
    if invalid_sections:
        raise DeliveryValidationError(
            "evidence sections must be JSON objects: " + ", ".join(invalid_sections)
        )
    if "observed_tip" in data and (
        not isinstance(data["observed_tip"], str) or not data["observed_tip"].strip()
    ):
        raise DeliveryValidationError("evidence observed_tip must be a non-empty string")
    return data


def cmd_delivery_record(args: argparse.Namespace) -> int:
    """Record bounded evidence using compare-and-swap generation."""
    repo_root = get_repo_root()
    task_json = _task_json(args, repo_root)
    try:
        evidence = _load_evidence(Path(args.evidence_file))
        intent, status, inventory, data = _status_for_task(
            repo_root, task_json, allow_base_checkout=True
        )
        store = _receipt_store(repo_root)
        receipt = _read_receipt(store, intent.candidate_id)
        if receipt is None:
            return _error_code(
                "RECEIPT_MISSING: run delivery register or start the task first", 2
            )
        _validate_receipt_binding(
            repo_root,
            intent,
            receipt,
            data,
            allow_base_checkout=True,
        )
        if receipt.get("outcome") in _TERMINAL_OUTCOMES and receipt.get("closure") in {
            "verified",
            "closed",
        }:
            return _error_code(
                "CANDIDATE_TERMINAL: terminal delivery receipts are immutable; archive cleanup is the only remaining transition",
                1,
            )
        tip = status.tip
        if tip is None:
            return _error_code(
                "CANDIDATE_TIP_UNRESOLVED: resolve the current branch or PR head before recording evidence",
                2,
            )
        evidence_tip = evidence.get("observed_tip")
        if evidence_tip and evidence_tip != tip:
            return _error_code(
                "evidence observed_tip does not match the current candidate tip", 2
            )
        for section, value in evidence.items():
            if section == "observed_tip":
                continue
            validate_evidence_mapping(
                {section: value}, intent, tip, inventory.base_oid
            )
            if isinstance(value, dict) and value.get("source") == "github":
                pull_request = _pr_for_candidate(intent, inventory, tip)
                if section == "review":
                    provider_verified = _github_review_matches_pr(value, pull_request)
                elif section == "checks":
                    provider_verified = _github_checks_match_pr(value, pull_request)
                else:
                    provider_verified = _github_evidence_matches_pr(value, pull_request)
                if not provider_verified:
                    return _error_code(
                        f"evidence {section} is not verified by the current GitHub pull request",
                        2,
                    )
            for field in ("head_oid", "observed_tip"):
                observed = value.get(field)
                if observed is not None and observed != tip:
                    return _error_code(
                        f"evidence {section}.{field} does not match the current candidate tip",
                        2,
                    )
        merged_evidence = dict(receipt.get("evidence") or {})
        merged_evidence.update(evidence)
        updated = store.update(
            intent.candidate_id,
            args.expected_generation,
            {
                "evidence": merged_evidence,
                "observed_tip": tip,
                "recorded_by": intent.owner,
                "phase": "integrated_pending_verification" if any(key in merged_evidence for key in ("postmerge", "acceptance")) else "review",
                "next_action": "Record the remaining exact evidence, then close the candidate with an explicit outcome.",
            },
        )
    except (DeliveryValidationError, DeliveryStoreError, ValueError) as error:
        return _error_code(str(error), 2)
    print(json.dumps(updated, indent=2, sort_keys=True))
    return 0


def _decision_evidence(
    args: argparse.Namespace,
    outcome: str,
    tip: str | None,
    intent: DeliveryIntent,
) -> dict[str, Any]:
    if outcome == "delivered":
        return {}
    if getattr(args, "decision_file", None):
        decision = _load_evidence(Path(args.decision_file))
        if "decision" not in decision:
            raise DeliveryValidationError("decision file must contain a decision section")
        decision_section = decision["decision"]
        if not isinstance(decision_section.get("reason"), str) or not decision_section["reason"].strip():
            raise DeliveryValidationError("decision section requires a non-empty reason")
        result = dict(decision)
    else:
        reason = getattr(args, "reason", None)
        if not reason:
            raise DeliveryValidationError("non-delivered closure requires --reason or --decision-file")
        if len(reason.encode("utf-8")) > 2048:
            raise DeliveryValidationError("decision reason exceeds the 2 KiB bound")
        if not intent.authority_ref:
            raise DeliveryValidationError(
                "non-delivered closure requires an explicit intent authority_ref"
            )
        result = {
            "decision": {
                "schema_version": _DELIVERY_EVIDENCE_SCHEMA,
                "source": "owner",
                "repository": intent.repository,
                "result": "verified",
                "reason": reason,
                "head_oid": tip,
                "authority_ref": intent.authority_ref,
                "decided_by": intent.owner,
                "decision_ref": "cli:explicit-decision",
                "evidence_ref": "cli:explicit-decision",
            }
        }
    if outcome == "superseded":
        replacement = getattr(args, "replacement", None)
        if not replacement and "replacement" not in result:
            raise DeliveryValidationError("superseded closure requires --replacement")
        if replacement:
            result["replacement"] = {
                "schema_version": _DELIVERY_EVIDENCE_SCHEMA,
                "source": "owner",
                "repository": intent.repository,
                "result": "verified",
                "head_oid": tip,
                "authority_ref": intent.authority_ref,
                "evidence_ref": "cli:explicit-replacement",
                "reference": replacement,
                "recovery_ref": replacement,
                "remaining_diff": "reviewed",
            }
    return result


def cmd_delivery_close(args: argparse.Namespace) -> int:
    """Validate and record an explicit terminal outcome."""
    repo_root = get_repo_root()
    task_json = _task_json(args, repo_root)
    try:
        intent, current, inventory, data = _status_for_task(
            repo_root, task_json, allow_base_checkout=True, refresh_remote=True
        )
        store = _receipt_store(repo_root)
        receipt = _read_receipt(store, intent.candidate_id)
        if receipt is None:
            return _error_code(
                "RECEIPT_MISSING: run delivery register or start the task first", 2
            )
        _validate_receipt_binding(
            repo_root,
            intent,
            receipt,
            data,
            allow_base_checkout=True,
        )
        existing_outcome = receipt.get("outcome")
        if (
            existing_outcome in _TERMINAL_OUTCOMES
            and existing_outcome != args.outcome
        ):
            return _error_code(
                "CANDIDATE_TERMINAL: TERMINAL_OUTCOME_IMMUTABLE: record cleanup or a new candidate instead of changing the terminal outcome",
                1,
            )
        if receipt.get("outcome") in _TERMINAL_OUTCOMES and receipt.get("closure") in {
            "verified",
            "closed",
        }:
            if current.outcome == args.outcome and not current.issues:
                print(json.dumps(receipt, indent=2, sort_keys=True))
                return 0
            return _error_code(
                "CANDIDATE_TERMINAL: a verified terminal outcome cannot be replaced",
                1,
            )
        changes = _decision_evidence(args, args.outcome, current.tip, intent)
        if changes:
            validate_evidence_mapping(
                changes, intent, current.tip, inventory.base_oid
            )
        prospective_evidence = dict(receipt.get("evidence") or {})
        prospective_evidence.update(changes)
        prospective = dict(receipt)
        prospective["evidence"] = prospective_evidence
        prospective["outcome"] = args.outcome
        prospective["closure"] = "verified"
        prospective["observed_tip"] = current.tip
        _validate_receipt_binding(
            repo_root,
            intent,
            prospective,
            data,
            allow_base_checkout=True,
        )
        status = classify_candidate(intent, inventory, prospective)
        transition_issues = [*status.issues, *validate_transition(intent, status, "close")]
        seen_codes: set[str] = set()
        transition_issues = [
            issue
            for issue in transition_issues
            if not (issue.code in seen_codes or seen_codes.add(issue.code))
        ]
        if transition_issues:
            for issue in transition_issues:
                print(f"{issue.code}: {issue.message}", file=sys.stderr)
            unavailable = any(
                issue.code.endswith("UNAVAILABLE")
                or issue.code in {"RECEIPT_MISSING", "CANDIDATE_TIP_UNRESOLVED"}
                for issue in transition_issues
            )
            return 2 if unavailable else 1
        expected_generation = args.expected_generation
        if expected_generation is None:
            expected_generation = int(receipt["generation"])
        updated = store.update(
            intent.candidate_id,
            expected_generation,
            {
                "evidence": prospective_evidence,
                "outcome": args.outcome,
                "closure": "verified",
                "phase": "verified",
                "observed_tip": current.tip,
                "next_action": "Perform only authorized cleanup, then archive or record closure." if args.outcome == "delivered" else "Retain the exact disposition and historical refs; no merge is implied.",
                "recorded_by": intent.owner,
            },
        )
    except (DeliveryValidationError, DeliveryStoreError, ValueError) as error:
        return _error_code(str(error), 2)
    print(json.dumps(updated, indent=2, sort_keys=True))
    return 0




def cmd_delivery_audit(args: argparse.Namespace) -> int:
    """Audit all local tasks and topology as a read-only operation."""
    repo_root = get_repo_root()
    report, statuses = project_audit(
        repo_root, owner=args.owner, refresh_remote=getattr(args, "refresh", False)
    )
    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(f"repository: {report['repository']}")
        print(f"base: {report['base_ref']}@{report['base_oid'] or 'unknown'}")
        print(f"coverage: {report['coverage']}")
        for status in statuses:
            print(f"{status.candidate_id}: {status.outcome or status.phase}; next: {status.next_action}")
        print(f"unowned refs: {len(report['unowned_refs'])}")
        print(f"unowned worktrees: {len(report['unowned_worktrees'])}")
    if args.strict and not strict_audit_pass(report, statuses, args.owner):
        return 1
    return 0


def cmd_delivery_next(args: argparse.Namespace) -> int:
    """Show the next owned action without dispatching it."""
    report, statuses = project_audit(get_repo_root(), owner=args.owner)
    pending = [status for status in statuses if status.outcome not in {"delivered", "superseded", "rejected", "cancelled"}]
    if args.format == "json":
        print(json.dumps([status.as_dict() for status in pending], indent=2, sort_keys=True))
    else:
        for status in pending:
            print(f"{status.candidate_id}: {status.next_action}")
        if not pending:
            print("no pending owned delivery actions")
    return 0


def cmd_delivery_checkpoint(args: argparse.Namespace) -> int:
    """Render the small deterministic checkpoint used for handoff."""
    _, statuses = project_audit(get_repo_root(), owner=args.owner)
    print(render_checkpoint(statuses))
    return 0


def prepare_task_start(repo_root: Path, task_json: Path) -> tuple[bool, str]:
    """Ensure a v1 task has an owner receipt before active state is set."""
    try:
        data, intent = _intent_task_data(task_json)
    except DeliveryValidationError as error:
        return False, str(error)
    if intent is None:
        return False, "LEGACY_UNCLASSIFIED: classify the task with delivery register before starting new implementation"
    current_branch = _current_branch(repo_root)
    declared_branch = _branch_name(intent.branch_ref)
    if declared_branch and current_branch and declared_branch != _branch_name(current_branch):
        return False, (
            f"BRANCH_MISMATCH: current branch {_branch_name(current_branch)!r} does not match "
            f"candidate branch {declared_branch!r}"
        )
    pending = pending_owned_candidate(repo_root, intent.owner, intent.candidate_id)
    if pending is not None:
        return False, (
            f"UNFINISHED_OWNED_CANDIDATE: {pending['candidate_id']} remains in "
            f"{pending['phase']} ({pending['task_path'] or 'task path unknown'}); "
            f"next: {pending['next_action']}"
        )
    try:
        receipt = ensure_receipt(repo_root, intent, data, phase="implementing")
        if receipt.get("outcome") in {"delivered", "superseded", "rejected", "cancelled"} and receipt.get("closure") in {"verified", "closed"}:
            return False, "CANDIDATE_TERMINAL: resume the existing terminal record instead of starting new work"
    except (DeliveryStoreError, KeyError, ValueError) as error:
        return False, str(error)
    return True, ""


def pause_active_task(repo_root: Path) -> tuple[bool, str, Any]:
    """Persist paused state before clearing the session pointer."""
    active = resolve_active_task(repo_root)
    if not active.task_path:
        return True, "", active
    task_json = repo_root / active.task_path / FILE_TASK_JSON
    try:
        data, intent = _intent_task_data(task_json)
        if intent is not None:
            store = _receipt_store(repo_root)
            receipt = ensure_receipt(repo_root, intent, data, phase="implementing")
            changes = {
                "session_state": "paused",
                "phase": receipt.get("phase") or "implementing",
                "next_action": "Resume the same candidate after reviewing its exact current tip and remaining evidence.",
                "recorded_by": intent.owner,
            }
            # A terminal outcome is durable product/disposition evidence. A
            # pause after verification may still precede physical cleanup, but
            # it must never reopen the candidate or erase its terminal claim.
            if not (
                receipt.get("outcome") in _TERMINAL_OUTCOMES
                and receipt.get("closure") in {"verified", "closed"}
            ):
                changes["closure"] = "open"
            updated = store.update(
                intent.candidate_id,
                int(receipt["generation"]),
                changes,
            )
            _ = updated
        previous = clear_active_task(repo_root)
        return True, "", previous
    except (DeliveryValidationError, DeliveryStoreError, ValueError) as error:
        return False, str(error), active


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


def record_archive_closure(
    repo_root: Path,
    intent: DeliveryIntent,
    archive_path: Path,
) -> tuple[bool, str]:
    """Record physical archive completion after the task move succeeds."""
    try:
        store = _receipt_store(repo_root)
        receipt = store.read(intent.candidate_id)
        task_data = read_json(archive_path / FILE_TASK_JSON)
        if not isinstance(task_data, dict):
            raise DeliveryValidationError(
                f"cannot read archived task JSON: {archive_path / FILE_TASK_JSON}"
            )
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
