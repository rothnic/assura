---
title: Basic Commands
description: Supported Assura CLI examples
---

## Check the Current Project

```bash
assura check
```

## Check a Path

```bash
assura check .
```

## Use a Specific Config

```bash
assura --config .assura/config.yml check .
```

## JSON Report

```bash
assura check --format json .
```

## Text Report for CI

```bash
assura check --format text
```

## Initialize a Project

```bash
assura init
```

## Migrate from LS-Lint

```bash
assura migrate .ls-lint.yml --output .assura/config.yml
assura check
```
