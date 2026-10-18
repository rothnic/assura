# NU05: Report and correct shadowed installer resolution

## Goal

After a successful Unix install, tell the user which version was installed and which `assura` their current PATH resolves. If another installation shadows the destination, print a concrete, safely quoted PATH correction that does not modify shell files or replace the other installation.

## Known behavior and constraints

- `website/public/install.sh` verifies a SHA-256 checksum and atomically installs both `assura` and `assura-full`; preserve those paths and failure behavior.
- Current post-install output only reports the destination and whether it is somewhere in PATH. An older path earlier in PATH can still win silently.
- `tests/release_process_contract_tests.py` provides small executable/archive fixture patterns. Installer verification must remain shell/fixture-only and must not build Rust binaries.
- This change is scoped to public Unix installer behavior and focused proof. No Windows installer change, startup-file edits, website rebuild, or release publication.

## Acceptance

- [x] Reproduce the old-first PATH case with old and new bin directories both present, and show the existing installer reports success without identifying the shadow.
- [x] After installation, report the installed version and actual `command -v assura` resolution.
- [x] If resolution is shadowed or absent, print a concrete safely quoted PATH correction; do not edit shell startup files or replace an installation outside `BIN_DIR`.
- [x] Verify `assura` and `assura-full` in the destination are both the new version.
- [x] Apply the printed correction in a fresh shell and verify `command -v assura` and `assura --version` select the installed candidate.
- [x] Preserve the normal successful path, checksum rejection, and transactional replacement behavior; cover no-PATH and new-first negative controls.
- [x] Run the focused fixture suite, `git diff --check`, and applicable Assura structure check. Record exact commands, exits, and test counts.

## Definition of Done

Implementation and focused evidence are committed coherently on `fix/nu05-installer-shadow`. Independent review remains a root-owned handoff; no merge or release claim is made by this candidate.

## Out of Scope

No automatic PATH persistence, shell profile modification, uninstall/rollback redesign, release/archive format change, Windows behavior change, global binary replacement, or website rebuild.
