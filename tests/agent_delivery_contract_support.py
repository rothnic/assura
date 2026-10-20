from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path

from common.delivery import GithubUnavailable

class FakeGithub:
    def __init__(self, pull_requests: list[dict]) -> None:
        self.pull_requests = pull_requests

    def list_pull_requests(self, repository: str) -> list[dict]:
        return self.pull_requests

def write_fake_github(path: Path, pull_requests: list[dict]) -> None:
    rows = repr(json.dumps(pull_requests))
    source = (
        "#!/usr/bin/env python3\nimport json,sys\n"
        f"rows=json.loads({rows})\n"
        "number=int(sys.argv[3]) if len(sys.argv)>3 and sys.argv[2]=='view' else None\n"
        "detail=next((row.copy() for row in rows if row.get('number')==number), {}) if number else None\n"
        "detail and detail.setdefault('reviews', [])\n"
        "detail and detail.setdefault('statusCheckRollup', [])\n"
        "print(json.dumps(detail if number else rows))\n"
    )
    path.write_text(source, encoding="utf-8")
    if os.name != "nt":
        path.chmod(0o755)

class UnavailableGithub:
    def list_pull_requests(self, repository: str) -> list[dict]:
        raise GithubUnavailable("GitHub evidence unavailable")

def git(repo: Path, *args: str) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        capture_output=True,
        text=True,
        check=True,
    )
    return result.stdout.strip()

def task_cli(repo: Path, *args: str) -> subprocess.CompletedProcess[str]:
    environment = os.environ.copy()
    environment["TRELLIS_CONTEXT_ID"] = "delivery-contract-test"
    fake_bin = repo / ".test-bin"
    if fake_bin.is_dir():
        environment["PATH"] = str(fake_bin) + os.pathsep + environment.get("PATH", "")
    return subprocess.run(
        [sys.executable, str(ROOT / ".trellis" / "scripts" / "task.py"), *args],
        cwd=repo,
        env=environment,
        capture_output=True,
        text=True,
        check=False,
    )

def fixture_repo() -> tuple[Path, str, str, tempfile.TemporaryDirectory[str]]:
    directory = tempfile.TemporaryDirectory()
    repo = Path(directory.name)
    git(repo, "init", "-b", "master")
    git(repo, "config", "user.email", "delivery@example.test")
    git(repo, "config", "user.name", "Delivery Test")
    (repo / ".trellis" / "tasks" / "01-01-candidate").mkdir(parents=True)
    (repo / ".trellis" / ".developer").write_text("name=tester\n", encoding="utf-8")
    (repo / "README.md").write_text("base\n", encoding="utf-8")
    git(repo, "add", ".")
    git(repo, "commit", "-m", "base")
    base_oid = git(repo, "rev-parse", "HEAD")
    git(repo, "checkout", "-b", "candidate")
    (repo / "candidate.txt").write_text("candidate\n", encoding="utf-8")
    git(repo, "add", "candidate.txt")
    git(repo, "commit", "-m", "candidate")
    tip = git(repo, "rev-parse", "HEAD")
    task = {
        "id": "candidate",
        "name": "candidate",
        "title": "Candidate",
        "status": "in_progress",
        "branch": "refs/heads/candidate",
        "base_branch": "master",
        "meta": {
            "delivery": {
                "schema_version": 1,
                "kind": "integration",
                "candidate_id": "candidate-1",
                "owner": "tester",
                "repository": "rothnic/assura",
                "base_ref": "refs/heads/master",
                "branch_ref": "refs/heads/candidate",
                "authority_ref": "test:delivery-authority",
            }
        },
    }
    (repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json").write_text(
        json.dumps(task), encoding="utf-8"
    )
    local_origin = repo / "rothnic" / "assura.git"
    local_origin.parent.mkdir(parents=True)
    git(repo, "init", "--bare", str(local_origin))
    git(repo, "remote", "add", "origin", "rothnic/assura.git")
    git(repo, "push", "origin", "master:refs/heads/master")
    fake_bin = repo / ".test-bin"
    fake_bin.mkdir()
    fake_gh = fake_bin / ("gh.py" if os.name == "nt" else "gh")
    fake_pull_requests = json.dumps(
        [
            {
                "number": 1,
                "state": "MERGED",
                "title": "Candidate",
                "url": "https://github.com/rothnic/assura/pull/1",
                "headRefName": "candidate",
                "headRefOid": tip,
                "baseRefName": "master",
                "baseRefOid": base_oid,
                "mergeCommit": {"oid": tip},
                "mergedAt": "2026-09-14T00:00:00Z",
            }
        ]
    )
    write_fake_github(fake_gh, json.loads(fake_pull_requests))
    if os.name == "nt":
        # Windows does not execute extensionless scripts from PATH. Keep the
        # same fixture command while providing its native command shim.
        (fake_bin / "gh.cmd").write_text(
            '@python "%~dp0gh.py" %*\n',
            encoding="utf-8",
        )
    else:
        fake_gh.chmod(0o755)
    exclude = repo / ".git" / "info" / "exclude"
    exclude.write_text(
        exclude.read_text(encoding="utf-8") + ".test-bin/\nrothnic/\n",
        encoding="utf-8",
    )
    return repo, base_oid, tip, directory

def local_integration_evidence(tip: str, base_oid: str, label: str) -> dict:
    return {
        "review": {
            "schema_version": "assura.delivery-evidence.v1",
            "source": "local",
            "repository": "rothnic/assura",
            "evidence_ref": f"test:review-{label}",
            "result": "approved",
            "head_oid": tip,
            "reviewer": "independent-reviewer",
            "reviewer_role": "architect",
            "review_id": f"review-{label}",
            "findings": [],
        },
        "checks": {
            "schema_version": "assura.delivery-evidence.v1",
            "source": "local",
            "repository": "rothnic/assura",
            "evidence_ref": f"test:checks-{label}",
            "result": "passed",
            "conclusion": "success",
            "head_oid": tip,
            "base_oid": base_oid,
            "run_id": f"run-{label}",
            "job_id": f"job-{label}",
        },
        "postmerge": {
            "schema_version": "assura.delivery-evidence.v1",
            "source": "local",
            "repository": "rothnic/assura",
            "evidence_ref": f"test:postmerge-{label}",
            "result": "verified",
            "head_oid": tip,
            "base_oid": base_oid,
            "merge_oid": base_oid,
        },
        "acceptance": {
            "schema_version": "assura.delivery-evidence.v1",
            "source": "owner",
            "repository": "rothnic/assura",
            "evidence_ref": f"test:acceptance-{label}",
            "result": "accepted",
            "head_oid": tip,
            "acceptance_ref": "prd.md#acceptance",
            "authority_ref": "test:delivery-authority",
            "approved_by": "release-authority",
            "artifact_sha256": "a" * 64,
        },
    }
