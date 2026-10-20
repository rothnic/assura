"""Contract tests for the R03 hosted-runner variance diagnostic."""

from __future__ import annotations

import os
import hashlib
import subprocess
import tempfile
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
        self.assertIn("default: variance", workflow)
        self.assertIn("if: ${{ inputs.mode == '' || inputs.mode == 'variance' }}", workflow)
        self.assertIn("if: ${{ inputs.mode == 'callgrind' }}", workflow)
        self.assertIn("scripts/nu11-callgrind-profile.py", workflow)
        self.assertIn("debian:bookworm-20260918-slim@sha256:", workflow)
        profile_job = workflow.split("  callgrind:", 1)[1]
        self.assertLess(profile_job.index("Reject source or ref mismatch"), profile_job.index("Bootstrap checkout prerequisites"))
        self.assertLess(profile_job.index("Bootstrap checkout prerequisites"), profile_job.index("uses: actions/checkout@v4"))
        self.assertLess(profile_job.index("Verify checked-out source"), profile_job.index("Install Rust"))
        self.assertLess(profile_job.index("Verify checked-out source"), profile_job.index("Admit resources before Rust and Node setup"))
        self.assertLess(profile_job.index("Admit resources before Rust and Node setup"), profile_job.index("Install Rust"))
        self.assertIn('module.record_resource_guard(evidence, "before-runtime-tools"', profile_job)
        self.assertIn("ASSURA_NU11_STARTED_UTC", profile_job)
        self.assertIn("snapshot.debian.org/archive/debian/20260918T000000Z", profile_job)
        self.assertNotIn("--allow-unauthenticated", profile_job)

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
        self.assertLess(
            runner.index("cargo rustc --release --bin assura"),
            runner.index('> "$output_dir/binaries-before.sha256"'),
        )

    def test_profile_source_verification_rejects_failed_or_dirty_status(self) -> None:
        workflow = WORKFLOW.read_text(encoding="utf-8")
        step = workflow.split("    - name: Verify checked-out source before tool setup\n", 1)[1].split("    - name: Admit resources", 1)[0]
        shell = "\n".join(line[8:] for line in step.split("      run: |\n", 1)[1].splitlines())
        self.assertIn('git config --system --add safe.directory "$GITHUB_WORKSPACE"', shell)
        self.assertNotIn("safe.directory '*'", shell)
        self.assertIn("${{ steps.nu11_bootstrap.outputs.evidence_path }}", workflow)
        self.assertNotIn("${{ runner.temp }}/nu11-bootstrap", workflow)
        with tempfile.TemporaryDirectory() as temporary_directory:
            root = Path(temporary_directory)
            (root / "nu11-bootstrap").mkdir()
            git = root / "git"
            git.write_text("#!/bin/sh\ncase \"$1\" in\n  config) exit 0;;\n  rev-parse) printf '%s\\n' \"$ASSURA_NU11_COMMIT\";;\n  status) printf '%s' \"$STUB_STATUS\"; exit \"$STUB_EXIT\";;\n  *) exit 99;;\nesac\n")
            git.chmod(0o700)
            for status, exit_code, expected in [("", "0", 0), ("?? foreign", "0", 1), ("", "128", 128)]:
                with self.subTest(status=status, exit_code=exit_code):
                    env = os.environ | {"PATH": str(root) + os.pathsep + os.environ["PATH"], "GITHUB_WORKSPACE": str(root), "RUNNER_TEMP": str(root), "ASSURA_NU11_COMMIT": "a" * 40, "STUB_STATUS": status, "STUB_EXIT": exit_code}
                    result = subprocess.run(["bash", "-c", shell], env=env, capture_output=True, text=True)
                    self.assertEqual(result.returncode, expected, result.stderr)

    def test_original_variance_runner_is_byte_identical_to_base(self) -> None:
        self.assertEqual(
            hashlib.sha256(RUNNER.read_bytes()).hexdigest(),
            "fb933e77bed02f3ae4a7cf3de7b871ac7861fbb0d554a906d124ada84f0572c2",
        )

    def test_invalid_collection_boundaries_stop_before_build_or_artifact_creation(self) -> None:
        cases = [
            (
                {"ASSURA_PERF_VARIANCE_RUNS": "2", "ASSURA_PERF_VARIANCE_COMMIT": "0" * 40},
                "ASSURA_PERF_VARIANCE_RUNS must be an integer of at least 3",
            ),
            (
                {"ASSURA_PERF_VARIANCE_COMMIT": "master"},
                "ASSURA_PERF_VARIANCE_COMMIT must be a full 40-character commit SHA",
            ),
            (
                {"ASSURA_PERF_VARIANCE_COMMIT": "0" * 40},
                "checked-out commit does not match ASSURA_PERF_VARIANCE_COMMIT",
            ),
        ]

        for overrides, expected_error in cases:
            with self.subTest(overrides=overrides), tempfile.TemporaryDirectory() as temporary_directory:
                temporary_root = Path(temporary_directory)
                output_dir = temporary_root / "evidence"
                cargo_target_dir = temporary_root / "target"
                environment = os.environ | {
                    "ASSURA_PERF_VARIANCE_OUTPUT_DIR": str(output_dir),
                    "CARGO_TARGET_DIR": str(cargo_target_dir),
                    **overrides,
                }
                completed = subprocess.run(
                    ["bash", str(RUNNER)],
                    cwd=REPOSITORY_ROOT,
                    env=environment,
                    check=False,
                    capture_output=True,
                    text=True,
                )

                self.assertEqual(completed.returncode, 2, completed.stderr)
                self.assertIn(expected_error, completed.stderr)
                self.assertFalse(output_dir.exists())
                self.assertFalse(cargo_target_dir.exists())


if __name__ == "__main__":
    unittest.main()
