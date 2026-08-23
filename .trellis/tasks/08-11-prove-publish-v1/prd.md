# Prove And Publish Assura v1.0

## Goal

Promote the v0.4 CLI contract to v1.0 only after real agent usage demonstrates
stability across repositories and supported hosts.

## Entry Criteria

- v0.4 is published and live-verified.
- The soak recorder stores bounded, privacy-safe session evidence and exact
  binary/config/adapter versions.

## Acceptance Criteria

- [ ] At least 30 consecutive calendar days are recorded after v0.4 publication.
- [ ] At least 50 distinct agent sessions are recorded.
- [ ] At least three representative repositories participate.
- [ ] Codex, Claude Code, OpenCode, and Pi each pass full managed lifecycle proof.
- [ ] The final 14 days have no incompatible CLI, config, schema, or adapter change.
- [ ] No severity-one or release-blocking issue remains unresolved.
- [ ] Supported CLI commands, exit codes, config notation, report schemas, and
  managed integration ownership rules are frozen and documented.
- [ ] Rust internals are private or explicitly documented as outside the v1 contract.
- [ ] v1.0 artifacts pass the same release smoke matrix as v0.4.

## Evidence Record

Maintain a dated machine-readable ledger plus a concise human summary. Counts
must be derivable from the ledger; manual assertions are insufficient.

## Review Blocking Criteria

Block if any count or duration is short, evidence cannot be reproduced,
contract changes occurred during the freeze, one host lacks lifecycle proof,
or library internals are accidentally presented as stable.
