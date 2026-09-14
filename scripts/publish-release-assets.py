#!/usr/bin/env python3
"""Publish release assets without replacing an existing conflicting asset."""

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


SCRIPT_DIR = Path(__file__).resolve().parent
CONTRACT_PATH = SCRIPT_DIR / "release-contract.py"
_SPEC = importlib.util.spec_from_file_location("assura_release_contract", CONTRACT_PATH)
if _SPEC is None or _SPEC.loader is None:  # pragma: no cover - import failure is environment setup
    raise RuntimeError(f"cannot load release contract helper: {CONTRACT_PATH}")
_CONTRACT = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_CONTRACT)

ReleaseContractError = _CONTRACT.ReleaseContractError
REQUIRED_ARCHIVES = _CONTRACT.REQUIRED_ARCHIVES
build_asset_manifest = _CONTRACT.build_asset_manifest
plan_asset_uploads = _CONTRACT.plan_asset_uploads
sha256_file = _CONTRACT.sha256_file


def _run_gh(arguments: list[str]) -> tuple[int, str, str]:
    try:
        result = subprocess.run(
            ["gh", *arguments],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
        )
    except OSError as error:
        raise ReleaseContractError(f"GitHub CLI unavailable: {error}") from error
    return result.returncode, result.stdout, result.stderr


def _read_release(repository: str, tag: str) -> dict[str, Any] | None:
    code, stdout, stderr = _run_gh(
        ["api", f"repos/{repository}/releases/tags/{tag}"]
    )
    if code != 0:
        detail = f"{stdout}\n{stderr}".lower()
        if "404" in detail or "not found" in detail:
            return None
        raise ReleaseContractError(stderr.strip() or "unable to inspect GitHub release")
    try:
        value = json.loads(stdout)
    except json.JSONDecodeError as error:
        raise ReleaseContractError("GitHub release API returned invalid JSON") from error
    if not isinstance(value, dict):
        raise ReleaseContractError("GitHub release API returned a non-object")
    return value


def _local_assets(
    assets_dir: Path,
    version: str,
) -> dict[str, dict[str, Any]]:
    manifest = build_asset_manifest(assets_dir, version, execute_archives=False)
    result: dict[str, dict[str, Any]] = {}
    for archive_name in REQUIRED_ARCHIVES:
        archive = Path(assets_dir) / archive_name
        checksum = Path(assets_dir) / f"{archive_name}.sha256"
        result[archive_name] = {**manifest[archive_name], "path": archive}
        result[checksum.name] = {
            "name": checksum.name,
            "size": checksum.stat().st_size,
            "sha256": sha256_file(checksum),
            "path": checksum,
        }
    return result


def _write_json(path: Path, value: Any) -> None:
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    fd, temporary_name = tempfile.mkstemp(prefix=f".{path.name}.", suffix=".tmp", dir=path.parent)
    temporary = Path(temporary_name)
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as output:
            json.dump(value, output, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    finally:
        try:
            temporary.unlink()
        except FileNotFoundError:
            pass


def publish_assets(
    repository: str,
    tag: str,
    version: str,
    assets_dir: Path,
) -> dict[str, Any]:
    """Create a release or upload only missing, identity-verified assets."""
    assets_dir = Path(assets_dir)
    local_assets = _local_assets(assets_dir, version)
    existing = _read_release(repository, tag)
    if existing is None:
        paths = [str(facts["path"]) for facts in local_assets.values()]
        code, _, stderr = _run_gh(
            [
                "release",
                "create",
                tag,
                *paths,
                "--verify-tag",
                "--repo",
                repository,
                "--title",
                f"Release {tag}",
                "--notes",
                f"Prebuilt Assura CLI archives for {tag}.",
            ]
        )
        if code != 0:
            raise ReleaseContractError(stderr.strip() or "unable to create GitHub release")
        action = "created"
        uploaded = list(local_assets)
        skipped: list[str] = []
    else:
        if existing.get("tag_name") != tag:
            raise ReleaseContractError("existing GitHub release tag does not match requested tag")
        if existing.get("draft") is True:
            raise ReleaseContractError("existing GitHub release is a draft; investigate before retry")
        plan = plan_asset_uploads(local_assets, existing.get("assets") or [])
        uploaded = plan["upload"]
        skipped = plan["skip"]
        if uploaded:
            code, _, stderr = _run_gh(
                [
                    "release",
                    "upload",
                    tag,
                    *(str(local_assets[name]["path"]) for name in uploaded),
                    "--repo",
                    repository,
                ]
            )
            if code != 0:
                raise ReleaseContractError(stderr.strip() or "unable to upload missing release assets")
            action = "uploaded"
        else:
            action = "verified"

    verified = _read_release(repository, tag)
    if verified is None:
        raise ReleaseContractError("GitHub release disappeared before post-publish verification")
    final_plan = plan_asset_uploads(local_assets, verified.get("assets") or [])
    if final_plan["upload"]:
        raise ReleaseContractError(
            "post-publish release verification is missing assets: "
            + ", ".join(final_plan["upload"])
        )
    return {
        "action": action,
        "repository": repository,
        "tag": tag,
        "release_url": verified.get("html_url") or verified.get("url"),
        "uploaded_assets": uploaded,
        "skipped_assets": skipped,
        "verified_assets": final_plan["skip"],
    }


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--assets-dir", required=True)
    parser.add_argument("--output", required=True)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        result = publish_assets(args.repository, args.tag, args.version, Path(args.assets_dir))
        _write_json(Path(args.output), result)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    except ReleaseContractError as error:
        print(f"release publication stopped: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
