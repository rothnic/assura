from __future__ import annotations

import json
from pathlib import Path
from unittest import mock

from agent_delivery_contract_support import (
    FakeGithub,
    fixture_repo,
    git,
    task_cli,
)
from common import delivery_cli
from common.delivery import DeliveryValidationError, classify_candidate
from common.delivery_store import DeliveryStore

class LifecycleInspectCases:
    def test_inspect_and_archive_reject_mismatched_receipt_bindings_read_only(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        mismatches = (
            ("repository", "elsewhere/assura"),
            ("owner", "other-owner"),
            ("task_path", ".trellis/tasks/01-02-other/task.json"),
            ("attached_branch_ref", "refs/heads/other-candidate"),
            ("attached_branch_ref", None),
            ("authority_ref", "test:other-authority"),
            ("authority_ref", None),
        )
        for field, replacement in mismatches:
            with self.subTest(field=field):
                repo, _, _, directory = fixture_repo()
                try:
                    started = task_cli(repo, "start", task_path)
                    self.assertEqual(started.returncode, 0, started.stderr)
                    paused = task_cli(repo, "finish")
                    self.assertEqual(paused.returncode, 0, paused.stderr)

                    task_json = repo / task_path / "task.json"
                    receipt_path = (
                        repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
                    )
                    store = DeliveryStore(receipt_path.parent)
                    receipt = store.read("candidate-1")
                    store.update(
                        "candidate-1",
                        int(receipt["generation"]),
                        {field: replacement},
                    )
                    before_receipt = receipt_path.read_bytes()
                    before_generation = store.read("candidate-1")["generation"]
                    before_task = task_json.read_bytes()
                    before_refs = git(repo, "show-ref", "--heads")
                    before_worktrees = git(repo, "worktree", "list", "--porcelain")
                    evidence_path = Path(directory.name) / "evidence.json"
                    evidence_path.write_text(
                        json.dumps({"observed_tip": git(repo, "rev-parse", "HEAD")}),
                        encoding="utf-8",
                    )

                    commands = (
                        (
                            "inspect",
                            lambda: task_cli(
                                repo,
                                "delivery",
                                "inspect",
                                task_path,
                                "--format",
                                "json",
                            ),
                            2,
                        ),
                        (
                            "record",
                            lambda: task_cli(
                                repo,
                                "delivery",
                                "record",
                                task_path,
                                "--evidence-file",
                                str(evidence_path),
                                "--expected-generation",
                                str(before_generation),
                            ),
                            2,
                        ),
                        (
                            "close",
                            lambda: task_cli(
                                repo,
                                "delivery",
                                "close",
                                task_path,
                                "--outcome",
                                "delivered",
                            ),
                            2,
                        ),
                        (
                            "archive",
                            lambda: task_cli(repo, "archive", task_path, "--no-commit"),
                            1,
                        ),
                    )
                    for name, run, expected_code in commands:
                        result = run()
                        with self.subTest(boundary=name):
                            self.assertEqual(
                                result.returncode,
                                expected_code,
                                result.stdout + result.stderr,
                            )
                            self.assertIn(
                                "RECEIPT_BINDING_CONFLICT",
                                result.stdout + result.stderr,
                            )
                            self.assertEqual(receipt_path.read_bytes(), before_receipt)
                            self.assertEqual(
                                store.read("candidate-1")["generation"],
                                before_generation,
                            )
                            self.assertEqual(task_json.read_bytes(), before_task)
                            self.assertEqual(git(repo, "show-ref", "--heads"), before_refs)
                            self.assertEqual(
                                git(repo, "worktree", "list", "--porcelain"),
                                before_worktrees,
                            )
                finally:
                    directory.cleanup()

    def test_remote_branch_binding_requires_same_ref_or_proven_oid_alias(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        repo, base_oid, tip, directory = fixture_repo()
        try:
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["branch"] = "refs/remotes/origin/feature"
            task["meta"]["delivery"]["branch_ref"] = "refs/remotes/origin/feature"
            task_json.write_text(json.dumps(task), encoding="utf-8")
            git(repo, "update-ref", "refs/remotes/origin/feature", tip)
            git(repo, "update-ref", "refs/remotes/upstream/feature", tip)
            git(repo, "checkout", "--detach", "refs/remotes/origin/feature")

            started = task_cli(repo, "start", task_path)
            self.assertEqual(started.returncode, 0, started.stderr)
            paused = task_cli(repo, "finish")
            self.assertEqual(paused.returncode, 0, paused.stderr)

            receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
            store = DeliveryStore(receipt_path.parent)
            receipt = store.read("candidate-1")
            self.assertEqual(
                receipt["attached_branch_ref"], "refs/remotes/origin/feature"
            )

            with mock.patch.object(
                delivery_cli, "classify_candidate", wraps=classify_candidate
            ) as classifier:
                delivery_cli._status_for_task(
                    repo, task_json, github=FakeGithub([])
                )
                classifier.assert_called_once()

            receipt = store.read("candidate-1")
            store.update(
                "candidate-1",
                int(receipt["generation"]),
                {"attached_branch_ref": "refs/remotes/upstream/feature"},
            )
            self.assertEqual(
                git(repo, "rev-parse", "refs/remotes/origin/feature"),
                git(repo, "rev-parse", "refs/remotes/upstream/feature"),
            )
            with mock.patch.object(
                delivery_cli, "classify_candidate", wraps=classify_candidate
            ) as classifier:
                delivery_cli._status_for_task(
                    repo, task_json, github=FakeGithub([])
                )
                classifier.assert_called_once()

            git(repo, "checkout", "master")
            with mock.patch.object(
                delivery_cli, "classify_candidate", wraps=classify_candidate
            ) as classifier:
                delivery_cli._status_for_task(
                    repo,
                    task_json,
                    github=FakeGithub([]),
                    allow_base_checkout=True,
                )
                classifier.assert_called_once()

            git(repo, "checkout", "--detach", "refs/remotes/origin/feature")
            git(repo, "update-ref", "refs/remotes/upstream/feature", base_oid)
            receipt = store.read("candidate-1")
            store.update(
                "candidate-1",
                int(receipt["generation"]),
                {"attached_branch_ref": "refs/remotes/upstream/feature"},
            )
            self.assertNotEqual(
                git(repo, "rev-parse", "refs/remotes/origin/feature"),
                git(repo, "rev-parse", "refs/remotes/upstream/feature"),
            )

            with self.subTest(boundary="before-classification"):
                with mock.patch.object(
                    delivery_cli, "classify_candidate", wraps=classify_candidate
                ) as classifier:
                    with self.assertRaisesRegex(
                        DeliveryValidationError, "RECEIPT_BINDING_CONFLICT"
                    ):
                        delivery_cli._status_for_task(
                            repo, task_json, github=FakeGithub([])
                        )
                    classifier.assert_not_called()

            before_task = task_json.read_bytes()
            before_receipt = receipt_path.read_bytes()
            before_generation = store.read("candidate-1")["generation"]
            refs_before = git(repo, "show-ref")
            worktrees_before = git(repo, "worktree", "list", "--porcelain")
            archive_root = repo / ".trellis" / "tasks" / "archive"
            archive_paths_before = (
                sorted(path.relative_to(repo).as_posix() for path in archive_root.rglob("task.json"))
                if archive_root.is_dir()
                else []
            )
            evidence_path = Path(directory.name) / "remote-ref-evidence.json"
            evidence_path.write_text(
                json.dumps({"observed_tip": tip}), encoding="utf-8"
            )
            commands = (
                (
                    "inspect",
                    ("delivery", "inspect", task_path, "--format", "json"),
                    2,
                ),
                (
                    "record",
                    (
                        "delivery",
                        "record",
                        task_path,
                        "--evidence-file",
                        str(evidence_path),
                        "--expected-generation",
                        str(before_generation),
                    ),
                    2,
                ),
                (
                    "close",
                    ("delivery", "close", task_path, "--outcome", "delivered"),
                    2,
                ),
                ("archive", ("archive", task_path, "--no-commit"), 1),
            )
            for name, arguments, expected_code in commands:
                with self.subTest(boundary=name):
                    result = task_cli(repo, *arguments)
                    self.assertEqual(
                        result.returncode,
                        expected_code,
                        result.stdout + result.stderr,
                    )
                    self.assertIn(
                        "RECEIPT_BINDING_CONFLICT", result.stdout + result.stderr
                    )
                    self.assertEqual(task_json.read_bytes(), before_task)
                    self.assertEqual(receipt_path.read_bytes(), before_receipt)
                    receipt = store.read("candidate-1")
                    self.assertEqual(receipt["generation"], before_generation)
                    self.assertIsNone(receipt["outcome"])
                    self.assertEqual(receipt["closure"], "open")
                    self.assertEqual(git(repo, "show-ref"), refs_before)
                    self.assertEqual(
                        git(repo, "worktree", "list", "--porcelain"),
                        worktrees_before,
                    )
                    archive_paths_after = (
                        sorted(
                            path.relative_to(repo).as_posix()
                            for path in archive_root.rglob("task.json")
                        )
                        if archive_root.is_dir()
                        else []
                    )
                    self.assertEqual(archive_paths_after, archive_paths_before)
                    self.assertTrue(task_json.is_file())
        finally:
            directory.cleanup()
