"""Computed delivery status, evidence validation, and checkpoint projections."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .delivery import (
    CandidateStatus,
    DeliveryIntent,
    DeliveryIssue,
    DeliveryValidationError,
    Inventory,
    _ALLOWED_OUTCOMES,
    _DELIVERY_EVIDENCE_SCHEMA,
    _EVIDENCE_SOURCES,
    _EVIDENCE_SOURCES_BY_SECTION,
    _EVIDENCE_SUCCESS_RESULTS,
    _FULL_OID,
    _SHA256,
    _TERMINAL_OUTCOMES,
    _branch_name,
    _git_oid,
    _worktree_matches_candidate,
    delivery_store_root,
    load_delivery_intent,
)
from .git import run_git


def _task_for_intent(intent: DeliveryIntent, inventory: Inventory) -> dict[str, Any] | None:
    if intent.task_json:
        task_name = intent.task_json.resolve().as_posix()
        for task in inventory.tasks:
            if (inventory.repo_root / task["task_path"]).resolve().as_posix() == task_name:
                return task
    for task in inventory.tasks:
        task_intent = task.get("intent")
        if isinstance(task_intent, dict) and task_intent.get("candidate_id") == intent.candidate_id:
            return task
    return None


def _candidate_ref_matches(
    intent: DeliveryIntent, inventory: Inventory
) -> list[dict[str, Any]]:
    """Return refs that can identify this candidate, without guessing."""
    task = _task_for_intent(intent, inventory)
    branch = intent.branch_ref or (task or {}).get("branch")
    branch_short = _branch_name(branch)
    if not branch_short:
        return []
    all_refs: list[dict[str, Any]] = []
    seen: set[tuple[Any, Any]] = set()
    for ref in (*inventory.refs, *inventory.remote_refs):
        key = (ref.get("name"), ref.get("oid"))
        if key not in seen:
            seen.add(key)
            all_refs.append(ref)
    exact = [ref for ref in all_refs if ref.get("name") == branch]
    if exact:
        return exact
    return [
        ref
        for ref in all_refs
        if ref.get("kind") in {"heads", "remotes"}
        and _branch_name(ref.get("name")) == branch_short
    ]


def _ref_for_candidate(intent: DeliveryIntent, inventory: Inventory) -> dict[str, Any] | None:
    matches = _candidate_ref_matches(intent, inventory)
    return matches[0] if len(matches) == 1 else None


def _candidate_pr_matches(
    intent: DeliveryIntent, inventory: Inventory, tip: str | None
) -> list[dict[str, Any]]:
    """Return PRs bound to the exact candidate branch/head when possible."""
    task = _task_for_intent(intent, inventory)
    branch_short = _branch_name(intent.branch_ref or (task or {}).get("branch"))
    branch_matches = [
        pr
        for pr in inventory.pull_requests
        if branch_short and _branch_name(pr.get("head_ref")) == branch_short
    ]
    if tip:
        exact_head = [pr for pr in branch_matches if pr.get("head_oid") == tip]
        if exact_head:
            return exact_head
        if not branch_matches:
            return [pr for pr in inventory.pull_requests if pr.get("head_oid") == tip]
    return branch_matches


def _pr_for_candidate(intent: DeliveryIntent, inventory: Inventory, tip: str | None) -> dict[str, Any] | None:
    matches = _candidate_pr_matches(intent, inventory, tip)
    return matches[0] if len(matches) == 1 else None


def _is_ancestor(repo_root: Path, older: str | None, newer: str | None) -> bool:
    if not older or not newer:
        return False
    code, _, _ = run_git(["merge-base", "--is-ancestor", older, newer], cwd=repo_root)
    return code == 0


def _evidence_section(receipt: dict[str, Any] | None, name: str) -> Any:
    if not receipt:
        return None
    evidence = receipt.get("evidence")
    if isinstance(evidence, dict) and name in evidence:
        return evidence[name]
    return receipt.get(name)


def _aggregate_children_satisfied(intent: DeliveryIntent, inventory: Inventory) -> bool:
    """Compute aggregate completion from current child delivery outcomes."""
    task = _task_for_intent(intent, inventory)
    children = task.get("children") if task else None
    if not isinstance(children, list) or not children:
        return False

    by_name: dict[str, dict[str, Any]] = {}
    for candidate in inventory.tasks:
        task_path = candidate.get("task_path")
        if isinstance(task_path, str):
            by_name[Path(task_path).parent.name] = candidate
        candidate_id = candidate.get("id")
        if isinstance(candidate_id, str) and candidate_id:
            by_name[candidate_id] = candidate

    try:
        from .delivery_store import DeliveryStore, DeliveryStoreError

        store = DeliveryStore(delivery_store_root(inventory.repo_root))
    except (DeliveryValidationError, OSError, ValueError):
        return False

    for child_name in children:
        if not isinstance(child_name, str) or not child_name.strip():
            return False
        child = by_name.get(child_name)
        child_intent_data = child.get("intent") if child else None
        child_task_path = child.get("task_path") if child else None
        if not isinstance(child_intent_data, dict) or not isinstance(child_task_path, str):
            return False
        try:
            child_intent = load_delivery_intent(
                inventory.repo_root / child_task_path / "task.json"
            )
            receipt_path = store.path_for(child_intent.candidate_id)
            if not receipt_path.is_file():
                return False
            child_receipt = store.read(child_intent.candidate_id)
            child_status = classify_candidate(child_intent, inventory, child_receipt)
        except (DeliveryStoreError, DeliveryValidationError, OSError, ValueError):
            return False
        if not child_status.delivered:
            return False
    return True


def _nonempty_string(value: Any) -> bool:
    """Return whether a value is a non-empty string."""
    return isinstance(value, str) and bool(value.strip())


def _full_oid(value: Any) -> bool:
    """Return whether a value is a complete Git object id."""
    return isinstance(value, str) and _FULL_OID.fullmatch(value) is not None


def _full_sha256(value: Any) -> bool:
    """Return whether a value is a complete SHA-256 digest."""
    return isinstance(value, str) and _SHA256.fullmatch(value) is not None


def _evidence_identifier(value: Any) -> bool:
    """Return whether an external identifier is present and bounded."""
    return isinstance(value, (str, int)) and bool(str(value).strip())


def _evidence_error(
    section: str,
    value: Any,
    intent: DeliveryIntent | None,
    tip: str | None,
    base_oid: str | None,
) -> str | None:
    """Return a precise failure for the typed delivery-evidence envelope."""
    if not isinstance(value, dict):
        return "evidence section must be an object"
    if value.get("schema_version") != _DELIVERY_EVIDENCE_SCHEMA:
        return "evidence section has an unsupported schema_version"
    source = value.get("source")
    if source not in _EVIDENCE_SOURCES:
        return "evidence section requires a supported source"
    allowed_sources = _EVIDENCE_SOURCES_BY_SECTION.get(section)
    if allowed_sources is not None and source not in allowed_sources:
        return f"evidence source {source!r} is not valid for {section}"
    if intent is not None and value.get("repository") != intent.repository:
        return "evidence repository does not match the delivery intent"
    if tip is None or not _full_oid(value.get("head_oid")) or value.get("head_oid") != tip:
        return "evidence head_oid does not match the current full candidate tip"
    if not _nonempty_string(value.get("evidence_ref")):
        return "evidence section requires a bounded evidence_ref"
    result = str(
        value.get("result") or value.get("status") or value.get("conclusion") or ""
    ).lower()
    if result not in _EVIDENCE_SUCCESS_RESULTS and value.get("verified") is not True:
        return "evidence section does not contain a successful result"

    if section in {"checks", "postmerge"}:
        if base_oid is None or value.get("base_oid") != base_oid:
            return "evidence base_oid does not match the current full integration base"
        if not _full_oid(value.get("base_oid")):
            return "evidence base_oid must be a full Git object id"

    if section == "review":
        reviewer = value.get("reviewer") or value.get("reviewer_name") or value.get("reviewer_id")
        if not _nonempty_string(reviewer):
            return "review evidence requires reviewer identity"
        if intent is not None and reviewer == intent.owner:
            return "review evidence requires a reviewer distinct from the candidate owner"
        if not _nonempty_string(value.get("reviewer_role") or value.get("role")):
            return "review evidence requires reviewer role"
        if not _nonempty_string(
            value.get("review_id") or value.get("review_url") or value.get("review_ref")
        ):
            return "review evidence requires a review reference"
        findings = value.get("findings", value.get("finding_dispositions"))
        if not isinstance(findings, (list, dict, str)) or (
            isinstance(findings, str) and not findings.strip()
        ):
            return "review evidence requires finding dispositions"
        if source == "github":
            if not _positive_identifier(value.get("pr_number")):
                return "GitHub review evidence requires a positive pr_number"
            if not _nonempty_string(value.get("review_id") or value.get("review_url")):
                return "GitHub review evidence requires a review id or URL"

    if section == "checks":
        if not _evidence_identifier(value.get("run_id") or value.get("workflow_run_id")):
            return "checks evidence requires a workflow run id"
        if not _evidence_identifier(value.get("job_id") or value.get("check_id")):
            return "checks evidence requires a check or job id"
        conclusion = str(value.get("conclusion") or "").lower()
        if conclusion not in {"success", "successful", "passed", "pass"}:
            return "checks evidence requires a successful conclusion"
        if source == "github" and not _positive_identifier(value.get("pr_number")):
            return "GitHub checks evidence requires a positive pr_number"

    if section == "postmerge":
        merge_oid = value.get("merge_oid") or value.get("integrated_oid")
        if not _full_oid(merge_oid):
            return "postmerge evidence requires a full merge_oid"

    if section == "acceptance":
        if intent is None or not _nonempty_string(intent.authority_ref):
            return "acceptance evidence requires an explicit intent authority_ref"
        if value.get("authority_ref") != intent.authority_ref:
            return "acceptance authority_ref does not match the delivery intent"
        if not _nonempty_string(value.get("approved_by")):
            return "acceptance evidence requires approved_by"
        if value.get("approved_by") == intent.owner:
            return "acceptance evidence requires an approver distinct from the candidate owner"
        acceptance_ref = value.get("acceptance_ref") or value.get("criterion_ref")
        if not _nonempty_string(acceptance_ref):
            return "acceptance evidence requires acceptance_ref"
        if intent.acceptance_ref and acceptance_ref != intent.acceptance_ref:
            return "acceptance_ref does not match the delivery intent"
        artifact_digest = value.get("artifact_sha256") or value.get("artifact_hash")
        if not (_full_sha256(artifact_digest) or _nonempty_string(value.get("measurement"))):
            return "acceptance evidence requires a full artifact digest or measurement"

    if section == "durable":
        digest = value.get("sha256") or value.get("artifact_sha256") or value.get("artifact_hash")
        if not _full_sha256(digest):
            return "durable evidence requires a full SHA-256 digest"

    if section == "decision":
        if intent is None or not _nonempty_string(intent.authority_ref):
            return "decision evidence requires an explicit intent authority_ref"
        if value.get("authority_ref") != intent.authority_ref:
            return "decision authority_ref does not match the delivery intent"
        if not _nonempty_string(value.get("decided_by")):
            return "decision evidence requires decided_by"
        if not _nonempty_string(value.get("reason")):
            return "decision evidence requires a reason"
        if not _nonempty_string(value.get("decision_ref")):
            return "decision evidence requires decision_ref"

    if section == "replacement":
        if intent is None or not _nonempty_string(intent.authority_ref):
            return "replacement evidence requires an explicit intent authority_ref"
        if value.get("authority_ref") != intent.authority_ref:
            return "replacement authority_ref does not match the delivery intent"
        if not all(
            _nonempty_string(value.get(field))
            for field in ("reference", "recovery_ref", "remaining_diff")
        ):
            return "replacement evidence requires reference, recovery_ref, and remaining_diff"

    return None


def _positive_identifier(value: Any) -> bool:
    """Return whether an identifier is a positive integer or numeric string."""
    if isinstance(value, bool):
        return False
    try:
        return int(value) > 0
    except (TypeError, ValueError):
        return False


def validate_evidence_mapping(
    evidence: dict[str, Any],
    intent: DeliveryIntent,
    tip: str | None,
    base_oid: str | None,
) -> None:
    """Reject evidence that cannot be used as a typed, current-bound receipt."""
    if not isinstance(evidence, dict) or not evidence:
        raise DeliveryValidationError("evidence must contain at least one section")
    if all(section == "observed_tip" for section in evidence):
        raise DeliveryValidationError("evidence must contain at least one typed section")
    observed_tip = evidence.get("observed_tip")
    if observed_tip is not None and observed_tip != tip:
        raise DeliveryValidationError("evidence observed_tip does not match the current tip")
    for section, value in evidence.items():
        if section == "observed_tip":
            continue
        if section == "release":
            if not _release_receipt_verified(value, intent, tip):
                raise DeliveryValidationError("release evidence does not satisfy the release receipt contract")
            continue
        error = _evidence_error(section, value, intent, tip, base_oid)
        if error:
            raise DeliveryValidationError(f"{section}: {error}")


def _github_evidence_matches_pr(
    value: Any, pull_request: dict[str, Any] | None
) -> bool:
    """Bind provider-sourced evidence to the PR selected by current inventory."""
    if not isinstance(value, dict) or value.get("source") != "github":
        return True
    if pull_request is None or not _positive_identifier(value.get("pr_number")):
        return False
    try:
        same_number = int(value["pr_number"]) == int(pull_request.get("number"))
    except (KeyError, TypeError, ValueError):
        return False
    return same_number and value.get("head_oid") == pull_request.get("head_oid")


def _github_review_matches_pr(
    value: Any, pull_request: dict[str, Any] | None
) -> bool:
    """Require a bound approved review in the current provider payload."""
    if not _github_evidence_matches_pr(value, pull_request):
        return False
    if not isinstance(value, dict) or not isinstance(pull_request, dict):
        return False
    if str(pull_request.get("review_decision") or "").lower() != "approved":
        return False
    reviews = pull_request.get("reviews")
    if not isinstance(reviews, list):
        return False
    review_id = value.get("review_id")
    review_url = value.get("review_url")
    reviewer = value.get("reviewer") or value.get("reviewer_name") or value.get("reviewer_id")
    for review in reviews:
        if not isinstance(review, dict):
            continue
        if str(review.get("state") or "").lower() != "approved":
            continue
        same_id = review_id is not None and str(review.get("id")) == str(review_id)
        same_url = review_url is not None and review.get("url") == review_url
        if not (same_id or same_url):
            continue
        author = review.get("author")
        if reviewer and author and str(author).lower() != str(reviewer).lower():
            continue
        return True
    return False


def _check_success(check: dict[str, Any]) -> bool:
    """Return whether one provider check has a successful terminal result."""
    conclusion = str(check.get("conclusion") or "").lower()
    return conclusion in {"success", "successful", "passed", "pass"}


def _github_checks_match_pr(
    value: Any, pull_request: dict[str, Any] | None
) -> bool:
    """Require the bound check and every reported required check to pass."""
    if not _github_evidence_matches_pr(value, pull_request):
        return False
    if not isinstance(value, dict) or not isinstance(pull_request, dict):
        return False
    checks = pull_request.get("checks")
    if not isinstance(checks, list) or not checks or not all(
        isinstance(check, dict) and _check_success(check) for check in checks
    ):
        return False
    check_id = value.get("check_id") or value.get("job_id")
    check_name = value.get("check_name")
    check_url = value.get("check_url")
    for check in checks:
        same_id = check_id is not None and str(check.get("id")) == str(check_id)
        same_name = check_name is not None and check.get("name") == check_name
        same_context = check_name is not None and check.get("context") == check_name
        same_url = check_url is not None and check.get("url") == check_url
        if same_id or same_name or same_context or same_url:
            return True
    return False


def _evidence_verified(
    value: Any,
    tip: str | None = None,
    base_oid: str | None = None,
    require_base: bool = False,
    section: str | None = None,
    intent: DeliveryIntent | None = None,
) -> bool:
    """Verify a section's result, source binding, and provenance fields.

    A paused candidate may record a subset of complete typed sections. A
    terminal predicate still requires every section it needs to carry exact
    repository, tip, identity, and result fields; a result flag plus an
    abbreviated SHA is never sufficient.
    """
    if section == "release":
        return intent is not None and _release_receipt_verified(value, intent, tip)
    if _evidence_error(section or "evidence", value, intent, tip, base_oid):
        return False
    if require_base or (isinstance(value, dict) and value.get("requires_base")):
        if base_oid is None or value.get("base_oid") != base_oid:
            return False

    if section == "review":
        return True

    if section == "checks":
        return True

    if section == "postmerge":
        return True

    if section == "acceptance":
        return True

    if section == "durable":
        return True

    if section == "decision":
        return True

    if section == "replacement":
        return True

    return True


def _required_facts(
    intent: DeliveryIntent,
    inventory: Inventory,
    receipt: dict[str, Any] | None,
    tip: str | None,
    integration: str,
    dirty: bool,
    pull_request: dict[str, Any] | None,
) -> tuple[dict[str, bool], list[DeliveryIssue]]:
    issues: list[DeliveryIssue] = []
    if intent.kind == "integration":
        review_evidence = _evidence_section(receipt, "review")
        checks_evidence = _evidence_section(receipt, "checks")
        review_verified = _evidence_verified(
            review_evidence,
            tip,
            inventory.base_oid,
            section="review",
            intent=intent,
        ) and (
            review_evidence.get("source") != "github"
            or _github_review_matches_pr(review_evidence, pull_request)
            if isinstance(review_evidence, dict)
            else False
        )
        checks_verified = _evidence_verified(
            checks_evidence,
            tip,
            inventory.base_oid,
            section="checks",
            intent=intent,
        ) and (
            checks_evidence.get("source") != "github"
            or _github_checks_match_pr(checks_evidence, pull_request)
            if isinstance(checks_evidence, dict)
            else False
        )
        if (
            isinstance(review_evidence, dict)
            and review_evidence.get("source") == "github"
            and not _github_review_matches_pr(review_evidence, pull_request)
        ):
            issues.append(
                DeliveryIssue(
                    "GITHUB_REVIEW_UNVERIFIED"
                    if _github_evidence_matches_pr(review_evidence, pull_request)
                    else "GITHUB_REVIEW_UNBOUND",
                    "review evidence is not independently verified by the current GitHub provider payload"
                    if _github_evidence_matches_pr(review_evidence, pull_request)
                    else "review evidence is not bound to the current candidate pull request",
                    intent.candidate_id,
                    next_action="Re-record the review against the exact current PR head and number.",
                )
            )
        if (
            isinstance(checks_evidence, dict)
            and checks_evidence.get("source") == "github"
            and not _github_checks_match_pr(checks_evidence, pull_request)
        ):
            issues.append(
                DeliveryIssue(
                    "GITHUB_CHECKS_UNVERIFIED"
                    if _github_evidence_matches_pr(checks_evidence, pull_request)
                    else "GITHUB_CHECKS_UNBOUND",
                    "checks evidence is not independently verified by the current GitHub provider payload"
                    if _github_evidence_matches_pr(checks_evidence, pull_request)
                    else "checks evidence is not bound to the current candidate pull request",
                    intent.candidate_id,
                    next_action="Re-record checks for the exact current PR head and number.",
                )
            )
        facts = {
            "integration_verified": integration in {"ancestry_integrated", "pr_merged"},
            "review_resolved": review_verified,
            "required_checks_pass": checks_verified,
            "postmerge_verified": _evidence_verified(_evidence_section(receipt, "postmerge"), tip, inventory.base_oid, require_base=True, section="postmerge", intent=intent),
            "acceptance_verified": _evidence_verified(_evidence_section(receipt, "acceptance"), tip, inventory.base_oid, section="acceptance", intent=intent),
        }
    elif intent.kind in {"artifact", "experiment"}:
        facts = {
            "integration_verified": True,
            "review_resolved": True,
            "required_checks_pass": True,
            "postmerge_verified": True,
            "acceptance_verified": _evidence_verified(_evidence_section(receipt, "acceptance"), tip, inventory.base_oid, section="acceptance", intent=intent),
            "durable_evidence_verified": _evidence_verified(_evidence_section(receipt, "durable"), tip, inventory.base_oid, section="durable", intent=intent),
        }
    elif intent.kind == "release":
        release_receipt = _evidence_section(receipt, "release")
        release_verified = _release_receipt_verified(
            release_receipt, intent, tip, inventory
        )
        facts = {
            "integration_verified": True,
            "review_resolved": True,
            "required_checks_pass": True,
            "postmerge_verified": True,
            # The technical release receipt proves package construction,
            # publication, and install behavior. It is not product acceptance.
            "acceptance_verified": _evidence_verified(
                _evidence_section(receipt, "acceptance"),
                tip,
                inventory.base_oid,
                section="acceptance",
                intent=intent,
            ),
            "tag_source_verified": release_verified,
            "version_verified": release_verified,
            "required_assets_verified": release_verified,
            "checksums_verified": release_verified,
            "install_verified": release_verified,
        }
    else:
        facts = {
            "integration_verified": True,
            "review_resolved": True,
            "required_checks_pass": True,
            "postmerge_verified": True,
            "acceptance_verified": _evidence_verified(_evidence_section(receipt, "acceptance"), tip, inventory.base_oid, section="acceptance", intent=intent),
            "all_required_children_satisfied": _aggregate_children_satisfied(intent, inventory),
        }

    if not inventory.base_oid:
        issues.append(DeliveryIssue("BASE_REF_UNRESOLVED", "integration ref is not resolved", intent.candidate_id, next_action="Resolve the base ref before closure."))
    needs_github_coverage = intent.kind == "integration" and (
        receipt is None
        or receipt.get("outcome") in {None, "delivered"}
    )
    if needs_github_coverage and inventory.coverage.get("github") != "complete":
        issues.append(DeliveryIssue("GITHUB_COVERAGE_UNAVAILABLE", "GitHub review/check coverage is incomplete", intent.candidate_id, next_action="Restore GitHub evidence access before claiming delivery."))
    if dirty:
        issues.append(DeliveryIssue("OWNED_WORKTREE_DIRTY", "candidate worktree has uncommitted changes", intent.candidate_id, next_action="Preserve and resolve the owned changes before closure."))
    return facts, issues


def _release_receipt_verified(
    value: Any,
    intent: DeliveryIntent,
    tip: str | None,
    inventory: Inventory | None = None,
) -> bool:
    """Validate the shared durable release receipt against the source tip."""
    if not isinstance(value, dict) or tip is None:
        return False
    if value.get("schema_version") != "assura.release-receipt.v1":
        return False
    if value.get("repository") != intent.repository or value.get("version") != intent.version:
        return False
    if value.get("tag") != f"v{intent.version}":
        return False
    commit_oid = value.get("commit_oid")
    tag_oid = value.get("tag_oid")
    if not isinstance(commit_oid, str) or commit_oid != tip or not _FULL_OID.fullmatch(commit_oid):
        return False
    if not isinstance(tag_oid, str) or not _FULL_OID.fullmatch(tag_oid):
        return False
    if inventory is not None:
        tag_name = f"refs/tags/v{intent.version}"
        tag_ref = next(
            (ref for ref in inventory.refs if ref.get("name") == tag_name),
            None,
        )
        if tag_ref is None or tag_ref.get("oid") != tag_oid:
            return False
        tagged_commit = _git_oid(inventory.repo_root, f"{tag_name}^{{commit}}")
        if tagged_commit != commit_oid:
            return False
    workflow_runs = value.get("workflow_runs")
    if not isinstance(workflow_runs, list) or not workflow_runs:
        return False
    if any(
        not isinstance(run, dict)
        or not _nonempty_string(run.get("workflow"))
        or not isinstance(run.get("run_id"), (str, int))
        or not str(run.get("run_id")).strip()
        for run in workflow_runs
    ):
        return False
    assets = value.get("assets")
    required_assets = set(intent.required_assets)
    if not required_assets or not isinstance(assets, list):
        return False
    seen_assets: set[str] = set()
    for asset in assets:
        if not isinstance(asset, dict):
            return False
        name = asset.get("name")
        digest = asset.get("sha256")
        size = asset.get("size")
        if (
            not isinstance(name, str)
            or name in seen_assets
            or name not in required_assets
            or not isinstance(digest, str)
            or not _SHA256.fullmatch(digest)
            or not isinstance(size, int)
            or size < 0
            or asset.get("checksum_name") != f"{name}.sha256"
            or not isinstance(asset.get("members"), list)
            or not all(isinstance(member, str) and member for member in asset["members"])
            or not set(
                ("assura.exe", "assura-full.exe")
                if name.endswith(".zip")
                else ("assura", "assura-full")
            ).issubset(asset["members"])
        ):
            return False
        seen_assets.add(name)
    if seen_assets != required_assets or value.get("checksums_verified") is not True:
        return False
    install = value.get("install")
    if not isinstance(install, dict) or install.get("verified") is not True:
        return False
    expected_output = f"assura {intent.version}"
    if install.get("assura") != expected_output or install.get("assura-full") != expected_output:
        return False
    publish = value.get("publish")
    return (
        isinstance(publish, dict)
        and publish.get("repository") == intent.repository
        and publish.get("tag") == f"v{intent.version}"
        and publish.get("action") in {"created", "uploaded", "verified"}
        and isinstance(publish.get("verified_assets"), list)
        and required_assets.issubset(set(publish["verified_assets"]))
    )


def validate_release_receipt(
    value: Any, intent: DeliveryIntent, tip: str | None
) -> bool:
    """Public predicate for the release receipt contract."""
    return _release_receipt_verified(value, intent, tip)


def _may_deliver(intent: DeliveryIntent, facts: dict[str, bool], dirty: bool, coverage_unknown: bool) -> bool:
    if coverage_unknown or dirty:
        return False
    if intent.kind == "integration":
        return all(facts.get(key, False) for key in ("integration_verified", "review_resolved", "required_checks_pass", "postmerge_verified", "acceptance_verified"))
    if intent.kind in {"artifact", "experiment"}:
        return facts.get("acceptance_verified", False) and facts.get("durable_evidence_verified", False)
    if intent.kind == "release":
        return all(facts.get(key, False) for key in ("tag_source_verified", "version_verified", "required_assets_verified", "checksums_verified", "install_verified", "acceptance_verified"))
    return facts.get("all_required_children_satisfied", False)


def classify_candidate(
    intent: DeliveryIntent,
    inventory: Inventory,
    receipt: dict[str, Any] | None = None,
) -> CandidateStatus:
    """Compute a truthful candidate status from current facts and evidence."""
    task = _task_for_intent(intent, inventory)
    ref_matches = _candidate_ref_matches(intent, inventory)
    ref = ref_matches[0] if len(ref_matches) == 1 else None
    tip = str(ref.get("oid")) if ref and ref.get("oid") else None
    pr_matches = _candidate_pr_matches(intent, inventory, tip)
    pr = pr_matches[0] if len(pr_matches) == 1 else None
    if not tip and pr and pr.get("head_oid"):
        tip = str(pr["head_oid"])
    if not tip and isinstance(receipt, dict) and _full_oid(receipt.get("observed_tip")):
        # A verified receipt remains a reconstruction source after an
        # authorized branch/worktree cleanup. It never overrides a current
        # ref; a current ref moving later will be reported as a tip change.
        tip = str(receipt["observed_tip"])
    pr_matches = _candidate_pr_matches(intent, inventory, tip)
    pr = pr_matches[0] if len(pr_matches) == 1 else None

    branch = intent.branch_ref or (task or {}).get("branch")
    dirty = any(
        _worktree_matches_candidate(worktree, intent, task, tip)
        and worktree.get("dirty") is True
        for worktree in inventory.worktrees
    )

    recorded_outcome = receipt.get("outcome") if isinstance(receipt, dict) else None
    if recorded_outcome not in _ALLOWED_OUTCOMES:
        recorded_outcome = None

    if intent.kind == "aggregate":
        integration = "aggregate"
    elif not tip or inventory.base_oid is None:
        integration = "unknown"
    elif pr and pr.get("merged"):
        merge_oid = pr.get("merge_oid") or pr.get("head_oid")
        has_post_review_tip = bool(
            pr.get("head_oid")
            and tip
            and tip != pr.get("head_oid")
            and not _is_ancestor(inventory.repo_root, tip, inventory.base_oid)
        )
        integration = (
            "post_review_commits"
            if has_post_review_tip
            else (
                "pr_merged"
                if _is_ancestor(inventory.repo_root, merge_oid, inventory.base_oid)
                else "post_review_commits"
            )
        )
    elif tip == inventory.base_oid:
        registered_tip = receipt.get("registered_tip") if isinstance(receipt, dict) else None
        registered_base = receipt.get("registered_base_oid") if isinstance(receipt, dict) else None
        integration = (
            "ancestry_integrated"
            if registered_tip and registered_base and registered_tip == tip and registered_tip != registered_base
            else "no_candidate_commit"
        )
    elif _is_ancestor(inventory.repo_root, tip, inventory.base_oid):
        integration = "ancestry_integrated"
    elif pr and str(pr.get("state")) == "closed":
        integration = "pr_closed_unmerged"
    elif pr:
        integration = "pr_open"
    else:
        integration = "unmerged"

    evidence = dict(receipt.get("evidence") or {}) if isinstance(receipt, dict) else {}
    facts, fact_issues = _required_facts(intent, inventory, receipt, tip, integration, dirty, pr)
    # A local ancestry relation is not enough to prove a complete delivery
    # record. GitHub API failures are coverage failures even when the branch
    # tip is already reachable from the base; otherwise a direct/local merge
    # could bypass the required review and remote-evidence boundary.
    remote_coverage_unknown = (
        intent.kind == "integration"
        and (receipt is None or receipt.get("outcome") in {None, "delivered"})
        and inventory.coverage.get("github") != "complete"
    )
    coverage_unknown = not bool(inventory.coverage.get("base_resolved")) or inventory.coverage.get("git") != "complete" or inventory.coverage.get("tasks") != "complete" or remote_coverage_unknown
    issues: list[DeliveryIssue] = list(fact_issues)
    if intent.kind != "aggregate" and not tip:
        issues.append(DeliveryIssue("CANDIDATE_TIP_UNRESOLVED", "candidate branch or PR head is not present in the inventory", intent.candidate_id, next_action="Resolve the branch/PR identity without deleting historical refs."))
    if len(ref_matches) > 1:
        issues.append(DeliveryIssue("CANDIDATE_REF_AMBIGUOUS", "multiple local refs could identify the candidate branch", intent.candidate_id, next_action="Bind the task to one exact branch ref before recording delivery evidence."))
    if len(pr_matches) > 1:
        issues.append(DeliveryIssue("PR_IDENTITY_AMBIGUOUS", "multiple pull requests could identify the candidate head", intent.candidate_id, next_action="Bind the candidate to one exact pull request and head before closure."))
    if receipt is None:
        issues.append(DeliveryIssue("RECEIPT_MISSING", "candidate has no resumable delivery receipt", intent.candidate_id, next_action="Register the candidate or recreate its receipt from preserved task and Git evidence."))
    for inventory_issue in inventory.issues:
        if inventory_issue.code != "WORKTREE_COVERAGE_UNAVAILABLE":
            continue
        if any(
            _worktree_matches_candidate(worktree, intent, task, tip)
            and worktree.get("state") != "available"
            for worktree in inventory.worktrees
        ):
            issues.append(
                DeliveryIssue(
                    inventory_issue.code,
                    inventory_issue.message,
                    intent.candidate_id,
                    inventory_issue.evidence_ref,
                    inventory_issue.next_action,
                )
            )
    if intent.kind != "aggregate" and integration in {"unmerged", "no_candidate_commit", "pr_open", "post_review_commits", "pr_closed_unmerged", "unknown"} and recorded_outcome not in {"superseded", "rejected", "cancelled"}:
        issues.append(DeliveryIssue("INTEGRATION_UNVERIFIED", f"current integration state is {integration}", intent.candidate_id, next_action="Review the exact candidate diff and integrate, supersede, reject, or hold it explicitly."))
    if isinstance(receipt, dict) and receipt.get("observed_tip") and tip and receipt.get("observed_tip") != tip:
        issues.append(DeliveryIssue("RECEIPT_TIP_CHANGED", "receipt evidence belongs to a different candidate tip", intent.candidate_id, next_action="Re-record evidence for the current exact tip."))

    if isinstance(receipt, dict) and receipt.get("outcome") not in _ALLOWED_OUTCOMES:
        issues.append(DeliveryIssue("RECEIPT_OUTCOME_INVALID", "receipt contains an unsupported outcome", intent.candidate_id, next_action="Preserve the receipt and record a supported outcome."))

    if recorded_outcome == "delivered":
        if not _may_deliver(intent, facts, dirty, coverage_unknown):
            issues.append(DeliveryIssue("UNSUPPORTED_DELIVERY_CLAIM", "receipt claims delivered without all current terminal evidence", intent.candidate_id, next_action="Revalidate required review, checks, integration, acceptance, and coverage evidence."))
            outcome = "unknown"
        else:
            outcome = "delivered"
    elif recorded_outcome in {"superseded", "rejected", "cancelled"}:
        decision = _evidence_section(receipt, "decision")
        if not isinstance(decision, dict) or not _evidence_verified(
            decision, tip, inventory.base_oid, section="decision", intent=intent
        ):
            issues.append(DeliveryIssue("OUTCOME_DECISION_UNVERIFIED", f"{recorded_outcome} lacks a verified decision bound to the current tip", intent.candidate_id, next_action="Record the decision, exact tip, and durable recovery evidence."))
            outcome = "unknown"
        elif recorded_outcome == "superseded":
            replacement = _evidence_section(receipt, "replacement")
            if not isinstance(replacement, dict) or not all(
                isinstance(replacement.get(field), str) and replacement[field].strip()
                for field in ("reference", "recovery_ref", "remaining_diff")
            ) or not _evidence_verified(
                replacement, tip, inventory.base_oid, section="replacement", intent=intent
            ):
                issues.append(DeliveryIssue("REPLACEMENT_UNRESOLVED", "superseded outcome lacks a verified replacement, recovery reference, or remaining-diff disposition", intent.candidate_id, next_action="Bind the candidate to the replacement task or PR, record the exact tip, and account for remaining diff."))
                outcome = "unknown"
            else:
                outcome = recorded_outcome
        else:
            outcome = recorded_outcome
    else:
        outcome = None

    closure = str(receipt.get("closure") or "open") if isinstance(receipt, dict) else "open"
    if outcome is None or outcome == "unknown":
        phase = "registered" if not receipt else str(receipt.get("phase") or "held")
        if integration in {"pr_open", "unmerged", "no_candidate_commit"}:
            phase = "review" if pr else "implementing"
        elif integration in {"ancestry_integrated", "pr_merged", "post_review_commits"}:
            phase = "integrated_pending_verification"
        if issues:
            phase = "held" if any(issue.code.endswith("UNAVAILABLE") or issue.code in {"INTEGRATION_UNVERIFIED", "UNSUPPORTED_DELIVERY_CLAIM", "OWNED_WORKTREE_DIRTY", "RECEIPT_MISSING", "CANDIDATE_TIP_UNRESOLVED"} for issue in issues) else phase
    else:
        phase = "verified"

    if outcome in _TERMINAL_OUTCOMES and closure not in {"verified", "closed"}:
        next_action = "Perform only authorized cleanup, then record closure."
    elif outcome in _TERMINAL_OUTCOMES:
        next_action = "No delivery action remains; retain the receipt and historical refs."
    elif dirty:
        next_action = "Preserve owned dirty changes and resolve their disposition before closure."
    elif integration in {"ancestry_integrated", "pr_merged"}:
        next_action = "Record review, checks, post-merge, and acceptance evidence for the exact integrated tip."
    elif pr and pr.get("state") == "open":
        next_action = "Complete review and required checks for the existing pull request."
    else:
        next_action = "Inspect the exact tip and choose implementation, review, integration, hold, or explicit disposition."

    return CandidateStatus(
        candidate_id=intent.candidate_id,
        owner=intent.owner,
        kind=intent.kind,
        task_path=(task or {}).get("task_path"),
        tip=tip,
        phase=phase,
        outcome=outcome,
        integration=integration,
        dirty=dirty,
        closure=closure,
        next_action=next_action,
        evidence=evidence,
        issues=tuple(issues),
    )


def validate_transition(intent: DeliveryIntent, status: CandidateStatus, action: str) -> list[DeliveryIssue]:
    """Validate an explicit lifecycle transition without mutating state."""
    if action == "archive":
        if status.outcome not in _TERMINAL_OUTCOMES:
            return [DeliveryIssue("ARCHIVE_REQUIRES_TERMINAL_OUTCOME", "physical archive requires a verified terminal delivery outcome", intent.candidate_id, next_action="Record and revalidate delivered, superseded, rejected, or cancelled evidence first.")]
        if status.closure not in {"verified", "closed"}:
            return [DeliveryIssue("ARCHIVE_REQUIRES_VERIFIED_CLOSURE", "physical archive requires a terminal outcome whose receipt is verified", intent.candidate_id, next_action="Record the terminal outcome before moving the task directory.")]
        if status.dirty:
            return [DeliveryIssue("ARCHIVE_WORKTREE_DIRTY", "physical archive cannot consume owned uncommitted changes", intent.candidate_id, next_action="Preserve and resolve the dirty worktree before archiving.")]
        return []
    if action == "close" and status.outcome not in _TERMINAL_OUTCOMES:
        specific = [
            issue
            for issue in status.issues
            if issue.code not in {"INTEGRATION_UNVERIFIED", "CANDIDATE_TIP_UNRESOLVED"}
        ]
        if specific:
            return specific
        return [DeliveryIssue("CLOSE_REQUIRES_VERIFIED_EVIDENCE", "candidate cannot close while its outcome is unknown", intent.candidate_id, next_action="Record the missing evidence or choose an explicitly supported disposition.")]
    return []


def render_checkpoint(statuses: list[CandidateStatus] | tuple[CandidateStatus, ...], max_bytes: int = 1024) -> str:
    """Render a deterministic three-to-six-line checkpoint."""
    ordered = sorted(statuses, key=lambda status: status.candidate_id)
    delivered = sum(status.outcome == "delivered" for status in ordered)
    terminal = sum(status.outcome in _TERMINAL_OUTCOMES for status in ordered)
    unresolved = len(ordered) - terminal
    held = sum(status.phase == "held" for status in ordered)
    lines = [
        f"- candidates: {len(ordered)}; terminal: {terminal}; delivered: {delivered}; unresolved: {unresolved}",
        f"- held: {held}; dirty-owned: {sum(status.dirty for status in ordered)}",
    ]
    for status in ordered[:3]:
        lines.append(f"- {status.candidate_id}: {status.outcome or status.phase}; next: {status.next_action}")
    if len(ordered) > 3:
        lines.append(f"- additional candidates: {len(ordered) - 3}; inspect the JSON audit for exact dispositions")
    result = "\n".join(lines)
    encoded = result.encode("utf-8")
    if len(encoded) <= max_bytes:
        return result
    suffix = "\n- checkpoint truncated; inspect JSON audit for exact dispositions"
    budget = max(0, max_bytes - len(suffix.encode("utf-8")))
    truncated = encoded[:budget].decode("utf-8", errors="ignore").rstrip()
    return truncated + suffix
