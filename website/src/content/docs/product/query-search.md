---
title: Query And Search
description: Planned query, graph expansion, keyword search, and semantic search layers.
---

Query and search are planned layers of the Project Intelligence Runtime. They
depend on the fact model and embedded graph/search storage goals, so the docs
must treat them as roadmap work until those goals land.

## Status

| Capability | Status | Notes |
| --- | --- | --- |
| Collection queries | Planned | Will expose modeled collection records from the content runtime. |
| Relation queries | Planned | Will traverse content-model relation edges and missing-target diagnostics. |
| Keyword search | Planned | Will search indexed document, section, instance, and diagnostic text. |
| Graph expansion | Planned | Will expand from a resource or model instance into related facts. |
| Local semantic search | Planned | Optional candidate retrieval only; it will not decide validation correctness. |

## Current Path

Today, use `assura check --format json .` for validation facts and content
runtime diagnostics. Query and search commands will be documented here after the
fact model, embedded store, and CLI goals add their supported surfaces.
