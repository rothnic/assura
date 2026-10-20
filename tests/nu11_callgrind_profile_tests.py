"""Focused negative controls for NU11's bounded Callgrind runner."""

from __future__ import annotations

import importlib.util
import json
import os
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path
from unittest.mock import patch


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = REPOSITORY_ROOT / "scripts" / "nu11-callgrind-profile.py"
SPEC = importlib.util.spec_from_file_location("nu11_callgrind_profile", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
NU11 = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = NU11
SPEC.loader.exec_module(NU11)


class Nu11ProfileContractTests(unittest.TestCase):
    def capture(self, root: Path, name: str, code: str, *, timeout: float = 3,
                limit: int = 4096, produced_file: Path | None = None,
                produced_file_limit: int = NU11.PROFILE_LIMIT):
        evidence = NU11.Evidence(root, {"PATH": os.environ.get("PATH", "")}, REPOSITORY_ROOT)
        return evidence, evidence.capture(
            name,
            [sys.executable, "-c", code],
            cwd=REPOSITORY_ROOT,
            timeout=timeout,
            limit=limit,
            produced_file=produced_file,
            produced_file_limit=produced_file_limit,
        )

    def test_source_binding_requires_exact_candidate_commit_and_clean_branch(self) -> None:
        commit = "a" * 40
        env = {
            "ASSURA_NU11_COMMIT": commit,
            "ASSURA_NU11_MODE": "callgrind",
            "GITHUB_SHA": commit,
            "GITHUB_REF": "refs/heads/feat/nu11",
        }
        self.assertEqual(NU11.validate_source(REPOSITORY_ROOT, env, commit, ""), commit)
        invalid_cases = [
            ({**env, "GITHUB_SHA": "b" * 40}, commit, ""),
            (env, "b" * 40, ""),
            ({**env, "GITHUB_REF": "refs/heads/master"}, commit, ""),
            ({**env, "GITHUB_REF": "refs/tags/v0"}, commit, ""),
            (env, commit, " M src/main.rs"),
        ]
        for invalid_env, head, status in invalid_cases:
            with self.subTest(head=head, status=status):
                with self.assertRaises(NU11.GateFailure):
                    NU11.validate_source(REPOSITORY_ROOT, invalid_env, head, status)

    def test_fixture_verifier_binds_exact_800_scope_inputs_and_rejects_corruption(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / ".assura").mkdir()
            files = []
            for relative, body in ((".ls-lint.yml", "ls:\n  .dir: kebab-case\n"),
                                   (".assura/config.yml", "structure:\n  ./: {}\n")):
                path = root / relative
                path.write_text(body, encoding="utf-8")
                files.append({"path": relative, "sha256": NU11.sha256_file(path), "size_bytes": path.stat().st_size})
            for index in range(800):
                relative = f"pkg-{index:04}/file-name.js"
                path = root / relative
                path.parent.mkdir()
                path.write_bytes(b"")
                files.append({"path": relative, "sha256": NU11.sha256_file(path), "size_bytes": 0})
            manifest = {
                "schema": "assura.nu11.fixture.v1",
                "source": {
                    "commit": "a" * 40,
                    "constructor": {
                        "path": "src/cli/performance_report/counterexample_fixtures.rs",
                        "sha256": NU11.sha256_file(REPOSITORY_ROOT / "src/cli/performance_report/counterexample_fixtures.rs"),
                    },
                    "exporter": {
                        "path": "src/cli/performance_report/fixture_tests.rs",
                        "sha256": NU11.sha256_file(REPOSITORY_ROOT / "src/cli/performance_report/fixture_tests.rs"),
                    },
                },
                "package_directory_count": 800,
                "package_file_count": 800,
                "rule_entry_count": 801,
                "assura_policy_nonempty": True,
                "configs": {
                    "ls_lint": {"path": ".ls-lint.yml", "sha256": NU11.sha256_file(root / ".ls-lint.yml")},
                    "assura": {"path": ".assura/config.yml", "sha256": NU11.sha256_file(root / ".assura/config.yml")},
                },
                "files": files,
            }
            (root / "nu11-fixture-manifest.json").write_text(json.dumps(manifest), encoding="utf-8")
            self.assertEqual(len(NU11.verify_fixture(root, REPOSITORY_ROOT, "a" * 40)["files"]), 802)

            (root / "pkg-0000/file-name.js").write_bytes(b"drift")
            with self.assertRaises(NU11.GateFailure):
                NU11.verify_fixture(root, REPOSITORY_ROOT, "a" * 40)

    def test_vendor_total_reader_rejects_missing_summary(self) -> None:
        self.assertEqual(NU11.annotator_total_ir("  12,345 (100.0%) PROGRAM TOTALS\n"), 12345)
        with self.assertRaises(NU11.GateFailure):
            NU11.annotator_total_ir("no profile totals here\n")

    def test_capture_retains_raw_streams_and_marks_overflow_before_rejection(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            evidence = NU11.Evidence(root, {"PATH": os.environ.get("PATH", "")}, REPOSITORY_ROOT)
            result = evidence.capture(
                "bounded-output",
                [sys.executable, "-c", "import sys; print('x' * 2048); print('err', file=sys.stderr)"],
                cwd=REPOSITORY_ROOT,
                timeout=5,
                limit=128,
            )
            self.assertTrue(result.overflow)
            self.assertTrue((root / result.stdout).is_file())
            self.assertTrue((root / result.stderr).is_file())
            stdout = root / result.stdout
            stderr = root / result.stderr
            self.assertEqual(stdout.stat().st_size + stderr.stat().st_size, 128)
            self.assertEqual(result.stdout_bytes, stdout.stat().st_size)
            self.assertEqual(result.stderr_bytes, stderr.stat().st_size)
            self.assertEqual(result.stdout_sha256, NU11.sha256_file(stdout))
            self.assertEqual(result.stderr_sha256, NU11.sha256_file(stderr))
            self.assertGreater(result.stdout_observed_bytes + result.stderr_observed_bytes, 128)
            self.assertEqual(result.stop_reason, "captured_output_overflow")
            self.assertFalse(result.timed_out)
            self.assertTrue(result.cleanup_complete)
            self.assertEqual(result.pid, result.pgid)
            stored = json.loads((root / "run.json").read_text())["commands"][0]
            self.assertEqual(stored["lifecycle"], "terminal")
            self.assertEqual(stored["cleanup_status"], "complete")
            with self.assertRaises(NU11.GateFailure):
                evidence.require(result)

    def test_normal_nonzero_exit_is_preserved_without_timeout_or_overflow(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            evidence, result = self.capture(Path(temporary), "nonzero", "raise SystemExit(7)")
            self.assertEqual(result.exit_code, 7)
            self.assertFalse(result.timed_out)
            self.assertFalse(result.overflow)
            self.assertIsNone(result.stop_reason)
            self.assertTrue(result.cleanup_complete)
            with self.assertRaises(NU11.GateFailure):
                evidence.require(result)

    def test_deadline_timeout_is_distinct_and_reaps_the_group(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            _, result = self.capture(Path(temporary), "timeout", "import time; time.sleep(10)", timeout=0.15)
            self.assertTrue(result.timed_out)
            self.assertFalse(result.overflow)
            self.assertEqual(result.stop_reason, "command_timeout")
            self.assertTrue(result.cleanup_complete)
            self.assertTrue(any(signal.startswith("SIGTERM:") for signal in result.termination_signals))

    def test_profile_file_overflow_is_not_reported_as_a_timeout(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            produced = root / "profile.out"
            code = f"from pathlib import Path; import time; p=Path({str(produced)!r}); p.write_bytes(b'x'*4096); time.sleep(10)"
            _, result = self.capture(
                root, "profile-overflow", code, timeout=3, produced_file=produced,
                produced_file_limit=32,
            )
            self.assertTrue(result.overflow)
            self.assertFalse(result.timed_out)
            self.assertEqual(result.stop_reason, "profile_output_overflow")
            self.assertTrue(result.cleanup_complete)

    def test_term_resistant_child_and_descendant_are_killed_within_a_bound(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            child = (
                "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(30)"
            )
            code = (
                "import signal,subprocess,sys,time; "
                "signal.signal(signal.SIGTERM, signal.SIG_IGN); "
                f"subprocess.Popen([sys.executable, '-c', {child!r}]); "
                "print('started', flush=True); time.sleep(30)"
            )
            started = time.monotonic()
            _, result = self.capture(Path(temporary), "resistant", code, timeout=0.15)
            self.assertLess(time.monotonic() - started, 3)
            self.assertEqual(result.stop_reason, "command_timeout")
            self.assertTrue(result.cleanup_complete)
            self.assertIn("SIGKILL:sent", result.termination_signals)

    def test_exited_parent_with_descendant_held_pipe_is_stopped_and_recorded(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            child = "import time; time.sleep(30)"
            code = f"import subprocess,sys; subprocess.Popen([sys.executable, '-c', {child!r}])"
            _, result = self.capture(Path(temporary), "held-pipe", code, timeout=0.2)
            self.assertTrue(result.timed_out)
            self.assertEqual(result.stop_reason, "command_timeout")
            self.assertTrue(result.cleanup_complete)
            self.assertTrue(result.direct_child_reaped)
            self.assertTrue(result.process_group_gone)

    def test_global_job_deadline_caps_a_command_timeout(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            evidence = NU11.Evidence(Path(temporary), {"PATH": os.environ.get("PATH", "")}, REPOSITORY_ROOT)
            evidence.started = time.monotonic() - NU11.JOB_TIMEOUT + 0.15
            with self.assertRaisesRegex(NU11.GateFailure, "45-minute budget"):
                evidence.capture(
                    "job-budget", [sys.executable, "-c", "import time; time.sleep(10)"],
                    cwd=REPOSITORY_ROOT, timeout=5,
                )
            stored = json.loads((Path(temporary) / "run.json").read_text())["commands"][0]
            self.assertEqual(stored["lifecycle"], "terminal")
            self.assertEqual(stored["stop_reason"], "job_budget_timeout")
            self.assertTrue(stored["timed_out"])
            self.assertEqual(stored["pid"], stored["pgid"])
            self.assertTrue(stored["cleanup_complete"])
            self.assertEqual(stored["cleanup_status"], "complete")

    def test_initial_receipt_failure_still_kills_child_and_keeps_cleanup_unknown(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            evidence = NU11.Evidence(root, {"PATH": os.environ.get("PATH", "")}, REPOSITORY_ROOT)
            code = "import time; time.sleep(30)"
            with patch.object(evidence, "save", side_effect=OSError("injected initial receipt failure")):
                with self.assertRaisesRegex(OSError, "injected initial receipt failure"):
                    evidence.capture(
                        "initial-save-failure", [sys.executable, "-c", code],
                        cwd=REPOSITORY_ROOT, timeout=3,
                    )
            record = evidence.commands[0]
            self.assertEqual(record["lifecycle"], "aborted")
            self.assertEqual(record["cleanup_status"], "unknown")
            self.assertIn("receipt_write_error", record)
            self.assertEqual(record["pid"], record["pgid"])
            self.assertIs(NU11._process_group_gone(record["pgid"]), True)
            self.assertFalse((root / "run.json").exists())

    def test_capture_file_open_failure_cleans_up_and_persists_abort(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            evidence = NU11.Evidence(root, {"PATH": os.environ.get("PATH", "")}, REPOSITORY_ROOT)
            original_open = Path.open
            stderr_name = "setup-open-failure.stderr"

            def fail_stderr_open(path: Path, *args, **kwargs):
                if path.name == stderr_name:
                    raise OSError("injected stderr capture open failure")
                return original_open(path, *args, **kwargs)

            with patch.object(Path, "open", fail_stderr_open):
                with self.assertRaisesRegex(OSError, "injected stderr capture open failure"):
                    evidence.capture(
                        "setup-open-failure", [sys.executable, "-c", "import time; time.sleep(30)"],
                        cwd=REPOSITORY_ROOT, timeout=3,
                    )
            stored = json.loads((root / "run.json").read_text())["commands"][0]
            self.assertEqual(stored["lifecycle"], "aborted")
            self.assertEqual(stored["failure"], "OSError: injected stderr capture open failure")
            self.assertTrue(stored["direct_child_reaped"])
            self.assertIs(stored["process_group_gone"], True)
            self.assertEqual(stored["cleanup_status"], "complete")

    def test_launch_pid_pgid_and_unknown_cleanup_are_persisted_before_exit(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            evidence = NU11.Evidence(root, {"PATH": os.environ.get("PATH", "")}, REPOSITORY_ROOT)
            results = []
            thread = threading.Thread(target=lambda: results.append(evidence.capture(
                "launch-receipt", [sys.executable, "-c", "import time; time.sleep(0.3)"],
                cwd=REPOSITORY_ROOT, timeout=2,
            )))
            thread.start()
            deadline = time.monotonic() + 1
            launch_record = None
            while time.monotonic() < deadline:
                try:
                    commands = json.loads((root / "run.json").read_text())["commands"]
                    if commands:
                        launch_record = commands[0]
                        break
                except (FileNotFoundError, json.JSONDecodeError):
                    pass
                time.sleep(0.01)
            self.assertIsNotNone(launch_record)
            assert launch_record is not None
            self.assertEqual(launch_record["lifecycle"], "running")
            self.assertEqual(launch_record["cleanup_status"], "unknown")
            self.assertEqual(launch_record["pid"], launch_record["pgid"])
            thread.join(timeout=2)
            self.assertFalse(thread.is_alive())
            self.assertEqual(len(results), 1)
            self.assertTrue(results[0].cleanup_complete)


if __name__ == "__main__":
    unittest.main()
