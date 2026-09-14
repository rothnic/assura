"""Delivery intent, inventory, evidence, and closure primitives.

This module is intentionally a read-oriented projection layer. Task JSON stores
durable intent, Git/GitHub provide current facts, and the receipt store holds
explicit decisions and bounded evidence bindings. No function in this module
merges, publishes, deletes, or executes metadata as shell code.
"""

from __future__ import annotations

import json
import re
import subprocess
from dataclasses import asdict, dataclass, field
from pathlib import Path
from typing import Any, Protocol
from urllib.parse import urlparse

from .git import run_git


class DeliveryValidationError(ValueError):
    """A task does not contain a valid delivery intent."""


class GithubUnavailable(RuntimeError):
    """GitHub evidence could not be collected completely."""


_SAFE_ID = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
_VERSION = re.compile(
    r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
    r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$"
)
_SAFE_ASSET = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")
_FULL_OID = re.compile(r"^[0-9a-fA-F]{40}$")
_SHA256 = re.compile(r"^[0-9a-fA-F]{64}$")
_REQUIRED_RELEASE_ASSETS = (
    "assura-linux-amd64.tar.gz",
    "assura-linux-musl-amd64.tar.gz",
    "assura-macos-amd64.tar.gz",
    "assura-macos-arm64.tar.gz",
    "assura-windows-amd64.zip",
)
_KINDS = {"integration", "artifact", "experiment", "release", "aggregate"}
_TERMINAL_OUTCOMES = {"delivered", "superseded", "rejected", "cancelled"}
_ALLOWED_OUTCOMES = _TERMINAL_OUTCOMES | {"unknown", None}
_EVIDENCE_SUCCESS_RESULTS = {
    "approved",
    "pass",
    "passed",
    "success",
    "successful",
    "verified",
    "accepted",
    "complete",
    "completed",
}
_DELIVERY_EVIDENCE_SCHEMA = "assura.delivery-evidence.v1"
_EVIDENCE_SOURCES = {
    "artifact",
    "ci",
    "github",
    "git",
    "local",
    "owner",
}
_EVIDENCE_SOURCES_BY_SECTION = {
    "review": {"github", "local"},
    "checks": {"ci", "github", "local"},
    "postmerge": {"ci", "git", "local"},
    "acceptance": {"owner"},
    "durable": {"artifact", "ci", "local"},
    "decision": {"owner"},
    "replacement": {"owner"},
}


@dataclass(frozen=True)
class DeliveryIntent:
    """Validated, durable delivery intent read from a task record."""

    schema_version: int
    kind: str
    candidate_id: str
    owner: str
    repository: str
    base_ref: str
    branch_ref: str | None = None
    acceptance_ref: str | None = None
    authority_ref: str | None = None
    version: str | None = None
    required_assets: tuple[str, ...] = ()
    task_json: Path | None = None


@dataclass(frozen=True)
class DeliveryIssue:
    """A bounded, actionable delivery finding."""

    code: str
    message: str
    candidate_id: str | None = None
    evidence_ref: str | None = None
    next_action: str = ""

    def as_dict(self) -> dict[str, Any]:
        """Serialize the issue using stable field names."""
        return asdict(self)


@dataclass(frozen=True)
class CandidateStatus:
    """Computed status for one delivery candidate."""

    candidate_id: str
    owner: str
    kind: str
    task_path: str | None
    tip: str | None
    phase: str
    outcome: str | None
    integration: str
    dirty: bool
    closure: str
    next_action: str
    evidence: dict[str, Any] = field(default_factory=dict)
    issues: tuple[DeliveryIssue, ...] = ()

    @property
    def delivered(self) -> bool:
        """Return whether the candidate has a verified delivered outcome."""
        return self.outcome == "delivered" and self.closure in {"verified", "closed"}

    def as_dict(self) -> dict[str, Any]:
        """Serialize the computed status."""
        data = asdict(self)
        data["issues"] = [issue.as_dict() for issue in self.issues]
        return data


@dataclass(frozen=True)
class Inventory:
    """Current repository topology and remote evidence coverage."""

    schema_version: int
    repository: str
    base_ref: str | None
    base_oid: str | None
    coverage: dict[str, Any]
    refs: tuple[dict[str, Any], ...]
    worktrees: tuple[dict[str, Any], ...]
    tasks: tuple[dict[str, Any], ...]
    pull_requests: tuple[dict[str, Any], ...]
    remote_refs: tuple[dict[str, Any], ...] = ()
    issues: tuple[DeliveryIssue, ...] = ()
    repo_root: Path = field(default_factory=Path, repr=False, compare=False)

    def as_dict(self) -> dict[str, Any]:
        """Serialize the inventory using deterministic list ordering."""
        return {
            "schema_version": self.schema_version,
            "repository": self.repository,
            "base_ref": self.base_ref,
            "base_oid": self.base_oid,
            "coverage": self.coverage,
            "refs": list(self.refs),
            "worktrees": list(self.worktrees),
            "tasks": list(self.tasks),
            "pull_requests": list(self.pull_requests),
            "remote_refs": list(self.remote_refs),
            "issues": [issue.as_dict() for issue in self.issues],
        }


class GithubReader(Protocol):
    """Minimal injectable GitHub reader used by audits and tests."""

    def list_pull_requests(self, repository: str) -> list[dict[str, Any]]:
        """Return all available pull requests for a repository."""


class SubprocessGithubReader:
    """Read pull-request facts through the installed GitHub CLI."""

    def __init__(self, executable: str = "gh") -> None:
        self.executable = executable

    def list_pull_requests(self, repository: str) -> list[dict[str, Any]]:
        """Query all pull requests without invoking a shell."""
        command = [
            self.executable,
            "pr",
            "list",
            "--repo",
            repository,
            "--state",
            "all",
            "--limit",
            "1000",
            "--json",
            "number,state,title,url,headRefName,headRefOid,baseRefName,baseRefOid,mergeCommit,mergedAt,reviewDecision,reviews,statusCheckRollup",
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
        return [item for item in value if isinstance(item, dict)]


def _required_string(data: dict[str, Any], field: str) -> str:
    value = data.get(field)
    if not isinstance(value, str) or not value.strip():
        raise DeliveryValidationError(f"delivery intent requires non-empty {field}")
    return value.strip()


def _optional_string(data: dict[str, Any], field: str) -> str | None:
    value = data.get(field)
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        raise DeliveryValidationError(f"delivery intent field {field} must be a string")
    return value.strip()


def validate_delivery_mapping(
    delivery: dict[str, Any], task_json: Path | None = None
) -> DeliveryIntent:
    """Validate an in-memory ``meta.delivery`` mapping before it is persisted."""
    schema_version = delivery.get("schema_version")
    if schema_version != 1:
        raise DeliveryValidationError("unsupported delivery intent schema_version")

    kind = _required_string(delivery, "kind")
    if kind not in _KINDS:
        raise DeliveryValidationError(f"unsupported delivery kind: {kind}")

    candidate_id = _required_string(delivery, "candidate_id")
    if _SAFE_ID.fullmatch(candidate_id) is None:
        raise DeliveryValidationError(f"unsafe delivery candidate id: {candidate_id!r}")

    version = _optional_string(delivery, "version")
    if version is not None and _VERSION.fullmatch(version) is None:
        raise DeliveryValidationError(f"invalid delivery version: {version!r}")
    required_assets_value = delivery.get("required_assets", ())
    if not isinstance(required_assets_value, (list, tuple)):
        raise DeliveryValidationError("delivery intent field required_assets must be a list")
    required_assets_list: list[str] = []
    for item in required_assets_value:
        asset = _required_string({"value": item}, "value")
        if _SAFE_ASSET.fullmatch(asset) is None:
            raise DeliveryValidationError(f"unsafe required asset name: {asset!r}")
        if asset in required_assets_list:
            raise DeliveryValidationError(f"duplicate required asset name: {asset!r}")
        required_assets_list.append(asset)
    required_assets = tuple(required_assets_list)
    if kind == "release" and not version:
        raise DeliveryValidationError("release delivery intent requires non-empty version")
    if kind == "release" and not required_assets:
        raise DeliveryValidationError(
            "release delivery intent requires at least one required asset"
        )
    if kind == "release" and set(required_assets) != set(_REQUIRED_RELEASE_ASSETS):
        raise DeliveryValidationError(
            "release delivery intent must name exactly the five supported platform archives"
        )

    return DeliveryIntent(
        schema_version=schema_version,
        kind=kind,
        candidate_id=candidate_id,
        owner=_required_string(delivery, "owner"),
        repository=_required_string(delivery, "repository"),
        base_ref=_required_string(delivery, "base_ref"),
        branch_ref=_optional_string(delivery, "branch_ref"),
        acceptance_ref=_optional_string(delivery, "acceptance_ref"),
        authority_ref=_optional_string(delivery, "authority_ref"),
        version=version,
        required_assets=required_assets,
        task_json=task_json,
    )


def load_delivery_intent(task_json: Path) -> DeliveryIntent:
    """Load and validate ``meta.delivery`` from a Trellis task JSON file."""
    task_json = Path(task_json)
    try:
        raw = json.loads(task_json.read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise DeliveryValidationError(f"cannot read task JSON: {task_json}") from error

    if not isinstance(raw, dict):
        raise DeliveryValidationError("task JSON must contain an object")
    meta = raw.get("meta")
    delivery = meta.get("delivery") if isinstance(meta, dict) else None
    if not isinstance(delivery, dict):
        raise DeliveryValidationError("task is missing meta.delivery intent")
    return validate_delivery_mapping(delivery, task_json)


def repository_identity(repo_root: Path) -> str:
    """Resolve the canonical ``owner/name`` from the origin remote."""
    code, stdout, _ = run_git(["remote", "get-url", "origin"], cwd=repo_root)
    remote = stdout.strip() if code == 0 else ""
    if not remote:
        raise DeliveryValidationError(
            "origin remote is unavailable; cannot establish exact repository identity"
        )
    if remote.startswith("git@") and ":" in remote:
        remote_path = remote.split(":", 1)[1]
    else:
        parsed = urlparse(remote)
        remote_path = parsed.path.lstrip("/")
    remote_path = remote_path.removesuffix(".git").strip("/")
    if not remote_path or "/" not in remote_path:
        raise DeliveryValidationError(
            f"origin remote does not identify an owner/name repository: {remote!r}"
        )
    return remote_path


def git_common_dir(repo_root: Path) -> Path:
    """Return the absolute Git common directory for linked worktrees."""
    code, stdout, stderr = run_git(["rev-parse", "--git-common-dir"], cwd=repo_root)
    if code != 0 or not stdout.strip():
        raise DeliveryValidationError(
            f"cannot resolve Git common directory: {stderr.strip() or 'unknown git error'}"
        )
    common = Path(stdout.strip())
    if not common.is_absolute():
        common = repo_root / common
    return common.resolve()


def delivery_store_root(repo_root: Path) -> Path:
    """Return the local receipt directory outside the task worktree."""
    return git_common_dir(repo_root) / "assura" / "delivery-v1"


def _git_oid(repo_root: Path, ref: str | None) -> str | None:
    if not ref:
        return None
    candidates = [ref]
    if not ref.startswith("refs/"):
        candidates.extend((f"refs/heads/{ref}", f"refs/remotes/origin/{ref}"))
    for candidate in candidates:
        code, stdout, _ = run_git(["rev-parse", "--verify", candidate], cwd=repo_root)
        if code == 0 and stdout.strip():
            return stdout.strip().splitlines()[0]
    return None


def _choose_base_ref(repo_root: Path) -> str | None:
    for ref in ("origin/master", "master", "origin/main", "main"):
        if _git_oid(repo_root, ref):
            return ref
    return None


def _branch_name(ref: str | None) -> str | None:
    if not ref:
        return None
    value = ref.strip()
    for prefix in ("refs/remotes/origin/", "refs/heads/", "origin/"):
        if value.startswith(prefix):
            return value[len(prefix):]
    return value


def _worktree_branch_matches(worktree_branch: str | None, branch_ref: str | None) -> bool:
    if not worktree_branch or not branch_ref:
        return False
    return _branch_name(worktree_branch) == _branch_name(branch_ref)


def _worktree_matches_candidate(
    worktree: dict[str, Any],
    intent: DeliveryIntent,
    task: dict[str, Any] | None,
    tip: str | None,
) -> bool:
    """Match attached, path-bound, or detached worktrees conservatively."""
    branch = intent.branch_ref or (task or {}).get("branch")
    if _worktree_branch_matches(worktree.get("branch_ref"), branch):
        return True

    configured_path = (task or {}).get("worktree_path")
    if isinstance(configured_path, str) and configured_path.strip():
        try:
            if Path(worktree.get("path", "")).resolve() == Path(configured_path).resolve():
                return True
        except (OSError, RuntimeError, TypeError):
            pass

    # A detached worktree has no branch identity. An exact candidate tip is
    # the strongest available local binding; treating matching dirt as owned
    # is safer than allowing it to disappear from closure accounting.
    return not worktree.get("branch_ref") and bool(
        tip and worktree.get("head_oid") == tip
    )


def _read_worktrees(repo_root: Path) -> tuple[list[dict[str, Any]], list[DeliveryIssue]]:
    code, stdout, stderr = run_git(["worktree", "list", "--porcelain"], cwd=repo_root)
    if code != 0:
        return [], [DeliveryIssue("WORKTREE_INVENTORY_UNAVAILABLE", stderr.strip() or "unable to list worktrees", next_action="Repair Git worktree access and rerun the audit.")]

    records: list[dict[str, Any]] = []
    issues: list[DeliveryIssue] = []
    current: dict[str, Any] = {}

    def finish() -> None:
        if not current:
            return
        path = Path(str(current.get("path", "")))
        if path.is_dir():
            status_code, status_out, status_err = run_git(
                ["status", "--porcelain", "--untracked-files=all"], cwd=path
            )
            if status_code == 0:
                current["dirty_paths"] = [line for line in status_out.splitlines() if line]
                current["dirty"] = bool(current["dirty_paths"])
                current["state"] = "available"
            else:
                current["dirty_paths"] = []
                current["dirty"] = None
                current["state"] = "unavailable"
                current["error"] = status_err.strip() or "unable to inspect worktree"
                issues.append(
                    DeliveryIssue(
                        "WORKTREE_COVERAGE_UNAVAILABLE",
                        current["error"],
                        evidence_ref=str(path),
                        next_action="Preserve the worktree and restore Git status access before closing or archiving candidates.",
                    )
                )
        else:
            current["dirty_paths"] = []
            current["dirty"] = None
            current["state"] = "missing"
            current["error"] = "worktree path is missing"
            issues.append(
                DeliveryIssue(
                    "WORKTREE_COVERAGE_UNAVAILABLE",
                    "worktree path is missing",
                    evidence_ref=str(path),
                    next_action="Restore the missing worktree or record its preserved ownership and disposition before closure.",
                )
            )
        records.append(dict(current))
        current.clear()

    for line in stdout.splitlines():
        if not line.strip():
            finish()
            continue
        key, _, value = line.partition(" ")
        if key == "worktree":
            finish()
            current["path"] = value
        elif key == "HEAD":
            current["head_oid"] = value
        elif key == "branch":
            current["branch_ref"] = value
        elif key == "detached":
            current["detached"] = True
        elif key == "bare":
            current["bare"] = True
    finish()
    records.sort(key=lambda item: str(item.get("path", "")))
    return records, issues


def _read_refs(repo_root: Path) -> tuple[list[dict[str, Any]], list[DeliveryIssue]]:
    format_string = "%(refname)%09%(objectname)%09%(symref)"
    code, stdout, stderr = run_git(
        ["for-each-ref", f"--format={format_string}", "refs/heads", "refs/remotes", "refs/tags"],
        cwd=repo_root,
    )
    if code != 0:
        return [], [DeliveryIssue("REF_INVENTORY_UNAVAILABLE", stderr.strip() or "unable to list refs", next_action="Repair Git ref access and rerun the audit.")]
    refs: list[dict[str, Any]] = []
    for line in stdout.splitlines():
        fields = line.split("\t", 2)
        if len(fields) < 2:
            continue
        name, oid = fields[:2]
        refs.append(
            {
                "name": name,
                "oid": oid,
                "symref": fields[2] if len(fields) == 3 and fields[2] else None,
                "kind": name.split("/", 2)[1] if name.startswith("refs/") else "other",
            }
        )
    refs.sort(key=lambda item: item["name"])
    return refs, []


def _read_remote_refs(
    repo_root: Path, remote: str = "origin"
) -> tuple[list[dict[str, Any]], list[DeliveryIssue]]:
    """Read advertised heads/tags without changing local refs.

    A local tracking ref is only a cache. Fleet-level audit completeness is
    therefore withheld until this explicit read-only advertisement succeeds.
    """
    code, stdout, stderr = run_git(
        ["ls-remote", "--heads", "--tags", remote], cwd=repo_root
    )
    if code != 0:
        return [], [
            DeliveryIssue(
                "REMOTE_REF_COVERAGE_UNAVAILABLE",
                stderr.strip() or f"unable to read advertised refs from {remote}",
                next_action="Restore remote ref access and rerun the audit with --refresh.",
            )
        ]

    refs: list[dict[str, Any]] = []
    for line in stdout.splitlines():
        oid, separator, name = line.partition("\t")
        if not separator or not _FULL_OID.fullmatch(oid) or not name.startswith("refs/"):
            return [], [
                DeliveryIssue(
                    "REMOTE_REF_COVERAGE_INVALID",
                    f"remote returned an invalid ref advertisement: {line!r}",
                    next_action="Inspect the remote response and rerun the audit.",
                )
            ]
        kind = name.split("/", 2)[1] if name.startswith("refs/") else "other"
        if kind not in {"heads", "tags"}:
            continue
        refs.append(
            {
                "name": name,
                "oid": oid,
                "kind": kind,
                "source": "remote-advertisement",
            }
        )
    refs.sort(key=lambda item: item["name"])
    return refs, []


def _read_tasks(repo_root: Path) -> tuple[list[dict[str, Any]], list[DeliveryIssue]]:
    tasks_root = repo_root / ".trellis" / "tasks"
    if not tasks_root.is_dir():
        return [], []
    tasks: list[dict[str, Any]] = []
    issues: list[DeliveryIssue] = []
    for task_json in sorted(tasks_root.rglob("task.json")):
        relative = task_json.parent.relative_to(repo_root).as_posix()
        try:
            raw = json.loads(task_json.read_text(encoding="utf-8"))
        except (OSError, UnicodeError, json.JSONDecodeError) as error:
            issues.append(DeliveryIssue("TASK_RECORD_UNREADABLE", str(error), evidence_ref=relative, next_action="Preserve the task and repair its JSON before classifying it."))
            continue
        if not isinstance(raw, dict):
            issues.append(DeliveryIssue("TASK_RECORD_INVALID", "task.json is not an object", evidence_ref=relative, next_action="Preserve the task and repair its schema before classifying it."))
            continue
        try:
            intent = load_delivery_intent(task_json)
            intent_data = asdict(intent)
            intent_data["task_json"] = task_json.relative_to(repo_root).as_posix()
            intent_data["required_assets"] = list(intent.required_assets)
            intent_error = None
        except DeliveryValidationError as error:
            intent_data = None
            intent_error = str(error)
            meta = raw.get("meta")
            if isinstance(meta, dict) and "delivery" in meta:
                issues.append(
                    DeliveryIssue(
                        "DELIVERY_INTENT_INVALID",
                        intent_error,
                        evidence_ref=relative,
                        next_action="Preserve the task and repair its versioned delivery intent before continuing.",
                    )
                )
        tasks.append(
            {
                "task_path": relative,
                "directory": relative.rsplit("/", 1)[0],
                "archived": "/archive/" in f"/{relative}/",
                "id": raw.get("id") or raw.get("name"),
                "title": raw.get("title") or raw.get("name"),
                "status": raw.get("status", "unknown"),
                "assignee": raw.get("assignee"),
                "branch": raw.get("branch"),
                "base_branch": raw.get("base_branch"),
                "worktree_path": raw.get("worktree_path"),
                "children": raw.get("children") if isinstance(raw.get("children"), list) else [],
                "meta": raw.get("meta") if isinstance(raw.get("meta"), dict) else {},
                "intent": intent_data,
                "intent_error": intent_error,
            }
        )
    tasks.sort(key=lambda item: item["task_path"])
    return tasks, issues


def _normalise_review(raw: Any) -> dict[str, Any]:
    """Keep the bounded identity/result fields needed to verify a review."""
    if not isinstance(raw, dict):
        return {}
    author = raw.get("author")
    author_name = (
        author.get("login")
        if isinstance(author, dict)
        else raw.get("author_login") or raw.get("reviewer")
    )
    return {
        "id": raw.get("id") or raw.get("databaseId"),
        "url": raw.get("url") or raw.get("html_url"),
        "author": author_name,
        "state": str(raw.get("state") or raw.get("status") or "").lower(),
    }


def _normalise_check(raw: Any) -> dict[str, Any]:
    """Keep bounded check identity and terminal result fields."""
    if not isinstance(raw, dict):
        return {}
    conclusion = raw.get("conclusion") or raw.get("state") or raw.get("bucket")
    return {
        "id": raw.get("databaseId") or raw.get("id"),
        "name": raw.get("name") or raw.get("context"),
        "context": raw.get("context"),
        "url": raw.get("detailsUrl") or raw.get("url") or raw.get("link"),
        "status": str(raw.get("status") or "").lower(),
        "conclusion": str(conclusion or "").lower(),
    }


def _normalise_pr(raw: dict[str, Any]) -> dict[str, Any]:
    merge = raw.get("mergeCommit")
    merge_oid = merge.get("oid") or merge.get("sha") if isinstance(merge, dict) else raw.get("merge_commit") or raw.get("mergeCommitOid")
    state = str(raw.get("state") or "").lower()
    merged_at = raw.get("mergedAt") or raw.get("merged_at")
    return {
        "number": raw.get("number"),
        "state": state,
        "title": raw.get("title"),
        "url": raw.get("url"),
        "head_ref": raw.get("headRefName") or raw.get("head_ref"),
        "head_oid": raw.get("headRefOid") or raw.get("head_oid") or raw.get("head_sha"),
        "base_ref": raw.get("baseRefName") or raw.get("base_ref"),
        "base_oid": raw.get("baseRefOid") or raw.get("base_oid"),
        "merge_oid": merge_oid,
        "merged_at": merged_at,
        "merged": state == "merged" or bool(merged_at),
        "review_decision": str(
            raw.get("reviewDecision") or raw.get("review_decision") or ""
        ).lower(),
        "reviews": [
            review
            for review in (
                _normalise_review(item)
                for item in raw.get("latestReviews", raw.get("reviews", []))
            )
            if review
        ],
        "checks": [
            check
            for check in (
                _normalise_check(item)
                for item in raw.get("statusCheckRollup", raw.get("checks", []))
            )
            if check
        ],
    }


def collect_inventory(
    repo_root: Path,
    github: GithubReader | None = None,
    base_ref: str | None = None,
    refresh_remote: bool = False,
) -> Inventory:
    """Collect local inventory and optionally refresh advertised remote refs.

    ``refresh_remote`` is deliberately explicit because ``git fetch`` would
    mutate refs and a local tracking ref is not proof of the remote state.
    """
    repo_root = Path(repo_root).resolve()
    identity_issue: DeliveryIssue | None = None
    try:
        repository = repository_identity(repo_root)
    except DeliveryValidationError as error:
        repository = "(unknown)"
        identity_issue = DeliveryIssue(
            "REPOSITORY_IDENTITY_UNAVAILABLE",
            str(error),
            next_action="Configure an exact origin owner/name remote and rerun the audit.",
        )
    selected_base = base_ref or _choose_base_ref(repo_root)
    base_oid = _git_oid(repo_root, selected_base)
    refs, ref_issues = _read_refs(repo_root)
    worktrees, worktree_issues = _read_worktrees(repo_root)
    tasks, task_issues = _read_tasks(repo_root)
    if refresh_remote:
        remote_refs, remote_ref_issues = _read_remote_refs(repo_root)
        remote_ref_state = "verified" if not remote_ref_issues else "unavailable"
    else:
        remote_refs, remote_ref_issues = [], []
        remote_ref_state = "local-tracking-only"
    issues = [
        *([identity_issue] if identity_issue is not None else []),
        *ref_issues,
        *worktree_issues,
        *task_issues,
        *remote_ref_issues,
    ]

    if identity_issue is not None:
        pull_requests = ()
        github_state = "unavailable"
    else:
        reader = github or SubprocessGithubReader()
        try:
            pull_requests = tuple(
                sorted(
                    (_normalise_pr(item) for item in reader.list_pull_requests(repository)),
                    key=lambda item: (str(item.get("number") or ""), str(item.get("head_ref") or "")),
                )
            )
            github_state = "complete"
        except GithubUnavailable as error:
            pull_requests = ()
            github_state = "unavailable"
            issues.append(DeliveryIssue("GITHUB_COVERAGE_UNAVAILABLE", str(error), next_action="Restore authenticated GitHub evidence access and rerun the audit; do not treat missing PRs as no PRs."))
        except Exception as error:
            pull_requests = ()
            github_state = "unavailable"
            issues.append(DeliveryIssue("GITHUB_COVERAGE_UNAVAILABLE", f"unexpected GitHub reader failure: {error}", next_action="Inspect the GitHub reader failure and rerun the audit."))

    coverage = {
        "git": "complete" if not ref_issues and not worktree_issues else "partial",
        "refs": "complete" if not ref_issues else "partial",
        "worktrees": "complete" if not worktree_issues else "partial",
        "tasks": "complete" if not task_issues else "partial",
        "github": github_state,
        "host": "local-only",
        "remote_refs": remote_ref_state,
        "complete": not issues and base_oid is not None and remote_ref_state == "verified",
        "base_resolved": base_oid is not None,
    }
    if base_oid is None:
        issues.append(DeliveryIssue("BASE_REF_UNRESOLVED", f"could not resolve integration ref {selected_base or '(none)'}", next_action="Refresh the integration ref or explicitly provide the correct base ref."))
        coverage["complete"] = False
    return Inventory(
        schema_version=1,
        repository=repository,
        base_ref=selected_base,
        base_oid=base_oid,
        coverage=coverage,
        refs=tuple(refs),
        worktrees=tuple(worktrees),
        tasks=tuple(tasks),
        pull_requests=pull_requests,
        remote_refs=tuple(remote_refs),
        issues=tuple(issues),
        repo_root=repo_root,
    )




# Preserve the historical common.delivery API while keeping status projection
# separate from intent and inventory collection.
from .delivery_status import (  # noqa: E402,F401
    classify_candidate,
    _github_checks_match_pr,
    _github_evidence_matches_pr,
    _github_review_matches_pr,
    _pr_for_candidate,
    render_checkpoint,
    validate_evidence_mapping,
    validate_release_receipt,
    validate_transition,
)
