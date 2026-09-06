---
name: assura-rust-quality
description: "Rust changes to Assura: preserve policy contracts and reduce maintenance cost."
---

# Assura Rust Quality

Use this skill for Assura Rust changes; do not use it to broaden product scope
or replace the Trellis workflow.

1. Read the changed surface's spec and one existing canonical example.
2. State the observable contract, invariant, and ownership boundary.
3. For config/model work, read [architecture](references/architecture.md).
4. For paths, reports, or subprocesses, read
   [errors and effects](references/errors-and-effects.md).
5. For traversal, cache, or concurrency, read [performance](references/performance.md).
6. Add a positive, negative, and valid-exception case for validation changes.
7. Keep one authoritative model per semantic role and canonical item path.
8. Preserve errors, checked scope, deterministic ordering, and exit behavior.
9. Do not split files or weaken policy merely to pass a numeric limit.
10. Read [tests and review](references/tests-and-review.md); run focused proof
    and the proper tier.

Report exact checks, remaining risks, and any policy or dependency change.

## References

- [Architecture](references/architecture.md): configuration or check pipeline.
- [Errors and effects](references/errors-and-effects.md): fallback, output, or
  subprocess behavior.
- [Performance](references/performance.md): comparable measurements and
  traversal/cache changes.
- [Tests and review](references/tests-and-review.md): meaningful fixtures and
  review-ready proof.
