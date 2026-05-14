---
title: Installation
description: How to install or build Assura
---

Assura is a Rust CLI. The current supported onboarding path uses Cargo.

## Requirements

- Rust 1.70.0 or later.
- Linux, macOS, or Windows.

## Install from Crates.io

```bash
cargo install assura
```

## Build from Source

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
