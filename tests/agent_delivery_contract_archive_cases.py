from __future__ import annotations

import json
import shutil
from pathlib import Path

from agent_delivery_contract_support import fixture_repo, git, task_cli
from common import delivery_cli
from common.delivery import load_delivery_intent
from common.delivery_cli import ensure_receipt
from common.delivery_store import DeliveryStore

class LifecycleArchiveCases:
    def test_delivery_inspect_keeps_missing_and_valid_receipt_behavior(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"

            missing = task_cli(
                repo, "delivery", "inspect", task_path, "--format", "json"
            )
            self.assertEqual(missing.returncode, 0, missing.stderr)
            self.assertEqual(json.loads(missing.stdout)["candidate_id"], "candidate-1")
            self.assertFalse(receipt_path.exists())

            started = task_cli(repo, "start", task_path)
            self.assertEqual(started.returncode, 0, started.stderr)
            before_task = task_json.read_bytes()
            before_receipt = receipt_path.read_bytes()
            store = DeliveryStore(receipt_path.parent)
            before_generation = store.read("candidate-1")["generation"]
            valid = task_cli(
                repo, "delivery", "inspect", task_path, "--format", "json"
            )

            self.assertEqual(valid.returncode, 0, valid.stderr)
            self.assertEqual(json.loads(valid.stdout)["candidate_id"], "candidate-1")
            self.assertEqual(store.read("candidate-1")["generation"], before_generation)
            self.assertEqual(receipt_path.read_bytes(), before_receipt)
            self.assertEqual(task_json.read_bytes(), before_task)
        finally:
            directory.cleanup()

    def test_archive_missing_receipt_keeps_explicit_error(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            before_task = task_json.read_bytes()

            archived = task_cli(repo, "archive", task_path, "--no-commit")

            self.assertEqual(archived.returncode, 1, archived.stdout + archived.stderr)
            self.assertIn("RECEIPT_MISSING", archived.stdout + archived.stderr)
            self.assertFalse(receipt_path.exists())
            self.assertEqual(task_json.read_bytes(), before_task)
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
        repo, base_before_merge, tip, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
            self.assertEqual(task_cli(repo, "finish").returncode, 0)
            git(repo, "checkout", "master")
            git(repo, "merge", "--ff-only", "candidate")
            base_after_merge = git(repo, "rev-parse", "HEAD")
            git(repo, "push", "origin", "master:refs/heads/master")
            unrelated_worktree = Path(directory.name) / "unrelated-missing"
            git(repo, "worktree", "add", "--detach", str(unrelated_worktree), base_before_merge)
            shutil.rmtree(unrelated_worktree)
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
            audit = task_cli(repo, "delivery", "audit", "--format", "json", "--refresh")
            self.assertEqual(audit.returncode, 0, audit.stderr)
            audit_report = json.loads(audit.stdout)
            self.assertFalse(audit_report["coverage"]["complete"])
            self.assertTrue(
                any(
                    issue["code"] == "WORKTREE_COVERAGE_UNAVAILABLE"
                    and issue["evidence_ref"] == str(unrelated_worktree.resolve())
                    for issue in audit_report["issues"]
                )
            )
            strict_audit = task_cli(
                repo,
                "delivery",
                "audit",
                "--format",
                "json",
                "--refresh",
                "--strict",
            )
            self.assertEqual(strict_audit.returncode, 1, strict_audit.stderr)
            self.assertFalse(json.loads(strict_audit.stdout)["coverage"]["complete"])
            archived = task_cli(repo, "archive", task_path, "--no-commit")

            self.assertEqual(archived.returncode, 0, archived.stderr)
            self.assertFalse((repo / task_path).exists())
            receipts = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            receipt = json.loads(receipts.read_text(encoding="utf-8"))
            self.assertEqual(receipt["outcome"], "delivered")
            self.assertEqual(receipt["closure"], "closed")
        finally:
            directory.cleanup()

    def test_archive_closure_helper_and_retry_reject_mismatched_authority(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        for consumer in ("closure-helper", "archive-recovery"):
            with self.subTest(consumer=consumer):
                repo, _, _, directory = fixture_repo()
                try:
                    self.assertEqual(task_cli(repo, "start", task_path).returncode, 0)
                    self.assertEqual(task_cli(repo, "finish").returncode, 0)
                    git(repo, "checkout", "master")
                    closed = task_cli(
                        repo,
                        "delivery",
                        "close",
                        task_path,
                        "--outcome",
                        "rejected",
                        "--reason",
                        "not selected",
                    )
                    self.assertEqual(closed.returncode, 0, closed.stderr)

                    source = repo / task_path
                    intent = load_delivery_intent(source / "task.json")
                    destination = (
                        repo
                        / ".trellis"
                        / "tasks"
                        / "archive"
                        / "2099-01"
                        / source.name
                    )
                    destination.parent.mkdir(parents=True)
                    shutil.move(str(source), str(destination))
                    task_json = destination / "task.json"
                    receipt_path = (
                        repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
                    )
                    store = DeliveryStore(receipt_path.parent)
                    receipt = store.read("candidate-1")
                    self.assertEqual(receipt["outcome"], "rejected")
                    self.assertEqual(receipt["closure"], "verified")
                    store.update(
                        "candidate-1",
                        int(receipt["generation"]),
                        {"authority_ref": "test:other-authority"},
                    )
                    task_bytes = task_json.read_bytes()
                    receipt_bytes = receipt_path.read_bytes()
                    generation = store.read("candidate-1")["generation"]
                    refs_before = git(repo, "show-ref", "--heads")
                    worktrees_before = git(repo, "worktree", "list", "--porcelain")

                    if consumer == "closure-helper":
                        allowed, message = delivery_cli.record_archive_closure(
                            repo, intent, destination
                        )
                        self.assertFalse(allowed, message)
                        output = message
                    else:
                        retried = task_cli(
                            repo, "archive", task_path, "--no-commit"
                        )
                        self.assertEqual(
                            retried.returncode,
                            1,
                            retried.stdout + retried.stderr,
                        )
                        output = retried.stdout + retried.stderr

                    self.assertIn("RECEIPT_BINDING_CONFLICT", output)
                    self.assertEqual(task_json.read_bytes(), task_bytes)
                    self.assertEqual(receipt_path.read_bytes(), receipt_bytes)
                    current = store.read("candidate-1")
                    self.assertEqual(current["generation"], generation)
                    self.assertEqual(current["outcome"], "rejected")
                    self.assertEqual(current["closure"], "verified")
                    self.assertTrue(task_json.is_file())
                    self.assertEqual(git(repo, "show-ref", "--heads"), refs_before)
                    self.assertEqual(
                        git(repo, "worktree", "list", "--porcelain"),
                        worktrees_before,
                    )
                finally:
                    directory.cleanup()

    def test_archive_closure_reloads_archived_intent_before_receipt_update(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        for field, value in (
            ("candidate_id", "drifted-candidate"),
            ("repository", "elsewhere/assura"),
            ("branch_ref", "refs/heads/other-candidate"),
            ("base_ref", "refs/heads/missing-base"),
        ):
            with self.subTest(field=field):
                repo, _, _, directory = fixture_repo()
                try:
                    task_json = repo / task_path / "task.json"
                    task_data = json.loads(task_json.read_text(encoding="utf-8"))
                    intent = load_delivery_intent(task_json)
                    ensure_receipt(repo, intent, task_data)
                    receipt_path = (
                        repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
                    )
                    before_receipt = receipt_path.read_bytes()

                    archive_path = (
                        repo / ".trellis" / "tasks" / f"archive-copy-{field}" / "candidate"
                    )
                    archive_path.mkdir(parents=True)
                    archived = json.loads(json.dumps(task_data))
                    archived["meta"]["delivery"][field] = value
                    archived_task_json = archive_path / "task.json"
                    archived_task_json.write_text(
                        json.dumps(archived), encoding="utf-8"
                    )
                    before_archived = archived_task_json.read_bytes()

                    allowed, message = delivery_cli.record_archive_closure(
                        repo, intent, archive_path
                    )

                    self.assertFalse(allowed, message)
                    self.assertIn("ARCHIVE_INTENT_MISMATCH", message)
                    self.assertEqual(receipt_path.read_bytes(), before_receipt)
                    self.assertEqual(archived_task_json.read_bytes(), before_archived)
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
            git(repo, "push", "origin", "master:refs/heads/master")
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
