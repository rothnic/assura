# Publish Claim-Complete v0.4

## Goal

Publish the first release in which the binary, support contract, website, docs,
examples, evidence, checksums, and release notes expose the same product.

## Entry Criteria

- Claim/release contract, watch/warm runtime, managed activation, and supported
  policy-depth tasks are complete.
- PR 140 and all required follow-up changes pass Linux, macOS, and Windows CI.

## Acceptance Criteria

- [ ] Package and workspace versions are v0.4.0.
- [ ] Changelog and release notes enumerate every new supported surface.
- [ ] Generated website evidence names v0.4.0 and current benchmark artifacts.
- [ ] Release archives and checksums exist for supported platforms.
- [ ] Install and upgrade smoke tests run the promoted commands from artifacts.
- [ ] GitHub release, docs, and production website are live and cross-linked.
- [ ] `release-readiness` returns ready with no unreleased marketed surfaces.

## Validation

```bash
cargo xtask full
cargo xtask release-readiness --format json
cargo xtask release-smoke
pnpm --dir website test:marketing
git diff --check
```

## Review Blocking Criteria

Block on a stale version, missing platform artifact, generated/source drift,
failed artifact command, missing checksum, or production claim not present in
the release binary.
