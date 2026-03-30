---
title: Installation
description: How to install Assura using Cargo or from source
---

Assura is distributed as a Rust crate and can be installed using Cargo.

## Requirements

- **Rust**: Version 1.70.0 or later
- **Operating System**: Linux, macOS, or Windows

## Install from Crates.io

The easiest way to install Assura:

```bash
cargo install assura
```

## Install from Source

For the latest development version:

```bash
git clone https://github.com/anomalyco/assura
cd assura
cargo build --release
```

The binary will be available at `target/release/assura`.

## Verify Installation

Check that Assura is installed correctly:

```bash
assura --version
```

## Shell Completions

Enable tab completion for your shell:

### Bash
```bash
assura completions bash > /usr/share/bash-completion/completions/assura
```

### Zsh
```bash
assura completions zsh > /usr/share/zsh/site-functions/_assura
```

### Fish
```bash
assura completions fish > ~/.config/fish/completions/assura.fish
```
