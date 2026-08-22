# Authorization benchmark suite

This directory contains the preserved exhaustive baseline harness plus the
locked bool-first follow-up checks. It contains no production shortcuts or
alternate authorization implementation.

Use `just bench can_any` for a filtered Criterion run, or `just bench-full`
for the Criterion timing suite plus the allocation and concurrency executables.

- `authorization_baseline` retains the pre-existing exhaustive groups and adds
  focused bool-versus-`Decision`, locked two-target/`Targets` cases, and the
  Knowledge Base workload. The follow-up run filters to those focused groups;
  it does not rerun the exhaustive matrix.
- `allocation_counts` preserves the established allocation scenarios and adds
  Access-1 allow and deny comparisons between `can`/`decide` and
  `can_runtime`/`decide_runtime`.
- `can_any_consistency` alternates policies with 1, 4, and 8 readers plus a
  writer, reporting calls, allows, impossible denies, and duration.
- `concurrency` remains the pre-existing fixed-work throughput harness; it is
  compiled for API compatibility but is not part of the locked follow-up run.

Lookup fixtures are constructed outside timed closures. Typed calls construct
their action and target inside the closure deliberately; runtime comparisons
use prebuilt keys. Results are consumed with `black_box`.

Criterion JSON lives in `target/criterion/<group>/<case>/new/estimates.json`
and its HTML report in `target/criterion/report/index.html`. See
[`docs/benchmark-baseline.md`](../docs/benchmark-baseline.md) for the checked-in
analysis. [`docs/bool-first-validation.md`](../docs/bool-first-validation.md)
records the targeted follow-up results while retaining the prior exhaustive
evidence in [`benchmark-results.md`](../benchmark-results.md).
