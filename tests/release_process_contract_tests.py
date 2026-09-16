#!/usr/bin/env python3
"""Focused tests for the version-to-release evidence contract."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import copy
import stat
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile
from collections.abc import Callable
from unittest import mock
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

_PUBLISHER_SPEC = importlib.util.spec_from_file_location(
    "assura_publish_release_assets", ROOT / "scripts" / "publish-release-assets.py"
)
assert _PUBLISHER_SPEC is not None and _PUBLISHER_SPEC.loader is not None
_PUBLISHER = importlib.util.module_from_spec(_PUBLISHER_SPEC)
_PUBLISHER_SPEC.loader.exec_module(_PUBLISHER)

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


def _make_tar(path: Path, version: str, full_version: str | None = None) -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        _fake_binary(root / "assura", version)
        _fake_binary(root / "assura-full", full_version or version)
        with tarfile.open(path, "w:gz") as archive:
            archive.add(root / "assura", arcname="assura")
            archive.add(root / "assura-full", arcname="assura-full")


def _make_zip(path: Path, version: str, full_version: str | None = None) -> None:
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        _fake_binary(root / "assura.exe", version)
        _fake_binary(root / "assura-full.exe", full_version or version)
        with zipfile.ZipFile(path, "w") as archive:
            archive.write(root / "assura.exe", "assura.exe")
            archive.write(root / "assura-full.exe", "assura-full.exe")


def _run_assert_version(version: str, binary: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(ROOT / "scripts" / "release-contract.py"),
            "assert-version",
            "--version",
            version,
            "--binary",
            str(binary),
        ],
        capture_output=True,
        text=True,
        check=False,
    )


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

    def test_archive_rejects_mismatched_companion_binary_versions(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive = Path(directory) / "assura-linux-amd64.tar.gz"
            _make_tar(archive, "0.4.0", "0.3.0")

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


class ReleaseBinaryCommandTests(unittest.TestCase):
    def test_assert_version_rejects_individually_wrong_assura_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "assura"
            _fake_binary(binary, "0.3.0")

            result = _run_assert_version("0.4.0", binary)

            self.assertNotEqual(result.returncode, 0)

    def test_assert_version_rejects_individually_wrong_assura_full_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            binary = Path(directory) / "assura-full"
            _fake_binary(binary, "0.3.0")

            result = _run_assert_version("0.4.0", binary)

            self.assertNotEqual(result.returncode, 0)


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

    def test_remote_annotated_tag_must_match_tag_and_commit_identity(self) -> None:
        tag_oid = "1" * 40
        commit_oid = "2" * 40
        with mock.patch.object(
            _PUBLISHER,
            "_run_gh",
            side_effect=[
                (
                    0,
                    json.dumps(
                        {
                            "ref": "refs/tags/v0.4.0",
                            "object": {"sha": tag_oid, "type": "tag"},
                        }
                    ),
                    "",
                ),
                (
                    0,
                    json.dumps(
                        {"object": {"sha": commit_oid, "type": "commit"}}
                    ),
                    "",
                ),
            ],
        ):
            identity = _PUBLISHER.verify_remote_tag(
                "rothnic/assura", "v0.4.0", tag_oid, commit_oid
            )

        self.assertEqual(
            identity,
            {"tag_oid": tag_oid, "commit_oid": commit_oid},
        )

    def test_remote_tag_mismatch_stops_existing_release_retry(self) -> None:
        with mock.patch.object(
            _PUBLISHER,
            "_run_gh",
            return_value=(
                0,
                json.dumps(
                    {
                        "ref": "refs/tags/v0.4.0",
                        "object": {"sha": "3" * 40, "type": "commit"},
                    }
                ),
                "",
            ),
        ):
            with self.assertRaises(_PUBLISHER.ReleaseContractError):
                _PUBLISHER.verify_remote_tag(
                    "rothnic/assura", "v0.4.0", "1" * 40, "2" * 40
                )

    def _release_assets(self, root: Path) -> dict[str, dict[str, object]]:
        for name in RELEASE_ARCHIVES:
            archive = root / name
            if name.endswith(".zip"):
                _make_zip(archive, "0.4.0")
            else:
                _make_tar(archive, "0.4.0")
            digest = hashlib.sha256(archive.read_bytes()).hexdigest()
            (root / f"{name}.sha256").write_text(
                f"{digest}  {name}\n", encoding="utf-8"
            )
        return _PUBLISHER._local_assets(root, "0.4.0")

    @staticmethod
    def _remote_asset(name: str, facts: dict[str, object]) -> dict[str, object]:
        return {
            "name": name,
            "size": facts["size"],
            "digest": "sha256:" + str(facts["sha256"]),
        }

    def _gh_tag_and_release_responses(
        self,
        calls: list[list[str]],
        release: dict[str, object],
        tag_oid: str = "1" * 40,
        commit_oid: str = "2" * 40,
    ) -> Callable[[list[str]], tuple[int, str, str]]:
        def run(arguments: list[str]) -> tuple[int, str, str]:
            calls.append(arguments)
            if arguments == [
                "api",
                "repos/rothnic/assura/git/ref/tags/v0.4.0",
            ]:
                return 0, json.dumps(
                    {
                        "ref": "refs/tags/v0.4.0",
                        "object": {"sha": tag_oid, "type": "tag"},
                    }
                ), ""
            if arguments == [
                "api",
                f"repos/rothnic/assura/git/tags/{tag_oid}",
            ]:
                return 0, json.dumps(
                    {"object": {"sha": commit_oid, "type": "commit"}}
                ), ""
            if arguments == [
                "api",
                "repos/rothnic/assura/releases/tags/v0.4.0",
            ]:
                return 0, json.dumps(release), ""
            raise AssertionError(f"unexpected GitHub command: {arguments!r}")

        return run

    def test_publish_assets_matching_release_is_verified_without_upload_or_create(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            local = self._release_assets(root)
            release = {
                "tag_name": "v0.4.0",
                "draft": False,
                "html_url": "https://example.test/release",
                "assets": [self._remote_asset(name, facts) for name, facts in local.items()],
            }
            calls: list[list[str]] = []
            with mock.patch.object(
                _PUBLISHER, "_run_gh", side_effect=self._gh_tag_and_release_responses(calls, release)
            ):
                result = _PUBLISHER.publish_assets(
                    "rothnic/assura", "v0.4.0", "0.4.0", root, "1" * 40, "2" * 40
                )

            self.assertEqual(result["action"], "verified")
            self.assertEqual(result["uploaded_assets"], [])
            self.assertEqual(
                calls,
                [
                    ["api", "repos/rothnic/assura/git/ref/tags/v0.4.0"],
                    ["api", "repos/rothnic/assura/git/tags/" + "1" * 40],
                    ["api", "repos/rothnic/assura/releases/tags/v0.4.0"],
                    ["api", "repos/rothnic/assura/releases/tags/v0.4.0"],
                ],
            )

    def test_publish_assets_uploads_only_missing_release_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            local = self._release_assets(root)
            missing = [RELEASE_ARCHIVES[0], f"{RELEASE_ARCHIVES[0]}.sha256"]
            initial_assets = [
                self._remote_asset(name, facts)
                for name, facts in local.items()
                if name not in missing
            ]
            release = {
                "tag_name": "v0.4.0",
                "draft": False,
                "assets": initial_assets,
            }
            complete_release = {**release, "assets": [
                self._remote_asset(name, facts) for name, facts in local.items()
            ]}
            calls: list[list[str]] = []
            responses = self._gh_tag_and_release_responses(calls, release)

            def run(arguments: list[str]) -> tuple[int, str, str]:
                if arguments[:2] == ["release", "upload"]:
                    calls.append(arguments)
                    return 0, "", ""
                response = responses(arguments)
                if arguments == [
                    "api",
                    "repos/rothnic/assura/releases/tags/v0.4.0",
                ] and calls.count(arguments) == 2:
                    return 0, json.dumps(complete_release), ""
                return response

            with mock.patch.object(_PUBLISHER, "_run_gh", side_effect=run):
                result = _PUBLISHER.publish_assets(
                    "rothnic/assura", "v0.4.0", "0.4.0", root, "1" * 40, "2" * 40
                )

            expected_paths = [str(local[name]["path"]) for name in missing]
            self.assertEqual(result["action"], "uploaded")
            self.assertEqual(result["uploaded_assets"], missing)
            self.assertEqual(
                calls[3],
                ["release", "upload", "v0.4.0", *expected_paths, "--repo", "rothnic/assura"],
            )

    def test_publish_assets_conflict_stops_before_upload(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            local = self._release_assets(root)
            first = RELEASE_ARCHIVES[0]
            release = {
                "tag_name": "v0.4.0",
                "draft": False,
                "assets": [
                    {
                        "name": first,
                        "size": local[first]["size"],
                        "digest": "sha256:" + "f" * 64,
                    }
                ],
            }
            calls: list[list[str]] = []
            with mock.patch.object(
                _PUBLISHER, "_run_gh", side_effect=self._gh_tag_and_release_responses(calls, release)
            ):
                with self.assertRaises(_PUBLISHER.ReleaseContractError):
                    _PUBLISHER.publish_assets(
                        "rothnic/assura", "v0.4.0", "0.4.0", root, "1" * 40, "2" * 40
                    )

            self.assertEqual(len(calls), 3)
            self.assertNotIn("release", calls[-1])

    def test_publish_assets_missing_remote_digest_stops_before_upload(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            local = self._release_assets(root)
            first = RELEASE_ARCHIVES[0]
            release = {
                "tag_name": "v0.4.0",
                "draft": False,
                "assets": [
                    {"name": first, "size": local[first]["size"]}
                ],
            }
            calls: list[list[str]] = []
            with mock.patch.object(
                _PUBLISHER, "_run_gh", side_effect=self._gh_tag_and_release_responses(calls, release)
            ):
                with self.assertRaises(_PUBLISHER.ReleaseContractError):
                    _PUBLISHER.publish_assets(
                        "rothnic/assura", "v0.4.0", "0.4.0", root, "1" * 40, "2" * 40
                    )

            self.assertEqual(len(calls), 3)
            self.assertNotIn("release", calls[-1])


class ReleaseReceiptTests(unittest.TestCase):
    def _receipt(
        self,
        install_proof: dict[str, str] | None = None,
        publish_overrides: dict[str, object] | None = None,
    ) -> dict[str, object]:
        assets = {
            name: {
                "sha256": hashlib.sha256(name.encode("utf-8")).hexdigest(),
                "size": len(name),
                "checksum_name": f"{name}.sha256",
                "members": (
                    ["assura.exe", "assura-full.exe"]
                    if name.endswith(".zip")
                    else ["assura", "assura-full"]
                ),
            }
            for name in RELEASE_ARCHIVES
        }
        publish = {
            "action": "verified",
            "repository": "rothnic/assura",
            "tag": "v0.4.0",
            "tag_oid": "1" * 40,
            "commit_oid": "2" * 40,
            "verified_assets": list(RELEASE_ARCHIVES),
        }
        if publish_overrides:
            publish.update(publish_overrides)
        return build_release_receipt(
            repository="rothnic/assura",
            version="0.4.0",
            tag="v0.4.0",
            tag_oid="1" * 40,
            commit_oid="2" * 40,
            workflow_runs=[{"workflow": "Release", "run_id": "123"}],
            assets=assets,
            install_proof=install_proof or {
                "assura": "assura 0.4.0",
                "assura-full": "assura 0.4.0",
            },
            publish=publish,
        )

    @staticmethod
    def _intent() -> DeliveryIntent:
        return DeliveryIntent(
            schema_version=1,
            kind="release",
            candidate_id="release-040",
            owner="tester",
            repository="rothnic/assura",
            base_ref="refs/heads/master",
            version="0.4.0",
            required_assets=tuple(RELEASE_ARCHIVES),
        )

    def test_receipt_binds_tag_commit_workflow_assets_and_install_proof(self) -> None:
        receipt = self._receipt()
        self.assertTrue(validate_release_receipt(receipt, self._intent(), "2" * 40))
        receipt["commit_oid"] = "3" * 40
        self.assertFalse(validate_release_receipt(receipt, self._intent(), "2" * 40))

    def test_receipt_requires_matching_full_publication_identities(self) -> None:
        cases = {
            "tag_oid mismatched": {"tag_oid": "3" * 40},
            "commit_oid mismatched": {"commit_oid": "3" * 40},
            "tag_oid missing": {"tag_oid": None},
            "commit_oid short": {"commit_oid": "short"},
        }
        for label, overrides in cases.items():
            with self.subTest(label=label), self.assertRaises(ReleaseContractError):
                self._receipt(publish_overrides=overrides)

    def test_receipt_install_proof_rejects_missing_and_wrong_versions(self) -> None:
        proofs = [
            {"assura-full": "assura 0.4.0"},
            {"assura": "assura 0.3.0", "assura-full": "assura 0.4.0"},
            {"assura": "assura 0.4.0", "assura-full": "assura 0.3.0"},
        ]
        for proof in proofs:
            with self.subTest(proof=proof), self.assertRaises(ReleaseContractError):
                self._receipt(proof)

    def test_validate_release_receipt_rejects_required_schema_field_variants(self) -> None:
        cases = {
            "schema_version missing": lambda value: value.pop("schema_version"),
            "repository mismatched": lambda value: value.update(repository="other/repo"),
            "version mismatched": lambda value: value.update(version="0.3.0"),
            "tag missing": lambda value: value.pop("tag"),
            "tag_oid invalid": lambda value: value.update(tag_oid="invalid"),
            "commit_oid missing": lambda value: value.pop("commit_oid"),
            "workflow_runs missing": lambda value: value.pop("workflow_runs"),
            "assets missing": lambda value: value.pop("assets"),
            "checksums_verified mismatched": lambda value: value.update(checksums_verified=False),
            "install missing": lambda value: value.pop("install"),
            "publish mismatched": lambda value: value.update(publish={}),
        }
        for label, mutate in cases.items():
            with self.subTest(label=label):
                candidate = copy.deepcopy(self._receipt())
                mutate(candidate)
                self.assertFalse(
                    validate_release_receipt(candidate, self._intent(), "2" * 40)
                )

    def test_manifest_requires_every_durable_archive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(ReleaseContractError):
                build_asset_manifest(Path(directory), "0.4.0")

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
        self.assertIn("--tag-oid", workflow)
        self.assertIn("--commit-oid", workflow)

    def test_ci_has_windows_negative_assert_version_contract(self) -> None:
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(
            encoding="utf-8"
        )
        self.assertIn("validate-windows-release-contract", workflow)
        self.assertIn("shell: pwsh", workflow)
        self.assertIn("continue-on-error: true", workflow)
        self.assertIn("steps.negative_assert_version.outcome", workflow)
        self.assertIn("python scripts/release-contract.py assert-version `", workflow)

    def test_existing_release_identity_must_include_the_requested_tag(self) -> None:
        publisher = (ROOT / "scripts" / "publish-release-assets.py").read_text(
            encoding="utf-8"
        )
        self.assertIn('existing.get("tag_name") != tag', publisher)
        self.assertIn("verify_remote_tag", publisher)


if __name__ == "__main__":
    unittest.main(verbosity=2)
