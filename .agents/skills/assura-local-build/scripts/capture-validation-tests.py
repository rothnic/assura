#!/usr/bin/env python3
"""Focused controls for capture-validation.py; uses only owned temporary children."""

from __future__ import annotations

import json
import contextlib
import importlib.util
import io
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import uuid
from unittest import mock


HERE = Path(__file__).resolve().parent
CAPTURE = HERE / "capture-validation.py"
PREFLIGHT = HERE / "local-preflight.py"


def archive_invocation(root: Path, label: str, result: subprocess.CompletedProcess, created: list[Path]) -> None:
    diagnostics = root / "diagnostics"
    diagnostics.mkdir(exist_ok=True)
    (diagnostics / f"{label}.wrapper.stdout.bin").write_bytes(result.stdout)
    (diagnostics / f"{label}.wrapper.stderr.bin").write_bytes(result.stderr)
    for run_dir in created:
        saved = diagnostics / run_dir.name
        saved.mkdir(exist_ok=True)
        for name in ("receipt.json", "stdout.bin", "stderr.bin"):
            source = run_dir / name
            if source.exists():
                shutil.copyfile(source, saved / name)


def invoke(
    root: Path,
    *command: str,
    label: str = "test",
    extra: tuple[str, ...] = (),
    expected_create: bool = True,
    repo_arg: Path | None = None,
) -> tuple[subprocess.CompletedProcess, Path | None]:
    evidence_root = root / "evidence"
    before = {item.name for item in evidence_root.iterdir()}
    result = subprocess.run(
        [sys.executable, str(CAPTURE), "--repo", str(repo_arg or root), "--evidence-root", str(evidence_root),
         "--label", label, *extra, "--", *command],
        cwd=root, env={**os.environ, "CAPTURE_TEST_VALUE": "selected-value"},
        stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
    )
    created = [item for item in evidence_root.iterdir() if item.name not in before]
    archive_invocation(root, label, result, created)
    if expected_create:
        assert len(created) == 1, created
        return result, created[0]
    assert not created, created
    return result, None


def receipt(root: Path, run_dir: Path, label: str, command: list[str]) -> dict:
    assert run_dir.parent == root / "evidence"
    assert run_dir.name.startswith(label + "-")
    record = json.loads((run_dir / "receipt.json").read_text())
    assert record["run_id"] == run_dir.name
    assert record["cwd"] == str(root)
    assert record["argv"] == command
    return record


def process_state(pid: int) -> str | None:
    result = subprocess.run(["ps", "-o", "stat=", "-p", str(pid)], capture_output=True, text=True, check=False)
    value = result.stdout.strip()
    return value or None


def stop_owned_fixture(pid: int | None, nonce: str | None = None) -> str:
    if pid is None:
        return "no child PID observed"
    state = process_state(pid)
    if state is None or state.startswith("Z"):
        return f"stopped ({state or 'absent'})"
    command = subprocess.run(["ps", "-o", "command=", "-p", str(pid)], capture_output=True, text=True, check=False).stdout
    if nonce is None or nonce not in command:
        return f"not signalled (PID identity did not contain fixture nonce; state={state})"
    os.kill(pid, signal.SIGKILL)
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        state = process_state(pid)
        if state is None or state.startswith("Z"):
            return f"stopped ({state or 'absent'})"
        time.sleep(0.02)
        raise AssertionError(f"nonce-verified fixture PID {pid} still active in state {state}")


def finish_wrapper(root: Path, label: str, wrapper: subprocess.Popen | None) -> None:
    if wrapper is None:
        return
    diagnostics = root / "diagnostics"
    diagnostics.mkdir(exist_ok=True)
    if wrapper.poll() is None:
        wrapper.send_signal(signal.SIGTERM)
    try:
        stdout, stderr = wrapper.communicate(timeout=6)
    except subprocess.TimeoutExpired as timeout:
        (diagnostics / f"{label}.cleanup-partial.stdout.bin").write_bytes(timeout.stdout or b"")
        (diagnostics / f"{label}.cleanup-partial.stderr.bin").write_bytes(timeout.stderr or b"")
        wrapper.kill()
        stdout, stderr = wrapper.communicate(timeout=2)
    (diagnostics / f"{label}.cleanup.stdout.bin").write_bytes(stdout or b"")
    (diagnostics / f"{label}.cleanup.stderr.bin").write_bytes(stderr or b"")


def start_interrupt_case(
    root: Path, label: str, command: list[str], nonce: str, ready_path: Path
) -> tuple[Path, subprocess.Popen, int]:
    evidence_root = root / "evidence"
    before = {item.name for item in evidence_root.iterdir()}
    wrapper = subprocess.Popen(
        [sys.executable, str(CAPTURE), "--repo", str(root), "--evidence-root", str(evidence_root),
         "--label", label, "--", *command],
        cwd=root, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
    )
    run_dir: Path | None = None
    child_pid: int | None = None
    try:
        deadline = time.monotonic() + 8
        while time.monotonic() < deadline:
            created = [item for item in evidence_root.iterdir() if item.name not in before]
            if len(created) > 1:
                raise AssertionError(f"multiple run directories for {label}: {created}")
            if created and (created[0] / "receipt.json").exists():
                run_dir = created[0]
                record = json.loads((run_dir / "receipt.json").read_text())
                assert record["run_id"] == run_dir.name and run_dir.name.startswith(label + "-")
                assert record["cwd"] == str(root) and record["argv"] == command
                child_pid = record["direct_child_pid"]
                if record["state"] == "running" and child_pid is not None:
                    if ready_path.exists() and ready_path.read_text() == nonce:
                        return run_dir, wrapper, child_pid
            if wrapper.poll() is not None:
                raise AssertionError(f"capture wrapper exited before fixture ready: {wrapper.returncode}")
            time.sleep(0.02)
        raise AssertionError(f"fixture readiness handshake timed out for {label}")
    except BaseException:
        try:
            finish_wrapper(root, label, wrapper)
        finally:
            stop_owned_fixture(child_pid, nonce)
        raise


def interrupted_case(
    root: Path,
    label: str,
    code: str,
    expected_return: int | None,
    expected_live: bool,
    expected_shell_status: int = 128 + signal.SIGTERM,
) -> str:
    nonce = str(uuid.uuid4())
    ready_path = root / f"{label}.ready"
    fixture = (
        "import pathlib,signal,sys,time;"
        + code
        + f";pathlib.Path({str(ready_path)!r}).write_text({nonce!r});time.sleep(30)"
    )
    command = [sys.executable, "-c", fixture]
    wrapper: subprocess.Popen | None = None
    child_pid: int | None = None
    try:
        run_dir, wrapper, child_pid = start_interrupt_case(root, label, command, nonce, ready_path)
        wrapper.send_signal(signal.SIGTERM)
        wrapper_stdout, wrapper_stderr = wrapper.communicate(timeout=7)
        archive_invocation(
            root, label,
            subprocess.CompletedProcess(wrapper.args, wrapper.returncode, wrapper_stdout, wrapper_stderr),
            [run_dir],
        )
        saved = receipt(root, run_dir, label, command)
        assert wrapper.returncode == expected_shell_status
        assert saved["state"] == "interrupted" and saved["returncode"] == expected_return
        assert saved["child_live"] is expected_live
        return stop_owned_fixture(child_pid, nonce)
    finally:
        try:
            finish_wrapper(root, label, wrapper)
        finally:
            stop_owned_fixture(child_pid, nonce)


def main() -> int:
    root = Path(tempfile.mkdtemp(prefix="dx03-capture-"))
    succeeded = False
    try:
        (root / "evidence").mkdir(mode=0o700)
        source = root / "source.txt"
        source.write_bytes(b"hash me\x00")
        payload = (
            "import os,sys;sys.stdout.buffer.write(b'out\\x00bytes');"
            "sys.stderr.buffer.write(b'err\\xffbytes');"
            "assert os.environ['CAPTURE_TEST_VALUE']=='selected-value';"
            "sys.exit(23 if sys.argv[1:] == ['space value','--','tail'] else 91)"
        )
        command = [sys.executable, "-c", payload, "space value", "--", "tail"]
        first, run_dir = invoke(
            root, *command, extra=("--record-env", "CAPTURE_TEST_VALUE", "--hash-file", "source.txt")
        )
        assert run_dir is not None
        saved = receipt(root, run_dir, "test", command)
        assert first.returncode == 23 and saved["state"] == "complete" and saved["returncode"] == 23
        assert saved["recorded_env"] == {"CAPTURE_TEST_VALUE": "selected-value"}
        assert saved["sha256"]["source.txt"] == "ff82f63706a47ba3951fb342a3905055a6a916fabd35941e5048ffa3eec8f622"
        assert (run_dir / "stdout.bin").read_bytes() == b"out\x00bytes"
        assert (run_dir / "stderr.bin").read_bytes() == b"err\xffbytes"
        before = (run_dir / "receipt.json").read_bytes()

        second_command = [sys.executable, "-c", "pass"]
        second, second_dir = invoke(root, *second_command)
        assert second_dir is not None
        assert second.returncode == 0 and (run_dir / "receipt.json").read_bytes() == before
        receipt(root, second_dir, "test", second_command)

        marker = root / "must-not-exist"
        invalid, _ = invoke(
            root, sys.executable, "-c", f"open({str(marker)!r},'w').close()",
            extra=("--record-env", "API_TOKEN"), expected_create=False,
        )
        assert invalid.returncode == 2 and not marker.exists()
        bad_hash, _ = invoke(
            root, sys.executable, "-c", "pass", extra=("--hash-file", "../outside"), expected_create=False
        )
        assert bad_hash.returncode == 2
        (root / "source-link").symlink_to("/etc/hosts")
        bad_symlink, _ = invoke(
            root, sys.executable, "-c", "pass", extra=("--hash-file", "source-link"), expected_create=False
        )
        assert bad_symlink.returncode == 2
        bad_label, _ = invoke(root, sys.executable, "-c", "pass", label="../bad", expected_create=False)
        assert bad_label.returncode == 2
        empty_command, _ = invoke(root, label="empty-command", expected_create=False)
        assert empty_command.returncode == 2
        unowned_repo, _ = invoke(
            root, sys.executable, "-c", "pass", repo_arg=Path("/System"), expected_create=False
        )
        assert unowned_repo.returncode == 2

        wrong_repo = root / "wrong-repo"
        wrong_repo.mkdir()
        marker.write_text("", encoding="utf-8")
        payload_command = [sys.executable, "-c", f"open({str(marker)!r},'w').write('launched')"]
        preflight_command = [sys.executable, str(PREFLIGHT), "--repo", str(wrong_repo), "--", *payload_command]
        preflight, preflight_dir = invoke(root, *preflight_command, label="preflight-reject")
        assert preflight_dir is not None
        preflight_record = receipt(root, preflight_dir, "preflight-reject", preflight_command)
        helper_record = json.loads((preflight_dir / "stderr.bin").read_text())
        assert preflight.returncode != 0 and preflight_record["returncode"] == preflight.returncode
        assert helper_record["ok"] is False and helper_record["command_started"] is False
        assert helper_record["argv"] == payload_command
        assert helper_record["cwd"] == str(root)
        assert helper_record["error"] == "caller cwd differs from --repo; no directory change was made"
        assert marker.read_text(encoding="utf-8") == ""

        zero_command = [sys.executable, "-c", "print('0 tests selected')"]
        zero_looking, zero_dir = invoke(root, *zero_command, label="zero-looking")
        assert zero_dir is not None
        zero_record = receipt(root, zero_dir, "zero-looking", zero_command)
        assert zero_looking.returncode == 0 and "test_count" not in zero_record and "passed" not in zero_record

        if os.name == "posix":
            known_exit = "import signal,sys;signal.signal(signal.SIGTERM,lambda *_:sys.exit(0))"
            known_teardown = interrupted_case(root, "interrupt-known", known_exit, 0, False)
            default_term = "import signal;signal.signal(signal.SIGTERM,signal.SIG_DFL)"
            signal_teardown = interrupted_case(root, "interrupt-signal-exit", default_term, -signal.SIGTERM, False)
            stubborn = "import signal;signal.signal(signal.SIGTERM,signal.SIG_IGN)"
            unknown_teardown = interrupted_case(root, "interrupt-unknown", stubborn, None, True)
            print(f"fixture teardown: known={known_teardown}; signal={signal_teardown}; unknown={unknown_teardown}")

        platform_guard_control()
        print("platform guard control: PASS (mocked non-POSIX; no Windows execution claimed)")
        fault_evidence = Path(os.environ.get("DX03_FAULT_EVIDENCE_ROOT", root / "fault-evidence"))
        fault_evidence.mkdir(mode=0o700)
        running_fault = fault_injection_control("running", fault_evidence)
        terminal_fault = fault_injection_control("complete", fault_evidence)
        assert running_fault == "running_receipt" and terminal_fault == "terminal_receipt"
        print("receipt persistence fault controls: running and terminal PASS")
        succeeded = True
        print("CONTROL_GROUPS=16")
        print("FAULT_STATES=2")
        print("capture-validation focused controls: PASS")
        print(f"owned fixture PID: {os.getpid()}; retained root: {root}")
        return 0
    except BaseException:
        print(f"capture-validation controls failed; preserved fixture root: {root}", file=sys.stderr)
        raise
    finally:
        if succeeded:
            shutil.rmtree(root)
            print(f"cleaned exact owned fixture root after teardown: {root}")


def fault_injection_control(fail_state: str, evidence: Path) -> str:
    """Inject one receipt persistence failure while retaining child diagnostics."""
    if fail_state not in {"running", "complete"}:
        raise ValueError(f"unsupported fault state: {fail_state}")
    evidence.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix=f"receipt-{fail_state}-"))
    (root / "evidence").mkdir(mode=0o700)
    snapshots: list[dict] = []
    child_holder: list[subprocess.Popen] = []
    capture_spec = importlib.util.spec_from_file_location("capture_validation", CAPTURE)
    assert capture_spec and capture_spec.loader
    capture = importlib.util.module_from_spec(capture_spec)
    capture_spec.loader.exec_module(capture)
    original_save = capture.save_receipt
    real_popen = subprocess.Popen

    def injected_save(run_dir: Path, record: dict) -> None:
        snapshots.append(json.loads(json.dumps(record)))
        if record.get("state") == fail_state:
            raise OSError(f"injected {fail_state} receipt write failure")
        original_save(run_dir, record)

    def tracked_popen(*args: object, **kwargs: object) -> subprocess.Popen:
        child = real_popen(*args, **kwargs)
        child_holder.append(child)
        return child

    command = [sys.executable, "-c", "import time; time.sleep(30)"] if fail_state == "running" else [sys.executable, "-c", "pass"]
    sys.argv = [str(CAPTURE), "--repo", str(root), "--evidence-root", str(root / "evidence"),
                "--label", f"fault-{fail_state}", "--", *command]
    stdout_capture, stderr_capture = io.StringIO(), io.StringIO()
    returned: int | None = None
    raised: str | None = None
    run_dir: Path | None = None
    try:
        with mock.patch.object(capture, "save_receipt", injected_save), \
             mock.patch.object(capture.subprocess, "Popen", tracked_popen), \
             contextlib.redirect_stdout(stdout_capture), contextlib.redirect_stderr(stderr_capture):
            try:
                returned = capture.main()
            except BaseException as exc:
                raised = f"{type(exc).__name__}: {exc}"
        runs = list((root / "evidence").iterdir())
        run_dir = runs[0] if len(runs) == 1 else None
        child = child_holder[0] if child_holder else None
        # Observe the wrapper's cleanup before the fixture can help it. The
        # finally block owns emergency cleanup when this control fails.
        observed_rc = child.poll() if child is not None else None
        child_live: bool | None = observed_rc is None if child is not None else None
        raw_dir = evidence / f"{fail_state}-{uuid.uuid4()}"
        raw_dir.mkdir(mode=0o700)
        (raw_dir / "wrapper.stdout.bin").write_bytes(stdout_capture.getvalue().encode())
        (raw_dir / "wrapper.stderr.bin").write_bytes(stderr_capture.getvalue().encode())
        (raw_dir / "attempted-receipts.json").write_text(json.dumps(snapshots, indent=2) + "\n")
        if run_dir is not None:
            for name in ("receipt.json", "stdout.bin", "stderr.bin"):
                if (run_dir / name).exists():
                    (raw_dir / name).write_bytes((run_dir / name).read_bytes())
        final_receipt = None
        if run_dir is not None and (run_dir / "receipt.json").exists():
            final_receipt = json.loads((run_dir / "receipt.json").read_text())
        record = {
            "fault_state": fail_state,
            "wrapper_return": returned,
            "wrapper_raised": raised,
            "actual_child_returncode": observed_rc,
            "child_live_before_fixture_cleanup": child_live,
            "run_dir": str(run_dir) if run_dir else None,
            "final_receipt": final_receipt,
            "raw_dir": str(raw_dir),
        }
        (raw_dir / "result.json").write_text(json.dumps(record, indent=2) + "\n")
        print(json.dumps(record, sort_keys=True))
        assert returned is not None and returned != 0, "capture failure must return a nonzero wrapper status"
        assert raised is None, f"capture failure escaped wrapper: {raised}"
        assert child_live is False, "wrapper must stop its child before fixture cleanup"
        assert final_receipt is not None and final_receipt["state"] == "interrupted"
        expected_phase = {"running": "running_receipt", "complete": "terminal_receipt"}[fail_state]
        assert final_receipt["capture_failure_phase"] == expected_phase
        assert final_receipt["capture_error"] == f"injected {fail_state} receipt write failure"
        assert final_receipt["returncode"] == observed_rc and final_receipt["child_live"] is False
        if fail_state == "running":
            assert observed_rc is not None and child_live is False, "child must be stopped after running receipt failure"
        else:
            assert observed_rc == 0 and child_live is False, "actual child success must remain separate from capture failure"
        print(f"fault control {fail_state}: PASS; evidence: {raw_dir}")
        return expected_phase
    finally:
        child = child_holder[0] if child_holder else None
        if child is not None and child.poll() is None:
            child.kill()
            child.wait(timeout=2)
        sys.argv = [str(CAPTURE)]
        if run_dir is not None:
            shutil.rmtree(run_dir)
        shutil.rmtree(root)


def platform_guard_control() -> None:
    root = Path(tempfile.mkdtemp(prefix="dx03-platform-guard-"))
    (root / "evidence").mkdir(mode=0o700)
    capture_spec = importlib.util.spec_from_file_location("capture_validation_platform", CAPTURE)
    assert capture_spec and capture_spec.loader
    capture = importlib.util.module_from_spec(capture_spec)
    capture_spec.loader.exec_module(capture)
    old_argv = sys.argv
    sys.argv = [str(CAPTURE), "--repo", str(root), "--evidence-root", str(root / "evidence"),
                "--label", "platform-guard", "--", sys.executable, "-c", "raise SystemExit(99)"]
    stderr_capture = io.StringIO()
    try:
        with mock.patch.object(capture.os, "name", "nt"), \
             mock.patch.object(capture.subprocess, "Popen") as popen, \
             contextlib.redirect_stderr(stderr_capture):
            returned = capture.main()
        assert returned == 2
        assert "POSIX platforms only" in stderr_capture.getvalue()
        assert not list((root / "evidence").iterdir())
        popen.assert_not_called()
    finally:
        sys.argv = old_argv
        shutil.rmtree(root)


if __name__ == "__main__":
    raise SystemExit(main())
