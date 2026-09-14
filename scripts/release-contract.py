#!/usr/bin/env python3
"""Verify durable Assura release assets and write bounded release evidence.

The release workflow calls this module on the build runner before upload and on
the publish runner after all five platform archives are present.  It does not
publish or overwrite anything; publication retry policy lives in
``publish-release-assets.py``.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
import subprocess
import sys
import tarfile
import tempfile
import zipfile
from pathlib import Path, PurePosixPath
from typing import Any, Iterable


REQUIRED_ARCHIVES = (
    "assura-linux-amd64.tar.gz",
    "assura-linux-musl-amd64.tar.gz",
    "assura-macos-amd64.tar.gz",
    "assura-macos-arm64.tar.gz",
    "assura-windows-amd64.zip",
)
_SHA256 = re.compile(r"^[0-9a-fA-F]{64}$")
_OID = re.compile(r"^[0-9a-fA-F]{40}$")


class ReleaseContractError(ValueError):
    """A release artifact or evidence record violates the contract."""


def _version(value: str) -> str:
    if not re.fullmatch(
        r"\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?"
        r"(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?",
        value,
    ):
        raise ReleaseContractError(f"invalid package version: {value!r}")
    return value


def verify_version_output(output: str, expected_version: str) -> dict[str, str]:
    """Require the exact public ``--version`` spelling for both binaries."""
    expected_version = _version(expected_version)
    actual = output.strip()
    expected = f"assura {expected_version}"
    if actual != expected:
        raise ReleaseContractError(
            f"binary version mismatch: expected {expected!r}, got {actual!r}"
        )
    return {"version": expected_version, "output": actual}


def verify_binary_version(binary: Path, expected_version: str) -> dict[str, str]:
    """Execute one built binary and verify its exact version output."""
    binary = Path(binary)
    try:
        result = subprocess.run(
            [str(binary), "--version"],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            check=False,
            timeout=30,
        )
    except (OSError, subprocess.SubprocessError) as error:
        raise ReleaseContractError(f"cannot execute {binary}: {error}") from error
    if result.returncode != 0:
        raise ReleaseContractError(
            f"binary version command failed for {binary} with exit {result.returncode}"
        )
    verified = verify_version_output(result.stdout, expected_version)
    return {"binary": binary.name, **verified}


def _safe_member_name(name: str) -> str:
    normalized = name.replace("\\", "/")
    path = PurePosixPath(normalized)
    if path.is_absolute() or ".." in path.parts or len(path.parts) != 1:
        raise ReleaseContractError(f"unsafe archive member: {name!r}")
    if path.name != normalized or not path.name:
        raise ReleaseContractError(f"invalid archive member: {name!r}")
    return path.name


def _expected_members(archive: Path) -> tuple[str, str]:
    if archive.name.endswith(".zip"):
        return "assura.exe", "assura-full.exe"
    if archive.name.endswith(".tar.gz"):
        return "assura", "assura-full"
    raise ReleaseContractError(f"unsupported release archive type: {archive.name}")


def _archive_members(archive: Path) -> tuple[list[str], dict[str, bytes]]:
    expected = _expected_members(archive)
    members: list[str] = []
    contents: dict[str, bytes] = {}
    if archive.name.endswith(".zip"):
        try:
            with zipfile.ZipFile(archive) as source:
                for info in source.infolist():
                    name = _safe_member_name(info.filename)
                    if name in members:
                        raise ReleaseContractError(f"duplicate archive member: {name}")
                    file_mode = info.external_attr >> 16
                    if stat.S_ISLNK(file_mode):
                        raise ReleaseContractError(
                            f"release archive member is a symbolic link: {name}"
                        )
                    if info.is_dir():
                        continue
                    members.append(name)
                    contents[name] = source.read(info)
        except (OSError, zipfile.BadZipFile) as error:
            raise ReleaseContractError(f"cannot read archive {archive}: {error}") from error
    else:
        try:
            with tarfile.open(archive, "r:gz") as source:
                for member in source.getmembers():
                    name = _safe_member_name(member.name)
                    if name in members:
                        raise ReleaseContractError(f"duplicate archive member: {name}")
                    if not member.isfile():
                        raise ReleaseContractError(
                            f"release archive member is not a regular file: {name}"
                        )
                    extracted = source.extractfile(member)
                    if extracted is None:
                        raise ReleaseContractError(f"cannot extract archive member: {name}")
                    members.append(name)
                    contents[name] = extracted.read()
        except (OSError, tarfile.TarError) as error:
            raise ReleaseContractError(f"cannot read archive {archive}: {error}") from error
    missing = sorted(set(expected) - set(members))
    if missing:
        raise ReleaseContractError(
            f"release archive {archive.name} is missing: {', '.join(missing)}"
        )
    return sorted(members), contents


def verify_archive(
    archive: Path,
    expected_version: str,
    execute: bool = True,
) -> dict[str, Any]:
    """Verify required archive members and, optionally, execute both binaries."""
    archive = Path(archive)
    expected_version = _version(expected_version)
    members, contents = _archive_members(archive)
    result: dict[str, Any] = {"archive": archive.name, "members": members}
    if not execute:
        return result

    with tempfile.TemporaryDirectory(prefix="assura-release-archive-") as directory:
        extracted_root = Path(directory)
        for name in _expected_members(archive):
            binary = extracted_root / name
            binary.write_bytes(contents[name])
            if os.name != "nt":
                binary.chmod(binary.stat().st_mode | 0o100)
            result[name] = verify_binary_version(binary, expected_version)
    result["version"] = expected_version
    return result


def sha256_file(path: Path) -> str:
    """Return the lowercase SHA-256 digest of one bounded release asset."""
    digest = hashlib.sha256()
    with Path(path).open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def verify_checksum_file(archive: Path, checksum: Path) -> dict[str, Any]:
    """Verify a checksum file names the archive and matches its bytes."""
    lines = [line.strip() for line in Path(checksum).read_text(encoding="utf-8").splitlines() if line.strip()]
    if len(lines) != 1:
        raise ReleaseContractError(f"checksum file must contain exactly one record: {checksum}")
    line = lines[0]
    parts = line.split(maxsplit=1)
    if len(parts) != 2 or not _SHA256.fullmatch(parts[0]):
        raise ReleaseContractError(f"invalid SHA-256 record: {checksum}")
    recorded_name = parts[1].lstrip("*").strip()
    if Path(recorded_name).name != Path(archive).name:
        raise ReleaseContractError(
            f"checksum {checksum.name} names {recorded_name!r}, expected {Path(archive).name!r}"
        )
    actual = sha256_file(archive)
    if parts[0].lower() != actual:
        raise ReleaseContractError(f"checksum mismatch for {Path(archive).name}")
    return {
        "name": Path(archive).name,
        "sha256": actual,
        "checksum_name": Path(checksum).name,
        "size": Path(archive).stat().st_size,
    }


def build_asset_manifest(
    assets_dir: Path,
    expected_version: str,
    execute_archives: bool = False,
) -> dict[str, dict[str, Any]]:
    """Verify all required durable archives and return deterministic metadata."""
    assets_dir = Path(assets_dir)
    manifest: dict[str, dict[str, Any]] = {}
    for name in REQUIRED_ARCHIVES:
        archive = assets_dir / name
        checksum = assets_dir / f"{name}.sha256"
        if not archive.is_file() or not checksum.is_file():
            raise ReleaseContractError(f"missing required release asset: {name}")
        metadata = verify_checksum_file(archive, checksum)
        archive_result = verify_archive(archive, expected_version, execute_archives)
        manifest[name] = {**metadata, "members": archive_result["members"]}
    return manifest


def build_release_build_metadata(
    asset_name: str,
    archive_name: str,
    version: str,
) -> dict[str, Any]:
    """Record the assertions completed by one durable build matrix job."""
    version = _version(version)
    if not asset_name or not archive_name:
        raise ReleaseContractError("release build metadata requires asset and archive names")
    if archive_name not in REQUIRED_ARCHIVES:
        raise ReleaseContractError(f"unsupported durable archive: {archive_name}")
    return {
        "schema_version": "assura.release-build.v1",
        "asset_name": asset_name,
        "archive_name": archive_name,
        "version": version,
        "binary_versions": {
            "assura": f"assura {version}",
            "assura-full": f"assura {version}",
        },
        "archive_verified": True,
    }


def _asset_digest(value: Any) -> str:
    if not isinstance(value, str):
        raise ReleaseContractError("existing release asset has no SHA-256 digest")
    digest = value.removeprefix("sha256:").lower()
    if not _SHA256.fullmatch(digest):
        raise ReleaseContractError(f"existing release asset has invalid digest: {value!r}")
    return digest


def plan_asset_uploads(
    local_assets: dict[str, dict[str, Any]],
    remote_assets: Iterable[dict[str, Any]],
) -> dict[str, list[str]]:
    """Plan a safe retry: matching assets skip, conflicts fail, missing upload."""
    remote_by_name: dict[str, dict[str, Any]] = {}
    for asset in remote_assets:
        name = asset.get("name")
        if not isinstance(name, str):
            continue
        if name in remote_by_name:
            raise ReleaseContractError(f"duplicate remote release asset: {name}")
        remote_by_name[name] = asset

    upload: list[str] = []
    skip: list[str] = []
    for name, local in local_assets.items():
        remote = remote_by_name.get(name)
        if remote is None:
            upload.append(name)
            continue
        expected_digest = local.get("sha256")
        expected_size = local.get("size")
        if not isinstance(expected_digest, str) or not _SHA256.fullmatch(expected_digest):
            raise ReleaseContractError(f"local release asset has invalid digest: {name}")
        if remote.get("size") != expected_size or _asset_digest(remote.get("digest")) != expected_digest.lower():
            raise ReleaseContractError(
                f"conflicting durable release asset will not be overwritten: {name}"
            )
        skip.append(name)
    return {"upload": upload, "skip": skip}


def _require_oid(value: str, field: str) -> str:
    if not _OID.fullmatch(value):
        raise ReleaseContractError(f"{field} must be a full 40-character Git object id")
    return value.lower()


def build_release_receipt(
    repository: str,
    version: str,
    tag: str,
    tag_oid: str,
    commit_oid: str,
    workflow_runs: list[dict[str, Any]],
    assets: dict[str, dict[str, Any]],
    install_proof: dict[str, str],
    publish: dict[str, Any],
) -> dict[str, Any]:
    """Build the durable release evidence record consumed by delivery audits."""
    version = _version(version)
    if tag != f"v{version}":
        raise ReleaseContractError(f"tag {tag!r} does not match package version {version!r}")
    if not repository or not isinstance(repository, str):
        raise ReleaseContractError("release receipt requires repository identity")
    if not workflow_runs or any(
        not isinstance(run, dict)
        or not isinstance(run.get("workflow"), str)
        or not run["workflow"].strip()
        or not isinstance(run.get("run_id"), (str, int))
        or not str(run["run_id"]).strip()
        for run in workflow_runs
    ):
        raise ReleaseContractError("release receipt requires workflow run evidence")
    tag_oid = _require_oid(tag_oid, "tag_oid")
    commit_oid = _require_oid(commit_oid, "commit_oid")
    if not isinstance(publish, dict) or publish.get("action") not in {
        "created",
        "uploaded",
        "verified",
    }:
        raise ReleaseContractError("release receipt requires a verified publication result")
    if publish.get("repository") != repository or publish.get("tag") != tag:
        raise ReleaseContractError(
            "release receipt publication result is bound to a different repository or tag"
        )
    if not isinstance(install_proof, dict):
        raise ReleaseContractError("release receipt requires an install proof object")
    installed: dict[str, str] = {}
    for binary in ("assura", "assura-full"):
        output = install_proof.get(binary)
        if not isinstance(output, str):
            raise ReleaseContractError(f"release receipt lacks installed proof for {binary}")
        installed[binary] = verify_version_output(output, version)["output"]
    normalized_assets = []
    for name in REQUIRED_ARCHIVES:
        metadata = assets.get(name)
        if not isinstance(metadata, dict):
            raise ReleaseContractError(f"release receipt lacks asset metadata for {name}")
        digest = metadata.get("sha256")
        if not isinstance(digest, str) or not _SHA256.fullmatch(digest):
            raise ReleaseContractError(f"release receipt has invalid asset digest for {name}")
        if not isinstance(metadata.get("size"), int) or metadata["size"] < 0:
            raise ReleaseContractError(f"release receipt has invalid asset size for {name}")
        if metadata.get("checksum_name") != f"{name}.sha256":
            raise ReleaseContractError(f"release receipt has invalid checksum metadata for {name}")
        members = metadata.get("members")
        if not isinstance(members, list) or not all(
            isinstance(member, str) and member for member in members
        ):
            raise ReleaseContractError(f"release receipt has invalid member metadata for {name}")
        expected_members = (
            {"assura.exe", "assura-full.exe"}
            if name.endswith(".zip")
            else {"assura", "assura-full"}
        )
        if not expected_members.issubset(members):
            raise ReleaseContractError(f"release receipt omits a binary member for {name}")
        normalized_assets.append(
            {
                "name": name,
                "size": metadata["size"],
                "sha256": digest.lower(),
                "checksum_name": metadata["checksum_name"],
                "members": members,
            }
        )
    verified_assets = publish.get("verified_assets")
    if not isinstance(verified_assets, list) or not all(
        isinstance(name, str) for name in verified_assets
    ) or not set(REQUIRED_ARCHIVES).issubset(verified_assets):
        raise ReleaseContractError("release receipt publication evidence omits a required archive")
    return {
        "schema_version": "assura.release-receipt.v1",
        "repository": repository,
        "version": version,
        "tag": tag,
        "tag_oid": tag_oid,
        "commit_oid": commit_oid,
        "workflow_runs": workflow_runs,
        "assets": normalized_assets,
        "checksums_verified": True,
        "install": {"verified": True, **installed},
        "publish": publish,
    }


def _read_json(path: Path) -> Any:
    try:
        return json.loads(Path(path).read_text(encoding="utf-8"))
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ReleaseContractError(f"cannot read JSON: {path}") from error


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


def _parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)

    binary = subparsers.add_parser("assert-version")
    binary.add_argument("--version", required=True)
    binary.add_argument("--binary", action="append", required=True)

    archive = subparsers.add_parser("assert-archive")
    archive.add_argument("--version", required=True)
    archive.add_argument("--archive", required=True)
    archive.add_argument("--no-execute", action="store_true")

    assets = subparsers.add_parser("verify-assets")
    assets.add_argument("--version", required=True)
    assets.add_argument("--assets-dir", required=True)
    assets.add_argument("--execute-archives", action="store_true")
    assets.add_argument("--output")

    metadata = subparsers.add_parser("build-metadata")
    metadata.add_argument("--asset-name", required=True)
    metadata.add_argument("--archive-name", required=True)
    metadata.add_argument("--version", required=True)
    metadata.add_argument("--output", required=True)

    receipt = subparsers.add_parser("receipt")
    receipt.add_argument("--repository", required=True)
    receipt.add_argument("--version", required=True)
    receipt.add_argument("--tag", required=True)
    receipt.add_argument("--tag-oid", required=True)
    receipt.add_argument("--commit-oid", required=True)
    receipt.add_argument("--workflow-run-id", action="append", required=True)
    receipt.add_argument("--workflow-name", default="Release")
    receipt.add_argument("--assets-dir", required=True)
    receipt.add_argument("--install-dir", required=True)
    receipt.add_argument("--publish-json", required=True)
    receipt.add_argument("--output", required=True)
    return parser


def main(argv: list[str] | None = None) -> int:
    args = _parser().parse_args(argv)
    try:
        if args.command == "assert-version":
            result = [verify_binary_version(Path(binary), args.version) for binary in args.binary]
        elif args.command == "assert-archive":
            result = verify_archive(Path(args.archive), args.version, execute=not args.no_execute)
        elif args.command == "verify-assets":
            result = build_asset_manifest(Path(args.assets_dir), args.version, args.execute_archives)
            if args.output:
                _write_json(Path(args.output), result)
        elif args.command == "build-metadata":
            result = build_release_build_metadata(
                args.asset_name,
                args.archive_name,
                args.version,
            )
            _write_json(Path(args.output), result)
        else:
            manifest = build_asset_manifest(Path(args.assets_dir), args.version)
            install_dir = Path(args.install_dir)
            suffix = ".exe" if os.name == "nt" else ""
            install = {
                binary: verify_binary_version(install_dir / f"{binary}{suffix}", args.version)["output"]
                for binary in ("assura", "assura-full")
            }
            publish = _read_json(Path(args.publish_json))
            result = build_release_receipt(
                repository=args.repository,
                version=args.version,
                tag=args.tag,
                tag_oid=args.tag_oid,
                commit_oid=args.commit_oid,
                workflow_runs=[
                    {"workflow": args.workflow_name, "run_id": run_id}
                    for run_id in args.workflow_run_id
                ],
                assets=manifest,
                install_proof=install,
                publish=publish,
            )
            _write_json(Path(args.output), result)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    except ReleaseContractError as error:
        print(f"release contract failed: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
