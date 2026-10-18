# Existing Beads store recovery

Use only after identity inspection, under an authorized execution goal. This
records an observed Beads 1.2.2 path, not a generic init/repair recipe. Current
context and installed source outrank these dated examples. Start with the
personal Beads skill; keep the coordinator as the sole issue writer.

## Discover before creating

Inspect local/parent stores, overrides, worktree redirects and user registry,
then `git ls-remote origin 'refs/dolt/*'`. Version 1.2.2 can automatically clone
`refs/dolt/data` from cwd's Git origin even with an explicit `BEADS_DIR`.
An absent local folder is not an empty project. Preserve existing issues,
relationships and project identity; do not create a parallel tracker.

`--skip-agents --skip-hooks` preserved Assura files in the observed init.
`--setup-exclude` nevertheless appended broad patterns to tracked `.gitignore`.
Preserve the exact diff and prior clean evidence before removing only an
attributable generated suffix. Prefer a precise local Git info/exclude entry
for tool runtime. Do not weaken Assura root rules or global ignore settings.

## Decode a preserved copy first

Capture source file hashes, ownership, process holders and remote ref. Retain
an untouched copy and use another disposable copy for experiments. Never edit
Dolt internals, delete locks, force init or run repair/GC on the original.
The observed embedded/system Dolt 2.3.1 read issues but failed `dolt_status`
with FlatBuffers unknown fields. A checksum-verified isolated vendor Dolt 2.5.0
read both; system installations remained unchanged. Verify the actual case;
do not assume an upgrade fixes every schema error.

External server identity needs `.beads/metadata.json` with supported server
mode, database, endpoint and existing project UUID. In 1.2.2 environment mode
is exactly `BEADS_DOLT_SERVER_MODE=1`, not `true`; env alone did not select
server mode while metadata was absent. Pass explicit `BEADS_DIR`, actor and
endpoint across worktrees. Verify process/data-dir/socket and `bd context
--json`; a successful command or zero issue count is not identity proof.

Beads readonly commands can still migrate on startup. A readonly server
rejects even startup's CREATE TABLE IF NOT EXISTS. Before testing a writable
server on a disposable copy, compare all schemas/rows, migration cursors,
project identity and commit ancestry. The observed app open added only local
ignored migrations 1..11 and eight empty runtime tables; all original rows,
schemas and history stayed intact. Recheck this before primary adoption.

## Missing lifecycle prefix

The recovered remote had 61 `ae` issues but no DB `issue_prefix`. Graph dry run
passed; actual create stopped without creating issues. Both `config set`
spellings and rename-prefix cannot restore this state in 1.2.2. Do not insert
SQL configuration or reinitialize over data to bypass that boundary.

The supported existing-server attach is `bd init --server --external
--database <existing-db> --prefix <verified-prefix>`, with explicit endpoint,
noninteractive mode, skipped agent/hooks setup, sandbox and empty `--remote`
opt-out. Test on a separate database copy first. Use a clean dedicated client
Git checkout: init stages metadata and can commit every already-staged file.
Do not run it against an unrelated or dirty index. Keep normal no-clobber
checks enabled, with no force/reinit/discard flags. Existing project UUID must
be adopted, never replaced. Record expected prefix/client metadata writes,
new commit ancestry and exact issue/relationship preservation. Explicit empty
remote suppresses origin inference; inspect ambient configuration as well.

For final adoption, native backup/sync and restoration into a new directory
proved the actual issues, owners, relationships and database head. Normal Dolt
remote push needs a fresh remote/base check and independent intake review;
never force or claim cross-machine sync from JSONL/export/backup success.
Store the current key-to-ID map and concise evidence in the selected task.
The first execution's proof is in task `10-17-assura-beads-execution/research`;
query Beads for live state instead of copying another queue.
