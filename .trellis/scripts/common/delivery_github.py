"""GitHub pull request readers and candidate query validation."""

from __future__ import annotations

import json
import re
import shutil
import subprocess
from datetime import datetime
from typing import Any, Protocol


class GithubUnavailable(RuntimeError):
    """GitHub evidence could not be collected completely."""


class GithubIdentityMismatch(ValueError):
    """GitHub data does not match the candidate identity snapshot."""


_UTC_TIMESTAMP = re.compile(
    r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?Z"
)
_FULL_OID = re.compile(r"(?:[0-9a-fA-F]{40}|[0-9a-fA-F]{64})")


def _valid_merge_snapshot(row: dict[str, Any]) -> bool:
    """Require provider merge fields to agree with the PR's terminal state."""
    state = row.get("state")
    merged_at = row.get("mergedAt")
    merge_commit = row.get("mergeCommit")
    if state == "MERGED":
        if not isinstance(merged_at, str) or _UTC_TIMESTAMP.fullmatch(merged_at) is None:
            return False
        try:
            datetime.fromisoformat(merged_at[:-1] + "+00:00")
        except ValueError:
            return False
        return (
            isinstance(merge_commit, dict)
            and isinstance(merge_commit.get("oid"), str)
            and _FULL_OID.fullmatch(merge_commit["oid"]) is not None
        )
    return state in {"OPEN", "CLOSED"} and merged_at is None and merge_commit is None


class GithubReader(Protocol):
    """Minimal injectable GitHub reader used by audits and tests."""

    def list_pull_requests(self, repository: str) -> list[dict[str, Any]]:
        """Return all available pull requests for a repository."""

    def candidate_pull_request(
        self, repository: str, head_ref: str, base_ref: str, head_oid: str
    ) -> tuple[list[dict[str, Any]], dict[str, Any]]:
        """Return a complete identity cohort and the exact current PR details."""


class SubprocessGithubReader:
    """Read pull-request facts through the installed GitHub CLI."""

    def __init__(self, executable: str = "gh") -> None:
        self.executable = executable

    def list_pull_requests(self, repository: str) -> list[dict[str, Any]]:
        """Query all pull requests without invoking a shell."""
        executable = shutil.which(self.executable) or self.executable
        field_sets = (
            "number,state,title,url,headRefName,headRefOid,baseRefName,baseRefOid,mergeCommit,mergedAt,reviewDecision,mergeStateStatus",
            "number,reviews",
            "number,statusCheckRollup",
        )
        rows: dict[str, dict[str, Any]] = {}
        for fields in field_sets:
            command = [
                executable,
                "pr",
                "list",
                "--repo",
                repository,
                "--state",
                "all",
                "--limit",
                "1000",
                "--json",
                fields,
            ]
            try:
                result = subprocess.run(
                    command,
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                    errors="replace",
                    check=False,
                )
            except OSError as error:
                raise GithubUnavailable(f"GitHub CLI unavailable: {error}") from error
            if result.returncode != 0:
                detail = result.stderr.strip() or "unknown GitHub CLI failure"
                raise GithubUnavailable(detail)
            try:
                value = json.loads(result.stdout or "[]")
            except json.JSONDecodeError as error:
                raise GithubUnavailable("GitHub CLI returned invalid JSON") from error
            if not isinstance(value, list):
                raise GithubUnavailable("GitHub CLI returned a non-list response")
            if len(value) >= 1000:
                raise GithubUnavailable(
                    "GitHub PR pagination is incomplete at the 1000-result limit"
                )
            for item in value:
                if not isinstance(item, dict) or item.get("number") is None:
                    continue
                key = str(item["number"])
                rows.setdefault(key, {}).update(item)
        return list(rows.values())

    def candidate_pull_request(
        self, repository: str, head_ref: str, base_ref: str, head_oid: str
    ) -> tuple[list[dict[str, Any]], dict[str, Any]]:
        """Resolve one candidate from a complete identity cohort, then read its details."""
        executable = shutil.which(self.executable) or self.executable
        identity_fields = (
            "number,state,headRefName,headRefOid,baseRefName,baseRefOid,"
            "mergeCommit,mergedAt,reviewDecision,mergeStateStatus"
        )
        command = [
            executable, "pr", "list", "--repo", repository, "--state", "all",
            "--limit", "1000", "--json", identity_fields,
        ]
        cohort = self._run_candidate_query(command, "identity cohort", list_response=True)
        if len(cohort) >= 1000:
            raise GithubUnavailable("GitHub identity cohort unavailable: pagination reached the 1000-result limit")
        seen: set[int] = set()
        for row in cohort:
            number = row.get("number")
            if not isinstance(number, int) or isinstance(number, bool) or number <= 0:
                raise GithubIdentityMismatch("PR identity mismatch: identity cohort contains an invalid number")
            if number in seen:
                raise GithubIdentityMismatch("PR identity mismatch: identity cohort contains duplicate numbers")
            seen.add(number)
            for field in ("headRefName", "headRefOid", "baseRefName", "baseRefOid", "state"):
                if not isinstance(row.get(field), str) or not row[field]:
                    raise GithubIdentityMismatch(f"PR identity mismatch: identity cohort has incomplete {field}")
            if row["state"].lower() not in {"open", "closed", "merged"}:
                raise GithubIdentityMismatch("PR identity mismatch: identity cohort has invalid state")
            for field in ("headRefOid", "baseRefOid"):
                if re.fullmatch(r"(?:[0-9a-fA-F]{40}|[0-9a-fA-F]{64})", row[field]) is None:
                    raise GithubIdentityMismatch(f"PR identity mismatch: identity cohort has invalid {field}")
            if not isinstance(row.get("mergeCommit"), (dict, type(None))):
                raise GithubIdentityMismatch("PR identity mismatch: identity cohort has malformed mergeCommit")
            merge = row.get("mergeCommit")
            if merge is not None and (
                not isinstance(merge.get("oid"), str)
                or re.fullmatch(r"(?:[0-9a-fA-F]{40}|[0-9a-fA-F]{64})", merge["oid"]) is None
            ):
                raise GithubIdentityMismatch("PR identity mismatch: identity cohort has malformed merge OID")
            if not _valid_merge_snapshot(row):
                raise GithubIdentityMismatch("PR identity mismatch: identity cohort has incoherent merge state")

        matches = [
            row for row in cohort
            if row["headRefName"] == head_ref
            and row["baseRefName"] == base_ref
            and row["headRefOid"] == head_oid
        ]
        if len(matches) != 1:
            raise GithubIdentityMismatch(
                "PR identity mismatch: candidate does not map to exactly one current pull request"
            )
        number = matches[0]["number"]
        detail_fields = (
            "number,state,title,url,headRefName,headRefOid,baseRefName,baseRefOid,"
            "mergeCommit,mergedAt,reviewDecision,mergeStateStatus,reviews,statusCheckRollup"
        )
        detail_command = [executable, "pr", "view", str(number), "--repo", repository, "--json", detail_fields]
        details = self._run_candidate_query(detail_command, "current PR details", list_response=False)
        if not isinstance(details, dict):
            raise GithubUnavailable("GitHub current PR details unavailable: response was not an object")
        for field in ("number", "state", "headRefName", "headRefOid", "baseRefName", "baseRefOid", "mergedAt"):
            if details.get(field) != matches[0].get(field):
                raise GithubIdentityMismatch(f"PR identity mismatch: current details changed {field}")
        if not isinstance(details.get("number"), int) or isinstance(details.get("number"), bool):
            raise GithubIdentityMismatch("PR identity mismatch: current details have invalid number")
        if not isinstance(details.get("title"), str) or not details["title"]:
            raise GithubIdentityMismatch("PR identity mismatch: current details have invalid title")
        if not isinstance(details.get("url"), str) or not details["url"].startswith("https://"):
            raise GithubIdentityMismatch("PR identity mismatch: current details have invalid URL")
        if details["state"].lower() not in {"open", "closed", "merged"}:
            raise GithubIdentityMismatch("PR identity mismatch: current details have invalid state")
        for field in ("reviews", "statusCheckRollup"):
            if not isinstance(details.get(field), list) or any(not isinstance(item, dict) for item in details[field]):
                raise GithubIdentityMismatch(f"PR identity mismatch: current details have malformed {field}")
        for review in details["reviews"]:
            author = review.get("author")
            if (
                not isinstance(review.get("id") or review.get("databaseId"), (str, int))
                or isinstance(review.get("id") or review.get("databaseId"), bool)
                or not isinstance(author, dict)
                or not isinstance(author.get("login"), str)
                or not isinstance(review.get("state"), str)
            ):
                raise GithubIdentityMismatch("PR identity mismatch: current details have malformed review identity")
        for check in details["statusCheckRollup"]:
            typename = check.get("__typename")
            if typename == "CheckRun":
                status = check.get("status")
                conclusion = check.get("conclusion")
                valid = (
                    isinstance(check.get("name"), str)
                    and bool(check["name"])
                    and isinstance(status, str)
                    and status in {"QUEUED", "IN_PROGRESS", "COMPLETED", "WAITING", "PENDING", "REQUESTED"}
                    and (
                        conclusion is None
                        or isinstance(conclusion, str) and conclusion in {
                            "ACTION_REQUIRED", "CANCELLED", "FAILURE", "NEUTRAL",
                            "SKIPPED", "STALE", "STARTUP_FAILURE", "SUCCESS", "TIMED_OUT",
                        }
                    )
                    and (
                        isinstance(conclusion, str)
                        if status == "COMPLETED"
                        else conclusion is None
                    )
                    and (check.get("detailsUrl") is None or isinstance(check.get("detailsUrl"), str))
                )
            elif typename == "StatusContext":
                valid = (
                    isinstance(check.get("context"), str)
                    and bool(check["context"])
                    and isinstance(check.get("state"), str)
                    and check["state"] in {"EXPECTED", "ERROR", "FAILURE", "PENDING", "SUCCESS"}
                    and (check.get("targetUrl") is None or isinstance(check.get("targetUrl"), str))
                )
            else:
                valid = False
            if not valid:
                raise GithubIdentityMismatch("PR identity mismatch: current details have malformed check identity")
        if details.get("mergeCommit") != matches[0].get("mergeCommit"):
            raise GithubIdentityMismatch("PR identity mismatch: current details changed mergeCommit")
        if not _valid_merge_snapshot(details):
            raise GithubIdentityMismatch("PR identity mismatch: current details have incoherent merge state")
        return cohort, details

    def _run_candidate_query(
        self, command: list[str], stage: str, *, list_response: bool
    ) -> Any:
        try:
            result = subprocess.run(
                command, capture_output=True, text=True, encoding="utf-8",
                errors="replace", check=False,
            )
        except OSError as error:
            raise GithubUnavailable(f"GitHub {stage} unavailable: {type(error).__name__}") from error
        if result.returncode != 0:
            raise GithubUnavailable(f"GitHub {stage} unavailable: CLI exited {result.returncode}")
        try:
            value = json.loads(result.stdout or "null")
        except json.JSONDecodeError as error:
            raise GithubUnavailable(f"GitHub {stage} unavailable: invalid JSON") from error
        if list_response:
            if not isinstance(value, list) or any(not isinstance(item, dict) for item in value):
                raise GithubUnavailable(f"GitHub {stage} unavailable: malformed list response")
        elif not isinstance(value, dict):
            raise GithubUnavailable(f"GitHub {stage} unavailable: malformed object response")
        return value
