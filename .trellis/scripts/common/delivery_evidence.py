"""Typed evidence predicates used by the delivery status projection."""

from __future__ import annotations

from typing import Any

from .delivery import (
    DeliveryIntent,
    _DELIVERY_EVIDENCE_SCHEMA,
    _EVIDENCE_SOURCES,
    _EVIDENCE_SOURCES_BY_SECTION,
    _EVIDENCE_SUCCESS_RESULTS,
    _FULL_OID,
    _SHA256,
)


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
    reviewer = value.get("reviewer") or value.get("reviewer_name") or value.get("reviewer_id")
    if review_id is None or not str(review_id).strip() or not reviewer or not str(reviewer).strip():
        return False
    for review in reviews:
        if not isinstance(review, dict):
            continue
        if str(review.get("state") or "").lower() != "approved":
            continue
        provider_review_id = review.get("id")
        if provider_review_id is None or str(provider_review_id) != str(review_id):
            continue
        author = review.get("author")
        if isinstance(author, dict):
            author = author.get("login")
        if not author or str(author).casefold() != str(reviewer).casefold():
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
    """Require a passing provider check and a clean provider merge state.

    GitHub includes conditionally skipped jobs in statusCheckRollup. They are
    not passes, but a skipped optional job is valid when GitHub reports the PR
    clean and the recorded check itself is a successful, bound job.
    """
    if not _github_evidence_matches_pr(value, pull_request):
        return False
    if not isinstance(value, dict) or not isinstance(pull_request, dict):
        return False
    if str(pull_request.get("merge_state_status") or "").lower() != "clean":
        return False
    checks = pull_request.get("checks")
    if not isinstance(checks, list) or not checks:
        return False
    if any(
        not isinstance(check, dict)
        or (
            not _check_success(check)
            and str(check.get("conclusion") or "").lower() != "skipped"
        )
        for check in checks
    ):
        return False
    run_id = value.get("run_id") or value.get("workflow_run_id")
    job_id = value.get("job_id") or value.get("check_id")
    check_name = value.get("check_name")
    check_url = value.get("check_url")
    if run_id is None or job_id is None:
        return False
    for check in checks:
        provider_run_id = check.get("run_id") or check.get("workflow_run_id")
        provider_job_id = check.get("job_id") or check.get("id")
        if provider_run_id is None or provider_job_id is None:
            continue
        same_run = str(provider_run_id) == str(run_id)
        same_job = str(provider_job_id) == str(job_id)
        same_name = check_name is not None and check.get("name") == check_name
        same_context = check_name is not None and check.get("context") == check_name
        same_url = check_url is not None and check.get("url") == check_url
        if (
            same_run
            and same_job
            and _check_success(check)
            and (same_name or same_context or same_url)
        ):
            return True
    return False
