# Authorization benchmarks

This directory benchmarks the supported production crate. It contains no
alternate authorization implementation.

## Routine production benchmarks

`just bench [filter]` runs the Criterion timing suite; for example,
`just bench can_any`. `just bench-full` adds the allocation and fixed-work
concurrency executables. The suite covers Access lookup/build/clone shapes,
typed and runtime checks, identifiers, `can`/`decide`, `can_any`, policies,
realistic requests, allocations, concurrency, and policy-snapshot consistency.

Fixtures are constructed outside timed closures unless construction is the
measurement. Typed calls deliberately construct their action and target inside
the closure, while runtime comparisons use prebuilt keys. Results are consumed
with `black_box`.

## Opt-in scaling

`just bench-scale [filter]` runs `authorization_scale`: the production
10k–1m ordered-Vec envelope and the 1–128 candidate `Targets<A>` curve. It is
excluded from routine checks and CI.

## Allocation and concurrency tools

`allocation_counts` uses `stats_alloc` around individual operations rather
than statistical timing. `concurrency` measures fixed-work throughput, while
`can_any_consistency` alternates policy snapshots and reports impossible
denials. These are executable benchmark targets, not Criterion groups.

## Indexed experiment

The unpublished [`indexed_access` experiment](../experiments/indexed_access/)
contains the direct production-Vec versus ordered-target-index comparison.
Run it with `just bench-index [filter]`, or run the complete opt-in research
suite with `just bench-study`. It is architectural research, not a supported
Access implementation.

## Reports

Durable results and methodology live in [`docs/benchmarks/`](../docs/benchmarks/):

- [`baseline-2026-08-22.md`](../docs/benchmarks/baseline-2026-08-22.md)
- [`bool-first-2026-08-22.md`](../docs/benchmarks/bool-first-2026-08-22.md)
- [`access-index-crossover-2026-08-24.md`](../docs/benchmarks/access-index-crossover-2026-08-24.md)

Generated Criterion JSON and HTML remain under ignored `target/criterion/`.
