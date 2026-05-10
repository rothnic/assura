#!/usr/bin/env python3
"""
Standalone reader for .trellis/config.yaml.

Callers such as hooks can read configuration without importing the full
task/repo helpers. Returns an empty dict on missing/malformed files so callers
stay simple.
"""

from __future__ import annotations

from pathlib import Path
from typing import Optional

try:
    from .simple_yaml import parse_simple_yaml
except ImportError:  # pragma: no cover - supports direct script execution.
    from simple_yaml import parse_simple_yaml


CONFIG_REL_PATH = ".trellis/config.yaml"


def read_trellis_config(repo_root: Optional[Path] = None) -> dict:
    """Read .trellis/config.yaml. Returns {} on missing or malformed file."""
    root = repo_root or Path.cwd()
    config_file = root / CONFIG_REL_PATH
    try:
        content = config_file.read_text(encoding="utf-8")
    except (FileNotFoundError, OSError):
        return {}
    try:
        parsed = parse_simple_yaml(content)
    except Exception:
        return {}
    return parsed if isinstance(parsed, dict) else {}
