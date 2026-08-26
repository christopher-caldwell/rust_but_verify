# Indexed Access experiment

Purpose: reproduce the comparison between production's ordered
`Vec<RoleAssignment>` and an ordered target index. This unpublished crate is
research only; it is neither supported nor a proposed public API.

The experimental representation retains the Vec as authoritative and adds a
`HashMap<TargetKey, Vec<usize>>`. Bucket indices preserve assignment order and
first-success provenance. Public-behavior parity tests reuse the production
test cases directly.

The [completed study](../../docs/benchmarks/access-index-crossover-2026-08-24.md)
kept Vec as the production default. Indexing was attractive only for large,
reused, mostly unique-target Access values; its build, clone, and memory costs
prevented it from being a better general representation.

From the repository root, run parity tests with `just check-experiment` and the
comparison with `just bench-index`. `just bench-study` runs the complete opt-in
research suite.
