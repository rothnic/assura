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
from types import SimpleNamespace
from unittest.mock import mock_open, patch


REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
SCRIPT = REPOSITORY_ROOT / "scripts" / "nu11-callgrind-profile.py"
SPEC = importlib.util.spec_from_file_location("nu11_callgrind_profile", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
NU11 = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = NU11
SPEC.loader.exec_module(NU11)


class Nu11ProfileContractTests(unittest.TestCase):
    def test_container_identity_decodes_quotes_and_rejects_wrong_values(self):
        cases = [
            ('ID=debian\nVERSION_ID="12"\n', True),
            ('ID="debian"\nVERSION_ID=12\n', True),
            ('ID=ubuntu\nVERSION_ID="12"\n', False),
            ('ID=debian\nVERSION_ID="120"\n', False),
            ('ID=debian\n', False),
        ]
        for raw, accepted in cases:
            with self.subTest(raw=raw), tempfile.TemporaryDirectory() as temporary:
                evidence = NU11.Evidence(Path(temporary), {}, REPOSITORY_ROOT)
                # Exercise the public stdlib decoder against the real assignment text.
                with patch.object(NU11.platform, "_os_release_cache", None), \
                        patch("builtins.open", mock_open(read_data=raw)), \
                        patch.object(NU11.Path, "read_bytes", return_value=raw.encode()):
                    if accepted:
                        NU11.require_container_identity(evidence)
                    else:
                        with self.assertRaisesRegex(NU11.GateFailure, "pinned Debian"):
                            NU11.require_container_identity(evidence)
                self.assertEqual((Path(temporary) / "os-release").read_bytes(), raw.encode())
                state = json.loads((Path(temporary) / "run.json").read_text())
                self.assertEqual(state["os_release_sha256"], NU11.sha256_bytes(raw.encode()))
                self.assertEqual(state["os_release"]["ID"], "debian" if "ubuntu" not in raw else "ubuntu")

    def resources(self):
        return {"memory_total": 1000, "memory_available": 500, "load_1m": 1,
                "cpu_count": 4, "disk_free": NU11.MIN_DISK_FREE,
                "psi_full_avg10": {"memory": 0, "io": 0}}

    def test_resource_pressure_rejects_memory_io_and_disk_and_unknown_counters(self):
        healthy = self.resources()
        self.assertEqual(NU11.resource_rejections(healthy), [])
        for bad in [{**healthy, "memory_available": 99},
                    {**healthy, "disk_free": NU11.MIN_DISK_FREE - 1},
                    {**healthy, "psi_full_avg10": {"memory": 5, "io": 0}},
                    {**healthy, "psi_full_avg10": {"memory": 0, "io": 5}}]:
            self.assertTrue(NU11.resource_rejections(bad))
        with self.assertRaises(NU11.GateFailure):
            NU11.resource_rejections({**healthy, "load_1m": float("nan")})

    def test_resource_guard_requires_two_high_load_samples_and_retains_rejection(self):
        with tempfile.TemporaryDirectory() as temporary:
            evidence = NU11.Evidence(Path(temporary), {}, REPOSITORY_ROOT)
            high = {**self.resources(), "load_1m": 9}
            with patch.object(NU11, "read_host_resources", side_effect=[high.copy(), high.copy()]), \
                    patch.object(NU11.time, "sleep") as sleep:
                with self.assertRaisesRegex(NU11.GateFailure, "two samples"):
                    NU11.record_resource_guard(evidence, "load", REPOSITORY_ROOT)
            sleep.assert_called_once_with(30)
            stored = json.loads((Path(temporary) / "run.json").read_text())
            self.assertEqual(len(stored["resource_snapshots"]), 2)
            self.assertTrue(stored["resource_snapshots"]["load-load-confirmation"]["rejection_reasons"])

    def test_transient_load_can_clear_and_missing_resource_data_stops(self):
        with tempfile.TemporaryDirectory() as temporary:
            evidence = NU11.Evidence(Path(temporary), {}, REPOSITORY_ROOT)
            with patch.object(NU11, "read_host_resources", side_effect=[{**self.resources(), "load_1m": 9}, self.resources()]), \
                    patch.object(NU11.time, "sleep"):
                NU11.record_resource_guard(evidence, "transient", REPOSITORY_ROOT)
            with patch.object(NU11, "read_host_resources", side_effect=OSError("missing PSI")):
                with self.assertRaisesRegex(NU11.GateFailure, "unavailable"):
                    NU11.record_resource_guard(evidence, "unknown", REPOSITORY_ROOT)
            self.assertEqual(evidence.state["resource_snapshots"]["unknown"]["outcome"], "unknown")

    def test_bootstrap_io_recheck_retains_initial_rejection_and_admits_after_one_idle(self):
        with tempfile.TemporaryDirectory() as temporary:
            evidence = NU11.Evidence(Path(temporary), {}, REPOSITORY_ROOT)
            io = {**self.resources(), "psi_full_avg10": {"memory": 0, "io": 15.86}}
            with patch.object(NU11, "read_host_resources", side_effect=[io, self.resources()]), patch.object(NU11.time, "sleep") as sleep:
                NU11.admit_after_bootstrap(evidence, REPOSITORY_ROOT)
            sleep.assert_called_once_with(30)
            state = json.loads((Path(temporary) / "run.json").read_text())
            self.assertTrue(state["resource_snapshots"]["before-runtime-tools"]["rejection_reasons"])
            self.assertEqual(state["bootstrap_admission_recheck"]["outcome"], "admitted_after_idle")
            self.assertEqual(len(state["resource_snapshots"]), 2)

    def test_bootstrap_second_io_rejection_is_terminal(self):
        with tempfile.TemporaryDirectory() as temporary:
            evidence = NU11.Evidence(Path(temporary), {}, REPOSITORY_ROOT)
            io = {**self.resources(), "psi_full_avg10": {"memory": 0, "io": 15.86}}
            with patch.object(NU11, "read_host_resources", side_effect=[io.copy(), io.copy()]) as read, patch.object(NU11.time, "sleep") as sleep:
                with self.assertRaisesRegex(NU11.GateFailure, "resource pressure"):
                    NU11.admit_after_bootstrap(evidence, REPOSITORY_ROOT)
            self.assertEqual(read.call_count, 2)
            sleep.assert_called_once_with(30)
            self.assertEqual(evidence.state["bootstrap_admission_recheck"]["outcome"], "rejected_or_unknown")

    def test_bootstrap_non_io_mixed_and_unknown_rejections_never_wait(self):
        for snapshot in [{**self.resources(), "memory_available": 1},
                         {**self.resources(), "psi_full_avg10": {"memory": 5, "io": 15.86}},
                         {**self.resources(), "load_1m": 9, "psi_full_avg10": {"memory": 0, "io": 15.86}},
                         {**self.resources(), "load_1m": float("nan")}]:
            with self.subTest(snapshot=snapshot), tempfile.TemporaryDirectory() as temporary:
                evidence = NU11.Evidence(Path(temporary), {}, REPOSITORY_ROOT)
                with patch.object(NU11, "read_host_resources", return_value=snapshot) as read, patch.object(NU11.time, "sleep") as sleep:
                    with self.assertRaises(NU11.GateFailure):
                        NU11.admit_after_bootstrap(evidence, REPOSITORY_ROOT)
                self.assertEqual(read.call_count, 1)
                sleep.assert_not_called()

    def test_bootstrap_io_recheck_requires_budget_before_sleep(self):
        with tempfile.TemporaryDirectory() as temporary:
            evidence = NU11.Evidence(Path(temporary), {}, REPOSITORY_ROOT)
            evidence.started -= NU11.JOB_TIMEOUT - 20
            io = {**self.resources(), "psi_full_avg10": {"memory": 0, "io": 15.86}}
            with patch.object(NU11, "read_host_resources", return_value=io) as read, patch.object(NU11.time, "sleep") as sleep:
                with self.assertRaisesRegex(NU11.GateFailure, "insufficient job budget"):
                    NU11.admit_after_bootstrap(evidence, REPOSITORY_ROOT)
            self.assertEqual(read.call_count, 1)
            sleep.assert_not_called()

    def test_filename_negative_rejects_abnormal_capture_even_with_expected_json(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / "negative.stdout").write_text(json.dumps({"success": False, "violations": [{"path": "FileName.js"}]}))
            evidence = NU11.Evidence(root, {}, REPOSITORY_ROOT)
            fields = {"name": "filename-negative-control", "stdout": "negative.stdout", "exit_code": 1, "timed_out": False,
                      "overflow": False, "stop_reason": None, "cleanup_complete": True}
            for exit_code in (0, 1):
                NU11.require_filename_negative(evidence, SimpleNamespace(**{**fields, "exit_code": exit_code}), root)
            for changes in [{"stop_reason": "descendant_process_remains"}, {"cleanup_complete": False},
                            {"timed_out": True}, {"overflow": True}, {"exit_code": 2}]:
                with self.assertRaises(NU11.GateFailure):
                    NU11.require_filename_negative(evidence, SimpleNamespace(**{**fields, **changes}), root)

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

    def test_pressure_during_a_long_capture_stops_and_reaps_the_owned_child(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            evidence = NU11.Evidence(root, {"PATH": os.environ.get("PATH", "")}, REPOSITORY_ROOT)
            with patch.object(NU11, "RESOURCE_CHECK_INTERVAL", 0), \
                    patch.object(NU11, "record_resource_guard", side_effect=NU11.GateFailure("injected pressure")) as guard:
                with self.assertRaisesRegex(NU11.GateFailure, "injected pressure"):
                    evidence.capture("pressure", [sys.executable, "-c", "import time; time.sleep(10)"],
                                     cwd=REPOSITORY_ROOT, timeout=3)
            guard.assert_called_once()
            stored = json.loads((root / "run.json").read_text())["commands"][0]
            self.assertEqual(stored["lifecycle"], "aborted")
            self.assertTrue(stored["cleanup_complete"])

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
            with self.assertRaisesRegex(NU11.GateFailure, "active run budget"):
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
