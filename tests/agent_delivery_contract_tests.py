#!/usr/bin/env python3
"""Focused tests for the Assura delivery ownership and closure contract."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / ".trellis" / "scripts"))

from common.delivery import (  # noqa: E402
    CandidateStatus,
    DeliveryValidationError,
    classify_candidate,
    collect_inventory,
    load_delivery_intent,
    render_checkpoint,
)
from common.delivery_cli import project_audit  # noqa: E402
from common.delivery_store import (  # noqa: E402
    DeliveryStore,
    ReceiptCorrupt,
    StaleGeneration,
)
from common.tasks import children_progress  # noqa: E402


class DeliveryIntentTests(unittest.TestCase):
    def test_loads_versioned_integration_intent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_json = Path(directory) / "task.json"
            task_json.write_text(
                json.dumps(
                    {
                        "id": "task-1",
                        "meta": {
                            "delivery": {
                                "schema_version": 1,
                                "kind": "integration",
                                "candidate_id": "candidate-1",
                                "owner": "nroth",
                                "repository": "rothnic/assura",
                                "base_ref": "refs/heads/master",
                                "branch_ref": "refs/heads/codex/candidate-1",
                                "acceptance_ref": "prd.md#acceptance",
                            }
                        },
                    }
                ),
                encoding="utf-8",
            )

            intent = load_delivery_intent(task_json)

            self.assertEqual(intent.candidate_id, "candidate-1")
            self.assertEqual(intent.kind, "integration")
            self.assertEqual(intent.owner, "nroth")
            self.assertEqual(intent.base_ref, "refs/heads/master")

    def test_rejects_unsafe_candidate_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_json = Path(directory) / "task.json"
            task_json.write_text(
                json.dumps(
                    {
                        "meta": {
                            "delivery": {
                                "schema_version": 1,
                                "kind": "integration",
                                "candidate_id": "../../overwrite",
                                "owner": "nroth",
                                "repository": "rothnic/assura",
                                "base_ref": "refs/heads/master",
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaises(DeliveryValidationError):
                load_delivery_intent(task_json)


class DeliveryStoreTests(unittest.TestCase):
    def test_compare_and_swap_rejects_stale_writer(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            store = DeliveryStore(Path(directory))
            first = store.create(
                "candidate-1",
                {"phase": "registered", "owner": "nroth"},
            )

            second = store.update(
                "candidate-1",
                first["generation"],
                {"phase": "review"},
            )

            self.assertEqual(second["generation"], first["generation"] + 1)
            with self.assertRaises(StaleGeneration):
                store.update(
                    "candidate-1",
                    first["generation"],
                    {"phase": "implementing"},
                )
            self.assertEqual(store.read("candidate-1")["phase"], "review")

    def test_corrupt_receipt_does_not_disappear_as_empty_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            store = DeliveryStore(Path(directory))
            store.create("candidate-1", {"phase": "registered"})
            receipt_path = store.path_for("candidate-1")
            receipt_path.write_text("{not-json", encoding="utf-8")

            with self.assertRaises(ReceiptCorrupt):
                store.read("candidate-1")


class FakeGithub:
    def __init__(self, pull_requests: list[dict]) -> None:
        self.pull_requests = pull_requests

    def list_pull_requests(self, repository: str) -> list[dict]:
        return self.pull_requests


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
            }
        },
    }
    (repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json").write_text(
        json.dumps(task), encoding="utf-8"
    )
    git(repo, "remote", "add", "origin", "https://github.com/rothnic/assura.git")
    return repo, base_oid, tip, directory


class DeliveryProjectionTests(unittest.TestCase):
    def test_missing_child_is_unknown_not_done(self) -> None:
        progress = children_progress(("missing", "delivered"), {"delivered": "outcome:delivered"})

        self.assertIn("1/2 delivered", progress)
        self.assertIn("unknown=1", progress)

    def test_inventory_reports_unmerged_candidate_without_inventing_delivery(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            inventory = collect_inventory(repo, github=FakeGithub([]), base_ref="refs/heads/master")
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            intent = load_delivery_intent(task_json)
            status = classify_candidate(intent, inventory)

            self.assertEqual(status.tip, tip)
            self.assertEqual(status.integration, "unmerged")
            self.assertIsNone(status.outcome)
            self.assertTrue(any(issue.code == "INTEGRATION_UNVERIFIED" for issue in status.issues))
        finally:
            directory.cleanup()

    def test_inventory_uses_merge_commit_for_squash_merge(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--squash", "candidate")
            git(repo, "commit", "-m", "squash candidate")
            merge_oid = git(repo, "rev-parse", "HEAD")
            inventory = collect_inventory(
                repo,
                github=FakeGithub(
                    [
                        {
                            "number": 12,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": tip,
                            "baseRefName": "master",
                            "baseRefOid": base_oid,
                            "mergeCommit": {"oid": merge_oid},
                        }
                    ]
                ),
                base_ref="refs/heads/master",
            )
            intent = load_delivery_intent(repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json")
            status = classify_candidate(intent, inventory)

            self.assertEqual(status.integration, "pr_merged")
        finally:
            directory.cleanup()

    def test_checkpoint_is_deterministic_and_bounded(self) -> None:
        statuses = [
            CandidateStatus(
                candidate_id=f"candidate-{index}",
                owner="tester",
                kind="integration",
                task_path=None,
                tip=None,
                phase="held",
                outcome=None,
                integration="unknown",
                dirty=False,
                closure="open",
                next_action="resolve ownership",
            )
            for index in range(20)
        ]

        first = render_checkpoint(statuses)
        second = render_checkpoint(list(reversed(statuses)))

        self.assertEqual(first, second)
        self.assertLessEqual(len(first.encode("utf-8")), 1024)
        self.assertGreaterEqual(len(first.splitlines()), 3)
        self.assertLessEqual(len(first.splitlines()), 6)

    def test_repeated_audit_is_identical_and_does_not_write(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            before = sorted(
                path.relative_to(repo).as_posix()
                for path in repo.rglob("*")
                if path.is_file()
            )
            first, _ = project_audit(repo, github=FakeGithub([]))
            second, _ = project_audit(repo, github=FakeGithub([]))
            after = sorted(
                path.relative_to(repo).as_posix()
                for path in repo.rglob("*")
                if path.is_file()
            )

            self.assertEqual(first, second)
            self.assertEqual(before, after)
        finally:
            directory.cleanup()


class DeliveryLifecycleCommandTests(unittest.TestCase):
    def test_create_seeds_versioned_delivery_intent(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            created = task_cli(repo, "create", "New candidate", "--slug", "new-candidate", "--assignee", "tester")
            self.assertEqual(created.returncode, 0, created.stderr)
            task_json = next((repo / ".trellis" / "tasks").glob("*-new-candidate")) / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            delivery = task["meta"]["delivery"]
            self.assertEqual(delivery["schema_version"], 1)
            self.assertEqual(delivery["candidate_id"], task_json.parent.name)
            self.assertEqual(delivery["owner"], "tester")
        finally:
            directory.cleanup()

    def test_start_registers_receipt_and_finish_is_only_a_pause(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            started = task_cli(repo, "start", task_path)
            self.assertEqual(started.returncode, 0, started.stderr)
            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            self.assertTrue(receipt_path.is_file())

            finished = task_cli(repo, "finish")
            self.assertEqual(finished.returncode, 0, finished.stderr)
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            self.assertEqual(receipt["session_state"], "paused")
            task = json.loads((repo / task_path / "task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["status"], "in_progress")
        finally:
            directory.cleanup()

    def test_archive_rejects_unmerged_task_without_mutation(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            task_json = repo / task_path / "task.json"
            before_task = task_json.read_bytes()
            before_receipt = (repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json").read_bytes()
            archived = task_cli(repo, "archive", task_path, "--no-commit")

            self.assertNotEqual(archived.returncode, 0)
            self.assertIn("INTEGRATION_UNVERIFIED", archived.stdout + archived.stderr)
            self.assertTrue(task_json.is_file())
            self.assertEqual(task_json.read_bytes(), before_task)
            self.assertEqual(
                (repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json").read_bytes(),
                before_receipt,
            )
        finally:
            directory.cleanup()

    def test_archive_accepts_verified_outcome_and_records_closed_receipt(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            base_after_merge = git(repo, "rev-parse", "HEAD")
            evidence_path = repo / "evidence.json"
            evidence_path.write_text(
                json.dumps(
                    {
                        "review": {"result": "approved", "head_oid": tip},
                        "checks": {"result": "passed", "head_oid": tip},
                        "postmerge": {"result": "verified", "head_oid": tip, "base_oid": base_after_merge},
                        "acceptance": {"result": "accepted", "head_oid": tip},
                    }
                ),
                encoding="utf-8",
            )
            recorded = task_cli(
                repo,
                "delivery",
                "record",
                task_path,
                "--evidence-file",
                str(evidence_path),
                "--expected-generation",
                "1",
            )
            self.assertEqual(recorded.returncode, 0, recorded.stderr)
            closed = task_cli(repo, "delivery", "close", task_path, "--outcome", "delivered")
            self.assertEqual(closed.returncode, 0, closed.stderr)
            archived = task_cli(repo, "archive", task_path, "--no-commit")

            self.assertEqual(archived.returncode, 0, archived.stderr)
            self.assertFalse((repo / task_path).exists())
            receipts = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            receipt = json.loads(receipts.read_text(encoding="utf-8"))
            self.assertEqual(receipt["outcome"], "delivered")
            self.assertEqual(receipt["closure"], "closed")
        finally:
            directory.cleanup()

    def test_archive_commit_does_not_sweep_unrelated_staged_or_dirty_paths(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            base_after_merge = git(repo, "rev-parse", "HEAD")
            evidence_path = repo / "evidence.json"
            evidence_path.write_text(
                json.dumps(
                    {
                        "review": {"result": "approved", "head_oid": tip},
                        "checks": {"result": "passed", "head_oid": tip},
                        "postmerge": {"result": "verified", "head_oid": tip, "base_oid": base_after_merge},
                        "acceptance": {"result": "accepted", "head_oid": tip},
                    }
                ),
                encoding="utf-8",
            )
            self.assertEqual(
                task_cli(repo, "delivery", "record", task_path, "--evidence-file", str(evidence_path), "--expected-generation", "1").returncode,
                0,
            )
            self.assertEqual(task_cli(repo, "delivery", "close", task_path, "--outcome", "delivered").returncode, 0)

            (repo / "unrelated.txt").write_text("staged\n", encoding="utf-8")
            git(repo, "add", "unrelated.txt")
            (repo / "foreign.txt").write_text("leave me\n", encoding="utf-8")
            archived = task_cli(repo, "archive", task_path)

            self.assertEqual(archived.returncode, 0, archived.stderr)
            committed_paths = git(repo, "show", "--format=", "--name-only", "HEAD").splitlines()
            self.assertTrue(any("archive" in path for path in committed_paths))
            self.assertNotIn("unrelated.txt", committed_paths)
            self.assertTrue((repo / "foreign.txt").is_file())
            self.assertIn("unrelated.txt", git(repo, "diff", "--cached", "--name-only").splitlines())
        finally:
            directory.cleanup()


if __name__ == "__main__":
    unittest.main(verbosity=2)
