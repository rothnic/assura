---
title: Introduction
description: Welcome to Assura - the dependency-aware file system validation engine
template: doc
sidebar:
  order: 1
---

import { Card, CardGrid, Aside, LinkButton } from '@astrojs/starlight/components';

Welcome to **Assura**, a powerful dependency-aware file system validation engine written in Rust.

## What is Assura?

Assura helps you maintain code quality and consistency by analyzing your project's file system structure, detecting issues early, and enforcing architectural rules. It's designed for modern development workflows, supporting everything from individual developers to large multi-agent teams.

<CardGrid>
  <Card title="Dependency Analysis" icon="graph">
    Detect circular dependencies and compute validation ordering using graph algorithms
  </Card>
  <Card title="Rule-Based Validation" icon="document">
    Configure severity levels (Critical, High, Medium, Low) for comprehensive validation
  </Card>
  <Card title="File System Watching" icon="seti:rust">
    Continuous validation during development with automatic file system monitoring
  </Card>
  <Card title="Parallel Execution" icon="rocket">
    High-performance validation using Rayon and Tokio for large codebases
  </Card>
  <Card title="Extensible Architecture" icon="puzzle">
    Plugin-based design allows custom validators and integrations
  </Card>
  <Card title="Multiple Formats" icon="list-format">
    Support for YAML, JSON, and TOML configuration files
  </Card>
</CardGrid>

## Who is Assura For?

Assura is perfect for:

- **Individual developers** who want to maintain consistent code quality
- **Development teams** working on large codebases with complex dependencies
- **AI agent workflows** where multiple agents collaborate on the same project
- **Open source projects** that need automated quality checks
- **Enterprise teams** with strict architectural requirements

## Quick Start

Get started with Assura in minutes:

```bash
# Install Assura
cargo install assura

# Create configuration
mkdir -p .assura
cat > .assura/config.yml << 'EOF'
name: My Project
rules:
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
EOF

# Run validation
assura validate
```

<LinkButton href="/guides/getting-started/" variant="primary">
  Get Started
</LinkButton>

## Key Features

### Dependency Graph Analysis

Assura builds an intelligent dependency graph of your project, enabling:

- **Circular dependency detection** - Catch architectural issues early
- **Validation ordering** - Ensure files are validated in the correct order
- **Impact analysis** - Understand how changes affect other parts of your codebase

### Flexible Rule System

Define validation rules that match your project's needs:

- **File naming conventions** - Enforce consistent naming patterns
- **File size limits** - Prevent files from growing too large
- **Line length restrictions** - Maintain readable code
- **Documentation requirements** - Ensure public APIs are documented
- **Custom constraints** - Create project-specific validation logic

### Multi-Agent Support

Built for modern AI-assisted development:

- **Agent profiles** - Different validation rules for different roles
- **Maturity phases** - Relaxed rules during early development
- **Synchronized configuration** - Share validation rules across your team
- **Progressive enforcement** - Gradually increase strictness

### Seamless Integration

Works with your existing tools:

- **CI/CD pipelines** - GitHub Actions, GitLab CI, CircleCI, and more
- **Git hooks** - Pre-commit and pre-push validation
- **IDE integration** - Real-time feedback in your editor
- **TypeScript plugins** - Extend with custom JavaScript/TypeScript logic

## Why Choose Assura?

<Aside type="tip" title="Learn More">
  Read our detailed comparison: [Why Assura?](/docs/why-assura/)
</Aside>

- **Fast** - Written in Rust for maximum performance
- **Flexible** - YAML, JSON, or TOML configuration
- **Extensible** - Plugin architecture for custom validators
- **Team-friendly** - Multi-agent support with role-based profiles
- **CI-ready** - Designed for automated workflows
- **Open source** - MIT licensed and community-driven

## Next Steps

Ready to dive in? Choose your path:

<div class="flex gap-4 flex-wrap">
  <LinkButton href="/guides/getting-started/" variant="primary">
    Getting Started Guide
  </LinkButton>
  <LinkButton href="/reference/configuration/">
    Configuration Reference
  </LinkButton>
  <LinkButton href="/examples/basic-setup/">
    View Examples
  </LinkButton>
</div>

## Getting Help

- **Documentation** - You're here! Browse the guides and reference
- **GitHub Discussions** - Ask questions and share ideas
- **Issue Tracker** - Report bugs or request features
- **Community** - Join our growing community of developers

<Aside type="note" title="Contributing">
  Assura is open source and welcomes contributions! Check out our [Contributing Guide](https://github.com/anomalyco/assura/blob/main/CONTRIBUTING.md) to get started.
</Aside>
