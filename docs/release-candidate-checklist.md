---
title: Release Candidate Checklist
status: active
---

# Release Candidate Checklist

Use this checklist for the v0.3.0 release candidate and any later pre-1.0
release until a newer release process replaces it.

## Release Scope

- Public CLI surface: `assura check`, `init`, `status`, `migrate`, `hooks`,
  `quality plan`, `performance-report`, `agent`, `content`, and
  `editor session`.
- Experimental daemon surface: `assura daemon status`, `assura daemon start`,
  `assura daemon stop`, `assura daemon restart`, `assura daemon doctor`,
  `assura daemon logs`, `assura daemon health`, `assura daemon check-path`,
  and `assura daemon references`.
- Stable agent feedback surface: `assura check --format agent`.
- Codex adapter: `assura check --format agent --agent codex`.
- Experimental local agent integration lifecycle: `assura agent integration`
  for Codex, OpenCode, Claude, and Pi bundles. Host-agent configuration remains
  manual opt-in.
- Project Intelligence local surfaces: `assura init --project-intelligence`,
  `assura content context-pack`, `assura content session`, `assura agent ...`,
  `assura editor session`, and `.assura/models/**` model artifacts.
- Supported beta local editor package: `integrations/editors/vscode` over the
  shared CLI, daemon, editor-session, and Markdown safe-fix contracts. No
  marketplace support is claimed.
- Supported extension-boundary documentation: first-party `extensions.*`
  configuration policies and local JSON integration contracts are supported;
  public third-party plugin APIs remain roadmap-only.
- Relationship notation: `structure` captures, `exists:1`, `needs`, and
  `provides`.
- Experimental extension surface: first-party specialized
  `extensions.custom_constraints`.
- Installable artifacts: GitHub release archives consumed by
  `website/public/install.sh` and `website/public/install.ps1`.

## Pre-Tag Checklist

| Gate | Command Or Evidence | Blocking Criteria |
| --- | --- | --- |
| Format | `cargo fmt --all -- --check` | Any formatting diff. |
| Rust tests | `cargo test --all-targets --quiet` | Any failed unit, integration, or benchmark-harness test. |
| Lint | `cargo clippy --all-targets --all-features -- -D warnings` | Any warning. |
| Primary release build | `cargo build --release --bin assura --no-default-features --features json-output,yaml-config` | Public launcher cannot build. |
| Self-check | `cargo run --quiet -- check --format json .` | Any Assura violation. |
| Fast gate | `cargo xtask fast` | Any local fast gate failure. |
| Docs | `cargo xtask docs` | Website build fails or docs links break. |
| Marketed release claims | `cargo xtask website-demo-data --check --released` | A core website claim is not supported, lacks verified/measured evidence, is absent from the candidate release, or its public command smoke fails. |
| Release readiness | `cargo xtask release-readiness --format json` | Version, release notes, latest GitHub release, support policy, checklist, or unreleased public-surface state is inconsistent. |
| Release smoke | `cargo xtask release-smoke` | Local archive install or first-run smoke fails. |
| Daemon surface | `cargo test --test daemon_cli_tests --quiet` | Daemon lifecycle, stale-state, or IPC fallback behavior regresses. |
| Agent integration lifecycle | `cargo test --test agent_surface_cli --quiet` | Codex, OpenCode, Claude, or Pi integration bundle lifecycle behavior regresses. |
| VS Code local package | `pnpm --dir integrations/editors/vscode test && pnpm --dir integrations/editors/vscode run build && pnpm --dir integrations/editors/vscode run doctor && pnpm --dir integrations/editors/vscode run package` | Local beta package tests, build smoke, doctor, or package smoke fails. |
| Extension API boundaries | `cargo xtask target-state` and `cargo xtask docs` | Public docs overclaim plugin APIs or omit supported/roadmap boundary wording. |
| Project Intelligence smoke | `cargo test --test project_intelligence_release_hardening --quiet` | Supported local Project Intelligence schemas, docs, or starter workflow drift. |
| Checksums | `cargo xtask release-smoke` and release workflow checksum steps | Archive checksum generation or verification fails. |
| Evidence | `cargo xtask evidence` | Goal status, review evidence, or stale command-surface checks fail. |
| Whitespace | `git diff --check` | Whitespace errors. |

## CI Checklist

Required PR checks before a release tag:

- Build Documentation.
- Check.
- Rustfmt.
- Clippy.
- Test Suite on Ubuntu stable.
- Test Suite on macOS stable.
- Code Coverage.
- Evidence Gates.
- Release Bundle Smoke.
- Windows Installer Smoke.
- Installable Adoption Smoke on Ubuntu x86_64.
- Installable Adoption Smoke on macOS arm64.
- Installable Adoption Smoke on macOS x86_64.
- Installable Adoption Smoke on Windows x86_64.
- Performance Report.
- GitGuardian Security Checks.

Block the release if any required check is red, canceled, or skipped without a
maintainer-owned exception recorded in the PR.

## Tag And Publish

1. Confirm `Cargo.toml` has the intended version.
2. Confirm `docs/release-notes.md` names the same version.
3. Create an annotated tag:

   ```bash
   git tag -a v0.3.0 -m "Release v0.3.0"
   git push origin v0.3.0
   ```

4. Wait for `.github/workflows/release.yml` to publish release assets.
5. Confirm the GitHub release contains these files and matching `.sha256`
   checksum files:
   - `assura-linux-amd64.tar.gz`
   - `assura-linux-musl-amd64.tar.gz`
   - `assura-macos-arm64.tar.gz`
   - `assura-macos-amd64.tar.gz`
   - `assura-windows-amd64.zip`
6. Confirm each archive is below `ASSURA_MAX_RELEASE_ARCHIVE_BYTES`
   unless the release PR explicitly approved a new budget.
7. Confirm the workflow generated and verified checksums before upload.

## Post-Tag Verification

Run the live public URL gate after the release exists:

```bash
cargo xtask release-live
ASSURA_VERSION=v0.3.0 cargo xtask release-live
```

The live gate verifies unauthenticated access to the install scripts, all
release assets, and their `.sha256` checksum files.

## Rollback Or Reissue

- If an asset is missing or corrupt, rerun the release workflow or upload the
  corrected asset with `gh release upload --clobber`.
- If a release note is wrong but binaries are valid, update the release notes
  in a follow-up docs PR and edit the GitHub release body.
- If a binary has a functional blocker, mark the release as prerelease, open a
  blocking issue, and publish a patched tag instead of asking users to repair
  local installs manually.

## Completion Evidence

A release-readiness PR is complete only when it links:

- the release notes;
- the release-train check;
- this checklist;
- the support policy;
- the compatibility matrix;
- the website release readiness page;
- the next roadmap iteration; and
- local plus CI validation evidence.
