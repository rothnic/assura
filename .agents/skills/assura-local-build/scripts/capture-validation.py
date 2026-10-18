#!/usr/bin/env python3
"""Capture one command invocation and its observed direct-child exit."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import uuid


SECRET_NAME = re.compile(r"(?:SECRET|TOKEN|PASSWORD|PASSWD|CREDENTIAL|PRIVATE.?KEY|API.?KEY|AUTH)", re.I)
ENV_NAME = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")
LABEL = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,63}$")


def timestamp() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def save_receipt(run_dir: Path, receipt: dict) -> None:
    temporary = run_dir / ".receipt.tmp"
    temporary.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(temporary, run_dir / "receipt.json")


def fail(message: str) -> int:
    print(f"capture-validation: {message}", file=sys.stderr)
    return 2


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", required=True)
    parser.add_argument("--evidence-root", required=True)
    parser.add_argument("--label", required=True)
    parser.add_argument("--record-env", action="append", default=[])
    parser.add_argument("--hash-file", action="append", default=[])
    parser.add_argument("command", nargs=argparse.REMAINDER)
    args = parser.parse_args()

    if not LABEL.fullmatch(args.label):
        return fail("invalid label")
    repo = Path(args.repo)
    evidence_root = Path(args.evidence_root)
    if not repo.is_absolute() or not evidence_root.is_absolute():
        return fail("repo and evidence-root must be absolute")
    if not repo.is_dir() or not evidence_root.is_dir():
        return fail("repo and evidence-root must already exist")
    if repo.stat().st_uid != os.getuid() or evidence_root.stat().st_uid != os.getuid():
        return fail("repo and evidence-root must be caller-owned")
    command = args.command[1:] if args.command[:1] == ["--"] else args.command
    if not command or not command[0]:
        return fail("a command is required after --")

    env_values = {}
    for name in args.record_env:
        if not ENV_NAME.fullmatch(name) or SECRET_NAME.search(name):
            return fail(f"environment name is invalid or secret-bearing: {name}")
        if name not in os.environ:
            return fail(f"requested environment variable is unset: {name}")
        env_values[name] = os.environ[name]

    hashes = {}
    repo_real = repo.resolve()
    for relative in args.hash_file:
        rel = Path(relative)
        if rel.is_absolute() or ".." in rel.parts:
            return fail(f"hash path must be repository-relative: {relative}")
        candidate = repo / rel
        try:
            resolved = candidate.resolve(strict=True)
            resolved.relative_to(repo_real)
        except (OSError, ValueError):
            return fail(f"hash path is missing or outside the repository: {relative}")
        if not candidate.is_file() or candidate.is_symlink():
            return fail(f"hash path must be an ordinary file: {relative}")
        digest = sha256_file(candidate)
        hashes[relative] = digest

    run_id = f"{args.label}-{uuid.uuid4()}"
    run_dir = evidence_root / run_id
    try:
        run_dir.mkdir(mode=0o700)
        run_dir.chmod(0o700)
    except OSError as exc:
        return fail(f"could not create fresh run directory: {exc}")

    started = timestamp()
    receipt = {
        "schema": "assura.validation-capture.v1",
        "run_id": run_id,
        "run_dir": str(run_dir),
        "cwd": os.getcwd(),
        "argv": command,
        "recorded_env": env_values,
        "sha256": hashes,
        "started_at": started,
        "ended_at": None,
        "direct_child_pid": None,
        "stdout_path": "stdout.bin",
        "stderr_path": "stderr.bin",
        "state": "claimed",
        "returncode": None,
    }
    save_receipt(run_dir, receipt)

    child: subprocess.Popen | None = None
    handled_signal: int | None = None
    handled_signal_time: float | None = None
    handled_signal_at: str | None = None

    def on_signal(signum: int, _frame: object) -> None:
        nonlocal handled_signal, handled_signal_time, handled_signal_at
        if handled_signal is None:
            handled_signal = signum
            handled_signal_time = time.monotonic()
            handled_signal_at = timestamp()

    previous_handlers = {}
    for signum in (signal.SIGINT, signal.SIGTERM):
        previous_handlers[signum] = signal.signal(signum, on_signal)

    stdout_path, stderr_path = run_dir / "stdout.bin", run_dir / "stderr.bin"
    try:
        with stdout_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
            if handled_signal is None:
                try:
                    child = subprocess.Popen(command, stdout=stdout, stderr=stderr)
                except OSError as exc:
                    receipt["launch_error"] = str(exc)
                    save_receipt(run_dir, receipt)
                    print(f"capture-validation: launch failed: {exc}", file=sys.stderr)
                    return 127
                receipt["state"] = "running"
                receipt["direct_child_pid"] = child.pid
                save_receipt(run_dir, receipt)
                if handled_signal is not None and child.poll() is None:
                    try:
                        os.kill(child.pid, handled_signal)
                    except ProcessLookupError:
                        pass

            if child is not None:
                signal_forwarded = False
                returncode = None
                while True:
                    if handled_signal is not None and not signal_forwarded:
                        if child.poll() is None:
                            try:
                                os.kill(child.pid, handled_signal)
                            except ProcessLookupError:
                                pass
                        signal_forwarded = True

                    if handled_signal is None:
                        wait_timeout = 0.1
                    else:
                        assert handled_signal_time is not None
                        remaining = 5 - (time.monotonic() - handled_signal_time)
                        if remaining <= 0:
                            returncode = child.poll()
                            break
                        wait_timeout = min(0.1, remaining)
                    try:
                        returncode = child.wait(timeout=wait_timeout)
                        break
                    except subprocess.TimeoutExpired:
                        continue

                receipt["state"] = "interrupted" if handled_signal is not None else "complete"
                receipt["returncode"] = returncode
                if handled_signal is not None:
                    receipt["signal"] = handled_signal
                    receipt["signal_received_at"] = handled_signal_at
                    receipt["child_live"] = returncode is None
                receipt["ended_at"] = timestamp()
                save_receipt(run_dir, receipt)
                if handled_signal is not None:
                    return 128 + handled_signal
                return 128 - returncode if returncode is not None and returncode < 0 else int(returncode or 0)

            receipt["state"] = "interrupted"
            receipt["signal"] = handled_signal
            receipt["target_started"] = False
            receipt["ended_at"] = timestamp()
            save_receipt(run_dir, receipt)
            return 128 + int(handled_signal or signal.SIGTERM)
    finally:
        for signum, handler in previous_handlers.items():
            signal.signal(signum, handler)


if __name__ == "__main__":
    raise SystemExit(main())
