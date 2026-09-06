# Performance references

Read this before changing traversal, prepared checks, cache behavior, or a
performance claim.

`src/cli/performance_report/mod.rs::performance_report_command` writes the
machine-readable report. `claim_summary.rs` limits the headline comparison to
the `assura-cli` and `ls-lint-cli` families in a real-repo or
realistic-equivalent cohort; cached, compiled, hot, and in-process rows are
diagnostic modes, not substitutes for that claim.

Use `.agents/skills/assura-performance-reporting/SKILL.md` for the current
build order, report commands, and host-comparability rules. Compare the same
candidate binary, fixture cohort, build profile, machine, and execution mode.
Cold and warm paths answer different questions; a prepared or changed-path
result cannot certify one-shot CLI performance.

For check behavior, `PreparedStructureCheck::check_path` is a whole-project
proof and `check_changed_path` is only a safe incremental response. Preserve
that scope distinction, config reload behavior, deterministic findings, and
the report's declared row family when optimizing.
