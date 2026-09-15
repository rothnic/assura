"""
JSON file I/O utilities.

Provides read_json and write_json as the single source of truth
for JSON file operations across all Trellis scripts.
"""

from __future__ import annotations

import json
import os
from pathlib import Path
import secrets
import stat


def read_json(path: Path) -> dict | None:
    """Read and parse a JSON file.

    Returns None if the file doesn't exist, is invalid JSON, or can't be read.
    """
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (FileNotFoundError, json.JSONDecodeError, OSError):
        return None


def write_json(path: Path, data: dict) -> bool:
    """Atomically replace a JSON file with pretty-formatted dict content.

    Returns True on success, False on error.
    """
    temporary_path: Path | None = None
    try:
        target = Path(path).resolve()
        try:
            target_mode = stat.S_IMODE(target.stat().st_mode)
        except FileNotFoundError:
            target_mode = None

        for _ in range(10):
            candidate_path = target.with_name(
                f".{target.name}.{secrets.token_hex(8)}.tmp"
            )
            try:
                descriptor = os.open(
                    candidate_path,
                    os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                    0o666,
                )
                temporary_path = candidate_path
                break
            except FileExistsError:
                continue
        else:
            return False

        with os.fdopen(descriptor, "w", encoding="utf-8") as temporary:
            if target_mode is not None:
                os.chmod(temporary_path, target_mode)
            json.dump(data, temporary, indent=2, ensure_ascii=False)
            temporary.flush()
            os.fsync(temporary.fileno())
        os.replace(temporary_path, target)
        return True
    except (OSError, IOError):
        return False
    finally:
        if temporary_path is not None:
            try:
                temporary_path.unlink()
            except OSError:
                pass
