"""Contract tests for the R03 independent hosted calibration collector."""

from __future__ import annotations

import os
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


if __name__ == "__main__":
    unittest.main()
