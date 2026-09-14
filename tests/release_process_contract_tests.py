#!/usr/bin/env python3
"""Focused tests for the version-to-release evidence contract."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import stat
import sys
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
sys.path.insert(0, str(ROOT / ".trellis" / "scripts"))

_SPEC = importlib.util.spec_from_file_location(
    "assura_release_contract", ROOT / "scripts" / "release-contract.py"
)
assert _SPEC is not None and _SPEC.loader is not None
_MODULE = importlib.util.module_from_spec(_SPEC)
_SPEC.loader.exec_module(_MODULE)

RELEASE_ARCHIVES = _MODULE.REQUIRED_ARCHIVES
ReleaseContractError = _MODULE.ReleaseContractError
build_asset_manifest = _MODULE.build_asset_manifest
build_release_receipt = _MODULE.build_release_receipt
plan_asset_uploads = _MODULE.plan_asset_uploads
verify_archive = _MODULE.verify_archive
verify_checksum_file = _MODULE.verify_checksum_file

from common.delivery import (  # noqa: E402
    DeliveryIntent,
    validate_release_receipt,
)


def _fake_binary(path: Path, version: str) -> None:
    path.write_text(
        "#!/bin/sh\n" f"printf '%s\\n' 'assura {version}'\n", encoding="utf-8"
    )
    path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def _make_tar(path: Path, version: str) -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        _fake_binary(root / "assura", version)
        _fake_binary(root / "assura-full", version)
        with tarfile.open(path, "w:gz") as archive:
            archive.add(root / "assura", arcname="assura")
            archive.add(root / "assura-full", arcname="assura-full")


def _make_zip(path: Path, version: str) -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        _fake_binary(root / "assura.exe", version)
        _fake_binary(root / "assura-full.exe", version)
        with zipfile.ZipFile(path, "w") as archive:
            archive.write(root / "assura.exe", "assura.exe")
            archive.write(root / "assura-full.exe", "assura-full.exe")


class ReleaseArchiveTests(unittest.TestCase):
    def test_archive_binaries_must_report_the_package_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "assura-linux-amd64.tar.gz"
            _make_tar(archive, "0.4.0")

            evidence = verify_archive(archive, "0.4.0")

            self.assertEqual(evidence["version"], "0.4.0")

    def test_wrong_version_archive_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "assura-linux-amd64.tar.gz"
            _make_tar(archive, "0.3.0")

            with self.assertRaises(ReleaseContractError):
                verify_archive(archive, "0.4.0")

    def test_windows_archive_contains_and_verifies_both_exe_companions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "assura-windows-amd64.zip"
            _make_zip(archive, "0.4.0")

            evidence = verify_archive(archive, "0.4.0")

            self.assertEqual(evidence["version"], "0.4.0")
            self.assertEqual(
                {name for name in evidence if name.endswith(".exe")},
                {"assura.exe", "assura-full.exe"},
            )

    def test_checksum_sidecar_rejects_corrupt_archive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / RELEASE_ARCHIVES[0]
            archive.write_bytes(b"release")
            checksum = root / f"{archive.name}.sha256"
            checksum.write_text(f"{'0' * 64}  {archive.name}\n", encoding="utf-8")

            with self.assertRaises(ReleaseContractError):
                verify_checksum_file(archive, checksum)


class ReleaseRetryTests(unittest.TestCase):
    def test_matching_existing_assets_are_noop_and_missing_assets_upload(self) -> None:
        local = {
            "assura-linux-amd64.tar.gz": {
                "size": 10,
                "sha256": "a" * 64,
            },
            "assura-macos-arm64.tar.gz": {
                "size": 11,
                "sha256": "b" * 64,
            },
        }
        remote = [
            {"name": "assura-linux-amd64.tar.gz", "size": 10, "digest": "sha256:" + "a" * 64}
        ]

        actions = plan_asset_uploads(local, remote)

        self.assertEqual(actions["upload"], ["assura-macos-arm64.tar.gz"])
        self.assertEqual(actions["skip"], ["assura-linux-amd64.tar.gz"])

    def test_conflicting_existing_asset_stops_retry(self) -> None:
        with self.assertRaises(ReleaseContractError):
            plan_asset_uploads(
                {"assura-linux-amd64.tar.gz": {"size": 10, "sha256": "a" * 64}},
                [{"name": "assura-linux-amd64.tar.gz", "size": 10, "digest": "sha256:" + "b" * 64}],
            )

    def test_missing_remote_digest_stops_retry(self) -> None:
        with self.assertRaises(ReleaseContractError):
            plan_asset_uploads(
                {"assura-linux-amd64.tar.gz": {"size": 10, "sha256": "a" * 64}},
                [{"name": "assura-linux-amd64.tar.gz", "size": 10}],
            )


class ReleaseReceiptTests(unittest.TestCase):
    def test_receipt_binds_tag_commit_workflow_assets_and_install_proof(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            assets = {}
            for name in RELEASE_ARCHIVES:
                archive = root / name
                archive.write_bytes(name.encode("utf-8"))
                digest = hashlib.sha256(archive.read_bytes()).hexdigest()
                checksum = root / f"{name}.sha256"
                checksum.write_text(f"{digest}  {name}\n", encoding="utf-8")
                assets[name] = {
                    "sha256": digest,
                    "size": archive.stat().st_size,
                    "checksum_name": checksum.name,
                    "members": (
                        ["assura.exe", "assura-full.exe"]
                        if name.endswith(".zip")
                        else ["assura", "assura-full"]
                    ),
                }

            receipt = build_release_receipt(
                repository="rothnic/assura",
                version="0.4.0",
                tag="v0.4.0",
                tag_oid="1" * 40,
                commit_oid="2" * 40,
                workflow_runs=[{"workflow": "Release", "run_id": "123"}],
                assets=assets,
                install_proof={"assura": "assura 0.4.0", "assura-full": "assura 0.4.0"},
                publish={
                    "action": "verified",
                    "repository": "rothnic/assura",
                    "tag": "v0.4.0",
                    "verified_assets": list(RELEASE_ARCHIVES),
                },
            )
            intent = DeliveryIntent(
                schema_version=1,
                kind="release",
                candidate_id="release-040",
                owner="tester",
                repository="rothnic/assura",
                base_ref="refs/heads/master",
                version="0.4.0",
                required_assets=tuple(RELEASE_ARCHIVES),
            )

            self.assertTrue(validate_release_receipt(receipt, intent, "2" * 40))
            receipt["commit_oid"] = "3" * 40
            self.assertFalse(validate_release_receipt(receipt, intent, "2" * 40))

    def test_manifest_requires_every_durable_archive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ReleaseContractError):
                build_asset_manifest(Path(directory), "0.4.0")

    def test_receipt_rejects_wrong_tag_and_failed_install_proof(self) -> None:
        with self.assertRaises(ReleaseContractError):
            build_release_receipt(
                repository="rothnic/assura",
                version="0.4.0",
                tag="v0.3.0",
                tag_oid="1" * 40,
                commit_oid="2" * 40,
                workflow_runs=[{"workflow": "Release", "run_id": "123"}],
                assets={},
                install_proof={},
                publish={
                    "action": "verified",
                    "repository": "rothnic/assura",
                    "tag": "v0.3.0",
                },
            )


class ReleaseWorkflowContractTests(unittest.TestCase):
    def test_release_workflow_has_no_unconditional_overwrite_path(self) -> None:
        workflow = (ROOT / ".github" / "workflows" / "release.yml").read_text(
            encoding="utf-8"
        )
        self.assertNotIn("--clobber", workflow)
        self.assertIn("release-contract.py assert-version", workflow)
        self.assertIn("release-contract.py assert-archive", workflow)
        self.assertIn("assura-release-evidence", workflow)
        self.assertIn("publish-release-assets.py", workflow)
        self.assertIn(
            "--verify-tag",
            (ROOT / "scripts" / "publish-release-assets.py").read_text(encoding="utf-8"),
        )

    def test_existing_release_identity_must_include_the_requested_tag(self) -> None:
        publisher = (ROOT / "scripts" / "publish-release-assets.py").read_text(
            encoding="utf-8"
        )
        self.assertIn('existing.get("tag_name") != tag', publisher)


if __name__ == "__main__":
    unittest.main(verbosity=2)
