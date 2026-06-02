---
title: Evidence and review policy
date: 2026-06-02
status: active
---

# Evidence And Review Policy

Goal PRs must be reviewable from repository files, PR text, CI links, and
checked artifacts. Chat history can explain why a decision happened, but it is
not completion evidence.

## Checked-In Evidence

Check in evidence that is small, durable, and useful for later agents:

- review records under `docs/analysis/`;
- templates or policy docs that define the proof contract;
- deterministic JSON fixtures or reports that are referenced by docs or tests;
- goal progress-log entries and Iteration 01 ledger updates; and
- source files, tests, and docs needed to reproduce the behavior.

## Generated Evidence

Keep large, machine-local, or environment-specific outputs under `target/`:

- local build caches and release archives;
- smoke-test working directories;
- benchmark scratch output;
- downloaded third-party repositories; and
- CI artifacts that already have GitHub-hosted artifact links.

The PR body or review record must name the command that regenerates any
generated evidence used for a completion claim.

## Review Feedback

Complex goal PRs require review-agent review before PR creation. Actionable
review-agent, Gemini, or human review findings must be either fixed or rejected
with a short rationale in the review record or PR body.

## Goal Closure

Before marking an execution goal complete:

- update the goal progress log;
- update the Iteration 01 ledger;
- link the PR, review record, validation commands, and CI evidence;
- confirm stable command surfaces remain current; and
- name the next goal path in copy/paste-ready form.
