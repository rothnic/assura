"""Contract tests for the bounded R03 Callgrind attribution driver."""

from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
DRIVER = REPOSITORY_ROOT / ".trellis" / "scripts" / "r03_callgrind_attribution.py"


class R03CallgrindAttributionDriverTests(unittest.TestCase):
    def test_plan_declares_the_reviewed_finite_diagnostic_contract(self) -> None:
        completed = subprocess.run(
            [sys.executable, str(DRIVER), "--plan"],
            cwd=REPOSITORY_ROOT,
            check=False,
            capture_output=True,
            text=True,
        )

        self.assertEqual(completed.returncode, 0, completed.stderr)
        plan = json.loads(completed.stdout)
        self.assertTrue(plan["diagnostic_only"])
        self.assertEqual(plan["timeout_seconds"], 30)
        self.assertEqual(plan["collection_sequence"], [
            "baseline",
            "candidate",
            "candidate",
            "baseline",
            "baseline",
            "candidate",
        ])
        self.assertEqual(plan["collection_samples"], 6)
        self.assertEqual(plan["positive_control"], "shell-loop-1000-no-io")
        self.assertIn("hosted no-slower", plan["non_acceptance_uses"])


if __name__ == "__main__":
    unittest.main()
