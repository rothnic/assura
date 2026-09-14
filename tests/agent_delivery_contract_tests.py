#!/usr/bin/env python3
"""Focused tests for the Assura delivery ownership and closure contract."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import types
import unittest
from unittest import mock
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / ".trellis" / "scripts"))

RELEASE_ARCHIVES = (
    "assura-linux-amd64.tar.gz",
    "assura-linux-musl-amd64.tar.gz",
    "assura-macos-amd64.tar.gz",
    "assura-macos-arm64.tar.gz",
    "assura-windows-amd64.zip",
)

from common.delivery import (  # noqa: E402
    CandidateStatus,
    DeliveryValidationError,
    GithubUnavailable,
    SubprocessGithubReader,
    classify_candidate,
    collect_inventory,
    load_delivery_intent,
    _read_worktrees,
    render_checkpoint,
)
from common.delivery_cli import _load_evidence, project_audit  # noqa: E402
from common.delivery_store import (  # noqa: E402
    DeliveryStore,
    ReceiptBusy,
    ReceiptCorrupt,
    StaleGeneration,
)
from common.tasks import children_progress, get_all_statuses  # noqa: E402


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

    def test_release_intent_requires_version_and_safe_unique_assets(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_json = Path(directory) / "task.json"
            base = {
                "schema_version": 1,
                "kind": "release",
                "candidate_id": "release-1",
                "owner": "nroth",
                "repository": "rothnic/assura",
                "base_ref": "refs/heads/master",
            }
            task_json.write_text(json.dumps({"meta": {"delivery": base}}), encoding="utf-8")
            with self.assertRaises(DeliveryValidationError):
                load_delivery_intent(task_json)

            base.update(
                {
                    "version": "0.4.0",
                    "required_assets": ["assura-linux.tar.gz", "assura-linux.tar.gz"],
                }
            )
            task_json.write_text(json.dumps({"meta": {"delivery": base}}), encoding="utf-8")
            with self.assertRaises(DeliveryValidationError):
                load_delivery_intent(task_json)

    def test_release_intent_requires_version_and_platform_assets(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_json = Path(directory) / "task.json"
            task_json.write_text(
                json.dumps(
                    {
                        "meta": {
                            "delivery": {
                                "schema_version": 1,
                                "kind": "release",
                                "candidate_id": "release-040",
                                "owner": "nroth",
                                "repository": "rothnic/assura",
                                "base_ref": "refs/heads/master",
                                "version": "0.4.0",
                                "required_assets": [
                                    "assura-linux-amd64.tar.gz",
                                    "assura-linux-musl-amd64.tar.gz",
                                    "assura-macos-amd64.tar.gz",
                                    "assura-macos-arm64.tar.gz",
                                    "assura-windows-amd64.zip",
                                ],
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )

            intent = load_delivery_intent(task_json)

            self.assertEqual(intent.version, "0.4.0")
            self.assertEqual(
                intent.required_assets,
                (
                    "assura-linux-amd64.tar.gz",
                    "assura-linux-musl-amd64.tar.gz",
                    "assura-macos-amd64.tar.gz",
                    "assura-macos-arm64.tar.gz",
                    "assura-windows-amd64.zip",
                ),
            )

    def test_release_receipt_json_is_loaded_as_one_evidence_section(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            evidence_path = Path(directory) / "release-receipt.json"
            evidence_path.write_text(
                json.dumps(
                    {
                        "schema_version": "assura.release-receipt.v1",
                        "repository": "rothnic/assura",
                        "version": "0.4.0",
                    }
                ),
                encoding="utf-8",
            )

            evidence = _load_evidence(evidence_path)

            self.assertEqual(
                evidence,
                {
                    "release": {
                        "schema_version": "assura.release-receipt.v1",
                        "repository": "rothnic/assura",
                        "version": "0.4.0",
                    }
                },
            )


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

    def test_competing_process_reports_busy_without_deleting_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            store_root = Path(directory)
            store = DeliveryStore(store_root)
            store.create("candidate-1", {"phase": "registered"})
            script = (
                "from common.delivery_store import DeliveryStore, ReceiptBusy\n"
                "import sys\n"
                "store = DeliveryStore(sys.argv[1])\n"
                "try:\n"
                "    store.update('candidate-1', 0, {'phase': 'review'})\n"
                "except ReceiptBusy:\n"
                "    print('busy')\n"
                "    raise SystemExit(0)\n"
                "raise SystemExit(1)\n"
            )
            environment = os.environ.copy()
            script_root = str(ROOT / ".trellis" / "scripts")
            environment["PYTHONPATH"] = script_root + os.pathsep + environment.get("PYTHONPATH", "")
            with store._locked("candidate-1"):
                competing = subprocess.run(
                    [sys.executable, "-c", script, str(store_root)],
                    env=environment,
                    capture_output=True,
                    text=True,
                    check=False,
                )

            self.assertEqual(competing.returncode, 0, competing.stderr)
            self.assertEqual(competing.stdout.strip(), "busy")
            self.assertEqual(store.read("candidate-1")["generation"], 0)

    def test_interrupted_atomic_receipt_write_preserves_previous_record(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            store = DeliveryStore(Path(directory))
            store.create("candidate-1", {"phase": "registered"})
            receipt_path = store.path_for("candidate-1")
            before = receipt_path.read_bytes()

            with mock.patch(
                "common.delivery_store.os.replace",
                side_effect=OSError("simulated interruption"),
            ):
                with self.assertRaises(OSError):
                    store._write_atomic(receipt_path, {"phase": "corrupted"})

            self.assertEqual(receipt_path.read_bytes(), before)
            self.assertEqual(store.read("candidate-1")["phase"], "registered")
            self.assertEqual(list(Path(directory).glob(".*.tmp")), [])

    def test_windows_lock_acquire_and_release_use_the_same_byte(self) -> None:
        calls: list[tuple[str, int]] = []

        class LockFile:
            def __init__(self) -> None:
                self.position = 17
                self.contents = b""

            def seek(self, position: int) -> None:
                self.position = position

            def read(self, size: int) -> bytes:
                return self.contents[:size]

            def write(self, value: bytes) -> None:
                self.contents = value

            def flush(self) -> None:
                return None

            def fileno(self) -> int:
                return 42

            def tell(self) -> int:
                return self.position

        lock_file = LockFile()
        fake_msvcrt = types.SimpleNamespace(
            LK_NBLCK=1,
            LK_UNLCK=2,
            locking=lambda _fd, mode, _size: calls.append(
                ("acquire" if mode == 1 else "release", lock_file.tell())
            ),
        )
        with mock.patch("common.delivery_store.os.name", "nt"):
            with mock.patch.dict(sys.modules, {"msvcrt": fake_msvcrt}):
                DeliveryStore._acquire_lock(lock_file)
                DeliveryStore._release_lock(lock_file)

        self.assertEqual(calls, [("acquire", 0), ("release", 0)])


class FakeGithub:
    def __init__(self, pull_requests: list[dict]) -> None:
        self.pull_requests = pull_requests

    def list_pull_requests(self, repository: str) -> list[dict]:
        return self.pull_requests


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
    git(repo, "remote", "add", "origin", "https://github.com/rothnic/assura.git")
    fake_bin = repo / ".test-bin"
    fake_bin.mkdir()
    fake_gh = fake_bin / "gh"
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
    fake_gh.write_text(
        "#!/usr/bin/env python3\nprint(" + repr(fake_pull_requests) + ")\n",
        encoding="utf-8",
    )
    fake_gh.chmod(0o755)
    exclude = repo / ".git" / "info" / "exclude"
    exclude.write_text(exclude.read_text(encoding="utf-8") + ".test-bin/\n", encoding="utf-8")
    return repo, base_oid, tip, directory


class DeliveryProjectionTests(unittest.TestCase):
    def test_github_coverage_failure_blocks_local_ancestry_delivery(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merged_oid = git(repo, "rev-parse", "HEAD")
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            receipt = store.read("candidate-1")
            evidence = {
                "review": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:review-1",
                    "result": "approved",
                    "head_oid": tip,
                    "reviewer": "independent-reviewer",
                    "reviewer_role": "architect",
                    "review_id": "review-1",
                    "findings": [],
                },
                "checks": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "evidence_ref": "test:checks-1",
                    "result": "passed",
                    "conclusion": "success",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "repository": "rothnic/assura",
                    "run_id": "run-1",
                    "job_id": "process-contracts",
                },
                "postmerge": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "evidence_ref": "test:postmerge-1",
                    "result": "verified",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "merge_oid": merged_oid,
                    "repository": "rothnic/assura",
                },
                "acceptance": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "owner",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:acceptance-1",
                    "result": "accepted",
                    "head_oid": tip,
                    "acceptance_ref": "prd.md#acceptance",
                    "authority_ref": "test:delivery-authority",
                    "approved_by": "release-authority",
                    "artifact_sha256": "a" * 64,
                },
            }
            store.update(
                "candidate-1",
                receipt["generation"],
                {
                    "evidence": evidence,
                    "observed_tip": tip,
                    "outcome": "delivered",
                    "closure": "verified",
                    "phase": "verified",
                },
            )

            inventory = collect_inventory(
                repo,
                github=UnavailableGithub(),
                base_ref="refs/heads/master",
            )
            intent = load_delivery_intent(repo / task_path / "task.json")
            status = classify_candidate(intent, inventory, store.read("candidate-1"))

            self.assertEqual(status.integration, "ancestry_integrated")
            self.assertNotEqual(status.outcome, "delivered")
            self.assertTrue(
                any(issue.code == "GITHUB_COVERAGE_UNAVAILABLE" for issue in status.issues)
            )
            self.assertTrue(
                any(issue.code == "UNSUPPORTED_DELIVERY_CLAIM" for issue in status.issues)
            )
        finally:
            directory.cleanup()

    def test_release_receipt_must_match_current_tag_source(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"].update(
                {
                    "kind": "release",
                    "version": "0.4.0",
                    "required_assets": list(RELEASE_ARCHIVES),
                }
            )
            task_json.write_text(json.dumps(task), encoding="utf-8")
            git(repo, "add", str(task_json.relative_to(repo)))
            git(repo, "commit", "-m", "record release intent")
            tip = git(repo, "rev-parse", "HEAD")
            git(repo, "tag", "v0.4.0")

            assets = [
                {
                    "name": name,
                    "size": 1,
                    "sha256": "a" * 64,
                    "checksum_name": f"{name}.sha256",
                    "members": [
                        "assura.exe" if name.endswith(".zip") else "assura",
                        "assura-full.exe" if name.endswith(".zip") else "assura-full",
                    ],
                }
                for name in RELEASE_ARCHIVES
            ]
            receipt = {
                "schema_version": "assura.release-receipt.v1",
                "repository": "rothnic/assura",
                "version": "0.4.0",
                "tag": "v0.4.0",
                "tag_oid": tip,
                "commit_oid": tip,
                "workflow_runs": [{"workflow": "Release", "run_id": "run-1"}],
                "assets": assets,
                "checksums_verified": True,
                "install": {
                    "verified": True,
                    "assura": "assura 0.4.0",
                    "assura-full": "assura 0.4.0",
                },
                "publish": {
                    "action": "verified",
                    "repository": "rothnic/assura",
                    "tag": "v0.4.0",
                    "verified_assets": list(RELEASE_ARCHIVES),
                },
            }
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "phase": "verified",
                    "outcome": "delivered",
                    "closure": "verified",
                    "observed_tip": tip,
                    "evidence": {"release": receipt},
                },
            )

            inventory = collect_inventory(repo, github=FakeGithub([]), base_ref="refs/heads/master")
            intent = load_delivery_intent(task_json)
            status = classify_candidate(intent, inventory, store.read("candidate-1"))
            self.assertNotEqual(status.outcome, "delivered")
            self.assertTrue(
                any(issue.code == "UNSUPPORTED_DELIVERY_CLAIM" for issue in status.issues)
            )

            acceptance = {
                "schema_version": "assura.delivery-evidence.v1",
                "source": "owner",
                "repository": "rothnic/assura",
                "evidence_ref": "test:release-acceptance-1",
                "result": "accepted",
                "head_oid": tip,
                "acceptance_ref": "prd.md#acceptance",
                "authority_ref": "test:delivery-authority",
                "approved_by": "release-authority",
                "artifact_sha256": "b" * 64,
            }
            current_receipt = store.read("candidate-1")
            store.update(
                "candidate-1",
                current_receipt["generation"],
                {"evidence": {"release": receipt, "acceptance": acceptance}},
            )
            status = classify_candidate(intent, inventory, store.read("candidate-1"))
            self.assertEqual(status.outcome, "delivered")

            git(repo, "tag", "-f", "v0.4.0", "master")
            changed_inventory = collect_inventory(
                repo, github=FakeGithub([]), base_ref="refs/heads/master"
            )
            changed_status = classify_candidate(
                intent, changed_inventory, store.read("candidate-1")
            )
            self.assertNotEqual(changed_status.outcome, "delivered")
            self.assertTrue(
                any(issue.code == "UNSUPPORTED_DELIVERY_CLAIM" for issue in changed_status.issues)
            )
        finally:
            directory.cleanup()

    def test_active_verified_receipt_counts_as_explicit_child_outcome(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merged_tip = git(repo, "rev-parse", "HEAD")
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "repository": "rothnic/assura",
                    "task_path": ".trellis/tasks/01-01-candidate/task.json",
                    "outcome": "delivered",
                    "closure": "verified",
                    "phase": "verified",
                    "observed_tip": merged_tip,
                    "registered_tip": merged_tip,
                    "registered_base_oid": git(repo, "rev-parse", "HEAD^") ,
                    "evidence": {
                        "review": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:review-parent-progress",
                            "result": "approved",
                            "head_oid": merged_tip,
                            "reviewer": "independent-reviewer",
                            "reviewer_role": "architect",
                            "review_id": "review-parent-progress",
                            "findings": [],
                        },
                        "checks": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:checks-parent-progress",
                            "result": "passed",
                            "conclusion": "success",
                            "head_oid": merged_tip,
                            "base_oid": merged_tip,
                            "run_id": "run-parent-progress",
                            "job_id": "job-parent-progress",
                        },
                        "postmerge": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:postmerge-parent-progress",
                            "result": "verified",
                            "head_oid": merged_tip,
                            "base_oid": merged_tip,
                            "merge_oid": merged_tip,
                        },
                        "acceptance": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "owner",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:acceptance-parent-progress",
                            "result": "accepted",
                            "head_oid": merged_tip,
                            "acceptance_ref": "prd.md#acceptance",
                            "authority_ref": "test:delivery-authority",
                            "approved_by": "release-authority",
                            "artifact_sha256": "d" * 64,
                        },
                    },
                },
            )

            statuses = get_all_statuses(
                repo / ".trellis" / "tasks", github=FakeGithub([])
            )

            self.assertEqual(
                statuses["01-01-candidate"],
                "outcome:delivered",
            )
            self.assertIn(
                "1/1 delivered",
                children_progress(("01-01-candidate",), statuses),
            )
        finally:
            directory.cleanup()

    def test_stale_terminal_receipt_does_not_count_as_child_outcome(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "repository": "rothnic/assura",
                    "task_path": ".trellis/tasks/01-01-candidate/task.json",
                    "outcome": "delivered",
                    "closure": "verified",
                    "phase": "verified",
                    "observed_tip": tip,
                },
            )
            git(repo, "checkout", "candidate")
            (repo / "moved.txt").write_text("moved\n", encoding="utf-8")
            git(repo, "add", "moved.txt")
            git(repo, "commit", "-m", "move candidate tip")
            git(repo, "checkout", "master")

            statuses = get_all_statuses(
                repo / ".trellis" / "tasks", github=FakeGithub([])
            )

            self.assertNotEqual(
                statuses["01-01-candidate"],
                "outcome:delivered",
            )
            self.assertIn(
                "unknown=1",
                children_progress(("01-01-candidate",), statuses),
            )
        finally:
            directory.cleanup()

    def test_missing_child_is_unknown_not_done(self) -> None:
        progress = children_progress(("missing", "delivered"), {"delivered": "outcome:delivered"})

        self.assertIn("1/2 delivered", progress)
        self.assertIn("unknown=1", progress)

    def test_completed_label_without_verified_receipt_is_unknown(self) -> None:
        progress = children_progress(
            ("legacy-completed", "verified"),
            {"legacy-completed": "completed", "verified": "outcome:delivered"},
        )

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

    def test_ambiguous_pull_requests_do_not_promote_a_candidate(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            inventory = collect_inventory(
                repo,
                github=FakeGithub(
                    [
                        {
                            "number": 21,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": tip,
                            "baseRefName": "master",
                            "baseRefOid": base_oid,
                            "mergeCommit": {"oid": tip},
                        },
                        {
                            "number": 22,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": tip,
                            "baseRefName": "master",
                            "baseRefOid": base_oid,
                            "mergeCommit": {"oid": tip},
                        },
                    ]
                ),
                base_ref="refs/heads/master",
            )
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )
            status = classify_candidate(intent, inventory)

            self.assertNotEqual(status.integration, "pr_merged")
            self.assertTrue(
                any(issue.code == "PR_IDENTITY_AMBIGUOUS" for issue in status.issues)
            )
        finally:
            directory.cleanup()

    def test_unavailable_worktree_is_a_coverage_issue(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            missing = Path(directory) / "missing-worktree"
            output = (
                f"worktree {missing}\n"
                + "HEAD "
                + "a" * 40
                + "\nbranch refs/heads/candidate\n\n"
            )
            with mock.patch(
                "common.delivery.run_git",
                return_value=(0, output, ""),
            ):
                records, issues = _read_worktrees(Path(directory))

            self.assertEqual(records[0]["state"], "missing")
            self.assertEqual(records[0]["dirty"], None)
            self.assertTrue(
                any(issue.code == "WORKTREE_COVERAGE_UNAVAILABLE" for issue in issues)
            )

    def test_inventory_serializes_the_task_json_path(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            inventory = collect_inventory(repo, github=FakeGithub([]), base_ref="refs/heads/master")

            task = next(item for item in inventory.tasks if item["id"] == "candidate")

            self.assertEqual(
                task["intent"]["task_json"],
                ".trellis/tasks/01-01-candidate/task.json",
            )
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

    def test_github_evidence_requires_provider_review_and_check_facts(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merged_oid = git(repo, "rev-parse", "HEAD")
            provider_pr = {
                "number": 14,
                "state": "MERGED",
                "headRefName": "candidate",
                "headRefOid": tip,
                "baseRefName": "master",
                "baseRefOid": base_oid,
                "mergeCommit": {"oid": tip},
                "reviewDecision": "APPROVED",
                "reviews": [
                    {
                        "id": "review-14",
                        "author": {"login": "independent-reviewer"},
                        "state": "APPROVED",
                    }
                ],
                "statusCheckRollup": [
                    {
                        "databaseId": "job-14",
                        "name": "process-contracts",
                        "detailsUrl": "https://github.com/rothnic/assura/actions/runs/run-14/job/job-14",
                        "conclusion": "SUCCESS",
                    }
                ],
            }
            evidence = {
                "review": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "github",
                    "repository": "rothnic/assura",
                    "evidence_ref": "https://github.com/rothnic/assura/pull/14#pullrequestreview-14",
                    "result": "approved",
                    "head_oid": tip,
                    "pr_number": 14,
                    "reviewer": "independent-reviewer",
                    "reviewer_role": "architect",
                    "review_id": "review-14",
                    "findings": [],
                },
                "checks": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "github",
                    "repository": "rothnic/assura",
                    "evidence_ref": "https://github.com/rothnic/assura/actions/runs/run-14",
                    "result": "passed",
                    "conclusion": "success",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "pr_number": 14,
                    "run_id": "run-14",
                    "job_id": "job-14",
                    "check_name": "process-contracts",
                },
                "postmerge": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "local",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:postmerge-14",
                    "result": "verified",
                    "head_oid": tip,
                    "base_oid": merged_oid,
                    "merge_oid": merged_oid,
                },
                "acceptance": {
                    "schema_version": "assura.delivery-evidence.v1",
                    "source": "owner",
                    "repository": "rothnic/assura",
                    "evidence_ref": "test:acceptance-14",
                    "result": "accepted",
                    "head_oid": tip,
                    "acceptance_ref": "prd.md#acceptance",
                    "authority_ref": "test:delivery-authority",
                    "approved_by": "release-authority",
                    "artifact_sha256": "c" * 64,
                },
            }
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "owner": "tester",
                    "phase": "verified",
                    "outcome": "delivered",
                    "closure": "verified",
                    "observed_tip": tip,
                    "evidence": evidence,
                },
            )
            inventory = collect_inventory(
                repo, github=FakeGithub([provider_pr]), base_ref="refs/heads/master"
            )

            status = classify_candidate(intent, inventory, store.read("candidate-1"))

            self.assertEqual(status.integration, "pr_merged")
            self.assertEqual(status.outcome, "delivered")

            changed_provider_pr = {**provider_pr, "reviewDecision": "CHANGES_REQUESTED"}
            changed_inventory = collect_inventory(
                repo,
                github=FakeGithub([changed_provider_pr]),
                base_ref="refs/heads/master",
            )
            changed_status = classify_candidate(
                intent, changed_inventory, store.read("candidate-1")
            )

            self.assertNotEqual(changed_status.outcome, "delivered")
            self.assertTrue(
                any(
                    issue.code == "GITHUB_REVIEW_UNVERIFIED"
                    for issue in changed_status.issues
                )
            )

            changed_checks = {
                **provider_pr,
                "statusCheckRollup": [
                    {
                        "databaseId": "job-14",
                        "name": "process-contracts",
                        "detailsUrl": "https://github.com/rothnic/assura/actions/runs/other-run/job/job-14",
                        "conclusion": "SUCCESS",
                    }
                ],
            }
            changed_checks_inventory = collect_inventory(
                repo,
                github=FakeGithub([changed_checks]),
                base_ref="refs/heads/master",
            )
            changed_checks_status = classify_candidate(
                intent, changed_checks_inventory, store.read("candidate-1")
            )
            self.assertNotEqual(changed_checks_status.outcome, "delivered")
            self.assertTrue(
                any(
                    issue.code == "GITHUB_CHECKS_UNVERIFIED"
                    for issue in changed_checks_status.issues
                )
            )
        finally:
            directory.cleanup()

    def test_post_merge_commit_after_reviewed_head_remains_outstanding(self) -> None:
        repo, base_oid, reviewed_tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "merge", "--squash", "candidate")
            git(repo, "commit", "-m", "squash candidate")
            merge_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "checkout", "candidate")
            (repo / "follow-up.txt").write_text("post-review\n", encoding="utf-8")
            git(repo, "add", "follow-up.txt")
            git(repo, "commit", "-m", "post-review change")
            current_tip = git(repo, "rev-parse", "HEAD")
            inventory = collect_inventory(
                repo,
                github=FakeGithub(
                    [
                        {
                            "number": 13,
                            "state": "MERGED",
                            "headRefName": "candidate",
                            "headRefOid": reviewed_tip,
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

            self.assertEqual(status.tip, current_tip)
            self.assertEqual(status.integration, "post_review_commits")
            self.assertTrue(any(issue.code == "INTEGRATION_UNVERIFIED" for issue in status.issues))
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

    def test_audit_reports_all_unowned_local_branches_and_orphan_receipts(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            git(repo, "branch", "unusual/topic")
            orphan_store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            orphan_store.create("orphan-receipt", {"phase": "held"})

            report, _ = project_audit(repo, github=FakeGithub([]))

            names = {item["name"] for item in report["unowned_refs"]}
            self.assertIn("refs/heads/unusual/topic", names)
            self.assertEqual(report["orphan_receipts"][0]["candidate_id"], "orphan-receipt")
            self.assertEqual(report["orphan_receipts"][0]["disposition"], "unresolved")
            self.assertTrue(all("disposition" in item for item in report["unowned_worktrees"]))
        finally:
            directory.cleanup()

    def test_audit_holds_duplicate_candidate_identity_across_tasks(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            original = json.loads(
                (
                    repo
                    / ".trellis"
                    / "tasks"
                    / "01-01-candidate"
                    / "task.json"
                ).read_text(encoding="utf-8")
            )
            duplicate = dict(original)
            duplicate.update(
                {
                    "id": "duplicate-task",
                    "name": "duplicate-task",
                    "title": "Duplicate task",
                }
            )
            duplicate_dir = repo / ".trellis" / "tasks" / "01-02-duplicate"
            duplicate_dir.mkdir(parents=True)
            (duplicate_dir / "task.json").write_text(
                json.dumps(duplicate), encoding="utf-8"
            )

            _, statuses = project_audit(repo, github=FakeGithub([]))

            duplicate_statuses = [
                status
                for status in statuses
                if status.candidate_id == "candidate-1"
            ]
            self.assertEqual(len(duplicate_statuses), 2)
            self.assertTrue(
                all(
                    status.outcome is None
                    and any(
                        issue.code == "DUPLICATE_CANDIDATE_ID"
                        for issue in status.issues
                    )
                    for status in duplicate_statuses
                )
            )
        finally:
            directory.cleanup()

    def test_audit_keeps_detached_worktree_and_invalid_intent_visible(self) -> None:
        repo, _, _, directory = fixture_repo()
        detached = Path(directory.name) / "detached-worktree"
        try:
            git(repo, "worktree", "add", "--detach", str(detached), "master")
            (detached / "detached.txt").write_text("detached\n", encoding="utf-8")
            git(detached, "add", "detached.txt")
            git(detached, "commit", "-m", "detached candidate")

            invalid_dir = repo / ".trellis" / "tasks" / "01-02-invalid"
            invalid_dir.mkdir(parents=True)
            (invalid_dir / "task.json").write_text(
                json.dumps(
                    {
                        "id": "invalid",
                        "assignee": "tester",
                        "meta": {"delivery": {"schema_version": 1, "kind": "release"}},
                    }
                ),
                encoding="utf-8",
            )

            report, _ = project_audit(repo, github=FakeGithub([]))

            detached_rows = [
                item for item in report["unowned_worktrees"] if item.get("detached")
            ]
            self.assertEqual(len(detached_rows), 1)
            self.assertEqual(detached_rows[0]["disposition"], "unresolved")
            self.assertEqual(len(report["invalid_intents"]), 1)
            self.assertEqual(
                report["invalid_intents"][0]["issues"][0]["code"],
                "DELIVERY_INTENT_INVALID",
            )
        finally:
            git(repo, "worktree", "remove", str(detached))
            directory.cleanup()

    def test_owned_detached_worktree_dirt_blocks_candidate_and_is_not_dropped(self) -> None:
        repo, _, tip, directory = fixture_repo()
        detached = Path(directory.name) / "owned-detached-worktree"
        try:
            git(repo, "worktree", "add", "--detach", str(detached), tip)
            (detached / "uncommitted.txt").write_text("preserve\n", encoding="utf-8")

            report, statuses = project_audit(repo, github=FakeGithub([]))

            status = next(item for item in statuses if item.candidate_id == "candidate-1")
            self.assertTrue(status.dirty)
            self.assertTrue(
                any(
                    item.get("path")
                    and Path(item["path"]).resolve() == detached.resolve()
                    and item.get("dirty") is True
                    for item in report["inventory"]["worktrees"]
                )
            )
            self.assertFalse(
                any(
                    item.get("path")
                    and Path(item["path"]).resolve() == detached.resolve()
                    for item in report["unowned_worktrees"]
                )
            )
        finally:
            git(repo, "worktree", "remove", "--force", str(detached))
            directory.cleanup()

    def test_audit_marks_tracking_only_remote_refs_as_incomplete(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            report, _ = project_audit(repo, github=FakeGithub([]))

            self.assertEqual(report["coverage"]["remote_refs"], "local-tracking-only")
            self.assertFalse(report["coverage"]["complete"])
        finally:
            directory.cleanup()

    def test_explicit_remote_refresh_proves_remote_ref_coverage(self) -> None:
        repo, _, _, directory = fixture_repo()
        advertised = [{"name": "refs/heads/master", "oid": "a" * 40, "kind": "heads"}]
        try:
            with mock.patch(
                "common.delivery._read_remote_refs",
                return_value=(advertised, []),
            ):
                report, _ = project_audit(
                    repo, github=FakeGithub([]), refresh_remote=True
                )

            self.assertEqual(report["coverage"]["remote_refs"], "verified")
            self.assertTrue(report["coverage"]["complete"])
            self.assertEqual(report["inventory"]["remote_refs"], advertised)
        finally:
            directory.cleanup()

    def test_remote_advertised_candidate_without_local_branch_remains_identifiable(self) -> None:
        repo, base_oid, tip, directory = fixture_repo()
        try:
            git(repo, "checkout", "master")
            git(repo, "branch", "-D", "candidate")
            advertised = [
                {"name": "refs/heads/master", "oid": base_oid, "kind": "heads"},
                {"name": "refs/heads/candidate", "oid": tip, "kind": "heads"},
            ]
            with mock.patch(
                "common.delivery._read_remote_refs",
                return_value=(advertised, []),
            ):
                inventory = collect_inventory(
                    repo,
                    github=FakeGithub(
                        [
                            {
                                "number": 31,
                                "state": "OPEN",
                                "headRefName": "candidate",
                                "headRefOid": tip,
                                "baseRefName": "master",
                                "baseRefOid": base_oid,
                            }
                        ]
                    ),
                    base_ref="refs/heads/master",
                    refresh_remote=True,
                )
            intent = load_delivery_intent(
                repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )

            status = classify_candidate(intent, inventory)

            self.assertEqual(status.tip, tip)
            self.assertEqual(status.integration, "pr_open")
            self.assertFalse(
                any(issue.code == "CANDIDATE_TIP_UNRESOLVED" for issue in status.issues)
            )
        finally:
            directory.cleanup()

    def test_github_reader_rejects_a_full_page_as_incomplete_coverage(self) -> None:
        payload = json.dumps([{"number": index} for index in range(1000)])
        completed = subprocess.CompletedProcess(
            ["gh"],
            0,
            stdout=payload,
            stderr="",
        )
        with mock.patch("common.delivery.subprocess.run", return_value=completed):
            with self.assertRaises(GithubUnavailable):
                SubprocessGithubReader().list_pull_requests("rothnic/assura")


class DeliveryLifecycleCommandTests(unittest.TestCase):
    def test_invalid_registration_does_not_mutate_task_json(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_dir = repo / ".trellis" / "tasks" / "01-02-release"
            task_dir.mkdir(parents=True)
            task_json = task_dir / "task.json"
            task_json.write_text(
                json.dumps(
                    {
                        "id": "release",
                        "status": "in_progress",
                        "branch": "refs/heads/candidate",
                    }
                ),
                encoding="utf-8",
            )
            before = task_json.read_bytes()

            registered = task_cli(
                repo,
                "delivery",
                "register",
                ".trellis/tasks/01-02-release",
                "--candidate",
                "release-1",
                "--owner",
                "tester",
                "--kind",
                "release",
            )

            self.assertNotEqual(registered.returncode, 0)
            self.assertEqual(task_json.read_bytes(), before)
            self.assertFalse(
                (repo / ".git" / "assura" / "delivery-v1" / "release-1.json").exists()
            )
        finally:
            directory.cleanup()

    def test_register_can_fill_missing_authority_without_replacing_intent(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"].pop("authority_ref")
            task_json.write_text(json.dumps(task), encoding="utf-8")

            registered = task_cli(
                repo,
                "delivery",
                "register",
                ".trellis/tasks/01-01-candidate",
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
                "--authority-ref",
                "test:delivery-authority",
            )

            self.assertEqual(registered.returncode, 0, registered.stderr)
            updated = json.loads(task_json.read_text(encoding="utf-8"))
            self.assertEqual(
                updated["meta"]["delivery"]["authority_ref"],
                "test:delivery-authority",
            )
            self.assertEqual(updated["meta"]["delivery"]["candidate_id"], "candidate-1")
        finally:
            directory.cleanup()

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

    def test_finish_does_not_reopen_a_verified_terminal_receipt(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            receipt = store.read("candidate-1")
            store.update(
                "candidate-1",
                receipt["generation"],
                {
                    "outcome": "delivered",
                    "closure": "verified",
                    "phase": "verified",
                },
            )

            finished = task_cli(repo, "finish")

            self.assertEqual(finished.returncode, 0, finished.stderr)
            terminal = store.read("candidate-1")
            self.assertEqual(terminal["outcome"], "delivered")
            self.assertEqual(terminal["closure"], "verified")
            self.assertEqual(terminal["session_state"], "paused")
        finally:
            directory.cleanup()

    def test_start_blocks_a_second_unfinished_candidate_for_the_same_owner(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            created = task_cli(
                repo,
                "create",
                "Second candidate",
                "--slug",
                "second-candidate",
                "--assignee",
                "tester",
            )
            self.assertEqual(created.returncode, 0, created.stderr)
            second_path = next(
                (repo / ".trellis" / "tasks").glob("*-second-candidate")
            )

            started = task_cli(repo, "start", str(second_path))

            self.assertNotEqual(started.returncode, 0)
            self.assertIn("UNFINISHED_OWNED_CANDIDATE", started.stderr)
            self.assertFalse(
                (repo / ".git" / "assura" / "delivery-v1" / f"{second_path.name}.json").exists()
            )
        finally:
            directory.cleanup()

    def test_start_blocks_a_same_owner_candidate_with_a_missing_receipt(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            second_dir = repo / ".trellis" / "tasks" / "01-02-missing-receipt"
            second_dir.mkdir(parents=True)
            second_task = json.loads(
                (repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json").read_text(
                    encoding="utf-8"
                )
            )
            second_task["id"] = "missing-receipt"
            second_task["name"] = "missing-receipt"
            second_task["branch"] = "refs/heads/missing-receipt"
            second_task["meta"]["delivery"]["candidate_id"] = "candidate-2"
            second_task["meta"]["delivery"]["branch_ref"] = "refs/heads/missing-receipt"
            (second_dir / "task.json").write_text(
                json.dumps(second_task), encoding="utf-8"
            )

            started = task_cli(repo, "start", ".trellis/tasks/01-01-candidate")

            self.assertNotEqual(started.returncode, 0)
            self.assertIn("UNFINISHED_OWNED_CANDIDATE", started.stderr)
            self.assertIn("receipt_missing", started.stderr)
        finally:
            directory.cleanup()

    def test_start_blocks_a_same_owner_candidate_with_a_corrupt_receipt(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            second_dir = repo / ".trellis" / "tasks" / "01-02-corrupt-receipt"
            second_dir.mkdir(parents=True)
            second_task = json.loads(
                (repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json").read_text(
                    encoding="utf-8"
                )
            )
            second_task["id"] = "corrupt-receipt"
            second_task["name"] = "corrupt-receipt"
            second_task["branch"] = "refs/heads/corrupt-receipt"
            second_task["meta"]["delivery"]["candidate_id"] = "candidate-2"
            second_task["meta"]["delivery"]["branch_ref"] = "refs/heads/corrupt-receipt"
            (second_dir / "task.json").write_text(
                json.dumps(second_task), encoding="utf-8"
            )
            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-2.json"
            receipt_path.parent.mkdir(parents=True)
            receipt_path.write_text("{broken", encoding="utf-8")

            started = task_cli(repo, "start", ".trellis/tasks/01-01-candidate")

            self.assertNotEqual(started.returncode, 0)
            self.assertIn("UNFINISHED_OWNED_CANDIDATE", started.stderr)
            self.assertIn("receipt_corrupt", started.stderr)
        finally:
            directory.cleanup()

    def test_start_rejects_a_receipt_bound_to_another_owner(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "owner": "another-owner",
                    "task_path": ".trellis/tasks/01-01-candidate/task.json",
                    "phase": "registered",
                },
            )

            started = task_cli(repo, "start", ".trellis/tasks/01-01-candidate")

            self.assertEqual(started.returncode, 1)
            self.assertIn("RECEIPT_BINDING_CONFLICT", started.stderr)
            self.assertEqual(store.read("candidate-1")["generation"], 0)
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
            evidence_path = Path(directory.name) / "evidence.json"
            evidence_path.write_text(
                json.dumps(
                    {
                        "review": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:review-1",
                            "result": "approved",
                            "head_oid": tip,
                            "reviewer": "independent-reviewer",
                            "reviewer_role": "architect",
                            "review_id": "review-1",
                            "findings": [],
                        },
                        "checks": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "evidence_ref": "test:checks-1",
                            "result": "passed",
                            "conclusion": "success",
                            "head_oid": tip,
                            "base_oid": base_after_merge,
                            "repository": "rothnic/assura",
                            "run_id": "run-1",
                            "job_id": "process-contracts",
                        },
                        "postmerge": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "evidence_ref": "test:postmerge-1",
                            "result": "verified",
                            "head_oid": tip,
                            "base_oid": base_after_merge,
                            "merge_oid": base_after_merge,
                            "repository": "rothnic/assura",
                        },
                        "acceptance": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "owner",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:acceptance-1",
                            "authority_ref": "test:delivery-authority",
                            "approved_by": "release-authority",
                            "result": "accepted",
                            "head_oid": tip,
                            "acceptance_ref": "prd.md#acceptance",
                            "artifact_sha256": "a" * 64,
                        },
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

    def test_archive_retry_recovers_after_the_task_move(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            rejected = task_cli(
                repo,
                "delivery",
                "close",
                task_path,
                "--outcome",
                "rejected",
                "--reason",
                "not selected for this release",
            )
            self.assertEqual(rejected.returncode, 0, rejected.stderr)

            source = repo / task_path
            destination = repo / ".trellis" / "tasks" / "archive" / "2099-01" / source.name
            destination.parent.mkdir(parents=True)
            shutil.move(str(source), str(destination))

            retried = task_cli(repo, "archive", task_path, "--no-commit")

            self.assertEqual(retried.returncode, 0, retried.stderr)
            receipt = json.loads(
                (repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertEqual(receipt["outcome"], "rejected")
            self.assertEqual(receipt["closure"], "closed")
        finally:
            directory.cleanup()

    def test_terminal_outcome_cannot_be_replaced_by_another_outcome(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            rejected = task_cli(
                repo,
                "delivery",
                "close",
                task_path,
                "--outcome",
                "rejected",
                "--reason",
                "deferred by product decision",
            )
            self.assertEqual(rejected.returncode, 0, rejected.stderr)

            replaced = task_cli(
                repo,
                "delivery",
                "close",
                task_path,
                "--outcome",
                "delivered",
            )

            self.assertEqual(replaced.returncode, 1)
            self.assertIn("TERMINAL_OUTCOME_IMMUTABLE", replaced.stderr)
        finally:
            directory.cleanup()

    def test_terminal_evidence_must_bind_the_current_candidate_tip(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            evidence_path = Path(directory.name) / "evidence.json"
            evidence_path.write_text(
                json.dumps(
                    {
                        "review": {"result": "approved"},
                        "checks": {"result": "passed"},
                        "postmerge": {"result": "verified"},
                        "acceptance": {"result": "accepted"},
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
            self.assertEqual(recorded.returncode, 2)
            self.assertIn("schema_version", recorded.stderr)

            closed = task_cli(repo, "delivery", "close", task_path, "--outcome", "delivered")

            self.assertNotEqual(closed.returncode, 0)
            self.assertIn("UNSUPPORTED_DELIVERY_CLAIM", closed.stdout + closed.stderr)
            receipt = json.loads(
                (repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json").read_text(
                    encoding="utf-8"
                )
            )
            self.assertIsNone(receipt["outcome"])
            self.assertEqual(receipt["observed_tip"], tip)
        finally:
            directory.cleanup()

    def test_terminal_receipts_cannot_be_overwritten_or_reclosed_with_another_outcome(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            rejected = task_cli(
                repo,
                "delivery",
                "close",
                task_path,
                "--outcome",
                "rejected",
                "--reason",
                "not selected",
            )
            self.assertEqual(rejected.returncode, 0, rejected.stderr)
            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            before = receipt_path.read_bytes()

            repeated = task_cli(
                repo,
                "delivery",
                "close",
                task_path,
                "--outcome",
                "rejected",
                "--reason",
                "ignored on an idempotent retry",
            )
            self.assertEqual(repeated.returncode, 0, repeated.stderr)
            self.assertEqual(receipt_path.read_bytes(), before)

            changed = task_cli(
                repo,
                "delivery",
                "close",
                task_path,
                "--outcome",
                "cancelled",
                "--reason",
                "must not replace the disposition",
            )
            self.assertNotEqual(changed.returncode, 0)
            self.assertIn("TERMINAL_OUTCOME_IMMUTABLE", changed.stderr)
            self.assertEqual(receipt_path.read_bytes(), before)
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
            evidence_path = Path(directory.name) / "evidence.json"
            evidence_path.write_text(
                json.dumps(
                    {
                        "review": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:review-2",
                            "result": "approved",
                            "head_oid": tip,
                            "reviewer": "independent-reviewer",
                            "reviewer_role": "architect",
                            "review_id": "review-2",
                            "findings": [],
                        },
                        "checks": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "evidence_ref": "test:checks-2",
                            "result": "passed",
                            "conclusion": "success",
                            "head_oid": tip,
                            "base_oid": base_after_merge,
                            "repository": "rothnic/assura",
                            "run_id": "run-2",
                            "job_id": "process-contracts",
                        },
                        "postmerge": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "local",
                            "evidence_ref": "test:postmerge-2",
                            "result": "verified",
                            "head_oid": tip,
                            "base_oid": base_after_merge,
                            "merge_oid": base_after_merge,
                            "repository": "rothnic/assura",
                        },
                        "acceptance": {
                            "schema_version": "assura.delivery-evidence.v1",
                            "source": "owner",
                            "repository": "rothnic/assura",
                            "evidence_ref": "test:acceptance-2",
                            "authority_ref": "test:delivery-authority",
                            "approved_by": "release-authority",
                            "result": "accepted",
                            "head_oid": tip,
                            "acceptance_ref": "prd.md#acceptance",
                            "artifact_sha256": "b" * 64,
                        },
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
