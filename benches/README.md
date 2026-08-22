# Authorization benchmark suite

This directory measures the public API as it exists today. It contains no
production shortcuts or alternate authorization implementation.

Use `just bench can_any` for a filtered Criterion run, or `just bench-full`
for the Criterion timing suite plus the allocation and concurrency executables.

- `authorization_baseline` covers lookup position/size, denial/data shapes,
  typed versus runtime calls, identifiers, access construction/cloning,
  decisions, `can_any`, policies, and request workloads.
- `allocation_counts` uses `stats_alloc` to account for one release-mode
  operation at a time; it is separate because allocator instrumentation
  distorts timing.
- `concurrency` is a fixed-work throughput harness. Thread setup and policy
  construction complete before its timed barrier.

Lookup fixtures are constructed outside timed closures. Typed calls construct
their action and target inside the closure deliberately; runtime comparisons
use prebuilt keys. Results are consumed with `black_box`.

Criterion JSON lives in `target/criterion/<group>/<case>/new/estimates.json`
and its HTML report in `target/criterion/report/index.html`. See
[`docs/benchmark-baseline.md`](../docs/benchmark-baseline.md) for the checked-in
analysis.
