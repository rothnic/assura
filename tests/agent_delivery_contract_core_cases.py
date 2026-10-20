from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import types
from pathlib import Path
from unittest import mock

from agent_delivery_contract_support import ROOT
from common.delivery import DeliveryValidationError, load_delivery_intent
from common.delivery_cli import _load_evidence
from common.delivery_store import DeliveryStore, ReceiptCorrupt, StaleGeneration
from common.io import write_json

class CommonJsonIOCases:
    def test_failed_atomic_json_replace_preserves_prior_file(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "task.json"
            path.write_text('{"prior": true}', encoding="utf-8")
            before = path.read_bytes()

            with mock.patch("os.replace", side_effect=OSError("simulated interruption")):
                self.assertFalse(write_json(path, {"replacement": True}))

            self.assertEqual(path.read_bytes(), before)
            self.assertEqual(list(Path(directory).glob(".task.json.*.tmp")), [])

class DeliveryIntentCases:
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

    def test_rejects_candidate_branch_equal_to_base_ref(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_json = Path(directory) / "task.json"
            task_json.write_text(
                json.dumps(
                    {
                        "meta": {
                            "delivery": {
                                "schema_version": 1,
                                "kind": "integration",
                                "candidate_id": "candidate-1",
                                "owner": "nroth",
                                "repository": "rothnic/assura",
                                "base_ref": "refs/remotes/origin/master",
                                "branch_ref": "refs/heads/master",
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                DeliveryValidationError, "branch_ref must differ from base_ref"
            ):
                load_delivery_intent(task_json)

    def test_rejects_candidate_branch_equal_to_non_origin_remote_base(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_json = Path(directory) / "task.json"
            task_json.write_text(
                json.dumps(
                    {
                        "meta": {
                            "delivery": {
                                "schema_version": 1,
                                "kind": "integration",
                                "candidate_id": "candidate-1",
                                "owner": "nroth",
                                "repository": "rothnic/assura",
                                "base_ref": "refs/remotes/upstream/master",
                                "branch_ref": "refs/heads/master",
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                DeliveryValidationError, "branch_ref must differ from base_ref"
            ):
                load_delivery_intent(task_json)

    def test_rejects_symbolic_remote_head_as_delivery_base(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_json = Path(directory) / "task.json"
            task_json.write_text(
                json.dumps(
                    {
                        "meta": {
                            "delivery": {
                                "schema_version": 1,
                                "kind": "integration",
                                "candidate_id": "candidate-1",
                                "owner": "nroth",
                                "repository": "rothnic/assura",
                                "base_ref": "refs/remotes/origin/HEAD",
                                "branch_ref": "refs/heads/master",
                            }
                        }
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                DeliveryValidationError, "base_ref must name an explicit remote branch"
            ):
                load_delivery_intent(task_json)

    def test_rejects_literal_head_aliases_for_delivery_refs(self) -> None:
        for field, value, base_ref, branch_ref in (
            ("base_ref", "HEAD", "HEAD", "refs/heads/candidate"),
            ("base_ref", "@", "@", "refs/heads/candidate"),
            ("branch_ref", "HEAD", "refs/heads/master", "HEAD"),
            ("branch_ref", "@", "refs/heads/master", "@"),
        ):
            with self.subTest(field=field, value=value):
                with tempfile.TemporaryDirectory() as directory:
                    task_json = Path(directory) / "task.json"
                    task_json.write_text(
                        json.dumps(
                            {
                                "meta": {
                                    "delivery": {
                                        "schema_version": 1,
                                        "kind": "integration",
                                        "candidate_id": "candidate-1",
                                        "owner": "nroth",
                                        "repository": "rothnic/assura",
                                        "base_ref": base_ref,
                                        "branch_ref": branch_ref,
                                    }
                                }
                            }
                        ),
                        encoding="utf-8",
                    )

                    with self.assertRaisesRegex(
                        DeliveryValidationError, f"{field} must name an explicit branch"
                    ):
                        load_delivery_intent(task_json)

    def test_rejects_symbolic_remote_head_as_candidate_branch(self) -> None:
        for branch_ref in (
            "refs/remotes/origin/HEAD",
            "refs/remotes/upstream/HEAD",
            "origin/HEAD",
        ):
            with self.subTest(branch_ref=branch_ref):
                with tempfile.TemporaryDirectory() as directory:
                    task_json = Path(directory) / "task.json"
                    task_json.write_text(
                        json.dumps(
                            {
                                "meta": {
                                    "delivery": {
                                        "schema_version": 1,
                                        "kind": "integration",
                                        "candidate_id": "candidate-1",
                                        "owner": "nroth",
                                        "repository": "rothnic/assura",
                                        "base_ref": "refs/heads/master",
                                        "branch_ref": branch_ref,
                                    }
                                }
                            }
                        ),
                        encoding="utf-8",
                    )

                    with self.assertRaisesRegex(
                        DeliveryValidationError,
                        "branch_ref must name an explicit remote branch",
                    ):
                        load_delivery_intent(task_json)

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

class DeliveryStoreCases:
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
