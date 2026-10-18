---
name: assura-local-build
description: "Resolve Assura build prerequisites and route isolated VPS validation."
---

# Assura Local Build

Use this skill before a long local validation run, or when build/validation
commands fail for environment reasons rather than Rust code errors.

## Local preflight

From the candidate root, wrap the exact command without changing its argv:

```bash
python3 .agents/skills/assura-local-build/scripts/local-preflight.py \
  --repo "$PWD" --socket-relative --require-bash-mapfile --website-tools \
  -- cargo xtask pr
```

The helper resolves the effective target with offline, locked, no-dependency
Cargo metadata. It emits one JSON record to stderr, then execs the command with
its inherited environment and exit behavior; capture the command's exit in your
outer receipt. A rejected preflight never launches that command. Dirty source
is reported, not cleaned. Metadata and identity probes are not build/test credit.

Choose profiles for the actual route: socket checks for Unix-socket tests,
`--require-bash-mapfile` for evidence/pr/full (the CI scope classifier), and
`--website-tools` for docs/pr/full. Omit unrelated profiles for ordinary Cargo.
Socket checks require an existing caller-owned `0700` TMPDIR with verified disk
backing; the projected default socket includes `.tmpXXXXXX` and the hot-session
filename. Darwin's 104-byte capacity includes NUL and applies only on Darwin.
`--socket-relative <suffix>` selects another known test suffix, not a shorter
stand-in for the command's actual longest path.

`--target-dir <path>` asserts the effective Cargo target and explicitly declares
an external directory task-dedicated; it does not set `CARGO_TARGET_DIR`. An
external target must already exist, belong to the caller and exclude other
writers. Confirm its workload ownership separately. `--tmpdir` likewise asserts
the inherited value without setting it. Set child-only environment explicitly
before invoking the helper; it creates no directories or global configuration.

Targets inside another Assura checkout are rejected. General Cargo commands may
use a declared external target. `xtask docs`, `pr`, `full`, `website-demo-data`
and `website-config-examples` require the local target: `run_pr` reaches
`run_docs` in `xtask/src/main.rs`, and the demo/example helpers read
`target/debug/assura-full` in `xtask/src/website_demo.rs`.

Website probes retain xtask's pnpm/npm/node fallback and check selected pnpm
against `website/package.json`. They record package/lock and executable hashes,
child cwd/PATH and Node identity for the current simple Astro script. This
no-build probe is not proof of arbitrary nested build subprocesses; retain
actual build-child evidence when required. No Node version is invented as a pin.

Run the fake-tool controls (no Cargo/website builds or installs):

```bash
python3 .agents/skills/assura-local-build/scripts/local-preflight-tests.py
```

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
`pnpm --dir website install --frozen-lockfile` from the repository root. When
running `cargo xtask docs` or `cargo xtask website-demo-data --check`, keep the
default checkout-local Cargo target directory (or set `CARGO_TARGET_DIR` to
`$PWD/target`). The xtask website helper builds `assura-full` with inherited
Cargo environment but then resolves `target/debug/assura-full` under the
checkout; an out-of-tree target therefore produces `No such file or directory`
and is invalid-environment evidence, not a product failure.

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
