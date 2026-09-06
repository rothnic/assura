"""Contract tests for the isolated agent-initialization evaluator."""

from __future__ import annotations

import json
import importlib.util
import os
import stat
import hashlib
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
EVALUATOR = REPOSITORY_ROOT / "scripts" / "evaluate-agent-init.py"
EVALUATOR_SPEC = importlib.util.spec_from_file_location("evaluate_agent_init", EVALUATOR)
assert EVALUATOR_SPEC and EVALUATOR_SPEC.loader
EVALUATOR_MODULE = importlib.util.module_from_spec(EVALUATOR_SPEC)
EVALUATOR_SPEC.loader.exec_module(EVALUATOR_MODULE)


class AgentInitEvaluatorTests(unittest.TestCase):
    """The evaluator must reject a permissive false-green setup."""

    def test_permissive_check_fails_the_required_negative_probe(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            (project / ".assura").mkdir()
            (project / ".assura" / "config.yml").write_text("rules: {}\n")

            binary = temporary_root / "always-green-assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)

            contract = {
                "schema": "assura.agent-init-evaluator.v1",
                "fixture_id": "permissive-false-green",
                "stack": "rust",
                "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
                "required_paths": [".assura/config.yml"],
                "forbidden_paths": [],
                "preserve_hashes": {},
                "positive_probes": [],
                "negative_probes": [
                    {
                        "id": "must-reject-bad-name",
                        "expected_rule": "naming",
                        "mutation": {"path": "src/BadName.rs", "contents": "pub fn bad() {}\n"},
                        "command": ["check", "."],
                        "cwd": ".",
                    }
                ],
                "native_commands": [],
                "required_hook_states": [],
            }
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps(contract))
            output_path = temporary_root / "result.json"

            completed = subprocess.run(
                [
                    sys.executable,
                    str(EVALUATOR),
                    "--project",
                    str(project),
                    "--contract",
                    str(contract_path),
                    "--assura-bin",
                    str(binary),
                    "--output",
                    str(output_path),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertFalse(result["acceptance_pass"])
            self.assertIn("must-reject-bad-name", result["critical_failures"])

    def test_partial_dimension_run_is_not_acceptance_eligible(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(
                json.dumps(
                    {
                        "schema": "assura.agent-init-evaluator.v1",
                        "fixture_id": "partial-scope",
                        "stack": "python",
                "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
                        "required_paths": [],
                        "forbidden_paths": [],
                        "preserve_hashes": {},
                        "positive_probes": [],
                        "negative_probes": [],
                        "native_commands": [],
                        "required_hook_states": [],
                    }
                )
            )
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [
                    sys.executable,
                    str(EVALUATOR),
                    "--project", str(project),
                    "--contract", str(contract_path),
                    "--assura-bin", str(binary),
                    "--output", str(output_path),
                    "--dimensions", "structure,policy",
                ],
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertEqual(result["verification_scope"], "partial")
            self.assertFalse(result["acceptance_eligible"])
            self.assertFalse(result["acceptance_pass"])

    def test_changed_preserved_file_fails_evaluation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            preserved = project / "AGENTS.md"
            preserved.write_text("original instructions\n")
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            expected_hash = hashlib.sha256(b"different instructions\n").hexdigest()
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(
                json.dumps(
                    {
                        "schema": "assura.agent-init-evaluator.v1",
                        "fixture_id": "preservation-failure",
                        "stack": "rust",
                "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
                        "required_paths": [],
                        "forbidden_paths": [],
                        "preserve_hashes": {"AGENTS.md": expected_hash},
                        "positive_probes": [],
                        "negative_probes": [],
                        "native_commands": [],
                        "required_hook_states": [],
                    }
                )
            )
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("preservation:AGENTS.md", result["critical_failures"])

    def test_missing_native_command_is_unavailable_and_fails(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "native-missing",
                "stack": "python", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [{"id": "pytest", "command": ["definitely-not-installed"], "cwd": "."}],
                "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("native:pytest", result["critical_failures"])
            self.assertIn("unavailable", {entry.get("state") for entry in result["command_evidence"]})

    def test_missing_required_path_and_forbidden_path_fail_evaluation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            (project / "scratch.tmp").write_text("not allowed\n")
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "structure-failure",
                "stack": "typescript", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": ["src/index.ts"],
                "forbidden_paths": ["scratch.tmp"], "preserve_hashes": {},
                "positive_probes": [], "negative_probes": [], "native_commands": [],
                "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("required_path:src/index.ts", result["critical_failures"])
            self.assertIn("forbidden_path:scratch.tmp", result["critical_failures"])

    def test_partial_structure_scope_does_not_run_native_commands(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "partial-structure",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [{"id": "absent", "command": ["definitely-not-installed"], "cwd": "."}],
                "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "structure"],
                check=False, capture_output=True, text=True,
            )
            result = json.loads(output_path.read_text())
            self.assertFalse(result["acceptance_eligible"])
            self.assertNotIn("native:absent", result["critical_failures"])
            self.assertNotIn("absent", {entry.get("native_id") for entry in result["command_evidence"]})

    def test_failing_required_positive_probe_fails_evaluation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "positive-failure",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {},
                "positive_probes": [{"id": "must-pass-check", "command": ["check", "."], "cwd": "."}],
                "negative_probes": [], "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("positive:must-pass-check", result["critical_failures"])

    def test_missing_required_hook_state_fails_evaluation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "hook-missing",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [],
                "required_hook_states": [{"id": "pre-commit", "path": ".git/hooks/pre-commit", "exists": True}],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("hook:pre-commit", result["critical_failures"])

    def test_zero_collected_native_tests_fail_evaluation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "zero-tests",
                "stack": "python", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [{
                    "id": "pytest", "command": ["sh", "-c", "echo '0 tests collected'"],
                    "cwd": ".", "require_collected_tests": True,
                }], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("native:pytest", result["critical_failures"])
            native_evidence = next(entry for entry in result["command_evidence"] if entry.get("native_id") == "pytest")
            self.assertEqual(native_evidence["state"], "fail")

    def test_unknown_contract_schema_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v0", "fixture_id": "wrong-schema",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("schema", result["critical_failures"])

    def test_missing_declared_native_cwd_fails_with_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "wrong-cwd",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [{"id": "native", "command": ["sh", "-c", "exit 0"], "cwd": "missing"}],
                "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("native:native", result["critical_failures"])
            native_evidence = next(entry for entry in result["command_evidence"] if entry.get("native_id") == "native")
            self.assertEqual(native_evidence["state"], "unavailable")

    def test_hand_configured_contract_reports_unavailable_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            (project / "src").mkdir()
            stable_file = project / "src" / "lib.rs"
            stable_file.write_text("pub fn stable() {}\n")
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "known-good",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": ["src/lib.rs"], "forbidden_paths": [],
                "preserve_hashes": {"src/lib.rs": hashlib.sha256(stable_file.read_bytes()).hexdigest()},
                "positive_probes": [{"id": "check", "command": ["check", "."], "cwd": "."}],
                "negative_probes": [],
                "native_commands": [{"id": "native", "command": ["sh", "-c", "exit 0"], "cwd": "."}],
                "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )
            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertTrue(result["acceptance_eligible"])
            self.assertFalse(result["acceptance_pass"])
            self.assertEqual(result["dimension_states"]["guidance"], "unavailable")
            self.assertEqual(
                result["contract_hash"], hashlib.sha256(contract_path.read_bytes()).hexdigest()
            )
            self.assertEqual(
                result["assura_binary_hash"], hashlib.sha256(binary.read_bytes()).hexdigest()
            )

    def test_public_output_redacts_private_command_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text(
                "#!/bin/sh\nif test -f BadName.rs; then echo snake_case >&2; exit 1; fi\necho private-evaluator-secret\nexit 0\n"
            )
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "redacted-publication",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {},
                "positive_probes": [{"id": "check", "command": ["check", "."], "cwd": "."}],
                "negative_probes": [{
                    "id": "reject-bad-name", "expected_rule": "snake_case",
                    "mutation": {"path": "BadName.rs", "contents": "x"},
                    "command": ["check", "."], "cwd": ".",
                }], "native_commands": [], "required_hook_states": [],
            }))
            private_output = temporary_root / "private-result.json"
            public_output = temporary_root / "public-result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(private_output), "--public-output", str(public_output),
                 "--dimensions", "policy"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            self.assertIn("private-evaluator-secret", private_output.read_text())
            publication = public_output.read_text()
            self.assertNotIn("private-evaluator-secret", publication)
            public_result = json.loads(publication)
            command_evidence = public_result["command_evidence"][0]
            self.assertNotIn("stdout", command_evidence)
            self.assertNotIn("stderr", command_evidence)
            self.assertNotIn("cwd", command_evidence)
            self.assertNotIn("fixture_id", public_result)
            self.assertNotIn("critical_failures", public_result)
            self.assertEqual(public_result["critical_failure_count"], 0)

    def test_full_evaluation_records_idempotence_without_mutating_source_fixture(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            source_file = project / "src.txt"
            source_file.write_text("frozen fixture\n")
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\ntouch evaluator-mutation-marker\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "idempotence",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": ["src.txt"], "forbidden_paths": [],
                "preserve_hashes": {"src.txt": hashlib.sha256(source_file.read_bytes()).hexdigest()},
                "positive_probes": [{"id": "check", "command": ["check", "."], "cwd": "."}],
                "negative_probes": [], "native_commands": [], "required_hook_states": [],
            }))
            first_output = temporary_root / "first-result.json"
            second_output = temporary_root / "second-result.json"
            invocation = [sys.executable, str(EVALUATOR), "--project", str(project),
                          "--contract", str(contract_path), "--assura-bin", str(binary)]

            first = subprocess.run(
                [*invocation, "--output", str(first_output)], check=False,
                capture_output=True, text=True,
            )
            second = subprocess.run(
                [*invocation, "--output", str(second_output)], check=False,
                capture_output=True, text=True,
            )

            self.assertNotEqual(first.returncode, 0, first.stdout + first.stderr)
            self.assertNotEqual(second.returncode, 0, second.stdout + second.stderr)
            self.assertEqual(source_file.read_text(), "frozen fixture\n")
            self.assertFalse((project / "evaluator-mutation-marker").exists())
            result = json.loads(second_output.read_text())
            self.assertEqual(result["dimension_states"]["idempotence"], "pass")

    def test_full_run_marks_uncontracted_dimensions_unavailable(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "missing-dimensions",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertEqual(result["dimension_states"]["guidance"], "unavailable")
            self.assertEqual(result["dimension_states"]["hooks"], "unavailable")
            self.assertEqual(result["dimension_states"]["native"], "unavailable")
            self.assertFalse(result["acceptance_pass"])

    def test_guidance_assertion_records_matching_fixture_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            (project / "agent-next.md").write_text("Inspect manifests before selecting a pattern.\n")
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "guidance-pass",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
                "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
                "positive_probes": [], "negative_probes": [], "native_commands": [],
                "required_hook_states": [],
                "guidance_assertions": [{
                    "id": "inspect-manifests", "path": "agent-next.md",
                    "contains": "Inspect manifests",
                }],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "guidance"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertEqual(result["dimension_states"]["guidance"], "pass")
            evidence = next(entry for entry in result["command_evidence"] if entry["kind"] == "guidance")
            self.assertTrue(evidence["matched"])

    def test_missing_guidance_fragment_fails_requested_dimension(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            (project / "agent-next.md").write_text("Inspect manifests.\n")
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "guidance-fail",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
                "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
                "positive_probes": [], "negative_probes": [], "native_commands": [],
                "required_hook_states": [],
                "guidance_assertions": [{
                    "id": "negative-proof", "path": "agent-next.md",
                    "contains": "Prove a negative case",
                }],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "guidance"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("guidance:negative-proof", result["critical_failures"])
            self.assertEqual(result["dimension_states"]["guidance"], "fail")

    def test_missing_guidance_file_fails_requested_dimension(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "guidance-missing-file",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
                "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
                "positive_probes": [], "negative_probes": [], "native_commands": [],
                "required_hook_states": [],
                "guidance_assertions": [{
                    "id": "handoff-exists", "path": ".assura/onboarding/agent-next.md",
                    "contains": "Inspect manifests",
                }],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "guidance"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("guidance:handoff-exists", result["critical_failures"])
            evidence = next(entry for entry in result["command_evidence"] if entry["kind"] == "guidance")
            self.assertFalse(evidence["exists"])
            self.assertFalse(evidence["matched"])

    def test_unsafe_guidance_assertion_is_rejected_by_contract_validation(self) -> None:
        contract = {
            "fixture_id": "unsafe-guidance", "stack": "rust",
            "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
            "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
            "positive_probes": [], "negative_probes": [], "native_commands": [],
            "required_hook_states": [],
            "guidance_assertions": [{"id": "escape", "path": "../AGENTS.md", "contains": "safe"}],
        }

        self.assertIn(
            "guidance_assertions[0]",
            EVALUATOR_MODULE.contract_validation_errors(contract),
        )

    def test_empty_guidance_path_is_rejected_by_contract_validation(self) -> None:
        contract = {
            "fixture_id": "empty-guidance-path", "stack": "rust",
            "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
            "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
            "positive_probes": [], "negative_probes": [], "native_commands": [],
            "required_hook_states": [],
            "guidance_assertions": [{"id": "empty", "path": "", "contains": "safe"}],
        }

        self.assertIn(
            "guidance_assertions[0]",
            EVALUATOR_MODULE.contract_validation_errors(contract),
        )

    def test_guidance_assertion_rejects_external_symlink_content(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            external_guide = temporary_root / "external-guide.md"
            external_guide.write_text("external evidence must not satisfy guidance\n")
            os.symlink(external_guide, project / "guide.md")
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "guidance-symlink",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
                "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
                "positive_probes": [], "negative_probes": [], "native_commands": [],
                "required_hook_states": [],
                "guidance_assertions": [{
                    "id": "no-external-content", "path": "guide.md",
                    "contains": "external evidence",
                }],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "guidance"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("guidance:no-external-content", result["critical_failures"])
            evidence = next(entry for entry in result["command_evidence"] if entry["kind"] == "guidance")
            self.assertEqual(evidence["reason"], "symlink_not_allowed")
            self.assertFalse(evidence["matched"])

    def test_timed_out_policy_command_records_captured_output_as_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            result = EVALUATOR_MODULE.run_command(
                Path("/bin/sh"),
                ["-c", "printf partial-output; sleep 1"],
                Path(temporary_directory),
                timeout_seconds=0.01,
            )

            self.assertEqual(result["state"], "fail")
            self.assertTrue(result["timed_out"])
            self.assertIsNone(result["exit"])
            self.assertIn("partial-output", result["stdout"])

    def test_relative_assura_binary_is_rejected_before_project_commands_run(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            project_binary = project / "assura"
            project_binary.write_text("#!/bin/sh\nexit 0\n")
            project_binary.chmod(project_binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "relative-binary",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [],
                "negative_probes": [{
                    "id": "reject-name", "expected_rule": "naming",
                    "mutation": {"path": "BadName.rs", "contents": "x"},
                    "command": ["check", "."], "cwd": ".",
                }], "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", "./assura",
                 "--output", str(output_path)],
                cwd=project, check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("assura_bin:absolute_path", result["critical_failures"])

    def test_contract_prompt_hash_is_recorded_in_private_result(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            prompt_hash = "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941"
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "prompt-provenance",
                "stack": "rust", "prompt_hash": prompt_hash,
                "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
                "positive_probes": [], "negative_probes": [],
                "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "structure"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("prompt_hash", result)
            self.assertEqual(result["prompt_hash"], prompt_hash)

    def test_contract_without_prompt_hash_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract = {
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "missing-prompt",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941",
                "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
                "positive_probes": [], "negative_probes": [],
                "native_commands": [], "required_hook_states": [],
            }
            del contract["prompt_hash"]
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps(contract))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "structure"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("contract:prompt_hash", result["critical_failures"])

    def test_negative_probe_requires_its_named_rule_in_command_output(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\necho unrelated failure >&2\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "wrong-rule",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [],
                "negative_probes": [{
                    "id": "reject-name", "expected_rule": "file_naming",
                    "mutation": {"path": "BadName.rs", "contents": "x"},
                    "command": ["check", "."], "cwd": ".",
                }], "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "policy"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("reject-name", result["critical_failures"])

    def test_policy_contract_without_negative_probes_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "no-negative",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "policy"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("contract:negative_probes", result["critical_failures"])

    def test_zero_collected_native_tests_on_stderr_fail_evaluation(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 1\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "stderr-zero-tests",
                "stack": "python", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [{
                    "id": "pytest", "command": ["sh", "-c", "echo '0 tests collected' >&2"],
                    "cwd": ".", "require_collected_tests": True,
                }], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "native"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            native_evidence = next(entry for entry in result["command_evidence"] if entry.get("native_id") == "pytest")
            self.assertEqual(native_evidence["state"], "fail")
            self.assertEqual(native_evidence["reason"], "zero_collected_tests")

    def test_each_negative_probe_uses_a_fresh_disposable_copy(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text(
                "#!/bin/sh\nif test -f first-violation; then echo naming >&2; exit 1; fi\nexit 0\n"
            )
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "independent-negatives",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [],
                "negative_probes": [
                    {"id": "reject-first", "expected_rule": "naming", "mutation": {"path": "first-violation", "contents": "x"},
                     "command": ["check", "."], "cwd": "."},
                    {"id": "reject-second", "expected_rule": "naming", "mutation": {"path": "second-violation", "contents": "x"},
                     "command": ["check", "."], "cwd": "."},
                ],
                "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "policy"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("reject-second", result["critical_failures"])
            self.assertEqual(result["dimension_states"]["policy"], "fail")

    def test_missing_contract_field_is_a_named_validation_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "missing-field",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("contract:native_commands", result["critical_failures"])

    def test_contract_cwd_cannot_escape_disposable_project(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "escaped-cwd",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {},
                "positive_probes": [{"id": "escape", "command": ["check", "."], "cwd": "../"}],
                "negative_probes": [], "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("contract:positive_probes[0].cwd", result["critical_failures"])

    def test_pytest_collected_zero_items_fails_a_required_native_test(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "pytest-zero-items",
                "stack": "python", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {}, "positive_probes": [], "negative_probes": [],
                "native_commands": [{
                    "id": "pytest", "command": ["sh", "-c", "echo 'collected 0 items'"],
                    "cwd": ".", "require_collected_tests": True,
                }], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path)],
                check=False, capture_output=True, text=True,
            )

            result = json.loads(output_path.read_text())
            native_evidence = next(
                entry for entry in result["command_evidence"] if entry.get("native_id") == "pytest"
            )
            self.assertEqual(native_evidence["state"], "fail")
            self.assertEqual(native_evidence["reason"], "zero_collected_tests")

    def test_missing_policy_probe_cwd_emits_failed_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            project = temporary_root / "project"
            project.mkdir()
            binary = temporary_root / "assura"
            binary.write_text("#!/bin/sh\nexit 0\n")
            binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
            contract_path = temporary_root / "contract.json"
            contract_path.write_text(json.dumps({
                "schema": "assura.agent-init-evaluator.v1", "fixture_id": "policy-missing-cwd",
                "stack": "rust", "prompt_hash": "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941", "required_paths": [], "forbidden_paths": [],
                "preserve_hashes": {},
                "positive_probes": [{
                    "id": "missing-cwd", "command": ["check", "."], "cwd": "missing",
                }], "negative_probes": [], "native_commands": [], "required_hook_states": [],
            }))
            output_path = temporary_root / "result.json"
            completed = subprocess.run(
                [sys.executable, str(EVALUATOR), "--project", str(project),
                 "--contract", str(contract_path), "--assura-bin", str(binary),
                 "--output", str(output_path), "--dimensions", "policy"],
                check=False, capture_output=True, text=True,
            )

            self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
            result = json.loads(output_path.read_text())
            self.assertIn("positive:missing-cwd", result["critical_failures"])
            evidence = result["command_evidence"][0]
            self.assertEqual(evidence["state"], "unavailable")


if __name__ == "__main__":
    unittest.main()
