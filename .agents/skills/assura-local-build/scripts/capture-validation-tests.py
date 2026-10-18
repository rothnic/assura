#!/usr/bin/env python3
"""Focused controls for capture-validation.py; uses only owned temporary children."""

from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import uuid


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


def stop_owned_fixture(pid: int | None) -> str:
    if pid is None:
        return "no child PID observed"
    state = process_state(pid)
    if state is None or state.startswith("Z"):
        return f"stopped ({state or 'absent'})"
    os.kill(pid, signal.SIGKILL)
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        state = process_state(pid)
        if state is None or state.startswith("Z"):
            return f"stopped ({state or 'absent'})"
        time.sleep(0.02)
    raise AssertionError(f"owned fixture PID {pid} still active in state {state}")


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
            stop_owned_fixture(child_pid)
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
        return stop_owned_fixture(child_pid)
    finally:
        try:
            finish_wrapper(root, label, wrapper)
        finally:
            stop_owned_fixture(child_pid)


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

        succeeded = True
        print("CONTROL_GROUPS=13")
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


if __name__ == "__main__":
    raise SystemExit(main())
