---
title: Deslopify Rust Hygiene Evaluation
status: active
---

# Deslopify Rust Hygiene Evaluation

## Scope

This records the Rust hygiene tooling decisions for
`.trellis/tasks/06-08-full-deslopify-plan`.

## Decisions

| Tool | Decision | Evidence |
| --- | --- | --- |
| `cargo audit` | Keep existing security workflow. | `.github/workflows/security.yml` already runs the audit for Cargo metadata changes and scheduled audits. |
| `cargo-machete` | Adopt as scoped dependency hygiene gate. | Installed `cargo-machete v0.9.2`; first run found unused `dashmap`, `ignore`, `notify`, `once_cell`, `serde_regex`, `time`, `serde`, and `serde_yaml`; those dependencies were removed; `node --run verify:hygiene` is now clean; CI installs pinned `cargo-machete 0.9.2`. |
| `cargo-deny` | Defer until a license/source policy is designed. | Tool was not installed locally. Cargo advisories are already covered by `cargo audit`; adopting deny without a reviewed license and banned-crate policy would add config noise before policy. |
| `cargo-semver-checks` | Defer until the Rust public surface contract is stricter. | Tool was not installed locally. The current Rust crate modules are explicitly unstable internal APIs before 1.0, so semver checks should follow a public-surface support-matrix rule instead of driving it. |
| `cargo-nextest` | Do not adopt yet. | `cargo-nextest 0.9.137` was installed, but an exploratory run was noisy, reported a `LEAK` classification, and was terminated. It is not a drop-in replacement for `cargo test` without more configuration and timing proof. |

## Gate Contract

`node --run verify:hygiene` runs `cargo machete`. The gate is scoped to Cargo
metadata and scheduled security checks through `.assura/config.yml` and the
security workflow. It is intentionally separate from the normal frequent edit
loop.

## Follow-Up

Cargo license/source policy, semver release drift, and nextest profiling should
be handled by dedicated planned rule or tooling tasks rather than hidden inside
this cleanup.
