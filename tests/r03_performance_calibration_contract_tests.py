"""Contract tests for the R03 pre-registered calibration classifier."""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
CLASSIFIER = REPOSITORY_ROOT / ".trellis" / "scripts" / "r03_performance_calibration.py"


class R03PerformanceCalibrationContractTests(unittest.TestCase):
    def classify(self, cohorts: list[dict[str, object]]) -> dict[str, object]:
        with tempfile.TemporaryDirectory() as temporary_directory:
            input_path = Path(temporary_directory) / "cohorts.json"
            input_path.write_text(json.dumps({"cohorts": cohorts}), encoding="utf-8")
            completed = subprocess.run(
                [sys.executable, str(CLASSIFIER), "--classify", str(input_path)],
                cwd=REPOSITORY_ROOT,
                check=False,
                capture_output=True,
                text=True,
            )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        return json.loads(completed.stdout)

    def test_plan_requires_independent_fingerprint_cohorts_without_changing_the_gate(self) -> None:
        completed = subprocess.run(
            [sys.executable, str(CLASSIFIER), "--plan"],
            cwd=REPOSITORY_ROOT,
            check=False,
            capture_output=True,
            text=True,
        )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        plan = json.loads(completed.stdout)
        self.assertEqual(plan["independent_jobs_per_fingerprint"], 3)
        self.assertIn("EPYC 9V74", plan["required_observed_cpu_model"])
        self.assertIn("unchanged", plan["production_gate"])
        self.assertIn("inconclusive", plan["classifications"])

    def test_classifier_distinguishes_stable_slow_stable_no_slower_and_inconclusive(self) -> None:
        def cohort(cpu: str, image: str, gate_exit: int, delta: float, jobs: int = 3) -> dict[str, object]:
            return {
                "fingerprint": f"{cpu}|{image}",
                "runs": [
                    {
                        "job_id": f"{cpu}-{image}-{index}",
                        "cpu_model": cpu,
                        "runner_image": image,
                        "source_sha": "a" * 40,
                        "assura_binary_sha256": "b" * 64,
                        "report_sha256": f"{index:064x}",
                        "report_exit": 0,
                        "gate_exit": gate_exit,
                        "paired_deltas_ms": [delta] * 16,
                    }
                    for index in range(jobs)
                ],
            }

        result = self.classify([
            cohort("stable-slow", "image-a", 1, 1.0),
            cohort("stable-no-slower", "image-a", 0, -1.0),
            cohort("inconclusive", "image-a", 0, -1.0, jobs=2),
        ])
        classifications = {item["fingerprint"]: item["classification"] for item in result["cohorts"]}
        self.assertEqual(classifications["stable-slow|image-a"], "stable-slow")
        self.assertEqual(classifications["stable-no-slower|image-a"], "stable-no-slower")
        self.assertEqual(classifications["inconclusive|image-a"], "inconclusive")
        self.assertEqual(result["required_observed_cpu_model"]["status"], "unproven")

    def test_mixed_or_nonfinite_provenance_is_inconclusive(self) -> None:
        run = {
            "job_id": "one",
            "cpu_model": "AMD EPYC 9V74",
            "runner_image": "ubuntu-24.04-v1",
            "source_sha": "a" * 40,
            "assura_binary_sha256": "b" * 64,
            "report_sha256": "c" * 64,
            "report_exit": 0,
            "gate_exit": 0,
            "paired_deltas_ms": [-1.0] * 16,
        }
        mixed = {"fingerprint": "AMD EPYC 9V74|ubuntu-24.04-v1", "runs": [run | {"job_id": str(index)} for index in range(3)]}
        mixed["runs"][1]["cpu_model"] = "AMD EPYC 7763"
        nonfinite = {"fingerprint": "AMD EPYC 9V74|ubuntu-24.04-v1", "runs": [run | {"job_id": str(index)} for index in range(3)]}
        nonfinite["runs"][2]["paired_deltas_ms"] = [float("nan")] * 16

        result = self.classify([mixed, nonfinite])
        self.assertEqual([item["classification"] for item in result["cohorts"]], ["inconclusive", "inconclusive"])
        self.assertEqual(result["required_observed_cpu_model"]["status"], "unproven")


if __name__ == "__main__":
    unittest.main()
