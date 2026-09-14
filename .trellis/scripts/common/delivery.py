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
_KINDS = {"integration", "artifact", "experiment", "release", "aggregate"}
_TERMINAL_OUTCOMES = {"delivered", "superseded", "rejected", "cancelled"}
_ALLOWED_OUTCOMES = _TERMINAL_OUTCOMES | {"unknown", None}


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
            "number,state,title,url,headRefName,headRefOid,baseRefName,baseRefOid,mergeCommit,mergedAt",
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

    schema_version = delivery.get("schema_version")
    if schema_version != 1:
        raise DeliveryValidationError("unsupported delivery intent schema_version")

    kind = _required_string(delivery, "kind")
    if kind not in _KINDS:
        raise DeliveryValidationError(f"unsupported delivery kind: {kind}")

    candidate_id = _required_string(delivery, "candidate_id")
    if _SAFE_ID.fullmatch(candidate_id) is None:
        raise DeliveryValidationError(f"unsafe delivery candidate id: {candidate_id!r}")

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
        task_json=task_json,
    )


def repository_identity(repo_root: Path) -> str:
    """Resolve the canonical ``owner/name`` from the origin remote."""
    code, stdout, _ = run_git(["remote", "get-url", "origin"], cwd=repo_root)
    remote = stdout.strip() if code == 0 else ""
    if remote.startswith("git@") and ":" in remote:
        remote_path = remote.split(":", 1)[1]
    else:
        parsed = urlparse(remote)
        remote_path = parsed.path.lstrip("/")
    remote_path = remote_path.removesuffix(".git").strip("/")
    return remote_path or repo_root.name


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


def _read_worktrees(repo_root: Path) -> tuple[list[dict[str, Any]], list[DeliveryIssue]]:
    code, stdout, stderr = run_git(["worktree", "list", "--porcelain"], cwd=repo_root)
    if code != 0:
        return [], [DeliveryIssue("WORKTREE_INVENTORY_UNAVAILABLE", stderr.strip() or "unable to list worktrees", next_action="Repair Git worktree access and rerun the audit.")]

    records: list[dict[str, Any]] = []
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
        else:
            current["dirty_paths"] = []
            current["dirty"] = None
            current["state"] = "missing"
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
    return records, []


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
            intent_data["task_json"] = relative
            intent_error = None
        except DeliveryValidationError as error:
            intent_data = None
            intent_error = str(error)
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
                "meta": raw.get("meta") if isinstance(raw.get("meta"), dict) else {},
                "intent": intent_data,
                "intent_error": intent_error,
            }
        )
    tasks.sort(key=lambda item: item["task_path"])
    return tasks, issues


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
    }


def collect_inventory(
    repo_root: Path,
    github: GithubReader | None = None,
    base_ref: str | None = None,
) -> Inventory:
    """Collect a complete local inventory and explicit remote coverage."""
    repo_root = Path(repo_root).resolve()
    repository = repository_identity(repo_root)
    selected_base = base_ref or _choose_base_ref(repo_root)
    base_oid = _git_oid(repo_root, selected_base)
    refs, ref_issues = _read_refs(repo_root)
    worktrees, worktree_issues = _read_worktrees(repo_root)
    tasks, task_issues = _read_tasks(repo_root)
    issues = [*ref_issues, *worktree_issues, *task_issues]

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
        "tasks": "complete" if not task_issues else "partial",
        "github": github_state,
        "complete": not issues and base_oid is not None,
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
        issues=tuple(issues),
        repo_root=repo_root,
    )


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


def _ref_for_candidate(intent: DeliveryIntent, inventory: Inventory) -> dict[str, Any] | None:
    task = _task_for_intent(intent, inventory)
    branch = intent.branch_ref or (task or {}).get("branch")
    branch_short = _branch_name(branch)
    if not branch_short:
        return None
    for ref in inventory.refs:
        if ref.get("name") == branch or _branch_name(ref.get("name")) == branch_short:
            return ref
    return None


def _pr_for_candidate(intent: DeliveryIntent, inventory: Inventory, tip: str | None) -> dict[str, Any] | None:
    task = _task_for_intent(intent, inventory)
    branch_short = _branch_name(intent.branch_ref or (task or {}).get("branch"))
    matches = [
        pr
        for pr in inventory.pull_requests
        if (branch_short and _branch_name(pr.get("head_ref")) == branch_short)
        or (tip and pr.get("head_oid") == tip)
    ]
    return sorted(matches, key=lambda item: (str(item.get("number") or ""), str(item.get("url") or "")))[-1] if matches else None


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


def _evidence_verified(value: Any, tip: str | None = None, base_oid: str | None = None) -> bool:
    if value is True:
        return True
    if not isinstance(value, dict):
        return False
    result = str(value.get("result") or value.get("status") or value.get("conclusion") or "").lower()
    if result not in {"approved", "pass", "passed", "success", "successful", "verified", "accepted", "complete", "completed"} and value.get("verified") is not True:
        return False
    observed_tip = value.get("head_oid") or value.get("observed_tip")
    if observed_tip and tip and observed_tip != tip:
        return False
    observed_base = value.get("base_oid") or value.get("merge_oid")
    if value.get("requires_base") and base_oid and observed_base and observed_base != base_oid:
        return False
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
        facts = {
            "integration_verified": integration in {"ancestry_integrated", "pr_merged"},
            "review_resolved": _evidence_verified(_evidence_section(receipt, "review"), tip, inventory.base_oid),
            "required_checks_pass": _evidence_verified(_evidence_section(receipt, "checks"), tip, inventory.base_oid),
            "postmerge_verified": _evidence_verified(_evidence_section(receipt, "postmerge"), tip, inventory.base_oid),
            "acceptance_verified": _evidence_verified(_evidence_section(receipt, "acceptance"), tip, inventory.base_oid),
        }
    elif intent.kind in {"artifact", "experiment"}:
        facts = {
            "integration_verified": True,
            "review_resolved": True,
            "required_checks_pass": True,
            "postmerge_verified": True,
            "acceptance_verified": _evidence_verified(_evidence_section(receipt, "acceptance"), tip, inventory.base_oid),
            "durable_evidence_verified": _evidence_verified(_evidence_section(receipt, "durable"), tip, inventory.base_oid),
        }
    elif intent.kind == "release":
        facts = {
            "integration_verified": True,
            "review_resolved": True,
            "required_checks_pass": True,
            "postmerge_verified": True,
            "acceptance_verified": _evidence_verified(_evidence_section(receipt, "acceptance"), tip, inventory.base_oid),
            "tag_source_verified": _evidence_verified(_evidence_section(receipt, "tag"), tip, inventory.base_oid),
            "version_verified": _evidence_verified(_evidence_section(receipt, "version"), tip, inventory.base_oid),
            "required_assets_verified": _evidence_verified(_evidence_section(receipt, "assets"), tip, inventory.base_oid),
            "checksums_verified": _evidence_verified(_evidence_section(receipt, "checksums"), tip, inventory.base_oid),
            "install_verified": _evidence_verified(_evidence_section(receipt, "install"), tip, inventory.base_oid),
        }
    else:
        facts = {
            "integration_verified": True,
            "review_resolved": True,
            "required_checks_pass": True,
            "postmerge_verified": True,
            "acceptance_verified": _evidence_verified(_evidence_section(receipt, "acceptance"), tip, inventory.base_oid),
            "all_required_children_satisfied": _evidence_verified(_evidence_section(receipt, "children"), tip, inventory.base_oid),
        }

    if not inventory.base_oid:
        issues.append(DeliveryIssue("BASE_REF_UNRESOLVED", "integration ref is not resolved", intent.candidate_id, next_action="Resolve the base ref before closure."))
    if intent.kind == "integration" and pull_request is not None and inventory.coverage.get("github") != "complete":
        issues.append(DeliveryIssue("GITHUB_COVERAGE_UNAVAILABLE", "GitHub review/check coverage is incomplete", intent.candidate_id, next_action="Restore GitHub evidence access before claiming delivery."))
    if dirty:
        issues.append(DeliveryIssue("OWNED_WORKTREE_DIRTY", "candidate worktree has uncommitted changes", intent.candidate_id, next_action="Preserve and resolve the owned changes before closure."))
    return facts, issues


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
    ref = _ref_for_candidate(intent, inventory)
    tip = str(ref.get("oid")) if ref and ref.get("oid") else None
    pr = _pr_for_candidate(intent, inventory, tip)
    if not tip and pr and pr.get("head_oid"):
        tip = str(pr["head_oid"])

    branch = intent.branch_ref or (task or {}).get("branch")
    dirty = any(
        _worktree_branch_matches(worktree.get("branch_ref"), branch) and worktree.get("dirty") is True
        for worktree in inventory.worktrees
    )

    if not tip or inventory.base_oid is None:
        integration = "unknown"
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
    elif pr and pr.get("merged"):
        merge_oid = pr.get("merge_oid") or pr.get("head_oid")
        has_post_review_tip = bool(
            ref
            and pr.get("head_oid")
            and tip
            and tip != pr.get("head_oid")
            and not _is_ancestor(inventory.repo_root, tip, inventory.base_oid)
        )
        integration = "post_review_commits" if has_post_review_tip else (
            "pr_merged" if _is_ancestor(inventory.repo_root, merge_oid, inventory.base_oid) else "post_review_commits"
        )
    elif pr and str(pr.get("state")) == "closed":
        integration = "pr_closed_unmerged"
    elif pr:
        integration = "pr_open"
    else:
        integration = "unmerged"

    evidence = dict(receipt.get("evidence") or {}) if isinstance(receipt, dict) else {}
    facts, fact_issues = _required_facts(intent, inventory, receipt, tip, integration, dirty, pr)
    remote_coverage_unknown = inventory.coverage.get("github") != "complete" and (
        pr is not None or integration not in {"ancestry_integrated"}
    )
    coverage_unknown = not bool(inventory.coverage.get("base_resolved")) or inventory.coverage.get("git") != "complete" or inventory.coverage.get("tasks") != "complete" or remote_coverage_unknown
    issues: list[DeliveryIssue] = list(fact_issues)
    if not tip:
        issues.append(DeliveryIssue("CANDIDATE_TIP_UNRESOLVED", "candidate branch or PR head is not present in the inventory", intent.candidate_id, next_action="Resolve the branch/PR identity without deleting historical refs."))
    if integration in {"unmerged", "no_candidate_commit", "pr_open", "post_review_commits", "pr_closed_unmerged", "unknown"}:
        issues.append(DeliveryIssue("INTEGRATION_UNVERIFIED", f"current integration state is {integration}", intent.candidate_id, next_action="Review the exact candidate diff and integrate, supersede, reject, or hold it explicitly."))
    if isinstance(receipt, dict) and receipt.get("observed_tip") and tip and receipt.get("observed_tip") != tip:
        issues.append(DeliveryIssue("RECEIPT_TIP_CHANGED", "receipt evidence belongs to a different candidate tip", intent.candidate_id, next_action="Re-record evidence for the current exact tip."))

    recorded_outcome = receipt.get("outcome") if isinstance(receipt, dict) else None
    if recorded_outcome not in _ALLOWED_OUTCOMES:
        issues.append(DeliveryIssue("RECEIPT_OUTCOME_INVALID", "receipt contains an unsupported outcome", intent.candidate_id, next_action="Preserve the receipt and record a supported outcome."))
        recorded_outcome = None

    if recorded_outcome == "delivered":
        if not _may_deliver(intent, facts, dirty, coverage_unknown):
            issues.append(DeliveryIssue("UNSUPPORTED_DELIVERY_CLAIM", "receipt claims delivered without all current terminal evidence", intent.candidate_id, next_action="Revalidate required review, checks, integration, acceptance, and coverage evidence."))
            outcome = "unknown"
        else:
            outcome = "delivered"
    elif recorded_outcome in {"superseded", "rejected", "cancelled"}:
        decision = _evidence_section(receipt, "decision")
        if not isinstance(decision, dict) or not _evidence_verified(decision, tip, inventory.base_oid):
            issues.append(DeliveryIssue("OUTCOME_DECISION_UNVERIFIED", f"{recorded_outcome} lacks a verified decision bound to the current tip", intent.candidate_id, next_action="Record the decision, exact tip, and durable recovery evidence."))
            outcome = "unknown"
        elif recorded_outcome == "superseded" and not _evidence_section(receipt, "replacement"):
            issues.append(DeliveryIssue("REPLACEMENT_UNRESOLVED", "superseded outcome lacks a replacement reference", intent.candidate_id, next_action="Bind the candidate to the replacement task or PR and account for remaining diff."))
            outcome = "unknown"
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
            phase = "held" if any(issue.code.endswith("UNAVAILABLE") or issue.code in {"INTEGRATION_UNVERIFIED", "UNSUPPORTED_DELIVERY_CLAIM", "OWNED_WORKTREE_DIRTY"} for issue in issues) else phase
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
        if status.dirty:
            return [DeliveryIssue("ARCHIVE_WORKTREE_DIRTY", "physical archive cannot consume owned uncommitted changes", intent.candidate_id, next_action="Preserve and resolve the dirty worktree before archiving.")]
        return []
    if action == "close" and status.outcome not in _TERMINAL_OUTCOMES:
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
