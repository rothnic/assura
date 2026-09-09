---
name: assura-local-build
description: "Resolve Assura build prerequisites and route isolated VPS validation."
---

# Assura Local Build

Use this skill when build or validation commands fail for local environment
reasons rather than Rust code errors.

## OpenSSL Discovery

Assura depends on `git2`, which can require `openssl-sys` during Cargo builds.
On Ubuntu or WSL, `libssl-dev` may be installed even when `pkg-config` is not
available. If Cargo reports that `openssl-sys` cannot find OpenSSL, check:

```bash
dpkg -s libssl-dev
which pkg-config
find /usr/lib -maxdepth 3 -name 'libssl.so*' -o -name 'libcrypto.so*'
find /usr/include -maxdepth 2 -path '/usr/include/openssl/ssl.h' -print
```

If headers exist under `/usr/include/openssl` and libraries exist under
`/usr/lib/x86_64-linux-gnu`, run Cargo with explicit paths:

```bash
OPENSSL_INCLUDE_DIR=/usr/include \
OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu \
cargo build
```

Use the same environment variables for `cargo run`, `cargo clippy`, `cargo
test`, and `cargo bench`.

## Network-Limited Environments

If a sandboxed Cargo command fails to resolve `index.crates.io` or
`static.crates.io`, retry the same command with normal network access before
changing code. Treat dependency download failures as environment blockers until
the command reaches compile, lint, or test output from this repository.

For Node/npm/pnpm commands, the same distinction applies. In this WSL
environment Node is managed by `fnm`, with the active Node installed under
`~/.local/share/fnm/node-versions/`. If `corepack` exists but `pnpm` is missing,
enable the Corepack shim for the active Node install:

```bash
corepack prepare pnpm@10.25.0 --activate
corepack enable pnpm
pnpm --version
```

If `npm ping` or package installation fails inside the Codex sandbox with
`EAI_AGAIN registry.npmjs.org`, retry with normal network access before
changing project files. A successful outside-sandbox check looks like:

```bash
npm ping
# npm notice PONG ...
```

The website build disables Astro telemetry through `package.json` so the build
does not need to create `~/.config/astro` in sandboxed validation.

## macOS Uninterruptible Tool Processes

On the always-on macOS development host, Homebrew-managed `cargo`, `git`, `gh`,
or Node processes can occasionally enter uninterruptible `U` state. Do not
interpret a silent command as a product failure or keep spawning retries.

Inspect the process state first. For Git and Trellis bookkeeping, use system
tools explicitly:

```bash
/usr/bin/git status --short --branch
/usr/bin/env PATH=/usr/bin:/bin:/usr/sbin:/sbin \
  /usr/bin/python3 ./.trellis/scripts/workflow_gate.py --platform codex
```

When Cargo or website validation is affected, use the exact-commit procedure
below to run the applicable validation on `vps`. Keep hosted Linux, macOS and
Windows CI as final cross-platform proof.

## Validation Pattern

After resolving prerequisites, resume the failed applicable command from its
original cwd. Use OpenSSL overrides only after confirming that specific Linux
layout. See [validation routing](../assura-goal-execution/references/validation-routing.md)
for the changed surface; do not run unrelated Rust, website and integration
suites as a universal recovery recipe. For website setup, use
`pnpm --dir website install --frozen-lockfile` from the repository root.

## Isolated VPS validation

1. Resolve the configured SSH alias with local SSH config. On Nick's current
   machine the development alias is `vps`; do not assume `vps-dev` resolves.
   Probe with batch mode and a bounded connection timeout. Read CPU/load,
   available RAM, free disk, installed toolchains and any existing owned job.
2. Choose one clean committed candidate and record its SHA/base. Create a
   unique local scratch directory with `mktemp -d` and an exact Git bundle:
   `git bundle create <scratch>/candidate.bundle HEAD`, followed by
   `git bundle verify <scratch>/candidate.bundle`. Retain the exact SHA.
3. Create a unique remote directory with `mktemp -d` under the approved
   development workspace (discover the home/path first). Transfer only the
   bundle, clone it there, and check out the recorded SHA detached. Verify
   remote HEAD, clean status and bundle checksum against the local values.
   Do not copy credentials, private evaluators, or another checkout's dirt.
4. Select a toolchain matching the applicable CI job explicitly (`cargo
   +<toolchain> ...`). Record `rustc +<toolchain> -Vv`, `cargo +<toolchain>
   --version`, OS, dependency lockfile hash
   and package-manager versions. A default nightly is not stable/MSRV proof.
   Confirm projected build size plus a safety margin fits available disk;
   low disk means choose a smaller applicable job or retain local execution.
   Never remove unrelated caches or build trees to make room.
5. Run one bounded heavy job initially. Keep process/session identity, stdout,
   stderr and actual exit (including failure). Use a dedicated log and status
   file outside tracked source for detached jobs; do not infer success from a
   disconnected SSH session or the last line of output. Reconnect to that job.
6. Retrieve logs/results and verify source identity remained unchanged. Record
   elapsed time and host utilization before claiming an efficiency improvement.
   Concurrent builds must not contaminate performance comparisons. Retain
   baseline/candidate rows, warm/cold distinctions and all gate thresholds.
7. Close only the exact owned remote directory after evidence is retrieved and
   no process uses it, or record its owner, next action and archive/restore path.

The existing `scripts/perf-vps-ls-lint-compare.sh` is a specialized unstaged-diff
diagnostic, not a clean-commit validation runner. It recreates a date/label
directory and copies the working tree; inspect those effects and ownership
before any use. Prefer the isolated bundle procedure for merge evidence.
Do not configure a self-hosted GitHub runner or change protections as part of
routine remote validation.
