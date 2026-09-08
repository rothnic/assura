"""Contract tests for the R03 independent hosted calibration collector."""

from __future__ import annotations

import os
import json
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "r03-performance-calibration.yml"
COLLECTOR = REPOSITORY_ROOT / "scripts" / "r03-performance-calibration-run.sh"


class R03PerformanceCalibrationCollectorContractTests(unittest.TestCase):
    def test_collector_requires_three_independent_pinned_runner_jobs(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        collector = COLLECTOR.read_text(encoding="utf-8")

        self.assertIn("workflow_dispatch:", workflow)
        self.assertIn("runs-on: ubuntu-24.04", workflow)
        self.assertIn("slot: [1, 2, 3]", workflow)
        self.assertIn("ASSURA_R03_CALIBRATION_SLOT", workflow)
        self.assertIn("r03-performance-calibration-${{ matrix.slot }}", workflow)
        self.assertIn("if: ${{ always() }}", workflow)
        self.assertNotIn("continue-on-error", workflow)

        self.assertIn("ASSURA_R03_CALIBRATION_COMMIT", collector)
        self.assertIn("ASSURA_R03_CALIBRATION_SLOT", collector)
        self.assertIn("ubuntu-latest", collector)
        self.assertIn("ImageOS", collector)
        self.assertIn("ImageVersion", collector)
        self.assertIn("--iterations 16", collector)
        self.assertIn("cargo xtask performance-no-slower", collector)
        self.assertIn("paired_deltas_ms", collector)
        self.assertIn("assura_binary_sha256", collector)
        self.assertIn("report_sha256", collector)
        self.assertIn("exit \"$overall_exit\"", collector)

    def test_invalid_boundaries_stop_before_build_or_artifact_creation(self) -> None:
        cases = [
            (
                {
                    "ASSURA_R03_CALIBRATION_COMMIT": "master",
                    "ASSURA_R03_CALIBRATION_SLOT": "1",
                    "ImageOS": "ubuntu24",
                    "ImageVersion": "20260901.1",
                },
                "ASSURA_R03_CALIBRATION_COMMIT must be a full 40-character commit SHA",
            ),
            (
                {
                    "ASSURA_R03_CALIBRATION_COMMIT": "0" * 40,
                    "ASSURA_R03_CALIBRATION_SLOT": "4",
                    "ImageOS": "ubuntu24",
                    "ImageVersion": "20260901.1",
                },
                "ASSURA_R03_CALIBRATION_SLOT must be one of 1, 2, or 3",
            ),
            (
                {
                    "ASSURA_R03_CALIBRATION_COMMIT": "0" * 40,
                    "ASSURA_R03_CALIBRATION_SLOT": "1",
                    "ImageOS": "ubuntu-latest",
                    "ImageVersion": "latest",
                },
                "runner image identity must be nonempty and immutable",
            ),
            (
                {
                    "ASSURA_R03_CALIBRATION_COMMIT": "0" * 40,
                    "ASSURA_R03_CALIBRATION_SLOT": "1",
                    "ImageOS": "ubuntu24",
                    "ImageVersion": "20260901.1",
                    "GITHUB_RUN_ID": "1",
                    "GITHUB_JOB": "collect",
                },
                "checked-out commit does not match ASSURA_R03_CALIBRATION_COMMIT",
            ),
        ]

        for overrides, expected_error in cases:
            with self.subTest(overrides=overrides), tempfile.TemporaryDirectory() as temporary_directory:
                output_dir = Path(temporary_directory) / "evidence"
                environment = os.environ | {
                    "ASSURA_R03_CALIBRATION_OUTPUT_DIR": str(output_dir),
                    **overrides,
                }
                completed = subprocess.run(
                    ["bash", str(COLLECTOR)],
                    cwd=REPOSITORY_ROOT,
                    env=environment,
                    check=False,
                    capture_output=True,
                    text=True,
                )

                self.assertEqual(completed.returncode, 2, completed.stderr)
                self.assertIn(expected_error, completed.stderr)
                self.assertFalse(output_dir.exists())

    def test_strict_gate_failure_retains_complete_classification_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as temporary_directory:
            temporary_root = Path(temporary_directory)
            scripts_dir = temporary_root / "scripts"
            scripts_dir.mkdir()
            runner = scripts_dir / COLLECTOR.name
            shutil.copy2(COLLECTOR, runner)
            subprocess.run(["git", "init", "--quiet"], cwd=temporary_root, check=True)
            subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=temporary_root, check=True)
            subprocess.run(["git", "config", "user.name", "R03 Test"], cwd=temporary_root, check=True)
            subprocess.run(["git", "add", "scripts"], cwd=temporary_root, check=True)
            subprocess.run(["git", "commit", "--quiet", "-m", "fixture"], cwd=temporary_root, check=True)
            source_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=temporary_root, text=True).strip()

            release_dir = temporary_root / "target" / "release"
            release_dir.mkdir(parents=True)
            report = temporary_root / "report.json"
            samples = list(range(16))
            report.write_text(json.dumps({"results": [
                {"fixture_id": "many_configured_scopes_regression", "row_family": "assura-cli", "distribution": {"samples_ms": samples}},
                {"fixture_id": "many_configured_scopes_regression", "row_family": "ls-lint-cli", "distribution": {"samples_ms": [sample - 0.5 for sample in samples]}},
            ]}), encoding="utf-8")
            assura = release_dir / "assura"
            assura.write_text("#!/usr/bin/env bash\ncp \"$CALIBRATION_REPORT\" \"$3\"\n", encoding="utf-8")
            assura.chmod(0o755)
            for binary_name in ("assura-full", "assura-check"):
                (release_dir / binary_name).write_text(binary_name, encoding="utf-8")

            shim_dir = temporary_root / "shim"
            shim_dir.mkdir()
            (shim_dir / "cargo").write_text("#!/usr/bin/env bash\n[[ \"$1\" == xtask ]] && exit 1\nexit 0\n", encoding="utf-8")
            (shim_dir / "lscpu").write_text("#!/usr/bin/env bash\necho 'Model name: AMD EPYC 9V74'\n", encoding="utf-8")
            for shim in shim_dir.iterdir():
                shim.chmod(0o755)

            output_dir = temporary_root / "evidence"
            environment = os.environ | {
                "PATH": f"{shim_dir}:{os.environ['PATH']}",
                "CALIBRATION_REPORT": str(report),
                "ASSURA_R03_CALIBRATION_COMMIT": source_sha,
                "ASSURA_R03_CALIBRATION_SLOT": "2",
                "ASSURA_R03_CALIBRATION_OUTPUT_DIR": str(output_dir),
                "ImageOS": "ubuntu24",
                "ImageVersion": "20260901.1",
                "GITHUB_RUN_ID": "123",
                "GITHUB_RUN_ATTEMPT": "1",
                "GITHUB_JOB": "collect",
            }
            completed = subprocess.run(
                ["bash", str(runner)],
                cwd=temporary_root,
                env=environment,
                check=False,
                capture_output=True,
                text=True,
            )

            self.assertEqual(completed.returncode, 1, completed.stderr)
            self.assertTrue((output_dir / "report.json").is_file())
            metadata = json.loads((output_dir / "run.json").read_text(encoding="utf-8"))
            self.assertEqual(metadata["job_id"], "123:1:collect:2")
            self.assertEqual(metadata["cpu_model"], "AMD EPYC 9V74")
            self.assertEqual(metadata["runner_image_name"], "ubuntu24")
            self.assertEqual(metadata["runner_image_version"], "20260901.1")
            self.assertEqual(metadata["source_sha"], source_sha)
            self.assertEqual(metadata["report_exit"], 0)
            self.assertEqual(metadata["gate_exit"], 1)
            self.assertEqual(metadata["binary_identity_exit"], 0)
            self.assertEqual(metadata["paired_deltas_ms"], [0.5] * 16)


if __name__ == "__main__":
    unittest.main()
