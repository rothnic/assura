# NU05 local candidate evidence

- Base: `refs/remotes/origin/master` at `778128a89d086e946ab77b0e68ad6b73a3af71a4`.
- Branch: `fix/nu05-installer-shadow`.
- Owner: `nu05-installer-shadow`; candidate delivery registration succeeded before implementation.
- Red: `python3 tests/installer_path_resolution_tests.py InstallerPathResolutionTests.test_old_first_path_reports_shadow_and_correction_selects_new_binaries` exited 1 on the original installer because it omitted `Installed version: assura 0.4.1` and the actual PATH resolution.
- Focused green: `TMPDIR=/Users/nroth/workspace/.assura-review-20261017/nu05 python3 tests/installer_path_resolution_tests.py` exited 0; 4 tests passed. Covers old-first shadow plus a fresh `/bin/sh` canary for `assura` and `assura-full`, new-first without an unnecessary correction, destination absent from PATH, and checksum rejection preserving both prior destination binaries. The temporary install directory includes spaces and an apostrophe to exercise shell quoting.
- Shell syntax: `sh -n website/public/install.sh` exited 0.
- Whitespace: `git diff --check` exited 0.
- Structure: installed `assura 0.4.1` ran `assura check --format json .`, exit 0, 1,967 files / 412 dirs, 15 low nonblocking max-line findings on unchanged paths, zero findings on changed paths. Raw result: `/Users/nroth/workspace/.assura-review-20261017/nu05/structure-check-final.json`.
- Task context: `python3 .trellis/scripts/task.py validate 10-17-nu05-installer-shadow` exited 0.
- Not run: Rust/Cargo or website build, per the owned shell-installer scope and Mac NU01 compiler-lane constraint.
- Delivery readback: `task.py delivery inspect` waited inside provider pull-request inventory (`gh`) and was interrupted after 15 seconds (exit 130). Registration succeeded; hosted review and merge state remain root-owned.
- Status: implementation and local checks complete; independent Luna review, review disposition, hosted PR checks, merge, and fresh human/runtime release acceptance remain open.
