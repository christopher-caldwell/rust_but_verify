# Access index crossover study — 2026-08-24

## Decision

| Status | Decision |
| --- | --- |
| Production default | Ordered `Vec<RoleAssignment>` |
| Indexed implementation | Research reference for large, reused, mostly unique-target Access values |
| Current production change | None |

**Keep `Vec<RoleAssignment>` as the default, but indexing is clearly valuable when one
constructed `Access` with mostly unique targets receives roughly 20–60 late/missing-target
checks.** The target index is not a universal improvement: first-assignment success is faster
with Vec at every measured size, indexed construction never wins, clone cost is 2–3.3× higher,
and retained memory is about 2.8–3.35× higher for unique-target data.

There is enough evidence for a future opt-in/indexed-strategy design iteration, but not for
changing production in this iteration. Production remains the ordered Vec implementation.

## Environment and method

- Machine: Apple M3, 24 GiB RAM
- OS: macOS 26.6.2 (Darwin 25.6.0, arm64)
- Rust: `rustc 1.94.0 (4a4ef493e 2026-03-02)`, LLVM 21.1.8
- Base commit: `70e9b8a1dc402fa5dbd8c783000f7a8faf177d86`, with this study's uncommitted
  benchmark/experiment changes applied
- Criterion 0.5.1, optimized `bench` profile. Values below are Criterion point estimates;
  unusually broad intervals are called out instead of being overinterpreted.
- Runtime action and target identifiers were preconstructed for raw lookup. Construction and
  lookup were timed separately. End-to-end Criterion setup clones equivalent prepared facts
  outside the timed routine; the timed routine receives those owned facts, builds and later drops
  Access, and executes N checks.
- Fixtures are deterministic. Both implementations are exercised from the same comparison
  executable, but fixture values are independently constructed in each crate.
- The experimental structure retains authoritative ordered assignments plus
  `HashMap<TargetKey, Vec<usize>>`. Bucket indices are inserted in assignment order.
- Allocator figures use `stats_alloc` with the retained Access alive at measurement. They include
  heap-backed strings and assignments; the difference between columns is the index cost. Total
  allocated bytes can exceed retained bytes because HashMap growth deallocates old tables.

## Production Vec scaling envelope

All values are `can_runtime`. The same-target role-deny path does a policy lookup for every fact;
the target-miss path only compares target keys. This explains their different slopes.

| Facts | First allow | Midpoint allow | Last allow | Target miss | Same target, role deny |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 10,000 | 27.2 ns | 67.4 µs | 130 µs | 37.1 µs | 133 µs |
| 25,000 | 27.1 ns | 163 µs | 323 µs | 91.4 µs | 328 µs |
| 50,000 | 28.5 ns | 330 µs | 676 µs | 184 µs | 646 µs |
| 100,000 | 26.9 ns | 647 µs | 1.29 ms | 363 µs | 1.29 ms |
| 250,000 | 27.2 ns | 1.61 ms | 3.39 ms | 951 µs | 3.59 ms |
| 500,000 | 28.9 ns | 3.77 ms | 6.87 ms | 1.84 ms | 6.55 ms |
| 1,000,000 | 27.1 ns | 6.52 ms | 13.38 ms | 3.69 ms | 13.09 ms |

Scaling remained linear within normal noise. The first-hit control stayed constant through one
million facts. At 500k/1m, full-scan authorization is millisecond-scale, but construction is also
material: complete assignments + Vec Access measured 34.5/69.9 ms, cloning 33.2/76.0 ms, and
incremental `add_role` 25.1/51.4 ms.

## `Targets<A>` count curve

Candidate construction is intentionally inside the timed public operation. A small Access shows
roughly linear candidate overhead. A denying Access shows the expected candidate-count × Access
scan interaction; early candidate success avoids later scans.

| Candidates | Access 1 deny | Access 1 first allows | Access 1 last allows | Access 1,000 deny | Access 1,000 first allows | Access 1,000 last allows |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 96.8 ns | 121 ns | 121 ns | 6.14 µs | 119 ns | 122 ns |
| 4 | 306 ns | 259 ns | 265 ns | 24.8 µs | 254 ns | 18.4 µs |
| 16 | 950 ns | 870 ns | 949 ns | 98.1 µs | 876 ns | 91.6 µs |
| 64 | 3.58 µs | 3.24 µs | 3.87 µs | 395 µs | 3.03 µs | 384 µs |
| 128 | 6.98 µs | 5.98 µs | 6.40 µs | 789 µs | 5.83 µs | 773 µs |

At Access 100, 128 candidates measured 84.3 µs all-deny, 5.73 µs first-allow, and
85.6 µs last-allow. No pathological million-by-128 case was needed.

## Raw lookup crossover

### Unique targets

The index's raw lookup crossover is extremely small for a prebuilt Access: last-hit becomes
faster at 3 facts; middle and missing paths at approximately 4 facts. At 3 facts, missing is a
tie (14.65 vs 14.78 ns). Vec first-hit remains ~25 ns while indexed first-hit is ~40 ns and has
**no lookup break-even**.

| Facts | Vec last | Indexed last | Vec missing | Indexed missing |
| ---: | ---: | ---: | ---: | ---: |
| 2 | 34.8 ns | 41.6 ns | 10.6 ns | 14.6 ns |
| 3 | 43.2 ns | 40.4 ns | 14.6 ns | 14.8 ns |
| 4 | 50.9 ns | 40.5 ns | 18.2 ns | 14.6 ns |
| 25 | 183 ns | 45.1 ns | 101 ns | 14.9 ns |
| 1,000 | 7.58 µs | 41.4 ns | 3.97 µs | 14.7 ns |
| 10,000 | 58.0 µs | 38.0 ns | 36.1 µs | 14.7 ns |
| 100,000 | 679 µs | 50.1 ns | 367 µs | 15.6 ns |
| 500,000 | 2.91 ms | 39.3 ns | 1.81 ms | 14.6 ns |
| 1,000,000 | 5.89 ms | 52.8 ns | 3.66 ms | 14.6 ns |

### Other data shapes

| Shape | Facts | Vec | Indexed | Interpretation |
| --- | ---: | ---: | ---: | --- |
| Same target, first granting role | 100,000 | 26.5 ns | 41.1 ns | Vec wins; index adds hashing |
| Same target, last granting role | 100,000 | 1.43 ms | 838 µs | Both linear; compact bucket skips target comparisons |
| Same target, all roles deny | 100,000 | 1.39 ms | 820 µs | Target-only index cannot eliminate role scan |
| Unique roles + targets, allow | 10,000 | 57.8 µs | 38.4 ns | Index locates one-role bucket |
| Unique roles + targets, deny | 10,000 | 57.9 µs | 27.9 ns | One bucket role check versus full scan |
| Realistic mixed, early assignment | 10,000 | 33.7 ns | 39.4 ns | Vec wins |
| Realistic mixed, late assignment | 10,000 | 39.8 µs | 37.6 ns | Index wins |
| Realistic mixed, target miss | 10,000 | 36.2 µs | 14.3 ns | Index wins |

The same-target case is important: indexing helps the constant factor of a late/deny scan, but
does not change O(n) behavior. It does not rescue a workload dominated by early success.

## Construction, clone, and decision matrix

This matrix uses unique targets and the last-hit lookup. `Build` clones already-created
assignments into Access, so both sides pay equivalent fact-clone cost. Approximate checks use
`(indexed build - Vec build) / (Vec lookup - indexed lookup)` and are only shown when indexed
lookup is faster.

| Facts | Vec lookup | Indexed lookup | Vec build | Indexed build | Approx checks to amortize index |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 25 | 183 ns | 45 ns | 1.73 µs | 5.01 µs | 24 |
| 50 | 395 ns | 42 ns | 3.34 µs | 9.86 µs | 19 |
| 100 | 765 ns | 40 ns | 6.50 µs | 19.47 µs | 18 |
| 250 | 1.61 µs | 41 ns | 16.2 µs | 51.8 µs | 23 |
| 500 | 3.58 µs | 48 ns | 32.6 µs | 102.8 µs | 20 |
| 1,000 | 7.58 µs | 41 ns | 65.1 µs | 201 µs | 18 |
| 5,000 | 27.9 µs | 38 ns | 326 µs | 942 µs | 22 |
| 10,000 | 58.0 µs | 38 ns | 652 µs | 1.89 ms | 21 |
| 100,000 | 679 µs | 50 ns | 6.54 ms | 24.5 ms | 26 |
| 500,000 | 2.91 ms | 39 ns | 33.1 ms | 195 ms | 56 |
| 1,000,000 | 5.89 ms | 53 ns | 66.4 ms | 418 ms | 60 |

Indexed construction was never faster. Relative build cost was ~2.9–3.2× through 10k,
3.7× at 100k, 5.9× at 500k, and 6.3× at one million. Complete assignments + Access and
incremental construction showed the same direction.

| Facts | Vec clone | Indexed clone | Ratio |
| ---: | ---: | ---: | ---: |
| 100 | 6.45 µs | 13.1 µs | 2.0× |
| 1,000 | 65.7 µs | 132 µs | 2.0× |
| 10,000 | 653 µs | 1.32 ms | 2.0× |
| 500,000 | 35.8 ms | 114 ms | 3.2× |
| 1,000,000 | 68.5 ms | 225 ms | 3.3× |

## Allocation and retained memory

Unique-target data is the expensive case for this index: every target adds a cloned key and a
bucket Vec. Allocation counts roughly double because each assignment's target key is duplicated
into the map and each unique target owns a bucket allocation.

| Facts | Vec retained | Indexed retained | Extra retained | Indexed / Vec | Indexed total allocated during build |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | 8.69 KiB | 24.1 KiB | 15.5 KiB | 2.78× | 40.4 KiB |
| 1,000 | 87.9 KB | 281 KB | 193 KB | 3.20× | 502 KB |
| 10,000 | 0.889 MB | 2.97 MB | 2.08 MB | 3.35× | 4.89 MB |
| 100,000 | 8.99 MB | 25.18 MB | 16.19 MB | 2.80× | 41.95 MB |
| 500,000 | 45.39 MB | 146.07 MB | 100.68 MB | 3.22× | 258.62 MB |
| 1,000,000 | 90.89 MB | 292.37 MB | 201.48 MB | 3.22× | 517.46 MB |

At one million facts, construction recorded 3,000,001 allocations for Vec versus 6,000,022 for
indexed. The indexed build deallocated 225.1 MB of transient growth tables. Clone retained bytes
were 90.9 MB Vec versus 264.9 MB indexed (2.91×). `size_of::<Access>()` was 24 versus 72 bytes,
but that static size is not used as a heap-memory conclusion.

## End-to-end amortization

Direct build + N-check measurements validated the independent arithmetic. These are
unique-target, last-assignment checks; early-assignment controls never cross because indexed
lookup remains slower.

| Facts | Last-check end-to-end crossover | Direct evidence around crossover |
| ---: | ---: | --- |
| 25 | ~23–25 checks | 22: Vec 4.61 µs vs indexed 4.71 µs; 25: 5.20 vs 4.80 µs |
| 100 | ~16–18 checks | 18: Vec 16.6 µs vs indexed 15.0 µs |
| 250 | ~21–22 checks | 20: Vec 37.7 µs vs indexed 39.9 µs; 22: 41.0 vs 40.1 µs |
| 1,000 | ~17–18 checks | 15: Vec 136 µs vs indexed 150 µs; 18: 159 vs 153 µs |
| 5,000 | ~21–22 checks | 20: Vec 666 µs vs indexed 701 µs; 22: 723 vs 703 µs |
| 10,000 | ~20–22 checks | 20: approximately tied at 1.39 ms; 22: Vec 1.50 vs indexed 1.40 ms |
| 100,000 | ~27–30 checks | 25: Vec 16.7 ms vs indexed 18.2 ms; 30: 19.7 vs 18.3 ms |
| 500,000 | at or below 50 checks | 50: Vec 156 ms vs indexed 146 ms |
| 1,000,000 | ~50–60 checks | 50: Vec 331 ms vs indexed 348 ms; 60: 392 vs 358 ms |

At 100k, one late check was 2.82 ms Vec versus 18.2 ms indexed end-to-end. At one million,
the construction penalty remains too large for one check. There is no tested size where indexed
was faster end-to-end for a single construction + check.

For the Vec-friendly first-assignment workload there is no lookup or end-to-end break-even: at
10k and 1,000 checks, Vec was 246 µs versus indexed 1.47 ms.

## Multi-target and concurrency observations

Indexing materially changes `can_any` when candidate lookups would scan most of Access:

| Access | Candidates/path | Vec | Indexed |
| ---: | --- | ---: | ---: |
| 100 | tuple two, all deny | 848 ns | 134 ns |
| 1,000 | 4, first candidate allows at late fact | 7.64 µs | 265 ns |
| 1,000 | 16, all deny | 59.1 µs | 997 ns |
| 1,000 | 64, last allows | 240 µs | 3.83 µs |
| 10,000 | 4, all deny | 145 µs | 274 ns |
| 10,000 | 64, all deny | 2.32 ms | 3.97 µs |

Candidate construction is included and becomes the indexed path's dominant cost. These are
prebuilt-Access lookup results; they do not erase the construction amortization requirement.

No separate giant indexed concurrency matrix was run: Access is immutable and the index adds no
synchronization, so single-thread lookup already answers the representation question. The
preserved production throughput benchmark completed, and the alternating-policy consistency
run observed zero impossible denies across 11.7M (1 reader), 16.6M (4), and 10.3M (8) calls.
The same snapshot contract also passed in the indexed shared behavior tests.

## Correctness and packaging evidence

The experimental crate aliases itself as `rust_but_verify` and includes the production public
integration tests. It passes bool/Decision equivalence, runtime and typed paths, tuple and
arbitrary-N target ordering, duplicate/order provenance exercised by the contract, policy
replacement, alternating-policy snapshot consistency, macro declarations, and compile-fail
ActionOn boundaries. The experiment's private Access tests also verify ordered facts.

The production package explicitly excludes `experiments/**`; generated root and experiment
`target/criterion` output is ignored. Only benchmark source and this architectural report are
intended to be committed.

## Direct answers

1. **How far can Vec reasonably scale?** Early-hit checks remain ~27 ns through one million.
   Full scans are comfortable at request-like sizes and still sub-millisecond around 50k–100k
   for target-miss/midpoint paths, but reach 3.7–13.4 ms at one million. “Reasonable” therefore
   depends on assignment position and latency budget, not merely fact count.
2. **When does indexed lookup become faster?** For unique targets: last at 3 facts,
   middle/missing at about 4. For first-hit: never. Same-target late/deny remains linear but is
   ~1.6–2× faster; same-target first-hit is slower.
3. **When is indexing faster end-to-end for one check?** Never in the tested 25–1,000,000 range.
4. **Checks needed to amortize?** Roughly 16–30 through 100k unique targets and 50–60 at
   500k–1m for repeated late hits. No break-even for early-hit workloads.
5. **Additional memory?** Unique targets retain about 2.8–3.35× total Vec memory: +16.2 MB at
   100k, +100.7 MB at 500k, and +201.5 MB at one million.
6. **What happens at 500k/1m?** Vec last lookup is 2.91/5.89 ms for unique targets and
   6.87/13.38 ms for repeated same-target role scans. Indexed build is 195/418 ms and retained
   memory 146/292 MB. Direct end-to-end crossover is at or below 50 / about 50–60 late checks.
7. **Many roles on the same target?** The target index cannot avoid scanning roles. It improves
   late/deny constants but retains linear scaling and loses on first grant.
8. **Does indexing improve `can_any`?** Yes, dramatically for late/missing candidate targets on
   a prebuilt Access; at 10k × 64 all-deny it reduced 2.32 ms to 3.97 µs. Build cost still needs
   amortization.
9. **Is Vec still the right default?** Yes. Expected small/request-scoped and early-success
   workloads favor its build, clone, memory, and often lookup cost.
10. **Future indexed option or automatic strategy?** An indexed option deserves a future
    iteration for large, reused, mostly unique-target Access values. This study does not justify
    an automatic threshold or production API design yet.

## Commands exercised

The study ran the complete existing Criterion timing suite (with shortened sampling), production
large lookup/build/Targets envelopes, the complete comparison groups, allocation executables,
targeted lookup and amortization refinements, production allocation/concurrency/snapshot
executables, and the shared indexed behavior tests. Normal fmt, clippy, test, docs, examples, and
package-content validation are recorded in the implementation handoff after their final run.
