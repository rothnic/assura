# Release Installation Paths

## Goal

Make the current Assura release straightforward to install on every supported
platform without requiring users to build from source or infer an asset name.

## Success Criteria

1. macOS and Linux have a single documented shell command that resolves the
   latest compatible GitHub release asset and verifies its checksum.
2. Windows has a single documented PowerShell command with the same release
   and checksum behavior.
3. The public installer and documentation agree on supported target triples,
   archive names, installation destination, and PATH guidance.
4. CI smoke tests cover each documented platform path against release-style
   assets; a missing asset, checksum, or unsupported platform fails clearly.
5. The installer does not require a source checkout, Cargo, Node, or a pinned
   preview revision.

## Non-Goals

- Publishing a GitHub release or creating a version tag.
- Adding package-manager distribution channels in this task.
- Changing the validated CLI behavior beyond installation and error messages.

## Evidence

- Current GitHub release asset inventory and checksums.
- Shell and PowerShell installer tests.
- Documentation and website build checks.
