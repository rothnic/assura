#!/usr/bin/env python3
"""Focused POSIX installer PATH-resolution regressions using fake release archives."""

from __future__ import annotations

import hashlib
import os
import shutil
import stat
import subprocess
import tarfile
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INSTALLER = ROOT / "website" / "public" / "install.sh"


def _fake_binary(path: Path, version: str) -> None:
    path.write_text(
        "#!/bin/sh\n"
        f"printf '%s\\n' 'assura {version}'\n",
        encoding="utf-8",
    )
    path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


class InstallerPathResolutionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="assura installer ")
        self.root = Path(self.temp.name)
        self.old_bin = self.root / "older bin"
        self.new_bin = self.root / "new install's bin"
        self.alias_bin = self.root / "alias bin"
        self.old_bin.mkdir()
        self.alias_bin.symlink_to(self.new_bin, target_is_directory=True)
        _fake_binary(self.old_bin / "assura", "0.4.0")
        _fake_binary(self.old_bin / "assura-full", "0.4.0")

        self.archive = self.root / "release.tar.gz"
        package = self.root / "package"
        package.mkdir()
        _fake_binary(package / "assura", "0.4.1")
        _fake_binary(package / "assura-full", "0.4.1")
        with tarfile.open(self.archive, "w:gz") as bundle:
            bundle.add(package / "assura", arcname="assura")
            bundle.add(package / "assura-full", arcname="assura-full")
        self.checksum = self.root / "release.tar.gz.sha256"
        digest = hashlib.sha256(self.archive.read_bytes()).hexdigest()
        self.checksum.write_text(f"{digest}  release.tar.gz\n", encoding="utf-8")

    def tearDown(self) -> None:
        self.temp.cleanup()

    def install(
        self, path: str, checksum: Path | None = None
    ) -> subprocess.CompletedProcess[str]:
        env = os.environ.copy()
        env.update(
            {
                "ASSURA_ASSET_URL": str(self.archive),
                "ASSURA_CHECKSUM_URL": str(checksum or self.checksum),
                "BIN_DIR": str(self.new_bin),
                "PATH": path,
            }
        )
        return subprocess.run(
            [str(INSTALLER)], capture_output=True, text=True, env=env, check=False
        )

    def test_old_first_path_reports_shadow_and_correction_selects_new_binaries(self) -> None:
        path = os.pathsep.join((str(self.old_bin), str(self.new_bin), os.environ["PATH"]))

        result = self.install(path)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Installed version: assura 0.4.1", result.stdout)
        self.assertIn(
            f"Current PATH resolves assura to {self.old_bin / 'assura'} (assura 0.4.0)",
            result.stdout,
        )
        self.assertIn("export PATH=", result.stdout)
        for binary, version in (
            (self.new_bin / "assura", "assura 0.4.1"),
            (self.new_bin / "assura-full", "assura 0.4.1"),
            (self.old_bin / "assura", "assura 0.4.0"),
            (self.old_bin / "assura-full", "assura 0.4.0"),
        ):
            output = subprocess.check_output([str(binary), "--version"], text=True)
            self.assertEqual(output.strip(), version)

        correction = next(
            line.strip()
            for line in result.stdout.splitlines()
            if line.strip().startswith("export PATH=")
        )
        canary = subprocess.run(
            [
                "/bin/sh",
                "-c",
                'eval "$1"; command -v assura; assura --version; assura-full --version',
                "installer-canary",
                correction,
            ],
            capture_output=True,
            text=True,
            env={**os.environ, "PATH": path},
            check=False,
        )
        self.assertEqual(canary.returncode, 0, canary.stderr)
        self.assertEqual(
            canary.stdout.splitlines(),
            [str(self.new_bin / "assura"), "assura 0.4.1", "assura 0.4.1"],
        )

    def test_new_first_path_does_not_suggest_a_correction(self) -> None:
        path = os.pathsep.join((str(self.new_bin), str(self.old_bin), os.environ["PATH"]))

        result = self.install(path)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("Installed version: assura 0.4.1", result.stdout)
        self.assertIn(
            f"Current PATH resolves assura to {self.new_bin / 'assura'} (assura 0.4.1)",
            result.stdout,
        )
        self.assertNotIn("export PATH=", result.stdout)

    def test_symlinked_path_to_destination_does_not_suggest_a_correction(self) -> None:
        path = os.pathsep.join((str(self.alias_bin), str(self.old_bin), os.environ["PATH"]))

        result = self.install(path)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(
            f"Current PATH resolves assura to {self.alias_bin / 'assura'} (assura 0.4.1)",
            result.stdout,
        )
        self.assertNotIn("export PATH=", result.stdout)

    def test_destination_missing_from_path_keeps_add_path_guidance(self) -> None:
        path = os.pathsep.join(("/usr/bin", "/bin"))
        if shutil.which("install", path=path) is None:
            self.skipTest("system install utility is unavailable on the controlled PATH")

        result = self.install(path)

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(f"Add {self.new_bin} to PATH before running assura.", result.stdout)
        self.assertIn("Current PATH resolves assura to no command", result.stdout)
        self.assertIn("export PATH=", result.stdout)

    def test_invalid_checksum_preserves_existing_destination_binaries(self) -> None:
        self.new_bin.mkdir()
        _fake_binary(self.new_bin / "assura", "0.4.0")
        _fake_binary(self.new_bin / "assura-full", "0.4.0")
        bad_checksum = self.root / "invalid.sha256"
        bad_checksum.write_text(f"{'0' * 64}  release.tar.gz\n", encoding="utf-8")

        result = self.install(os.environ["PATH"], bad_checksum)

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("checksum mismatch", result.stderr)
        for binary in (self.new_bin / "assura", self.new_bin / "assura-full"):
            output = subprocess.check_output([str(binary), "--version"], text=True)
            self.assertEqual(output.strip(), "assura 0.4.0")


if __name__ == "__main__":
    unittest.main()
