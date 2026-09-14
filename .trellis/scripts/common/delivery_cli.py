"""Command handlers for the explicit Trellis delivery workflow."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any

from .active_task import clear_active_task, resolve_active_task
from .delivery import (
    CandidateStatus,
    DeliveryIntent,
    DeliveryIssue,
    DeliveryValidationError,
    _branch_name,
    _choose_base_ref,
    _git_oid,
    classify_candidate,
    collect_inventory,
    delivery_store_root,
    load_delivery_intent,
    render_checkpoint,
    repository_identity,
    validate_transition,
)
from .delivery_store import DeliveryStore, DeliveryStoreError, ReceiptCorrupt, StaleGeneration
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
}
_MAX_EVIDENCE_BYTES = 64 * 1024


def _error(message: str) -> int:
    print(f"Error: {message}", file=sys.stderr)
    return 1


def _task_dir(args: argparse.Namespace, repo_root: Path) -> Path:
    return resolve_task_dir(args.task, repo_root)


def _task_json(args: argparse.Namespace, repo_root: Path) -> Path:
    return _task_dir(args, repo_root) / FILE_TASK_JSON


def _current_branch(repo_root: Path) -> str | None:
    code, stdout, _ = run_git(["branch", "--show-current"], cwd=repo_root)
    return stdout.strip() if code == 0 and stdout.strip() else None


def _normalise_ref(ref: str | None) -> str | None:
    if not ref:
        return None
    value = ref.strip()
    if value.startswith("refs/"):
        return value
    if value.startswith("origin/"):
        return f"refs/remotes/{value}"
    return f"refs/heads/{value}"


def _intent_task_data(task_json: Path) -> tuple[dict[str, Any], DeliveryIntent | None]:
    data = read_json(task_json)
    if not isinstance(data, dict):
        raise DeliveryValidationError(f"cannot read task JSON: {task_json}")
    try:
        return data, load_delivery_intent(task_json)
    except DeliveryValidationError:
        return data, None


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


def _initial_receipt(
    repo_root: Path,
    intent: DeliveryIntent,
    data: dict[str, Any],
    phase: str = "registered",
) -> dict[str, Any]:
    return {
        "repository": intent.repository,
        "task_path": str(intent.task_json.relative_to(repo_root).as_posix()) if intent.task_json else None,
        "owner": intent.owner,
        "phase": phase,
        "session_state": "attached",
        "outcome": None,
        "closure": "open",
        "next_action": "Implement the candidate and record exact review/check/acceptance evidence.",
        "observed_tip": _tip_for_intent(repo_root, intent, data),
        "registered_tip": _tip_for_intent(repo_root, intent, data),
        "registered_base_oid": _git_oid(repo_root, intent.base_ref),
        "evidence_kind": "lifecycle",
        "evidence": {},
        "recorded_by": intent.owner,
        "authority_ref": intent.authority_ref,
    }


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
        return existing
    return store.create(intent.candidate_id, _initial_receipt(repo_root, intent, data, phase))


def register_task(
    repo_root: Path,
    task_json: Path,
    candidate_id: str,
    owner: str,
    kind: str = "integration",
    base_ref: str | None = None,
    acceptance_ref: str | None = "prd.md#acceptance",
    authority_ref: str | None = None,
) -> DeliveryIntent:
    """Persist ownership intent and create its resumable receipt."""
    data = read_json(task_json)
    if not isinstance(data, dict):
        raise DeliveryValidationError(f"cannot read task JSON: {task_json}")
    existing = data.get("meta", {}).get("delivery") if isinstance(data.get("meta"), dict) else None
    if isinstance(existing, dict):
        if existing.get("candidate_id") != candidate_id or existing.get("owner") != owner:
            raise DeliveryValidationError("task already has a different delivery owner or candidate")
        intent = load_delivery_intent(task_json)
        ensure_receipt(repo_root, intent, data)
        return intent

    chosen_base = _normalise_ref(base_ref) or _normalise_ref(data.get("base_branch")) or _normalise_ref(_choose_base_ref(repo_root))
    if not chosen_base:
        raise DeliveryValidationError("cannot register delivery without a resolvable base ref")
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
    meta = data.get("meta") if isinstance(data.get("meta"), dict) else {}
    meta["delivery"] = delivery
    data["meta"] = meta
    if not write_json(task_json, data):
        raise DeliveryValidationError(f"cannot write delivery intent: {task_json}")
    intent = load_delivery_intent(task_json)
    ensure_receipt(repo_root, intent, data)
    return intent


def _status_for_task(
    repo_root: Path,
    task_json: Path,
    github: Any = None,
    receipt_override: dict[str, Any] | None = None,
) -> tuple[DeliveryIntent, CandidateStatus, Any, dict[str, Any]]:
    data = read_json(task_json)
    if not isinstance(data, dict):
        raise DeliveryValidationError(f"cannot read task JSON: {task_json}")
    intent = load_delivery_intent(task_json)
    inventory = collect_inventory(repo_root, github=github, base_ref=intent.base_ref)
    store = _receipt_store(repo_root)
    receipt = receipt_override if receipt_override is not None else _read_receipt(store, intent.candidate_id)
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
        )
    except (DeliveryValidationError, DeliveryStoreError, ValueError) as error:
        return _error(str(error))
    print(f"registered: {intent.candidate_id} owner={intent.owner}")
    return 0


def cmd_delivery_inspect(args: argparse.Namespace) -> int:
    """Inspect one current candidate without writing state."""
    repo_root = get_repo_root()
    task_json = _task_json(args, repo_root)
    try:
        _, status, _, _ = _status_for_task(repo_root, task_json)
    except DeliveryValidationError as error:
        return _error(str(error))
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
    unknown = sorted(set(data) - _EVIDENCE_SECTIONS)
    if unknown:
        raise DeliveryValidationError(f"unsupported evidence sections: {', '.join(unknown)}")
    if not data:
        raise DeliveryValidationError("evidence file must contain at least one evidence section")
    return data


def cmd_delivery_record(args: argparse.Namespace) -> int:
    """Record bounded evidence using compare-and-swap generation."""
    repo_root = get_repo_root()
    task_json = _task_json(args, repo_root)
    try:
        evidence = _load_evidence(Path(args.evidence_file))
        intent, status, inventory, data = _status_for_task(repo_root, task_json)
        store = _receipt_store(repo_root)
        receipt = _read_receipt(store, intent.candidate_id)
        if receipt is None:
            return _error("RECEIPT_MISSING: run delivery register or start the task first")
        tip = status.tip
        evidence_tip = evidence.get("observed_tip")
        if evidence_tip and tip and evidence_tip != tip:
            return _error("evidence observed_tip does not match the current candidate tip")
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
        return _error(str(error))
    print(json.dumps(updated, indent=2, sort_keys=True))
    return 0


def _decision_evidence(args: argparse.Namespace, outcome: str, tip: str | None) -> dict[str, Any]:
    if outcome == "delivered":
        return {}
    if getattr(args, "decision_file", None):
        decision = _load_evidence(Path(args.decision_file))
        if "decision" not in decision:
            raise DeliveryValidationError("decision file must contain a decision section")
        result = dict(decision)
    else:
        reason = getattr(args, "reason", None)
        if not reason:
            raise DeliveryValidationError("non-delivered closure requires --reason or --decision-file")
        result = {"decision": {"result": "verified", "reason": reason, "head_oid": tip}}
    if outcome == "superseded":
        replacement = getattr(args, "replacement", None)
        if not replacement and "replacement" not in result:
            raise DeliveryValidationError("superseded closure requires --replacement")
        if replacement:
            result["replacement"] = {"reference": replacement}
    return result


def cmd_delivery_close(args: argparse.Namespace) -> int:
    """Validate and record an explicit terminal outcome."""
    repo_root = get_repo_root()
    task_json = _task_json(args, repo_root)
    try:
        intent, current, inventory, data = _status_for_task(repo_root, task_json)
        store = _receipt_store(repo_root)
        receipt = _read_receipt(store, intent.candidate_id)
        if receipt is None:
            return _error("RECEIPT_MISSING: run delivery register or start the task first")
        changes = _decision_evidence(args, args.outcome, current.tip)
        prospective_evidence = dict(receipt.get("evidence") or {})
        prospective_evidence.update(changes)
        prospective = dict(receipt)
        prospective["evidence"] = prospective_evidence
        prospective["outcome"] = args.outcome
        prospective["closure"] = "verified"
        prospective["observed_tip"] = current.tip
        status = classify_candidate(intent, inventory, prospective)
        transition_issues = validate_transition(intent, status, "close")
        if transition_issues:
            for issue in transition_issues:
                print(f"{issue.code}: {issue.message}", file=sys.stderr)
            return 1
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
        return _error(str(error))
    print(json.dumps(updated, indent=2, sort_keys=True))
    return 0


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


def project_audit(repo_root: Path, owner: str | None = None, github: Any = None) -> tuple[dict[str, Any], list[CandidateStatus]]:
    """Build the current audit projection without writing anything."""
    inventory = collect_inventory(repo_root, github=github)
    store = _receipt_store(repo_root)
    statuses: list[CandidateStatus] = []
    legacy: list[dict[str, Any]] = []
    bound_branches: set[str] = set()
    for task in inventory.tasks:
        intent_data = task.get("intent")
        if not isinstance(intent_data, dict):
            legacy_status = _legacy_status(task)
            legacy.append(legacy_status.as_dict())
            if owner is None or legacy_status.owner == owner:
                statuses.append(legacy_status)
            continue
        task_json = repo_root / task["task_path"] / FILE_TASK_JSON
        try:
            intent = load_delivery_intent(task_json)
            receipt = _read_receipt(store, intent.candidate_id)
            status = classify_candidate(intent, inventory, receipt)
            if owner is None or status.owner == owner:
                statuses.append(status)
            if intent.branch_ref:
                bound_branches.add(_branch_name(intent.branch_ref) or intent.branch_ref)
        except (DeliveryValidationError, DeliveryStoreError) as error:
            issue = DeliveryIssue("CANDIDATE_UNRESOLVED", str(error), task.get("task_path"), next_action="Preserve the task and resolve its schema or receipt before closure.")
            statuses.append(CandidateStatus(str(task.get("id") or task["task_path"]), str(task.get("assignee") or "unknown"), "unknown", task["task_path"], None, "held", None, "unknown", False, "open", issue.next_action, issues=(issue,)))

    ignored_ref_names = {"master", "main", "HEAD"}
    unowned_refs = []
    for ref in inventory.refs:
        if ref.get("kind") != "heads":
            continue
        branch = _branch_name(ref.get("name"))
        if not branch or branch in ignored_ref_names or branch in bound_branches:
            continue
        if branch.startswith(("codex/", "goal/", "feature/", "fix/", "archive/", "maturity/")):
            unowned_refs.append({"name": ref["name"], "oid": ref["oid"], "disposition": "unresolved", "next_action": "Bind to an owner/task or record a preserved historical disposition."})

    worktree_branches = {_branch_name(item.get("branch_ref")) for item in inventory.worktrees}
    unowned_worktrees = [
        item for item in inventory.worktrees
        if item.get("branch_ref") and _branch_name(item.get("branch_ref")) not in bound_branches
    ]
    report = {
        "schema_version": 1,
        "repository": inventory.repository,
        "base_ref": inventory.base_ref,
        "base_oid": inventory.base_oid,
        "coverage": inventory.coverage,
        "inventory": inventory.as_dict(),
        "candidates": [status.as_dict() for status in sorted(statuses, key=lambda item: item.candidate_id)],
        "legacy": legacy,
        "unowned_refs": sorted(unowned_refs, key=lambda item: item["name"]),
        "unowned_worktrees": sorted(unowned_worktrees, key=lambda item: str(item.get("path", ""))),
        "issues": [issue.as_dict() for issue in inventory.issues],
    }
    return report, sorted(statuses, key=lambda item: item.candidate_id)


def cmd_delivery_audit(args: argparse.Namespace) -> int:
    """Audit all local tasks and topology as a read-only operation."""
    repo_root = get_repo_root()
    report, statuses = project_audit(repo_root, owner=args.owner)
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
    if args.strict and any(status.outcome is None or status.dirty for status in statuses):
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
    try:
        ensure_receipt(repo_root, intent, data, phase="implementing")
    except (DeliveryStoreError, ValueError) as error:
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
            updated = store.update(
                intent.candidate_id,
                int(receipt["generation"]),
                {
                    "session_state": "paused",
                    "closure": "open",
                    "phase": receipt.get("phase") or "implementing",
                    "next_action": "Resume the same candidate after reviewing its exact current tip and remaining evidence.",
                    "recorded_by": intent.owner,
                },
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
        inventory = collect_inventory(repo_root, github=github, base_ref=intent.base_ref)
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
    except (DeliveryStoreError, ValueError) as error:
        return False, str(error)
