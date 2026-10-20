from __future__ import annotations

import json
from pathlib import Path
from unittest import mock

from agent_delivery_contract_support import fixture_repo, git, task_cli
from common import delivery_cli, delivery_receipts
from common.delivery import DeliveryValidationError
from common.delivery_store import DeliveryStore

class LifecycleRegistrationCases:
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

    def test_register_fills_authority_and_recovers_legacy_receipt_idempotently(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"].pop("authority_ref")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "task_path": f"{task_path}/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                },
            )

            registered = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
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
            receipt = store.read("candidate-1")
            self.assertEqual(receipt["attached_branch_ref"], "refs/heads/candidate")
            self.assertEqual(receipt["authority_ref"], "test:delivery-authority")
            self.assertEqual(receipt["generation"], 1)

            repeated = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
                "--authority-ref",
                "test:delivery-authority",
            )

            self.assertEqual(repeated.returncode, 0, repeated.stderr)
            self.assertEqual(store.read("candidate-1")["generation"], 1)
        finally:
            directory.cleanup()

    def test_register_rejects_conflicting_receipt_authority_without_mutation(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_json = repo / ".trellis/tasks/01-01-candidate/task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"].pop("authority_ref")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            before_task = task_json.read_bytes()
            store = DeliveryStore(repo / ".git/assura/delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "task_path": ".trellis/tasks/01-01-candidate/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                    "authority_ref": "test:older-authority",
                },
            )
            before_receipt = store.read("candidate-1")

            with self.assertRaisesRegex(
                DeliveryValidationError, "receipt authority_ref does not match"
            ):
                delivery_cli.register_task(
                    repo,
                    task_json,
                    "candidate-1",
                    "tester",
                    authority_ref="test:delivery-authority",
                )

            self.assertEqual(task_json.read_bytes(), before_task)
            self.assertEqual(store.read("candidate-1"), before_receipt)
        finally:
            directory.cleanup()

    def test_register_retries_after_task_write_failure_without_advancing_receipt(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_json = repo / ".trellis/tasks/01-01-candidate/task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"].pop("authority_ref")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            before_task = task_json.read_bytes()
            store = DeliveryStore(repo / ".git/assura/delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "task_path": ".trellis/tasks/01-01-candidate/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                },
            )

            with mock.patch.object(delivery_cli, "write_json", return_value=False):
                with self.assertRaisesRegex(
                    DeliveryValidationError, "cannot write delivery intent"
                ):
                    delivery_cli.register_task(
                        repo,
                        task_json,
                        "candidate-1",
                        "tester",
                        authority_ref="test:delivery-authority",
                    )

            self.assertEqual(task_json.read_bytes(), before_task)
            receipt = store.read("candidate-1")
            self.assertEqual(receipt["authority_ref"], "test:delivery-authority")
            self.assertEqual(receipt["attached_branch_ref"], "refs/heads/candidate")
            self.assertEqual(receipt["generation"], 1)

            delivery_cli.register_task(
                repo,
                task_json,
                "candidate-1",
                "tester",
                authority_ref="test:delivery-authority",
            )

            updated = json.loads(task_json.read_text(encoding="utf-8"))
            self.assertEqual(
                updated["meta"]["delivery"]["authority_ref"],
                "test:delivery-authority",
            )
            self.assertEqual(store.read("candidate-1")["generation"], 1)
        finally:
            directory.cleanup()

    def test_fresh_registration_writes_task_before_receipt_and_recovers_after_failure(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_json = repo / ".trellis/tasks/01-01-candidate/task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"].pop("delivery")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            candidate_id = "fresh-write-failure"
            receipt_path = (
                repo / ".git" / "assura" / "delivery-v1" / f"{candidate_id}.json"
            )
            original_write = delivery_cli.write_json

            def write_then_report_failure(path: Path, value: dict) -> bool:
                succeeded = original_write(path, value)
                if path.resolve() == task_json.resolve():
                    return False
                return succeeded

            with mock.patch.object(
                delivery_cli, "write_json", side_effect=write_then_report_failure
            ):
                with self.assertRaisesRegex(
                    DeliveryValidationError, "cannot write delivery intent"
                ):
                    delivery_cli.register_task(
                        repo, task_json, candidate_id, "tester"
                    )

            self.assertFalse(
                receipt_path.exists(),
                "a failed fresh task write must not leave an orphan receipt",
            )
            persisted = json.loads(task_json.read_text(encoding="utf-8"))
            self.assertEqual(
                persisted["meta"]["delivery"]["candidate_id"], candidate_id
            )

            recovered = delivery_cli.register_task(
                repo, task_json, candidate_id, "tester"
            )
            self.assertEqual(recovered.candidate_id, candidate_id)
            self.assertTrue(receipt_path.is_file())
        finally:
            directory.cleanup()

    def test_partial_registration_binding_blocks_retry_record_close_and_archive(self) -> None:
        task_path = ".trellis/tasks/01-01-candidate"
        commands = ("register-retry", "record", "close", "archive")
        for command_name in commands:
            with self.subTest(command=command_name):
                repo, _, tip, directory = fixture_repo()
                try:
                    task_json = repo / task_path / "task.json"
                    task = json.loads(task_json.read_text(encoding="utf-8"))
                    task["meta"]["delivery"].pop("authority_ref")
                    task_json.write_text(json.dumps(task), encoding="utf-8")
                    before_task = task_json.read_bytes()
                    receipt_path = repo / ".git" / "assura" / "delivery-v1" / "candidate-1.json"
                    store = DeliveryStore(receipt_path.parent)
                    store.create(
                        "candidate-1",
                        {
                            "repository": "rothnic/assura",
                            "owner": "tester",
                            "task_path": f"{task_path}/task.json",
                            "phase": "registered",
                            "session_state": "attached",
                            "outcome": None,
                            "closure": "open",
                            "evidence": {},
                            "attached_branch_ref": "refs/heads/candidate",
                        },
                    )

                    with mock.patch.object(delivery_cli, "write_json", return_value=False):
                        with self.assertRaisesRegex(
                            DeliveryValidationError, "cannot write delivery intent"
                        ):
                            delivery_cli.register_task(
                                repo,
                                task_json,
                                "candidate-1",
                                "tester",
                                authority_ref="test:delivery-authority",
                            )

                    partial_receipt = store.read("candidate-1")
                    self.assertEqual(
                        partial_receipt["authority_ref"], "test:delivery-authority"
                    )
                    self.assertEqual(partial_receipt["generation"], 1)
                    before_receipt = receipt_path.read_bytes()
                    before_generation = partial_receipt["generation"]
                    before_refs = git(repo, "show-ref", "--heads")
                    before_worktrees = git(repo, "worktree", "list", "--porcelain")
                    evidence_path = Path(directory.name) / "record-evidence.json"
                    evidence_path.write_text(
                        json.dumps({"observed_tip": tip}), encoding="utf-8"
                    )

                    if command_name == "register-retry":
                        result = task_cli(
                            repo,
                            "delivery",
                            "register",
                            task_path,
                            "--candidate",
                            "candidate-1",
                            "--owner",
                            "tester",
                        )
                        expected_code = 2
                    elif command_name == "record":
                        result = task_cli(
                            repo,
                            "delivery",
                            "record",
                            task_path,
                            "--evidence-file",
                            str(evidence_path),
                            "--expected-generation",
                            str(before_generation),
                        )
                        expected_code = 2
                    elif command_name == "close":
                        result = task_cli(
                            repo,
                            "delivery",
                            "close",
                            task_path,
                            "--outcome",
                            "delivered",
                        )
                        expected_code = 2
                    else:
                        result = task_cli(
                            repo, "archive", task_path, "--no-commit"
                        )
                        expected_code = 1

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
                    self.assertTrue(task_json.is_file())
                    self.assertEqual(git(repo, "show-ref", "--heads"), before_refs)
                    self.assertEqual(
                        git(repo, "worktree", "list", "--porcelain"),
                        before_worktrees,
                    )
                finally:
                    directory.cleanup()

    def test_register_rejects_receipt_created_during_recovery_before_authority_write(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"].pop("authority_ref")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            before_task = task_json.read_bytes()
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            original_read = delivery_receipts._read_receipt
            reads = 0

            def read_after_competing_create(
                receipt_store: DeliveryStore, candidate_id: str
            ) -> dict[str, object] | None:
                nonlocal reads
                reads += 1
                if reads == 2:
                    receipt_store.create(
                        candidate_id,
                        {
                            "repository": "rothnic/assura",
                            "task_path": f"{task_path}/task.json",
                            "owner": "tester",
                            "phase": "registered",
                            "session_state": "attached",
                            "outcome": None,
                            "closure": "open",
                            "evidence": {},
                            "attached_branch_ref": "refs/heads/other",
                            "authority_ref": "test:delivery-authority",
                        },
                    )
                return original_read(receipt_store, candidate_id)

            with mock.patch.object(
                delivery_cli, "_read_receipt", side_effect=read_after_competing_create
            ), mock.patch.object(
                delivery_receipts, "_read_receipt", side_effect=read_after_competing_create
            ):
                with self.assertRaisesRegex(
                    DeliveryValidationError, "receipt branch does not match"
                ):
                    delivery_cli.register_task(
                        repo,
                        task_json,
                        "candidate-1",
                        "tester",
                        authority_ref="test:delivery-authority",
                    )

            self.assertEqual(task_json.read_bytes(), before_task)
            receipt = store.read("candidate-1")
            self.assertEqual(receipt["attached_branch_ref"], "refs/heads/other")
            self.assertEqual(receipt["generation"], 0)
        finally:
            directory.cleanup()

    def test_register_new_intent_rejects_concurrent_receipt_before_task_write(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"].pop("delivery")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            before_task = task_json.read_bytes()
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            original_read = delivery_receipts._read_receipt
            reads = 0

            def read_after_competing_create(
                receipt_store: DeliveryStore, candidate_id: str
            ) -> dict[str, object] | None:
                nonlocal reads
                reads += 1
                if reads == 2:
                    receipt_store.create(
                        candidate_id,
                        {
                            "repository": "rothnic/assura",
                            "task_path": f"{task_path}/task.json",
                            "owner": "tester",
                            "phase": "registered",
                            "session_state": "attached",
                            "outcome": None,
                            "closure": "open",
                            "evidence": {},
                            "attached_branch_ref": "refs/heads/other",
                            "authority_ref": "test:delivery-authority",
                        },
                    )
                return original_read(receipt_store, candidate_id)

            with mock.patch.object(
                delivery_cli, "_read_receipt", side_effect=read_after_competing_create
            ), mock.patch.object(
                delivery_receipts, "_read_receipt", side_effect=read_after_competing_create
            ):
                with self.assertRaisesRegex(
                    DeliveryValidationError, "receipt branch does not match"
                ):
                    delivery_cli.register_task(
                        repo,
                        task_json,
                        "candidate-1",
                        "tester",
                        authority_ref="test:delivery-authority",
                    )

            self.assertEqual(task_json.read_bytes(), before_task)
            receipt = store.read("candidate-1")
            self.assertEqual(receipt["attached_branch_ref"], "refs/heads/other")
            self.assertEqual(receipt["generation"], 0)
        finally:
            directory.cleanup()

    def test_register_repairs_legacy_receipt_from_declared_candidate_branch(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            before_task = task_json.read_bytes()
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "task_path": f"{task_path}/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                },
            )

            repaired = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
            )

            self.assertEqual(repaired.returncode, 0, repaired.stderr)
            receipt = store.read("candidate-1")
            self.assertEqual(receipt["attached_branch_ref"], "refs/heads/candidate")
            self.assertEqual(receipt["generation"], 1)
            self.assertEqual(task_json.read_bytes(), before_task)

            repeated = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
            )

            self.assertEqual(repeated.returncode, 0, repeated.stderr)
            self.assertEqual(store.read("candidate-1")["generation"], 1)
            self.assertEqual(task_json.read_bytes(), before_task)
        finally:
            directory.cleanup()

    def test_register_rejects_legacy_receipt_from_base_checkout_without_mutation(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            before_task = task_json.read_bytes()
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "task_path": f"{task_path}/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                },
            )
            git(repo, "checkout", "master")

            rejected = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
            )

            self.assertEqual(rejected.returncode, 2)
            self.assertIn("does not match declared candidate branch", rejected.stderr)
            self.assertEqual(store.read("candidate-1")["generation"], 0)
            self.assertNotIn("attached_branch_ref", store.read("candidate-1"))
            self.assertEqual(task_json.read_bytes(), before_task)
        finally:
            directory.cleanup()

    def test_register_rejects_detached_legacy_receipt_without_mutation_when_filling_authority(
        self,
    ) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["meta"]["delivery"].pop("authority_ref")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            before_task = task_json.read_bytes()

            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "task_path": f"{task_path}/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                },
            )
            before_receipt = store.read("candidate-1")
            git(repo, "checkout", "--detach", "master")

            rejected = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
                "--authority-ref",
                "test:delivery-authority",
            )

            self.assertEqual(rejected.returncode, 2)
            self.assertIn("explicit recovery requires a live checkout", rejected.stderr)
            self.assertEqual(task_json.read_bytes(), before_task)
            self.assertEqual(store.read("candidate-1"), before_receipt)
        finally:
            directory.cleanup()

    def test_register_rejects_legacy_recovery_from_base_checkout_when_branch_ref_is_missing(
        self,
    ) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            task["branch"] = "refs/heads/master"
            delivery = task["meta"]["delivery"]
            delivery["base_ref"] = "refs/heads/master"
            delivery.pop("branch_ref")
            delivery.pop("authority_ref")
            task_json.write_text(json.dumps(task), encoding="utf-8")
            before_task = task_json.read_bytes()

            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "task_path": f"{task_path}/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                },
            )
            before_receipt = store.read("candidate-1")
            git(repo, "checkout", "master")

            rejected = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
                "--authority-ref",
                "test:delivery-authority",
            )

            self.assertNotEqual(rejected.returncode, 0)
            self.assertIn("base", rejected.stderr.lower())
            self.assertEqual(task_json.read_bytes(), before_task)
            self.assertEqual(store.read("candidate-1"), before_receipt)
        finally:
            directory.cleanup()

    def test_register_rejects_legacy_receipt_from_wrong_branch_without_mutation(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            before_task = task_json.read_bytes()
            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-1",
                {
                    "repository": "rothnic/assura",
                    "task_path": f"{task_path}/task.json",
                    "owner": "tester",
                    "phase": "registered",
                    "session_state": "attached",
                    "outcome": None,
                    "closure": "open",
                    "evidence": {},
                },
            )
            git(repo, "checkout", "-b", "wrong-candidate")

            rejected = task_cli(
                repo,
                "delivery",
                "register",
                task_path,
                "--candidate",
                "candidate-1",
                "--owner",
                "tester",
            )

            self.assertEqual(rejected.returncode, 2)
            self.assertIn("does not match declared candidate branch", rejected.stderr)
            self.assertEqual(store.read("candidate-1")["generation"], 0)
            self.assertNotIn("attached_branch_ref", store.read("candidate-1"))
            self.assertEqual(task_json.read_bytes(), before_task)
        finally:
            directory.cleanup()
