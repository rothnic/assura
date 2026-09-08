"""Contract tests for the R03 hosted-runner variance diagnostic."""

from __future__ import annotations

import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
WORKFLOW = REPOSITORY_ROOT / ".github" / "workflows" / "r03-performance-variance.yml"
RUNNER = REPOSITORY_ROOT / "scripts" / "r03-performance-variance.sh"


class R03PerformanceVarianceContractTests(unittest.TestCase):
    def test_diagnostic_preserves_exact_cold_gate_evidence_without_relaxing_it(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        runner = RUNNER.read_text(encoding="utf-8")

        self.assertIn("workflow_dispatch:", workflow)
        self.assertIn("runs-on: ubuntu-latest", workflow)
        self.assertIn("if: ${{ always() }}", workflow)
        self.assertIn("r03-performance-variance", workflow)
        self.assertIn("ASSURA_PERF_VARIANCE_COMMIT", workflow)
        self.assertIn("full 40-character commit SHA", workflow)

        self.assertIn("--iterations 16", runner)
        self.assertIn("cargo xtask performance-no-slower", runner)
        self.assertIn("ASSURA_PERF_VARIANCE_RUNS:-3", runner)
        self.assertIn("run_count < 3", runner)
        self.assertIn("^[0-9a-f]{40}$", runner)
        self.assertIn("requested_commit", runner)
        self.assertIn("git rev-parse HEAD", runner)
        self.assertIn("cargo rustc --release --bin assura --no-default-features --features json-output,yaml-config -- -C target-feature=+crt-static -C link-arg=-lgcc_eh", runner)
        self.assertIn("sha256sum", runner)
        self.assertIn("assura-full", runner)
        self.assertIn("report-${run}.json", runner)
        self.assertIn("gate_exit", runner)
        self.assertIn("exit 1", runner)
        self.assertNotIn("continue-on-error", workflow)


if __name__ == "__main__":
    unittest.main()
