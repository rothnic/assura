#!/usr/bin/env python3
"""Bounded candidate-branch Callgrind observation for NU11."""

from __future__ import annotations

import hashlib
import json
import math
import os
import platform
import re
import selectors
import shutil
import signal
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Sequence


COMMIT_RE = re.compile(r"^[0-9a-f]{40}$")
VALGRIND_VERSION = "1:3.19.0-1"
SNAPSHOT = "20260918T000000Z"
IMAGE = "debian:bookworm-20260918-slim@sha256:f3034a6ec3c1205360777c4aae76234998866ad18806ae62b63a3f84ccad782b"
PROFILE_COUNT = 3
CONTROL_TIMEOUT = 30
PROFILE_TIMEOUT = 120
# Leave five minutes of the outer job for bounded cleanup and artifact upload.
JOB_TIMEOUT = 40 * 60
CAPTURE_LIMIT = 8 * 1024 * 1024
PROFILE_LIMIT = 128 * 1024 * 1024
EVIDENCE_LIMIT = 512 * 1024 * 1024
MIN_DISK_FREE = 10 * 1024 * 1024 * 1024
RESOURCE_CHECK_INTERVAL = 300
TERM_GRACE_SECONDS = 0.5
REAP_GRACE_SECONDS = 0.5
PIPE_DRAIN_SECONDS = 1.0
ENV_KEYS = (
    "PATH", "HOME", "LANG", "LC_ALL", "TMPDIR", "CARGO_HOME", "RUSTUP_HOME",
    "CARGO_TARGET_DIR", "RUSTUP_TOOLCHAIN", "GITHUB_ACTIONS", "GITHUB_REF",
    "GITHUB_SHA", "GITHUB_RUN_ID", "GITHUB_RUN_ATTEMPT", "GITHUB_WORKFLOW",
    "GITHUB_JOB", "CI", "ASSURA_NU11_COMMIT", "ASSURA_NU11_MODE", "ASSURA_NU11_FIXTURE_ROOT",
    "ASSURA_NU11_STARTED_UTC",
)


class GateFailure(RuntimeError):
    pass


@dataclass
class Result:
    name: str
    argv: list[str]
    cwd: str
    started_utc: float
    ended_utc: float
    exit_code: int | None
    timed_out: bool
    overflow: bool
    stop_reason: str | None
    pid: int
    pgid: int
    direct_child_reaped: bool
    process_group_gone: bool | None
    cleanup_complete: bool
    termination_signals: list[str]
    stdout: str
    stderr: str
    stdout_sha256: str
    stderr_sha256: str
    stdout_bytes: int
    stderr_bytes: int
    stdout_observed_bytes: int
    stderr_observed_bytes: int

    def as_json(self) -> dict[str, Any]:
        return self.__dict__.copy()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _process_group_gone(pgid: int) -> bool | None:
    """Return true only when the OS proves this capture group is absent."""
    try:
        os.killpg(pgid, 0)
    except ProcessLookupError:
        return True
    except PermissionError:
        return False
    except OSError:
        return None
    return False


def _wait_for_process_group_exit(pgid: int) -> bool | None:
    deadline = time.monotonic() + REAP_GRACE_SECONDS
    while True:
        gone = _process_group_gone(pgid)
        if gone is True or gone is None or time.monotonic() >= deadline:
            return gone
        time.sleep(0.025)


def _terminate_process_group(process: subprocess.Popen[bytes], pgid: int) -> list[str]:
    """Escalate TERM to KILL, with bounded waits and explicit signal receipts."""
    sent: list[str] = []
    for sig, label in ((signal.SIGTERM, "SIGTERM"), (signal.SIGKILL, "SIGKILL")):
        gone = _process_group_gone(pgid)
        if gone is True:
            sent.append(f"{label}:group_absent")
            break
        try:
            os.killpg(pgid, sig)
            sent.append(f"{label}:sent")
        except ProcessLookupError:
            sent.append(f"{label}:group_absent")
            break
        except OSError as error:
            sent.append(f"{label}:error:{error.__class__.__name__}")
            if label == "SIGKILL":
                break
        grace = TERM_GRACE_SECONDS if sig == signal.SIGTERM else REAP_GRACE_SECONDS
        deadline = time.monotonic() + grace
        while time.monotonic() < deadline:
            if _process_group_gone(pgid) is True:
                break
            if process.poll() is not None and sig == signal.SIGTERM:
                # Descendants may still own pipes or the process group. Continue
                # the bounded TERM grace, then KILL if the group remains.
                pass
            time.sleep(0.025)
    return sent


def validate_source(repo: Path, env: dict[str, str], head: str, status: str) -> str:
    commit = env.get("ASSURA_NU11_COMMIT", "")
    if not COMMIT_RE.fullmatch(commit):
        raise GateFailure("ASSURA_NU11_COMMIT must be a full lowercase commit SHA")
    if env.get("ASSURA_NU11_MODE") != "callgrind":
        raise GateFailure("ASSURA_NU11_MODE must be callgrind")
    if env.get("GITHUB_SHA") != commit or head != commit:
        raise GateFailure("workflow SHA, input commit, and checked-out HEAD must match")
    ref = env.get("GITHUB_REF", "")
    if not ref.startswith("refs/heads/") or ref in {"refs/heads/master", "refs/heads/main"}:
        raise GateFailure("callgrind mode requires a non-default candidate branch")
    if status:
        raise GateFailure("refusing to profile a dirty source checkout")
    if not repo.is_dir():
        raise GateFailure("repository root is unavailable")
    return commit


def run_git(repo: Path, argv: Sequence[str]) -> str:
    completed = subprocess.run(
        ["git", *argv], cwd=repo, check=False, capture_output=True, text=True, timeout=10
    )
    if completed.returncode != 0:
        raise GateFailure(f"git {' '.join(argv)} failed: {completed.stderr.strip()}")
    return completed.stdout.strip()


def limited_environment(source: dict[str, str], *, tmpdir: Path, target: Path) -> dict[str, str]:
    result = {key: source[key] for key in ENV_KEYS if key in source}
    result.update({
        "TMPDIR": str(tmpdir),
        "CARGO_TARGET_DIR": str(target),
        "ASSURA_NU11_FIXTURE_ROOT": str(target.parent / "fixture"),
    })
    return result


class Evidence:
    def __init__(self, root: Path, env: dict[str, str], repo: Path):
        self.root = root
        self.repo = repo
        self.env = env
        self.started = time.monotonic()
        self.commands: list[dict[str, Any]] = []
        self.state: dict[str, Any] = {"schema": "assura.nu11.profile-run.v1", "commands": self.commands}

    def save(self) -> None:
        temporary = self.root / ".run.json.tmp"
        temporary.write_text(json.dumps(self.state, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        temporary.replace(self.root / "run.json")

    def check_budget(self) -> None:
        if time.monotonic() - self.started > JOB_TIMEOUT:
            raise GateFailure("NU11 active run budget expired")
        total = sum(path.stat().st_size for path in self.root.rglob("*") if path.is_file())
        if total > EVIDENCE_LIMIT:
            raise GateFailure("NU11 retained evidence exceeded 512 MiB")

    def capture(
        self,
        name: str,
        argv: Sequence[str],
        *,
        cwd: Path,
        timeout: int,
        env: dict[str, str] | None = None,
        limit: int = CAPTURE_LIMIT,
        produced_file: Path | None = None,
        produced_file_limit: int = PROFILE_LIMIT,
    ) -> Result:
        remaining_job = JOB_TIMEOUT - (time.monotonic() - self.started)
        if remaining_job <= 0:
            raise GateFailure("NU11 active run budget expired")
        if timeout <= 0 or limit <= 0:
            raise ValueError("capture timeout and output limit must be positive")
        output_dir = self.root / "commands"
        output_dir.mkdir(exist_ok=True)
        stdout_path = output_dir / f"{name}.stdout"
        stderr_path = output_dir / f"{name}.stderr"
        start = time.time()
        start_mono = time.monotonic()
        command_env = env or self.env
        process = subprocess.Popen(
            list(argv), cwd=cwd, env=command_env, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
            start_new_session=True,
        )
        pid = process.pid
        pgid = pid
        record: dict[str, Any] = {
            "name": name,
            "argv": list(argv),
            "cwd": str(cwd),
            "pid": pid,
            "pgid": pgid,
            "started_utc": start,
            "lifecycle": "running",
            "cleanup_status": "unknown",
            "environment": {key: command_env.get(key) for key in ENV_KEYS if key in command_env},
        }
        command_deadline = start_mono + timeout
        job_deadline = self.started + JOB_TIMEOUT
        deadline = min(command_deadline, job_deadline)
        deadline_reason = "job_budget_timeout" if job_deadline <= command_deadline else "command_timeout"
        stop_reason: str | None = None
        termination_signals: list[str] = []
        selector: selectors.BaseSelector | None = None
        streams: dict[Path, Any] = {}
        sizes = {stdout_path: 0, stderr_path: 0}
        observed = {stdout_path: 0, stderr_path: 0}
        total = 0
        pipes_closed = False
        direct_child_reaped = False
        process_group_gone: bool | None = None
        terminal_record_saved = False
        last_resource_check = start_mono
        resource_checks = 0

        def close_resources() -> None:
            nonlocal selector
            active_selector, selector = selector, None
            if active_selector is not None:
                for key in list((active_selector.get_map() or {}).values()):
                    try:
                        active_selector.unregister(key.fileobj)
                    except (KeyError, ValueError):
                        pass
                try:
                    active_selector.close()
                except OSError:
                    pass
            for stream in streams.values():
                try:
                    stream.close()
                except OSError:
                    pass
            for pipe in (process.stdout, process.stderr):
                if pipe is not None:
                    try:
                        pipe.close()
                    except OSError:
                        pass

        try:
            self.commands.append(record)
            self.save()
            selector = selectors.DefaultSelector()
            assert process.stdout is not None and process.stderr is not None
            selector.register(process.stdout, selectors.EVENT_READ, stdout_path)
            selector.register(process.stderr, selectors.EVENT_READ, stderr_path)
            for path in sizes:
                streams[path] = path.open("wb")

            while selector.get_map() or process.poll() is None:
                if time.monotonic() - last_resource_check >= RESOURCE_CHECK_INTERVAL:
                    resource_checks += 1
                    record_resource_guard(self, f"{name}-resources-{resource_checks}", cwd, command_deadline=deadline)
                    last_resource_check = time.monotonic()
                remaining = deadline - time.monotonic()
                if (
                    produced_file is not None
                    and produced_file.exists()
                    and produced_file.stat().st_size > produced_file_limit
                ):
                    stop_reason = "profile_output_overflow"
                    break
                if remaining <= 0 and (process.poll() is None or selector.get_map()):
                    stop_reason = deadline_reason
                    break
                for key, _ in selector.select(timeout=min(max(remaining, 0.01), 0.1)):
                    block = os.read(key.fileobj.fileno(), 64 * 1024)
                    if not block:
                        selector.unregister(key.fileobj)
                        continue
                    path = key.data
                    observed[path] += len(block)
                    available = max(0, limit - total)
                    if len(block) > available:
                        stop_reason = "captured_output_overflow"
                    if available:
                        written = block[:available]
                        streams[path].write(written)
                        sizes[path] += len(written)
                        total += len(written)
                    if stop_reason is not None:
                        break
                if stop_reason is not None:
                    break

            if stop_reason is None:
                # A completed direct child must not leave an unnoticed member in
                # this capture's process group. This is a local runner contract.
                process_group_gone = _process_group_gone(pgid)
                if process_group_gone is False:
                    stop_reason = "descendant_process_remains"

            if stop_reason is not None:
                termination_signals = _terminate_process_group(process, pgid)

            drain_deadline = time.monotonic() + PIPE_DRAIN_SECONDS
            while selector.get_map() and time.monotonic() < drain_deadline:
                for key, _ in selector.select(timeout=min(0.05, drain_deadline - time.monotonic())):
                    block = os.read(key.fileobj.fileno(), 64 * 1024)
                    if not block:
                        selector.unregister(key.fileobj)
                        continue
                    path = key.data
                    observed[path] += len(block)
                    available = max(0, limit - total)
                    if len(block) > available and stop_reason is None:
                        stop_reason = "captured_output_overflow"
                    written = block[:available]
                    if written:
                        streams[path].write(written)
                        sizes[path] += len(written)
                        total += len(written)
            pipes_closed = not selector.get_map()
            close_resources()
            pipes_closed = True

            try:
                process.wait(timeout=REAP_GRACE_SECONDS)
                direct_child_reaped = True
            except subprocess.TimeoutExpired:
                if stop_reason is None:
                    stop_reason = "child_reap_timeout"
                termination_signals.extend(_terminate_process_group(process, pgid))
                try:
                    process.wait(timeout=REAP_GRACE_SECONDS)
                    direct_child_reaped = True
                except subprocess.TimeoutExpired:
                    direct_child_reaped = False
            process_group_gone = _wait_for_process_group_exit(pgid)
            if (
                produced_file is not None
                and produced_file.exists()
                and produced_file.stat().st_size > produced_file_limit
                and stop_reason is None
            ):
                stop_reason = "profile_output_overflow"
            overflow = stop_reason in {"captured_output_overflow", "profile_output_overflow"}
            timed_out = stop_reason in {"command_timeout", "job_budget_timeout"}
            cleanup_complete = direct_child_reaped and process_group_gone is True and pipes_closed
            if not cleanup_complete and stop_reason is None:
                stop_reason = "cleanup_incomplete"
            result = Result(
                name=name,
                argv=list(argv),
                cwd=str(cwd),
                started_utc=start,
                ended_utc=time.time(),
                exit_code=process.returncode,
                timed_out=timed_out,
                overflow=overflow,
                stop_reason=stop_reason,
                pid=pid,
                pgid=pgid,
                direct_child_reaped=direct_child_reaped,
                process_group_gone=process_group_gone,
                cleanup_complete=cleanup_complete,
                termination_signals=termination_signals,
                stdout=str(stdout_path.relative_to(self.root)),
                stderr=str(stderr_path.relative_to(self.root)),
                stdout_sha256=sha256_file(stdout_path),
                stderr_sha256=sha256_file(stderr_path),
                stdout_bytes=sizes[stdout_path],
                stderr_bytes=sizes[stderr_path],
                stdout_observed_bytes=observed[stdout_path],
                stderr_observed_bytes=observed[stderr_path],
            )
            record.update(result.as_json())
            record["lifecycle"] = "terminal"
            record["cleanup_status"] = (
                "complete" if cleanup_complete else "unknown" if process_group_gone is None else "incomplete"
            )
            self.save()
            terminal_record_saved = True
            self.check_budget()
            return result
        except BaseException as error:
            if terminal_record_saved:
                raise
            termination_signals.extend(_terminate_process_group(process, pgid))
            try:
                process.wait(timeout=REAP_GRACE_SECONDS)
                direct_child_reaped = True
            except subprocess.TimeoutExpired:
                direct_child_reaped = False
            process_group_gone = _wait_for_process_group_exit(pgid)
            close_resources()
            record.update({
                "lifecycle": "aborted",
                "cleanup_status": (
                    "complete"
                    if direct_child_reaped and process_group_gone is True
                    else "unknown" if process_group_gone is None else "incomplete"
                ),
                "direct_child_reaped": direct_child_reaped,
                "process_group_gone": process_group_gone,
                "cleanup_complete": direct_child_reaped and process_group_gone is True,
                "termination_signals": termination_signals,
                "failure": f"{error.__class__.__name__}: {error}",
                "ended_utc": time.time(),
            })
            try:
                self.save()
            except BaseException as save_error:
                record["cleanup_status"] = "unknown"
                record["receipt_write_error"] = f"{save_error.__class__.__name__}: {save_error}"
            raise
        finally:
            close_resources()

    def require(self, result: Result, *, exit_code: int = 0, context: str | None = None) -> None:
        if (
            result.timed_out
            or result.overflow
            or result.stop_reason is not None
            or not result.cleanup_complete
            or result.exit_code != exit_code
        ):
            raise GateFailure(context or f"{result.name} failed or exceeded its bounded capture")


def read_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, data: Any) -> None:
    path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def annotator_total_ir(output: str) -> int:
    """Read the vendor annotator's PROGRAM TOTALS summary, never the raw profile."""
    match = re.search(r"(?m)^\s*([0-9][0-9,]*)\s+\([^)]*\)\s+PROGRAM TOTALS\s*$", output)
    if match is None:
        raise GateFailure("vendor annotation has no numeric PROGRAM TOTALS Ir")
    return int(match.group(1).replace(",", ""))


def verify_fixture(root: Path, repo: Path, commit: str) -> dict[str, Any]:
    manifest_path = root / "nu11-fixture-manifest.json"
    manifest = read_json(manifest_path)
    if manifest.get("schema") != "assura.nu11.fixture.v1":
        raise GateFailure("fixture exporter returned an unsupported manifest")
    source = manifest.get("source", {})
    if source.get("commit") != commit:
        raise GateFailure("fixture manifest source commit does not match the selected workflow input")
    for key, relative in (
        ("constructor", "src/cli/performance_report/counterexample_fixtures.rs"),
        ("exporter", "src/cli/performance_report/fixture_tests.rs"),
    ):
        if source.get(key, {}).get("path") != relative or source[key].get("sha256") != sha256_file(repo / relative):
            raise GateFailure(f"fixture manifest source identity mismatch: {relative}")
    if manifest.get("package_directory_count") != 800 or manifest.get("package_file_count") != 800:
        raise GateFailure("exported fixture does not contain the accepted 800-scope workload")
    if manifest.get("rule_entry_count") != 801 or manifest.get("assura_policy_nonempty") is not True:
        raise GateFailure("exported fixture policy is incomplete or empty")
    entries = manifest.get("files")
    if not isinstance(entries, list) or len(entries) != 802:
        raise GateFailure("fixture manifest must bind 800 files and both generated configs")
    paths: set[str] = set()
    for entry in entries:
        relative = entry.get("path")
        if not isinstance(relative, str) or relative in paths or Path(relative).is_absolute() or ".." in Path(relative).parts:
            raise GateFailure("fixture manifest contains a duplicate or unsafe path")
        paths.add(relative)
        file_path = root / relative
        if not file_path.is_file() or file_path.stat().st_size != entry.get("size_bytes") or sha256_file(file_path) != entry.get("sha256"):
            raise GateFailure(f"fixture input hash mismatch: {relative}")
    expected = {".ls-lint.yml", ".assura/config.yml"}
    expected.update(f"pkg-{index:04}/file-name.js" for index in range(800))
    if paths != expected:
        raise GateFailure("fixture manifest does not match the exact accepted file set")
    expected_directories = {f"pkg-{index:04}" for index in range(800)} | {".assura"}
    actual_directories = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_dir() and not path.is_symlink()
    }
    if actual_directories != expected_directories or any(path.is_symlink() for path in root.rglob("*")):
        raise GateFailure("fixture contains an unexpected directory or symbolic link")
    config = manifest.get("configs", {})
    for key, relative in (("ls_lint", ".ls-lint.yml"), ("assura", ".assura/config.yml")):
        if config.get(key, {}).get("path") != relative or config[key].get("sha256") != sha256_file(root / relative):
            raise GateFailure(f"fixture manifest does not bind {relative}")
    return manifest


def cli_command(binary: Path, fixture: Path) -> list[str]:
    return [str(binary), "--config", str(fixture / ".assura/config.yml"), "check", "--format", "json", str(fixture)]


def validate_cli_output(result: Result, root: Path, *, expected_success: bool) -> dict[str, Any]:
    stdout = (root / result.stdout).read_text(encoding="utf-8")
    try:
        payload = json.loads(stdout)
    except json.JSONDecodeError as error:
        raise GateFailure(f"Assura check did not emit valid JSON: {error}") from error
    success = payload.get("success") is True
    if success != expected_success:
        raise GateFailure("Assura CLI result did not match the positive/negative control")
    if expected_success:
        if payload.get("files_checked") != 801 or payload.get("dirs_checked") != 800 or payload.get("violations") != []:
            raise GateFailure("Assura CLI did not check the accepted 801-file/800-directory fixture")
    else:
        violations = payload.get("violations")
        if not isinstance(violations, list) or not any("FileName.js" in json.dumps(item) for item in violations):
            raise GateFailure("negative filename control lacked its expected violation")
    return payload


def command_hashes(paths: Sequence[Path]) -> dict[str, str]:
    return {str(path): sha256_file(path) for path in paths}


def read_host_resources(repo: Path) -> dict[str, Any]:
    memory = {}
    for line in Path("/proc/meminfo").read_text().splitlines():
        key, value = line.split(":", 1)
        if key in {"MemTotal", "MemAvailable"}:
            amount, unit = value.split()
            if unit != "kB":
                raise GateFailure("unsupported Linux memory counter unit")
            memory[key] = int(amount) * 1024
    pressure = {}
    for kind in ("memory", "io"):
        rows = Path(f"/proc/pressure/{kind}").read_text().splitlines()
        full = next(row for row in rows if row.startswith("full "))
        pressure[kind] = float(dict(field.split("=", 1) for field in full.split()[1:])["avg10"])
    return {
        "memory_total": memory["MemTotal"],
        "memory_available": memory["MemAvailable"],
        "psi_full_avg10": pressure,
        "load_1m": os.getloadavg()[0],
        "cpu_count": len(os.sched_getaffinity(0)),
        "disk_free": shutil.disk_usage(repo).free,
    }


def resource_rejections(snapshot: dict[str, Any]) -> list[str]:
    values = [snapshot[key] for key in ("memory_total", "memory_available", "load_1m", "cpu_count", "disk_free")]
    values.extend(snapshot["psi_full_avg10"][kind] for kind in ("memory", "io"))
    if any(isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value) or value < 0 for value in values):
        raise GateFailure("invalid resource counter")
    if snapshot["memory_total"] <= 0 or snapshot["cpu_count"] <= 0:
        raise GateFailure("resource capacity is unknown")
    reasons = []
    if snapshot["memory_available"] < snapshot["memory_total"] * 0.10:
        reasons.append("available memory below 10 percent")
    for kind in ("memory", "io"):
        if snapshot["psi_full_avg10"][kind] >= 5:
            reasons.append(f"{kind} PSI full avg10 at least 5 percent")
    if snapshot["disk_free"] < MIN_DISK_FREE:
        reasons.append("free disk below the 10 GiB build/evidence reserve")
    return reasons


def record_resource_guard(evidence: Evidence, label: str, repo: Path, *, command_deadline: float | None = None) -> None:
    # The infrastructure policy requires two elevated-load samples 30s apart.
    for sample in range(2):
        key = label if sample == 0 else f"{label}-load-confirmation"
        try:
            snapshot = read_host_resources(repo)
            reasons = resource_rejections(snapshot)
        except (OSError, ValueError, KeyError, StopIteration, GateFailure) as error:
            evidence.state.setdefault("resource_snapshots", {})[key] = {"outcome": "unknown", "failure": str(error)}
            evidence.save()
            raise GateFailure(f"resource admission unavailable: {label}") from error
        high_load = snapshot["load_1m"] > 2 * snapshot["cpu_count"]
        if high_load and sample == 1:
            reasons.append("load above twice CPU capacity in two samples 30 seconds apart")
        snapshot.update(observed_utc=time.time(), observed_monotonic=time.monotonic(), rejection_reasons=reasons)
        evidence.state.setdefault("resource_snapshots", {})[key] = snapshot
        evidence.save()
        if reasons:
            raise GateFailure(f"resource pressure stops collection: {'; '.join(reasons)}")
        if not high_load:
            return
        evidence.check_budget()
        if JOB_TIMEOUT - (time.monotonic() - evidence.started) <= 30:
            raise GateFailure("insufficient job budget to confirm elevated host load")
        if command_deadline is not None and command_deadline - time.monotonic() <= 30:
            raise GateFailure("insufficient command budget to confirm elevated host load")
        time.sleep(30)
        evidence.check_budget()


def require_filename_negative(evidence: Evidence, result: Result, root: Path) -> None:
    if result.exit_code not in (0, 1):
        raise GateFailure("negative filename control returned an unexpected exit")
    evidence.require(result, exit_code=result.exit_code)
    validate_cli_output(result, root, expected_success=False)


def record_host_snapshot(evidence: Evidence, label: str, repo: Path) -> None:
    result = evidence.capture(
        label,
        [
            "sh", "-c",
            "uname -a; cat /proc/loadavg; cat /proc/pressure/cpu /proc/pressure/memory /proc/pressure/io 2>/dev/null || true; "
            "df -B1 .; free -b 2>/dev/null || true; "
            "cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_cur_freq 2>/dev/null || true; "
            "ps -eo pid,ppid,stat,etime,comm --sort=pid",
        ],
        cwd=repo,
        timeout=CONTROL_TIMEOUT,
    )
    evidence.require(result, context=f"host telemetry capture failed: {label}")
    record_resource_guard(evidence, label, repo)


def execute(evidence: Evidence, repo: Path, fixture: Path, env: dict[str, str], run_root: Path) -> None:
    record_resource_guard(evidence, "host-admission", repo)
    apt_sources = run_root / "debian-snapshot.list"
    apt_sources.write_text(
        f"deb [check-valid-until=no] https://snapshot.debian.org/archive/debian/{SNAPSHOT} bookworm main\n",
        encoding="utf-8",
    )
    apt_opts = ["-o", f"Dir::Etc::sourcelist={apt_sources}", "-o", "Dir::Etc::sourceparts=-", "-o", "Acquire::Check-Valid-Until=false"]
    evidence.state.update({
        "source_commit": env["ASSURA_NU11_COMMIT"],
        "workflow_ref": env["GITHUB_REF"],
        "workflow_sha": env["GITHUB_SHA"],
        "container_image": IMAGE,
        "snapshot": SNAPSHOT,
        "valgrind_version": VALGRIND_VERSION,
        "architecture": platform.machine(),
        "started_utc": time.time(),
        "environment_allowlist": {key: env.get(key) for key in ENV_KEYS if key in env},
    })
    evidence.save()

    os_release = Path("/etc/os-release").read_text(encoding="utf-8")
    if "ID=debian" not in os_release or "VERSION_ID=12" not in os_release:
        raise GateFailure("profile job is not running in the pinned Debian Bookworm container")
    if platform.machine() not in {"x86_64", "amd64"}:
        raise GateFailure("profile job architecture is not the reviewed amd64 platform")
    if os.geteuid() != 0:
        raise GateFailure("pinned container package acquisition requires its isolated root user")

    for name, argv, timeout in (
        ("apt-update", ["apt-get", *apt_opts, "update"], 120),
        ("apt-resolve-download", ["apt-get", *apt_opts, "--download-only", "install", "-y", "--no-install-recommends", f"valgrind={VALGRIND_VERSION}", "git", "ca-certificates", "curl", "build-essential", "pkg-config"], 240),
        ("apt-install-tools", ["apt-get", *apt_opts, "install", "-y", "--no-install-recommends", f"valgrind={VALGRIND_VERSION}", "git", "ca-certificates", "curl", "build-essential", "pkg-config"], 240),
    ):
        result = evidence.capture(name, argv, cwd=run_root, timeout=timeout)
        evidence.require(result, context=f"pinned apt operation failed: {name}")

    downloaded = sorted(Path("/var/cache/apt/archives").glob("*.deb"))
    if not downloaded or not any(path.name.startswith("valgrind_") for path in downloaded):
        raise GateFailure("snapshot did not resolve the pinned Valgrind package and dependencies")
    package_dir = run_root / "packages"
    package_dir.mkdir()
    package_records = []
    for package in downloaded:
        copied = package_dir / package.name
        shutil.copy2(package, copied)
        package_records.append({"file": copied.name, "sha256": sha256_file(copied), "size_bytes": copied.stat().st_size})
    valgrind_path = Path(shutil.which("valgrind") or "")
    annotate_path = Path(shutil.which("callgrind_annotate") or "")
    if not valgrind_path.is_file() or not annotate_path.is_file():
        raise GateFailure("pinned Valgrind tools are unavailable after package installation")
    list_files = sorted(Path("/var/lib/apt/lists").glob("*"))
    package_inventory = evidence.capture(
        "package-inventory",
        ["sh", "-c", "dpkg-query -W -f='${Package} ${Version} ${Architecture}\\n' | sort && sha256sum /var/lib/apt/lists/*"],
        cwd=repo,
        timeout=CONTROL_TIMEOUT,
    )
    evidence.require(package_inventory, context="package metadata inventory failed")
    metadata_dir = run_root / "apt-metadata"
    metadata_dir.mkdir()
    metadata_records = []
    for source in list_files:
        if source.is_file():
            copied = metadata_dir / source.name
            shutil.copy2(source, copied)
            metadata_records.append({"file": copied.name, "sha256": sha256_file(copied), "size_bytes": copied.stat().st_size})
    evidence.check_budget()
    versions = evidence.capture("valgrind-version", [str(valgrind_path), "--version"], cwd=repo, timeout=CONTROL_TIMEOUT)
    evidence.require(versions, context="Valgrind version probe failed")
    version_text = (run_root / versions.stdout).read_text(encoding="utf-8").strip()
    if "3.19.0" not in version_text:
        raise GateFailure("installed Valgrind version does not match the pinned package")
    package_version = evidence.capture("valgrind-package-version", ["dpkg-query", "-W", "-f=${Version}", "valgrind"], cwd=repo, timeout=CONTROL_TIMEOUT)
    evidence.require(package_version, context="Valgrind package version query failed")
    actual_package_version = (run_root / package_version.stdout).read_text(encoding="utf-8").strip()
    if actual_package_version != VALGRIND_VERSION:
        raise GateFailure(f"Valgrind package version mismatch: {actual_package_version}")
    evidence.state["tool_hashes"] = command_hashes([valgrind_path, annotate_path])
    annotate_version = evidence.capture("annotator-version", [str(annotate_path), "--version"], cwd=repo, timeout=CONTROL_TIMEOUT)
    evidence.require(annotate_version, context="vendor annotator version probe failed")
    evidence.state["apt_packages"] = package_records
    evidence.state["apt_metadata_files"] = metadata_records
    evidence.state["apt_source_sha256"] = sha256_file(apt_sources)
    evidence.save()

    rust = evidence.capture("rust-identity", ["sh", "-c", "rustc --version --verbose && cargo --version && python3 --version && node --version && npm --version"], cwd=repo, timeout=CONTROL_TIMEOUT)
    evidence.require(rust, context="Rust/Node toolchain identity probe failed")
    if shutil.which("node") is None or shutil.which("npm") is None:
        raise GateFailure("Node and npm are required by the unchanged strict performance matrix")

    test_env = dict(env)
    test_env["ASSURA_NU11_COMMIT"] = env["ASSURA_NU11_COMMIT"]
    test_env["ASSURA_NU11_FIXTURE_ROOT"] = str(fixture)
    export = evidence.capture(
        "fixture-export",
        ["cargo", "test", "--release", "--lib", "--", "--ignored", "--nocapture", "export_nu11_accepted_many_scope_fixture"],
        cwd=repo,
        timeout=600,
        env=test_env,
    )
    evidence.require(export, context="fixture exporter test failed")
    manifest = verify_fixture(fixture, repo, env["ASSURA_NU11_COMMIT"])
    write_json(run_root / "fixture-manifest-copy.json", manifest)

    build_commands = [
        ("build-assura-json", ["cargo", "build", "--release", "--bin", "assura", "--no-default-features", "--features", "json-output,yaml-config"], env),
        ("build-assura-static", ["cargo", "rustc", "--release", "--bin", "assura", "--no-default-features", "--features", "json-output,yaml-config", "--", "-C", "target-feature=+crt-static", "-C", "link-arg=-lgcc_eh"], env),
        ("build-assura-full", ["cargo", "build", "--release", "--bin", "assura-full"], env),
        ("build-assura-check", ["cargo", "build", "--release", "-p", "assura-check-cli"], env),
        ("build-xtask", ["cargo", "build", "--release", "-p", "xtask"], env),
    ]
    for name, argv, build_env in build_commands:
        result = evidence.capture(name, argv, cwd=repo, timeout=900, env=build_env)
        evidence.require(result, context=f"pre-profile build failed: {name}")

    build_target = Path(env["CARGO_TARGET_DIR"])
    twin_target = build_target.parent / f"{build_target.name}-symbols"
    if twin_target.exists():
        raise GateFailure("refusing to reuse an existing symbol-twin build directory")
    twin_env = dict(env)
    twin_env.update({"CARGO_TARGET_DIR": str(twin_target), "CARGO_PROFILE_RELEASE_STRIP": "none", "CARGO_PROFILE_RELEASE_DEBUG": "2"})
    twin = evidence.capture("build-symbol-twin", ["cargo", "build", "--release", "--bin", "assura-full"], cwd=repo, timeout=1200, env=twin_env)
    evidence.require(twin, context="symbol-retaining diagnostic twin build failed")
    production = build_target / "release/assura-full"
    diagnostic = twin_target / "release/assura-full"
    assura = build_target / "release/assura"
    xtask = build_target / "release/xtask"
    check_binary = build_target / "release/assura-check"
    required_binaries = [production, diagnostic, assura, xtask, check_binary]
    if not all(path.is_file() for path in required_binaries):
        raise GateFailure("one or more required prebuilt executables are missing")
    evidence.state["binary_hashes_before"] = command_hashes(required_binaries)
    evidence.state["source_inputs"] = command_hashes([
        repo / "Cargo.lock", repo / "Cargo.toml", repo / ".cargo/config.toml",
        repo / ".github/workflows/r03-performance-variance.yml", repo / ".assura/config.yml",
        repo / "scripts/nu11-callgrind-profile.py", repo / "scripts/r03-performance-variance.sh",
        repo / "src/cli/performance_report/counterexample_fixtures.rs",
        repo / "src/cli/performance_report/fixture_tests.rs",
    ])
    evidence.save()

    # Capture and validate every control before any profile is started.
    none = evidence.capture("valgrind-none", [str(valgrind_path), "--tool=none", "/usr/bin/true"], cwd=repo, timeout=CONTROL_TIMEOUT)
    evidence.require(none, context="Valgrind tool=none health control failed")
    true_raw = run_root / "callgrind-true.out"
    true_result = evidence.capture("callgrind-true", [str(valgrind_path), "--tool=callgrind", "--cache-sim=no", "--branch-sim=no", f"--callgrind-out-file={true_raw}", "/usr/bin/true"], cwd=repo, timeout=CONTROL_TIMEOUT, produced_file=true_raw)
    evidence.require(true_result, context="Callgrind true control failed")
    loop_raw = run_root / "callgrind-loop.out"
    loop = evidence.capture("callgrind-fixed-loop", [str(valgrind_path), "--tool=callgrind", "--cache-sim=no", "--branch-sim=no", f"--callgrind-out-file={loop_raw}", "/bin/bash", "-c", "i=0; while ((i < 100000)); do ((i += 1)); done"], cwd=repo, timeout=CONTROL_TIMEOUT, produced_file=loop_raw)
    evidence.require(loop, context="Callgrind fixed loop control failed")
    for label, path in (("true", true_raw), ("loop", loop_raw)):
        if not path.is_file() or path.stat().st_size == 0 or path.stat().st_size > PROFILE_LIMIT:
            raise GateFailure(f"Callgrind {label} control output is absent or over limit")
    malformed = run_root / "malformed-callgrind.out"
    malformed.write_text("this is deliberately not a Callgrind profile\n", encoding="utf-8")
    malformed_hash = sha256_file(malformed)
    evidence.state["malformed_control_input"] = {"path": str(malformed.relative_to(run_root)), "sha256": malformed_hash}
    evidence.save()
    bad_annotation = evidence.capture("annotator-malformed-negative", [str(annotate_path), str(malformed)], cwd=repo, timeout=CONTROL_TIMEOUT)
    evidence.require(bad_annotation, exit_code=1, context="pinned vendor annotator accepted malformed profile")
    evidence.state["malformed_control"] = {"sha256": malformed_hash, "exit_code": bad_annotation.exit_code, "stdout": bad_annotation.stdout, "stderr": bad_annotation.stderr}
    evidence.save()

    control_totals: dict[str, int] = {}
    for label, raw in (("true", true_raw), ("loop", loop_raw)):
        annotation = evidence.capture(f"annotate-{label}", [str(annotate_path), "--inclusive=no", "--threshold=0.0", str(raw)], cwd=repo, timeout=CONTROL_TIMEOUT)
        evidence.require(annotation, context=f"vendor annotation failed for {label} control")
        annotated = (run_root / annotation.stdout).read_text(encoding="utf-8", errors="replace")
        control_totals[label] = annotator_total_ir(annotated)
    if control_totals["loop"] <= control_totals["true"]:
        raise GateFailure("fixed no-I/O loop did not exceed true in vendor-reported Ir")
    evidence.state["sensitivity_control_ir"] = control_totals
    evidence.save()
    compatibility = evidence.capture("diagnostic-cli-compatibility", cli_command(diagnostic, fixture), cwd=fixture, timeout=PROFILE_TIMEOUT)
    evidence.require(compatibility, context="diagnostic twin CLI compatibility failed")
    validate_cli_output(compatibility, run_root, expected_success=True)
    compatibility_raw = run_root / "callgrind-target-compatibility.out"
    compatibility_profile = evidence.capture(
        "callgrind-target-compatibility",
        [str(valgrind_path), "--tool=callgrind", "--cache-sim=no", "--branch-sim=no", f"--callgrind-out-file={compatibility_raw}", *cli_command(diagnostic, fixture)],
        cwd=fixture,
        timeout=PROFILE_TIMEOUT,
        limit=PROFILE_LIMIT,
        produced_file=compatibility_raw,
    )
    evidence.state["target_compatibility_raw"] = {
        "exists": compatibility_raw.is_file(),
        "size_bytes": compatibility_raw.stat().st_size if compatibility_raw.is_file() else None,
        "sha256": sha256_file(compatibility_raw) if compatibility_raw.is_file() else None,
    }
    evidence.save()
    evidence.require(compatibility_profile, context="Callgrind target compatibility control failed")
    if not compatibility_raw.is_file() or not compatibility_raw.stat().st_size or compatibility_raw.stat().st_size > PROFILE_LIMIT:
        raise GateFailure("Callgrind target compatibility output is absent or outside size bound")
    validate_cli_output(compatibility_profile, run_root, expected_success=True)
    compatibility_annotation = evidence.capture(
        "annotate-target-compatibility",
        [str(annotate_path), "--inclusive=no", "--threshold=0.0", str(compatibility_raw)],
        cwd=repo,
        timeout=CONTROL_TIMEOUT,
    )
    evidence.require(compatibility_annotation, context="vendor annotator could not read target compatibility profile")
    compatibility_text = (run_root / compatibility_annotation.stdout).read_text(encoding="utf-8", errors="replace")
    evidence.state["target_compatibility_ir"] = annotator_total_ir(compatibility_text)
    if "assura::cli::" not in compatibility_text and "assura::config::" not in compatibility_text:
        raise GateFailure("target compatibility profile did not resolve Assura functions")
    evidence.save()

    # Negative control uses one exact rename and guaranteed restoration.
    original = fixture / "pkg-0000/file-name.js"
    invalid = fixture / "pkg-0000/FileName.js"
    original_bytes = original.read_bytes()
    original.rename(invalid)
    try:
        negative = evidence.capture("filename-negative-control", cli_command(diagnostic, fixture), cwd=fixture, timeout=PROFILE_TIMEOUT)
        require_filename_negative(evidence, negative, run_root)
    finally:
        if invalid.exists():
            invalid.rename(original)
        if not original.exists() or original.read_bytes() != original_bytes:
            raise GateFailure("negative filename control failed to restore exact fixture bytes")
    verify_fixture(fixture, repo, env["ASSURA_NU11_COMMIT"])
    positive = evidence.capture("post-restore-positive-control", cli_command(diagnostic, fixture), cwd=fixture, timeout=PROFILE_TIMEOUT)
    evidence.require(positive, context="fixture positive recheck failed after restoration")
    validate_cli_output(positive, run_root, expected_success=True)

    env_file = run_root / "profile-environment.json"
    write_json(env_file, {key: env.get(key) for key in ENV_KEYS if key in env})
    raw_profiles: list[Path] = []
    profile_totals: dict[str, int] = {}
    for index in range(1, PROFILE_COUNT + 1):
        raw = run_root / f"profile-{index}.callgrind"
        record_host_snapshot(evidence, f"host-before-profile-{index}", repo)
        profile = evidence.capture(
            f"profile-{index}",
            [str(valgrind_path), "--tool=callgrind", "--cache-sim=no", "--branch-sim=no", f"--callgrind-out-file={raw}", *cli_command(diagnostic, fixture)],
            cwd=fixture,
            timeout=PROFILE_TIMEOUT,
            limit=PROFILE_LIMIT,
            produced_file=raw,
        )
        evidence.state.setdefault("partial_profiles", {})[raw.name] = {
            "exists": raw.is_file(),
            "size_bytes": raw.stat().st_size if raw.is_file() else None,
            "sha256": sha256_file(raw) if raw.is_file() else None,
        }
        evidence.save()
        evidence.require(profile, context=f"Callgrind profile {index} failed or timed out")
        if not raw.is_file() or not raw.stat().st_size or raw.stat().st_size > PROFILE_LIMIT:
            raise GateFailure(f"Callgrind profile {index} output missing or outside size bound")
        raw_profiles.append(raw)
        validate_cli_output(profile, run_root, expected_success=True)
        annotation = evidence.capture(f"annotate-profile-{index}", [str(annotate_path), "--inclusive=no", "--threshold=0.0", str(raw)], cwd=repo, timeout=CONTROL_TIMEOUT)
        evidence.require(annotation, context=f"vendor annotator failed for profile {index}")
        annotated = (run_root / annotation.stdout).read_text(encoding="utf-8", errors="replace")
        profile_totals[raw.name] = annotator_total_ir(annotated)
        if "assura::cli::" not in annotated and "assura::config::" not in annotated:
            raise GateFailure(f"profile {index} did not resolve Assura target functions")
        evidence.state.setdefault("profile_annotations", {})[raw.name] = {
            "raw_sha256": sha256_file(raw),
            "total_ir": profile_totals[raw.name],
            "vendor_stdout": annotation.stdout,
            "vendor_stdout_sha256": annotation.stdout_sha256,
        }
        evidence.save()
        record_host_snapshot(evidence, f"host-after-profile-{index}", repo)
    evidence.state["profile_hashes"] = {path.name: sha256_file(path) for path in raw_profiles}
    evidence.state["profile_total_ir"] = profile_totals
    evidence.state["binary_hashes_after_profiles"] = command_hashes(required_binaries)
    if evidence.state["binary_hashes_before"] != evidence.state["binary_hashes_after_profiles"]:
        raise GateFailure("a measured executable changed during the three profile collection")
    evidence.save()

    report = run_root / "strict-production-report.json"
    record_host_snapshot(evidence, "host-before-strict-matrix", repo)
    strict_report = evidence.capture("strict-production-matrix", [str(assura), "performance-report", "--output", str(report), "--iterations", "16"], cwd=repo, timeout=900, produced_file=report, produced_file_limit=PROFILE_LIMIT)
    evidence.state["strict_report"] = {
        "exists": report.is_file(),
        "size_bytes": report.stat().st_size if report.is_file() else None,
        "sha256": sha256_file(report) if report.is_file() else None,
    }
    evidence.save()
    evidence.require(strict_report, context="unchanged strict production report failed")
    gate = evidence.capture("strict-production-gate", [str(xtask), "performance-no-slower", str(report)], cwd=repo, timeout=CONTROL_TIMEOUT)
    evidence.require(gate, context="unchanged strict production gate failed")
    record_host_snapshot(evidence, "host-after-strict-matrix", repo)
    evidence.state["binary_hashes_after_gate"] = command_hashes(required_binaries)
    if evidence.state["binary_hashes_before"] != evidence.state["binary_hashes_after_gate"]:
        raise GateFailure("strict production matrix changed a measured executable")
    evidence.state["outcome"] = "completed"
    evidence.state["ended_utc"] = time.time()
    evidence.save()


def main() -> int:
    repo = Path(__file__).resolve().parents[1]
    source_env = dict(os.environ)
    try:
        head = run_git(repo, ["rev-parse", "HEAD"])
        status = run_git(repo, ["status", "--porcelain", "--untracked-files=normal"])
        commit = validate_source(repo, source_env, head, status)
    except (GateFailure, subprocess.SubprocessError, OSError) as error:
        print(f"NU11 source preflight failed: {error}", file=sys.stderr)
        return 2

    run_id = source_env.get("GITHUB_RUN_ID", "")
    attempt = source_env.get("GITHUB_RUN_ATTEMPT", "")
    if not run_id.isdigit() or not attempt.isdigit():
        print("NU11 workflow run id and attempt must be numeric", file=sys.stderr)
        return 2
    started_utc = source_env.get("ASSURA_NU11_STARTED_UTC", "")
    if not started_utc.isdigit() or not 0 <= time.time() - int(started_utc) < JOB_TIMEOUT:
        print("NU11 workflow start time is missing, future, or outside the active budget", file=sys.stderr)
        return 2
    root = repo / "target/performance/nu11-callgrind-profile" / f"{commit}-{run_id}-{attempt}"
    if root.exists():
        print(f"refusing to overwrite NU11 evidence: {root}", file=sys.stderr)
        return 2
    root.parent.mkdir(parents=True, exist_ok=True)
    root.mkdir(mode=0o700)
    os.umask(0o077)
    tmpdir = root / "tmp"
    tmpdir.mkdir(mode=0o700)
    target = repo / "target/nu11-callgrind-build" / f"{commit}-{run_id}-{attempt}"
    if target.exists():
        print(f"refusing to reuse NU11 build directory: {target}", file=sys.stderr)
        return 2
    target.parent.mkdir(parents=True, exist_ok=True)
    env = limited_environment(source_env, tmpdir=tmpdir, target=target)
    evidence = Evidence(root, env, repo)
    evidence.started -= time.time() - int(started_utc)
    try:
        execute(evidence, repo, Path(env["ASSURA_NU11_FIXTURE_ROOT"]), env, root)
    except (GateFailure, OSError, subprocess.SubprocessError, json.JSONDecodeError) as error:
        evidence.state["outcome"] = "failed_or_inconclusive"
        evidence.state["failure"] = str(error)
        evidence.state["ended_utc"] = time.time()
        evidence.save()
        print(f"NU11 diagnostic stopped: {error}", file=sys.stderr)
        return 1
    print(f"NU11 diagnostic evidence: {root}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
