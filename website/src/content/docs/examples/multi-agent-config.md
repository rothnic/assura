---
title: Multi-Agent Configuration
description: How to configure Assura for multi-agent development workflows
template: doc
sidebar:
  order: 5
---

import { Tabs, TabItem, Aside, Steps, Card, CardGrid } from '@astrojs/starlight/components';

This example shows how to configure Assura for projects with multiple AI agents or development teams, ensuring consistent validation across all contributors.

## Overview

When multiple agents (human or AI) work on the same codebase, maintaining consistent code quality is crucial. Assura's multi-agent configuration enables:

- **Shared validation rules** across all agents
- **Agent-specific overrides** for different roles
- **Synchronized configurations** via version control
- **Progressive enforcement** based on maturity

<CardGrid>
  <Card title="Shared Rules" icon="document">
    Common validation standards for all agents
  </Card>
  <Card title="Agent Profiles" icon="users">
    Role-specific validation configurations
  </Card>
  <Card title="Progressive Maturity" icon="rocket">
    Relaxed rules during early development phases
  </Card>
  <Card title="Synchronization" icon="sync">
    Version-controlled configuration
  </Card>
</CardGrid>

## Basic Multi-Agent Setup

### Directory Structure

```
project/
├── .assura/
│   ├── config.yml          # Main configuration
│   ├── base.yml            # Base rules (shared)
│   ├── agents/
│   │   ├── architect.yml   # Architect agent profile
│   │   ├── developer.yml   # Developer agent profile
│   │   └── reviewer.yml    # Reviewer agent profile
│   └── maturity/
│       ├── alpha.yml       # Alpha phase rules
│       ├── beta.yml        # Beta phase rules
│       └── release.yml     # Release phase rules
└── AGENTS.md               # Agent coordination guide
```

### Main Configuration

Create `.assura/config.yml`:

```yaml
# Multi-agent project configuration
name: Multi-Agent Project
description: Configuration for collaborative AI development
version: "1.0"

# Import base configuration
extends: ./base.yml

# Import agent-specific profile
# Set via environment variable: ASSURA_AGENT_PROFILE=developer
extends_profile: ${ASSURA_AGENT_PROFILE:-developer}

# Import maturity-based overrides
extends_maturity: ${PROJECT_MATURITY:-alpha}

settings:
  parallel: true
  max_workers: 8
  cache_enabled: true

includes:
  - "src/**/*.rs"
  - "tests/**/*.rs"
  - "docs/**/*.md"

excludes:
  - "target/**/*"
  - ".agent-*/**/*"
```

### Base Configuration

Create `.assura/base.yml`:

```yaml
# Base rules shared by all agents
rules:
  # Critical rules - never bypass
  - name: dependency-check
    severity: critical
    check_circular: true
    max_depth: 10
  
  - name: security-check
    severity: critical
    check_vulnerabilities: true
  
  # Standard rules
  - name: file-naming
    severity: high
    pattern: "^[a-z][a-z0-9_]*\\.rs$"
  
  - name: file-size
    severity: medium
    max_size: "500KB"

per_file_overrides:
  # All agents can generate files in these directories
  - path: "generated/**/*"
    rules:
      - name: file-naming
        enabled: false
      - name: documentation
        enabled: false
```

## Agent Profiles

### Architect Agent

Create `.assura/agents/architect.yml`:

```yaml
# Architect agent: Focuses on design and structure
rules:
  # Strict architecture rules
  - name: architecture-check
    severity: critical
    layers:
      - name: "domain"
        pattern: "src/domain/**"
        allowed_dependencies: []
      
      - name: "application"
        pattern: "src/application/**"
        allowed_dependencies: ["domain"]
      
      - name: "infrastructure"
        pattern: "src/infrastructure/**"
        allowed_dependencies: ["domain", "application"]
      
      - name: "presentation"
        pattern: "src/presentation/**"
        allowed_dependencies: ["application"]
  
  # Require documentation for public APIs
  - name: documentation
    severity: high
    require_public: true
    require_module: true
    min_description_length: 20
  
  # Enforce design patterns
  - name: design-pattern-check
    severity: medium
    patterns:
      - name: "repository"
        required_in: ["src/infrastructure/**"]
      - name: "use-case"
        required_in: ["src/application/**"]
```

### Developer Agent

Create `.assura/agents/developer.yml`:

```yaml
# Developer agent: Focuses on implementation
rules:
  # Standard code quality
  - name: line-length
    severity: low
    max_length: 100
    ignore_urls: true
  
  - name: import-order
    severity: low
    groups:
      - "std"
      - "external"
      - "crate"
      - "super"
      - "self"
    alphabetical: true
  
  # Allow TODOs during development
  - name: todo-detection
    severity: low
    allowed_patterns:
      - "TODO(#"
      - "TODO:"
    require_issue_reference: false
```

### Reviewer Agent

Create `.assura/agents/reviewer.yml`:

```yaml
# Reviewer agent: Strict validation for code review
rules:
  # Maximum strictness
  - name: documentation
    severity: high
    require_public: true
    require_module: true
    require_traits: true
    min_description_length: 30
  
  # No TODOs allowed
  - name: todo-detection
    severity: high
    allowed_patterns: []
    require_issue_reference: true
  
  # Strict line limits
  - name: line-length
    severity: medium
    max_length: 80
    ignore_urls: true
    ignore_comments: false
  
  # Comprehensive testing checks
  - name: test-coverage
    severity: medium
    min_coverage: 80
    require_integration_tests: true
```

## Maturity-Based Configuration

### Alpha Phase

Create `.assura/maturity/alpha.yml`:

```yaml
# Alpha phase: Rapid development, relaxed rules
maturity: alpha

description: |
  Early development phase with relaxed validation.
  Focus on feature development over code polish.

rules:
  # Relaxed documentation
  - name: documentation
    severity: low
    require_public: false
    require_module: false
  
  # Allow TODOs
  - name: todo-detection
    severity: low
  
  # Relaxed line length
  - name: line-length
    severity: low
    max_length: 120

settings:
  fail_fast: false
```

### Beta Phase

Create `.assura/maturity/beta.yml`:

```yaml
# Beta phase: Feature complete, testing and polish
maturity: beta

description: |
  Beta phase with moderate validation.
  Focus on stability and documentation.

rules:
  # Require documentation for public APIs
  - name: documentation
    severity: medium
    require_public: true
    require_module: false
    min_description_length: 10
  
  # TODOs should have issue references
  - name: todo-detection
    severity: medium
    allowed_patterns: ["TODO(#"]
    require_issue_reference: true
  
  # Standard line length
  - name: line-length
    severity: low
    max_length: 100

settings:
  fail_fast: false
```

### Release Phase

Create `.assura/maturity/release.yml`:

```yaml
# Release phase: Production ready, strict validation
maturity: release

description: |
  Release phase with strict validation.
  All code must meet production standards.

rules:
  # Strict documentation
  - name: documentation
    severity: high
    require_public: true
    require_module: true
    min_description_length: 20
  
  # No TODOs allowed
  - name: todo-detection
    severity: high
    allowed_patterns: []
    require_issue_reference: true
  
  # Strict line limits
  - name: line-length
    severity: medium
    max_length: 80
  
  # Test coverage requirements
  - name: test-coverage
    severity: high
    min_coverage: 80

settings:
  fail_fast: true
```

## Switching Configurations

### Using Environment Variables

```bash
# Set agent profile
export ASSURA_AGENT_PROFILE=architect

# Set maturity phase
export PROJECT_MATURITY=beta

# Run validation
assura validate
```

### Using CLI Flags

```bash
# Use specific agent profile
assura validate --agent-profile developer

# Use specific maturity
assura validate --maturity release

# Combine both
assura validate --agent-profile reviewer --maturity release
```

### Using Git Branches

Automatically detect phase from branch:

```yaml
# In .assura/config.yml
extends_maturity: |
  ${GIT_BRANCH}
    main: release
    release/*: release
    develop: beta
    feature/*: alpha
    default: alpha
```

## Agent Coordination File

Create `AGENTS.md` at project root:

```markdown
# Agent Coordination Guide

## Available Profiles

### Architect
- **Use when**: Designing APIs, defining structure
- **Activate**: `export ASSURA_AGENT_PROFILE=architect`
- **Focus**: Architecture validation, design patterns

### Developer
- **Use when**: Implementing features, writing code
- **Activate**: `export ASSURA_AGENT_PROFILE=developer`
- **Focus**: Code style, basic quality

### Reviewer
- **Use when**: Reviewing code, preparing for merge
- **Activate**: `export ASSURA_AGENT_PROFILE=reviewer`
- **Focus**: Comprehensive validation, documentation

## Maturity Phases

### Alpha (feature branches)
- Relaxed rules for rapid development
- Focus on functionality

### Beta (develop branch)
- Moderate validation
- Focus on stability and docs

### Release (main branch)
- Strict validation
- Production quality required

## Quick Start

```bash
# Before starting work
cd project
export ASSURA_AGENT_PROFILE=developer
export PROJECT_MATURITY=alpha

# Validate as you work
assura watch

# Before committing
export ASSURA_AGENT_PROFILE=reviewer
assura validate
```

## CI/CD Integration

CI uses reviewer profile with release maturity:

```yaml
# .github/workflows/validate.yml
- name: Validate
  run: |
    export ASSURA_AGENT_PROFILE=reviewer
    export PROJECT_MATURITY=release
    assura validate --format check
```
```

## Team Workflow Example

<Steps>

1. **Feature Development**

   ```bash
   git checkout -b feature/new-api
   export ASSURA_AGENT_PROFILE=developer
   export PROJECT_MATURITY=alpha
   assura watch  # Continuous validation
   ```

2. **Feature Complete**

   ```bash
   export ASSURA_AGENT_PROFILE=architect
   assura validate  # Architecture check
   git commit -m "feat: implement new API"
   ```

3. **Pre-Review**

   ```bash
   export ASSURA_AGENT_PROFILE=reviewer
   export PROJECT_MATURITY=beta
   assura validate  # Comprehensive check
   ```

4. **Pull Request**

   CI automatically runs with reviewer + release profile.

5. **Post-Merge**

   ```bash
   git checkout develop
   export PROJECT_MATURITY=beta
   assura validate
   ```

</Steps>

## Synchronization Strategy

### Option 1: Version Control (Recommended)

Store all configurations in Git:

```bash
git add .assura/
git add AGENTS.md
git commit -m "docs: update agent configurations"
```

Benefits:
- All agents use same config
- Version history
- Pull request reviews

### Option 2: Shared Configuration Server

For large organizations:

```yaml
# .assura/config.yml
extends_remote: https://config.company.com/assura/base.yml
agent_profiles_url: https://config.company.com/assura/agents/
```

### Option 3: Container Images

Bundle configuration in Docker:

```dockerfile
FROM rust:1.70

# Install Assura
RUN cargo install assura

# Copy team configuration
COPY .assura/ /project/.assura/
ENV ASSURA_CONFIG_PATH=/project/.assura/config.yml

WORKDIR /project
```

## Conflict Resolution

When agents disagree on configuration:

```yaml
# .assura/config.yml
conflict_resolution:
  strategy: "strictest"  # Options: strictest, most_recent, manual
  manual_reviewers:
    - lead-architect
    - tech-lead
```

Override priority:
1. Local user config
2. Agent profile
3. Maturity phase
4. Base configuration

<Aside type="tip" title="Best Practices">
  - Use Git branches to track maturity
  - Document profile selection in PRs
  - Regular sync meetings for config updates
  - Version your configurations
  - Use CI to enforce release standards
</Aside>

<Aside type="note" title="Migration Guide">
  When adding Assura to an existing multi-agent project:
  1. Start with alpha maturity
  2. Run validation and fix critical issues
  3. Gradually increase strictness
  4. Document agent responsibilities
</Aside>
