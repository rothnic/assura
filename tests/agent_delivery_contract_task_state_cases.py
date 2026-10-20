from __future__ import annotations

import json
import shutil
from pathlib import Path

from agent_delivery_contract_support import fixture_repo, git, task_cli
from common.delivery import DeliveryValidationError, load_delivery_intent
from common.delivery_cli import ensure_receipt
from common.delivery_store import DeliveryStore

class LifecycleTaskStateCases:
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

    def test_shared_candidate_receipt_rejects_a_second_branch_attachment(self) -> None:
        repo, _, _, directory = fixture_repo()
        alternate = Path(directory.name) / "candidate-alternate"
        try:
            task_json = repo / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            intent = load_delivery_intent(task_json)
            first = ensure_receipt(repo, intent, task, phase="implementing")
            self.assertEqual(first["attached_branch_ref"], "refs/heads/candidate")

            git(repo, "worktree", "add", "-b", "candidate-alternate", str(alternate), "candidate")
            alternate_task_json = (
                alternate / ".trellis" / "tasks" / "01-01-candidate" / "task.json"
            )
            alternate_task_json.parent.mkdir(parents=True, exist_ok=True)
            alternate_task = json.loads(json.dumps(task))
            alternate_task_json.write_text(
                json.dumps(alternate_task), encoding="utf-8"
            )
            alternate_intent = load_delivery_intent(alternate_task_json)

            with self.assertRaises(DeliveryValidationError) as raised:
                ensure_receipt(
                    alternate,
                    alternate_intent,
                    alternate_task,
                    phase="implementing",
                )

            self.assertIn("RECEIPT_BINDING_CONFLICT", str(raised.exception))
            self.assertIn("branch", str(raised.exception))
            self.assertEqual(
                DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
                .read("candidate-1")["attached_branch_ref"],
                "refs/heads/candidate",
            )
        finally:
            if alternate.is_dir():
                git(repo, "worktree", "remove", "--force", str(alternate))
            directory.cleanup()

    def test_delivery_record_rejects_a_second_branch_before_receipt_mutation(self) -> None:
        repo, _, _, directory = fixture_repo()
        alternate = Path(directory.name) / "candidate-alternate"
        try:
            task_path = ".trellis/tasks/01-01-candidate"
            task_json = repo / task_path / "task.json"
            task = json.loads(task_json.read_text(encoding="utf-8"))
            intent = load_delivery_intent(task_json)
            ensure_receipt(repo, intent, task, phase="implementing")

            git(repo, "worktree", "add", "-b", "candidate-alternate", str(alternate), "candidate")
            alternate_task_json = alternate / task_path / "task.json"
            alternate_task_json.parent.mkdir(parents=True, exist_ok=True)
            alternate_task_json.write_text(json.dumps(task), encoding="utf-8")
            shutil.copytree(repo / ".test-bin", alternate / ".test-bin")
            evidence_path = Path(directory.name) / "alternate-evidence.json"
            evidence_path.write_text(json.dumps({"review": {}}), encoding="utf-8")

            recorded = task_cli(
                alternate,
                "delivery",
                "record",
                task_path,
                "--evidence-file",
                str(evidence_path),
                "--expected-generation",
                "0",
            )

            self.assertEqual(recorded.returncode, 2)
            self.assertIn("RECEIPT_BINDING_CONFLICT", recorded.stderr)

            closed = task_cli(
                alternate,
                "delivery",
                "close",
                task_path,
                "--outcome",
                "rejected",
                "--reason",
                "must not close from a second branch",
            )

            self.assertEqual(closed.returncode, 2)
            self.assertIn("RECEIPT_BINDING_CONFLICT", closed.stderr)
            self.assertEqual(
                DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
                .read("candidate-1")["generation"],
                0,
            )
        finally:
            if alternate.is_dir():
                git(repo, "worktree", "remove", "--force", str(alternate))
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

    def test_start_guard_rejects_misbound_terminal_receipt(self) -> None:
        repo, _, _, directory = fixture_repo()
        try:
            candidate_path = ".trellis/tasks/01-01-candidate"
            candidate_json = repo / candidate_path / "task.json"
            other_path = ".trellis/tasks/01-02-other"
            other_json = repo / other_path / "task.json"
            task = json.loads(candidate_json.read_text(encoding="utf-8"))
            other_task = json.loads(json.dumps(task))
            other_task.update(
                {"id": "other", "name": "other", "title": "Other candidate"}
            )
            other_task["branch"] = "refs/heads/other-candidate"
            other_task["meta"]["delivery"].update(
                {
                    "candidate_id": "candidate-2",
                    "branch_ref": "refs/heads/other-candidate",
                }
            )
            other_json.parent.mkdir(parents=True)
            other_json.write_text(json.dumps(other_task), encoding="utf-8")

            store = DeliveryStore(repo / ".git" / "assura" / "delivery-v1")
            store.create(
                "candidate-2",
                {
                    "repository": "rothnic/assura",
                    "task_path": f"{other_path}/task.json",
                    "owner": "tester",
                    "attached_branch_ref": "refs/heads/candidate",
                    "authority_ref": "test:delivery-authority",
                    "phase": "verified",
                    "outcome": "delivered",
                    "closure": "verified",
                    "evidence": {},
                },
            )
            before_task = candidate_json.read_bytes()
            before_other = other_json.read_bytes()
            before_receipt = store.read("candidate-2")
            candidate_receipt = store.path_for("candidate-1")

            started = task_cli(repo, "start", candidate_path)

            self.assertNotEqual(started.returncode, 0, started.stderr)
            self.assertIn("UNFINISHED_OWNED_CANDIDATE", started.stderr)
            self.assertIn("RECEIPT_BINDING_CONFLICT", started.stderr)
            self.assertEqual(candidate_json.read_bytes(), before_task)
            self.assertEqual(other_json.read_bytes(), before_other)
            self.assertFalse(candidate_receipt.exists())
            self.assertEqual(store.read("candidate-2"), before_receipt)
        finally:
            directory.cleanup()
