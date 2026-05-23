---
title: Installation
description: How to install or build Assura
---

Assura ships as a prebuilt CLI. Normal usage does not require Rust.

## Requirements

- Linux, macOS, or Windows.

## Install A Release Binary

Download the latest archive for your platform from
[GitHub Releases](https://github.com/rothnic/assura/releases/latest).

Linux x64:

```bash
curl -L https://github.com/rothnic/assura/releases/latest/download/assura-linux-amd64.tar.gz | tar xz
sudo install -m 755 assura assura-full /usr/local/bin/
```

macOS Apple Silicon:

```bash
curl -L https://github.com/rothnic/assura/releases/latest/download/assura-macos-arm64.tar.gz | tar xz
sudo install -m 755 assura assura-full /usr/local/bin/
```

macOS Intel:

```bash
curl -L https://github.com/rothnic/assura/releases/latest/download/assura-macos-amd64.tar.gz | tar xz
sudo install -m 755 assura assura-full /usr/local/bin/
```

Windows users can download `assura-windows-amd64.exe.zip`, extract
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
