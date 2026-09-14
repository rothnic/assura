"""Atomic, generation-aware storage for delivery observations.

The delivery store is deliberately small. Task files hold durable intent;
this module holds mutable observations and evidence bindings that must survive
session changes without allowing a stale writer to replace newer state.
"""

from __future__ import annotations

import errno
import json
import os
import re
import tempfile
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator


_SAFE_ID = re.compile(r"^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$")


class DeliveryStoreError(RuntimeError):
    """Base error for delivery receipt failures."""


class ReceiptCorrupt(DeliveryStoreError):
    """A receipt exists but cannot be decoded as a delivery record."""


class StaleGeneration(DeliveryStoreError):
    """An update was based on a receipt older than the stored receipt."""


class ReceiptExists(DeliveryStoreError):
    """A candidate receipt already exists."""


class ReceiptBusy(DeliveryStoreError):
    """Another process currently owns the candidate receipt lock."""


def _validate_candidate_id(candidate_id: str) -> str:
    if not isinstance(candidate_id, str) or not _SAFE_ID.fullmatch(candidate_id):
        raise ValueError(f"unsafe delivery candidate id: {candidate_id!r}")
    return candidate_id


class DeliveryStore:
    """Store one JSON receipt per delivery candidate.

    Args:
        root: Directory reserved for delivery receipts. It is created lazily.
    """

    def __init__(self, root: Path) -> None:
        self.root = Path(root)

    def path_for(self, candidate_id: str) -> Path:
        """Return the validated receipt path for ``candidate_id``."""
        return self.root / f"{_validate_candidate_id(candidate_id)}.json"

    def create(self, candidate_id: str, values: dict[str, Any]) -> dict[str, Any]:
        """Create generation-zero receipt data without overwriting existing data."""
        candidate_id = _validate_candidate_id(candidate_id)
        self._validate_values(values)
        path = self.path_for(candidate_id)
        with self._locked(candidate_id):
            if path.exists():
                raise ReceiptExists(f"delivery receipt already exists: {candidate_id}")
            record = {
                "schema_version": 1,
                "candidate_id": candidate_id,
                "generation": 0,
                **values,
            }
            self._write_atomic(path, record)
            return record

    def read(self, candidate_id: str) -> dict[str, Any]:
        """Read a receipt, raising when it is missing or corrupt."""
        path = self.path_for(candidate_id)
        try:
            data = json.loads(path.read_text(encoding="utf-8"))
        except FileNotFoundError as error:
            raise DeliveryStoreError(f"delivery receipt not found: {candidate_id}") from error
        except (OSError, UnicodeError, json.JSONDecodeError) as error:
            raise ReceiptCorrupt(f"cannot read delivery receipt: {candidate_id}") from error
        if not isinstance(data, dict) or data.get("candidate_id") != candidate_id:
            raise ReceiptCorrupt(f"invalid delivery receipt: {candidate_id}")
        if data.get("schema_version") != 1 or not isinstance(data.get("generation"), int):
            raise ReceiptCorrupt(f"unsupported delivery receipt: {candidate_id}")
        return data

    def update(
        self,
        candidate_id: str,
        expected_generation: int,
        changes: dict[str, Any],
    ) -> dict[str, Any]:
        """Compare-and-swap a receipt and return the new generation."""
        candidate_id = _validate_candidate_id(candidate_id)
        self._validate_values(changes)
        path = self.path_for(candidate_id)
        with self._locked(candidate_id):
            current = self.read(candidate_id)
            actual_generation = current["generation"]
            if actual_generation != expected_generation:
                raise StaleGeneration(
                    f"delivery receipt {candidate_id} is generation "
                    f"{actual_generation}, expected {expected_generation}"
                )
            updated = {
                **current,
                **changes,
                "generation": actual_generation + 1,
            }
            self._write_atomic(path, updated)
            return updated

    @staticmethod
    def _validate_values(values: dict[str, Any]) -> None:
        if not isinstance(values, dict):
            raise TypeError("delivery receipt values must be a dictionary")
        reserved = {"schema_version", "candidate_id", "generation"}
        overlap = reserved.intersection(values)
        if overlap:
            raise ValueError(f"receipt values cannot replace reserved fields: {sorted(overlap)}")

    def _lock_path(self, candidate_id: str) -> Path:
        _validate_candidate_id(candidate_id)
        return self.root / f"{candidate_id}.lock"

    @contextmanager
    def _locked(self, candidate_id: str) -> Iterator[None]:
        """Hold a process-releasing lock while mutating one receipt."""
        self.root.mkdir(parents=True, exist_ok=True)
        lock_path = self._lock_path(candidate_id)
        lock_path.touch(exist_ok=True)
        with lock_path.open("r+b") as lock_file:
            self._acquire_lock(lock_file)
            try:
                yield
            finally:
                self._release_lock(lock_file)

    @staticmethod
    def _acquire_lock(lock_file: Any) -> None:
        if os.name == "nt":
            import msvcrt

            try:
                lock_file.seek(0)
                if lock_file.read(1) != b"0":
                    lock_file.seek(0)
                    lock_file.write(b"0")
                    lock_file.flush()
            except PermissionError as error:
                # Windows byte-range locks can reject even the probe read
                # while another process owns the byte. Surface the same
                # recoverable condition as msvcrt.LK_NBLCK below.
                raise ReceiptBusy("delivery receipt is busy") from error
            lock_file.seek(0)
            try:
                msvcrt.locking(lock_file.fileno(), msvcrt.LK_NBLCK, 1)
            except OSError as error:
                raise ReceiptBusy("delivery receipt is busy") from error
            return

        import fcntl

        try:
            fcntl.flock(lock_file.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ReceiptBusy("delivery receipt is busy") from error
        except OSError as error:
            if error.errno in {errno.EACCES, errno.EAGAIN}:
                raise ReceiptBusy("delivery receipt is busy") from error
            raise

    @staticmethod
    def _release_lock(lock_file: Any) -> None:
        if os.name == "nt":
            import msvcrt

            lock_file.seek(0)
            msvcrt.locking(lock_file.fileno(), msvcrt.LK_UNLCK, 1)
            return

        import fcntl

        fcntl.flock(lock_file.fileno(), fcntl.LOCK_UN)

    @staticmethod
    def _write_atomic(path: Path, data: dict[str, Any]) -> None:
        path.parent.mkdir(parents=True, exist_ok=True)
        fd, temporary_name = tempfile.mkstemp(
            prefix=f".{path.name}.",
            suffix=".tmp",
            dir=path.parent,
        )
        temporary_path = Path(temporary_name)
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as temporary:
                json.dump(data, temporary, indent=2, ensure_ascii=False, sort_keys=True)
                temporary.write("\n")
                temporary.flush()
                os.fsync(temporary.fileno())
            os.replace(temporary_path, path)
        finally:
            try:
                temporary_path.unlink()
            except FileNotFoundError:
                pass
