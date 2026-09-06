---
title: Installation
description: How to install or build Assura
---

Assura ships as a prebuilt CLI. Normal usage does not require Rust.

## Requirements

- Linux, macOS, or Windows.

## Install With Script

Linux and macOS:

```bash
curl -fsSL https://assura.dev/install.sh | sh
```

The installer detects your platform, downloads the matching release archive,
verifies its published SHA-256 sidecar, and installs `assura` plus its internal `assura-full` companion into
`$HOME/.local/bin` by default. It does not clone this repository or build from
source. Open a new terminal after installation if `$HOME/.local/bin` is not
already on `PATH`. Override the destination with `BIN_DIR`:

```bash
curl -fsSL https://assura.dev/install.sh | sudo env BIN_DIR=/usr/local/bin sh
```

Windows PowerShell:

```powershell
irm https://assura.dev/install.ps1 | iex
```

The PowerShell installer downloads and verifies `assura-windows-amd64.zip`,
installs `assura.exe` plus `assura-full.exe` into
`%LOCALAPPDATA%\Programs\Assura\bin` by default. Override the destination with
`$env:BIN_DIR` before running the script. The install directory is added to the
current session and user `PATH`; restart other terminals after installation.

## Supported Release Archives

Release and adoption smoke tests cover these installable archives:

| Platform | Archive |
| --- | --- |
| Linux x86_64 (glibc) | `assura-linux-amd64.tar.gz` |
| Linux x86_64 (musl/Alpine) | `assura-linux-musl-amd64.tar.gz` |
| macOS Apple Silicon | `assura-macos-arm64.tar.gz` |
| macOS Intel | `assura-macos-amd64.tar.gz` |
| Windows x86_64 | `assura-windows-amd64.zip` |

Each smoke installs from a release-style archive and proves `assura --version`,
`assura init`, `assura status --format json`, `assura check --format json`, a
failing validation case, and an LS-Lint migration walkthrough.

## Manual Release Archive

Download the latest archive for your platform from
[GitHub Releases](https://github.com/rothnic/assura/releases/latest). Extract
the archive and keep `assura` plus `assura-full` in the same directory.

Linux x64:

```bash
curl -L https://github.com/rothnic/assura/releases/latest/download/assura-linux-amd64.tar.gz | tar xz
sudo install -m 755 assura assura-full /usr/local/bin/
```

Windows users can download `assura-windows-amd64.zip`, extract
`assura.exe` and `assura-full.exe`, and place both files on `PATH`.

Each published archive has a `.sha256` sidecar. Both one-line installers verify
that sidecar automatically before installing.

Release archives include the public `assura` command and an internal
`assura-full` companion used for less common commands such as `init`, `migrate`,
and `performance-report`. Keep both files in the same directory and run
`assura`.

## Install With Cargo

Use Cargo only when you already have a Rust development environment:

```bash
cargo install assura
```

## Build from Source

Source builds require Rust 1.86.0 or later.

```bash
git clone https://github.com/rothnic/assura
cd assura
cargo build --release
```

The binary is available at `target/release/assura`.

## Verify

```bash
assura --version
assura --help
```

The first useful project check is:

```bash
assura init --no-git-hooks
assura status --format json
assura check --format json .
```

See [Adoption Walkthrough](/guides/adoption-walkthrough/) for the empty-project
and LS-Lint migration paths used by CI.
