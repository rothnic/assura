from __future__ import annotations

import json
import os
import shutil
from pathlib import Path

from agent_delivery_contract_support import (
    fixture_repo,
    git,
    local_integration_evidence,
    task_cli,
    write_fake_github,
)
from common.delivery_store import DeliveryStore

class LifecycleCloseCases:
    def test_record_and_close_reject_invalid_merge_snapshots_without_writes(self) -> None:
        cases = (
            ("malformed OPEN timestamp", "record", "OPEN", {"value": "not-a-time"}, None, None, None, "PR identity mismatch"),
            ("contradictory CLOSED timestamp", "close", "CLOSED", "2026-10-17T12:34:56Z", None, None, None, "PR identity mismatch"),
            ("MERGED missing timestamp", "record", "MERGED", None, "tip", None, None, "PR identity mismatch"),
            ("MERGED missing merge OID", "close", "MERGED", "2026-10-17T12:34:56Z", None, None, None, "PR identity mismatch"),
            ("unfinished CheckRun with SUCCESS", "record", "OPEN", None, None, "IN_PROGRESS", "SUCCESS", "PR identity mismatch"),
            ("pending CheckRun remains unresolved", "record", "OPEN", None, None, "IN_PROGRESS", None, "not verified by the current GitHub pull request"),
        )
        for label, command, state, merged_at, merge_oid, check_status, conclusion, expected_error in cases:
            with self.subTest(case=label):
                repo, base_oid, tip, directory = fixture_repo()
                try:
                    task_path = ".trellis/tasks/01-01-candidate"
                    # The task fixture is created after the initial commit and
                    # Trellis task data is intentionally ignored by the repo.
                    # Track it in this disposable repository so CLI state checks
                    # reach candidate-provider validation instead of rejecting
                    # the fixture as an untracked task directory.
                    git(repo, "add", "-f", f"{task_path}/task.json")
                    git(repo, "commit", "-m", "add delivery task fixture")
                    tip = git(repo, "rev-parse", "HEAD")
                    self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
                    self.assertEqual(task_cli(repo, "finish").returncode, 0)
                    task_json = repo / task_path / "task.json"
                    receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
                    store = DeliveryStore(receipt_path.parent)
                    fake_gh = repo / ".test-bin" / ("gh.py" if os.name == "nt" else "gh")
                    pull_request = {
                        "number": 1,
                        "state": state,
                        "title": "Candidate",
                        "url": "https://github.com/rothnic/assura/pull/1",
                        "headRefName": "candidate",
                        "headRefOid": tip,
                        "baseRefName": "master",
                        "baseRefOid": base_oid,
                        "mergeCommit": {"oid": tip} if merge_oid == "tip" else None,
                        "mergedAt": merged_at,
                    }
                    check_evidence = check_status is not None
                    if check_status is not None:
                        pull_request.update({
                            "mergeStateStatus": "CLEAN",
                            "statusCheckRollup": [{
                                "__typename": "CheckRun", "databaseId": "202",
                                "name": "build", "status": check_status,
                                "conclusion": conclusion,
                                "detailsUrl": "https://github.com/rothnic/assura/actions/runs/101/job/202",
                            }],
                        })
                    write_fake_github(fake_gh, [pull_request])
                    evidence_path = Path(directory.name) / "record-evidence.json"
                    evidence = {"observed_tip": tip}
                    if check_evidence:
                        evidence = {
                            "checks": {
                                "schema_version": "assura.delivery-evidence.v1",
                                "source": "github",
                                "repository": "rothnic/assura",
                                "evidence_ref": "https://github.com/rothnic/assura/actions/runs/101",
                                "result": "passed",
                                "conclusion": "success",
                                "head_oid": tip,
                                "base_oid": base_oid,
                                "pr_number": 1,
                                "run_id": "101",
                                "job_id": "202",
                                "check_name": "build",
                            }
                        }
                    evidence_path.write_text(json.dumps(evidence), encoding="utf-8")
                    before_task = task_json.read_bytes()
                    before_receipt = receipt_path.read_bytes()
                    before_generation = store.read("candidate-1")["generation"]
                    if command == "record":
                        result = task_cli(
                            repo, "delivery", "record", task_path,
                            "--evidence-file", str(evidence_path),
                            "--expected-generation", str(before_generation),
                        )
                    else:
                        result = task_cli(
                            repo, "delivery", "close", task_path,
                            "--outcome", "delivered",
                        )
                    self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
                    self.assertIn(expected_error, result.stdout + result.stderr)
                    self.assertEqual(task_json.read_bytes(), before_task)
                    self.assertEqual(receipt_path.read_bytes(), before_receipt)
                    self.assertEqual(store.read("candidate-1")["generation"], before_generation)
                finally:
                    directory.cleanup()

    def test_first_delivered_close_cannot_reuse_historical_active_base(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merge_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "push", "origin", "master:refs/heads/master")

            evidence_path = Path(directory.name) / "first-closure-evidence.json"
            evidence_path.write_text(
                json.dumps(local_integration_evidence(tip, merge_oid, "first-closure")),
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

            (repo / "unrelated.txt").write_text("unrelated base advance\n", encoding="utf-8")
            git(repo, "add", "unrelated.txt")
            git(repo, "commit", "-m", "unrelated base advance")
            git(repo, "push", "origin", "master:refs/heads/master")

            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            before_receipt = receipt_path.read_bytes()
            closed = task_cli(repo, "delivery", "close", task_path, "--outcome", "delivered")

            self.assertNotEqual(closed.returncode, 0, closed.stdout + closed.stderr)
            self.assertIn("UNSUPPORTED_DELIVERY_CLAIM", closed.stdout + closed.stderr)
            self.assertEqual(receipt_path.read_bytes(), before_receipt)
            self.assertIsNone(json.loads(before_receipt)["outcome"])
        finally:
            directory.cleanup()

    def test_close_rejects_candidate_unavailable_worktree_without_mutation(self) -> None:
        repo, _, tip, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            merged_oid = git(repo, "rev-parse", "HEAD")
            git(repo, "push", "origin", "master:refs/heads/master")

            missing_worktree = Path(directory.name) / "candidate-missing"
            git(repo, "worktree", "add", str(missing_worktree), "candidate")
            shutil.rmtree(missing_worktree)

            evidence_path = Path(directory.name) / "evidence.json"
            evidence_path.write_text(
                json.dumps(local_integration_evidence(tip, merged_oid, "owned-missing")),
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

            task_json = repo / task_path / "task.json"
            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            before_task = task_json.read_bytes()
            before_receipt = receipt_path.read_bytes()
            before_refs = git(repo, "show-ref")
            before_worktrees = git(repo, "worktree", "list", "--porcelain")

            closed = task_cli(repo, "delivery", "close", task_path, "--outcome", "delivered")

            self.assertEqual(closed.returncode, 2, closed.stdout + closed.stderr)
            self.assertIn("WORKTREE_COVERAGE_UNAVAILABLE", closed.stdout + closed.stderr)
            self.assertEqual(task_json.read_bytes(), before_task)
            self.assertEqual(receipt_path.read_bytes(), before_receipt)
            self.assertEqual(git(repo, "show-ref"), before_refs)
            self.assertEqual(
                git(repo, "worktree", "list", "--porcelain"), before_worktrees
            )
        finally:
            directory.cleanup()
