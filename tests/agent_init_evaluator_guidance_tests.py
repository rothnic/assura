"""Guidance-assertion contract tests for the isolated initialization evaluator."""

from __future__ import annotations

import importlib.util
import json
import os
import stat
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
PROMPT_HASH = "2257e02d8f8d56f70937ca8ecc2993e3e4743888a68e7a5e21ca9e348f114941"


class GuidanceAssertionTests(unittest.TestCase):
    """Guidance evidence must be fixture-owned, bounded, and non-bypassable."""

    def evaluate_guidance(self, project: Path, assertion: dict[str, str]) -> dict[str, object]:
        root = project.parent
        binary = root / "assura"
        binary.write_text("#!/bin/sh\nexit 0\n")
        binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
        contract_path = root / "contract.json"
        contract_path.write_text(json.dumps({
            "schema": "assura.agent-init-evaluator.v1", "fixture_id": "guidance",
            "stack": "rust", "prompt_hash": PROMPT_HASH,
            "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
            "positive_probes": [], "negative_probes": [], "native_commands": [],
            "required_hook_states": [], "guidance_assertions": [assertion],
        }))
        output_path = root / "result.json"
        completed = subprocess.run(
            [sys.executable, str(EVALUATOR), "--project", str(project),
             "--contract", str(contract_path), "--assura-bin", str(binary),
             "--output", str(output_path), "--dimensions", "guidance"],
            check=False, capture_output=True, text=True,
        )
        self.assertNotEqual(completed.returncode, 0, completed.stdout + completed.stderr)
        return json.loads(output_path.read_text())

    def test_matching_guidance_passes_requested_dimension(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory) / "project"
            project.mkdir()
            (project / "agent-next.md").write_text("Inspect manifests before selecting a pattern.\n")
            result = self.evaluate_guidance(project, {
                "id": "inspect-manifests", "path": "agent-next.md", "contains": "Inspect manifests",
            })
            self.assertEqual(result["dimension_states"]["guidance"], "pass")

    def test_missing_fragment_fails_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory) / "project"
            project.mkdir()
            (project / "agent-next.md").write_text("Inspect manifests.\n")
            result = self.evaluate_guidance(project, {
                "id": "negative-proof", "path": "agent-next.md", "contains": "Prove a negative case",
            })
            self.assertIn("guidance:negative-proof", result["critical_failures"])

    def test_missing_file_fails_guidance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            project = Path(directory) / "project"
            project.mkdir()
            result = self.evaluate_guidance(project, {
                "id": "handoff-exists", "path": ".assura/onboarding/agent-next.md", "contains": "Inspect manifests",
            })
            evidence = next(item for item in result["command_evidence"] if item["kind"] == "guidance")
            self.assertFalse(evidence["exists"])

    def test_unsafe_and_empty_paths_are_rejected(self) -> None:
        for path in ("../AGENTS.md", ""):
            errors = EVALUATOR_MODULE.contract_validation_errors({
                "fixture_id": "guidance", "stack": "rust", "prompt_hash": PROMPT_HASH,
                "required_paths": [], "forbidden_paths": [], "preserve_hashes": {},
                "positive_probes": [], "negative_probes": [], "native_commands": [],
                "required_hook_states": [],
                "guidance_assertions": [{"id": "invalid", "path": path, "contains": "safe"}],
            })
            self.assertIn("guidance_assertions[0]", errors)

    def test_external_symlink_cannot_supply_guidance_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            project = root / "project"
            project.mkdir()
            external = root / "external-guide.md"
            external.write_text("external evidence must not satisfy guidance\n")
            os.symlink(external, project / "guide.md")
            result = self.evaluate_guidance(project, {
                "id": "no-external-content", "path": "guide.md", "contains": "external evidence",
            })
            evidence = next(item for item in result["command_evidence"] if item["kind"] == "guidance")
            self.assertEqual(evidence["reason"], "symlink_not_allowed")
            self.assertFalse(evidence["matched"])
