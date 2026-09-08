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
        self.assertIn("EPYC 9V74", plan["required_observed_fingerprint"])
        self.assertIn("unchanged", plan["production_gate"])
        self.assertIn("inconclusive", plan["classifications"])

    def test_classifier_distinguishes_stable_slow_stable_no_slower_and_inconclusive(self) -> None:
        def cohort(fingerprint: str, gate_exit: int, delta: float, jobs: int = 3) -> dict[str, object]:
            return {
                "fingerprint": fingerprint,
                "runs": [
                    {"job_id": f"{fingerprint}-{index}", "gate_exit": gate_exit, "paired_deltas_ms": [delta] * 16}
                    for index in range(jobs)
                ],
            }

        result = self.classify([
            cohort("stable-slow", 1, 1.0),
            cohort("stable-no-slower", 0, -1.0),
            cohort("inconclusive", 0, -1.0, jobs=2),
        ])
        classifications = {item["fingerprint"]: item["classification"] for item in result["cohorts"]}
        self.assertEqual(classifications["stable-slow"], "stable-slow")
        self.assertEqual(classifications["stable-no-slower"], "stable-no-slower")
        self.assertEqual(classifications["inconclusive"], "inconclusive")


if __name__ == "__main__":
    unittest.main()
