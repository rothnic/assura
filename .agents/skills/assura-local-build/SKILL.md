---
name: assura-local-build
description: Use when Cargo, Clippy, tests, benchmarks, or Assura self-checks fail for local platform, OpenSSL, pkg-config, or network-limited WSL reasons before changing product code.
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

## Validation Pattern

After resolving local build prerequisites, run:

```bash
cargo fmt --all -- --check
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo build
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo clippy --all-targets --all-features -- -D warnings
OPENSSL_INCLUDE_DIR=/usr/include OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu cargo run --quiet -- check --format json .
cd website && pnpm install --frozen-lockfile && pnpm build
cd integrations/agents/codex && npm install && npm run lint && npm run build
```
