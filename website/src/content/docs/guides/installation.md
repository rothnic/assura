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
curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sh
```

The installer detects your platform, downloads the matching release archive,
and installs `assura` plus its internal `assura-full` companion into
`$HOME/.local/bin` by default. Override the destination with `BIN_DIR`:

```bash
curl -fsSL https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.sh | sudo env BIN_DIR=/usr/local/bin sh
```

Windows PowerShell:

```powershell
irm https://raw.githubusercontent.com/rothnic/assura/master/website/public/install.ps1 | iex
```

The PowerShell installer downloads `assura-windows-amd64.zip` and installs
`assura.exe` plus `assura-full.exe` into
`%LOCALAPPDATA%\Programs\Assura\bin` by default. Override the destination with
`$env:BIN_DIR` before running the script.

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

Release archives include the public `assura` command and an internal
`assura-full` companion used for less common commands such as `init`, `migrate`,
and `performance-report`. Keep both files in the same directory and run
`assura`.

## Install From Crates.io

Use Cargo when you already have a Rust development environment:

```bash
cargo install assura
```

## Build from Source

Source builds require Rust 1.70.0 or later.

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

The supported output formats for `assura check` are `text`, `json`, and `yaml`.
