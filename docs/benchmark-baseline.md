# Authorization baseline: 2026-08-22

Measurement only: no production authorization code changed.

## Method and environment

| Item | Value |
| --- | --- |
| OS / CPU / RAM | macOS 26.6.2; Apple M3, 8 physical and 8 logical cores; 24 GiB |
| Toolchain | rustc/Cargo 1.94.0, `aarch64-apple-darwin` |
| Profile / source | optimized Cargo `bench`; `261997d6b4b8` |
| Broad Criterion run | 20 samples, 200 ms warm-up, 500 ms measurement |
| Stability reruns | 50 samples, 1 s warm-up, 2 s measurement |

Fixtures (`Policy`, `Authorizer`, `Access`, and runtime keys) are constructed
outside lookup closures. Typed calls deliberately construct action/target in
the closure; runtime comparisons use prebuilt keys. Allocation and concurrency
use separate release-mode executables. Values are Criterion slope estimates;
ranges are 95% confidence intervals. Raw JSON is at
`target/criterion/<group>/<case>/new/estimates.json`; HTML is at
`target/criterion/report/index.html`.

## Lookup curve and stress thresholds

| Access assignments | Runtime full deny | Runtime last allow | Typed full deny |
| ---: | ---: | ---: | ---: |
| 0 | 10 ns | — | 70 ns |
| 1 | 16 ns | 103 ns (first) | 77 ns |
| 5 | 76 ns | 183 ns | 139 ns |
| 10 | 140 ns | 252 ns | 202 ns |
| 25 | 345 ns | 440 ns | 403 ns |
| 50 | 669 ns | 760 ns | 733 ns |
| 100 | 1.33 µs | 1.42 µs | 1.41 µs |
| 250 | 3.54 µs | 3.46 µs (3.45–3.48) | 4.13 µs |
| 500 | 6.63 µs | 6.73 µs | 6.71 µs |
| 1,000 | 13.71 µs (13.66–13.77) | 13.36 µs (13.30–13.44) | 13.27 µs |
| 2,500 | 32.93 µs | 32.87 µs | 33.88 µs |
| 5,000 | 65.96 µs | 65.72 µs | 66.64 µs |
| 10,000 | 132.08 µs | 133.85 µs | 132.96 µs |

At 1,000 facts, runtime first/midpoint/last allow was 103 ns / 6.70 µs /
13.36 µs. Position matters as an ordinary `Vec` scan predicts. A short-run
250-entry runtime-last anomaly (8.8 µs) was rerun and measured 3.46 µs.

| Threshold | First tested size above it (full deny and last allow) |
| --- | ---: |
| 1 µs | 100 |
| 5 µs | 500 |
| 10 µs | 1,000 |
| 50 µs | 5,000 |
| 100 µs | 10,000 |

At 1,000 entries, no-target-match denial was 3.65 µs; target-match/no-grant
was 13.16 µs; repeated matching non-granting roles were 13.18 µs. Policy
lookup on matching targets, rather than target comparison alone, causes most
of this denial difference. At that size: same-role/unique-target shape was
7.59 µs, unique-roles/same-target 15.21 µs, unique roles/targets 7.51 µs,
and the mixed fixture with its useful fact near the start 103 ns.

## Typed, decisions, identifiers, and allocations

| Case | Time |
| --- | ---: |
| Typed / runtime empty deny | 64–70 ns / 9–10 ns |
| Typed / runtime allow, Access 1 | ~164 ns / 104 ns (102–105) |
| Typed / runtime deny, Access 1 | ~77 ns / ~16 ns |
| Typed / runtime deny, Access 1,000 | 13.27 / 13.18 µs |
| Runtime allow / deny, Access 1, short ID | 115 / 23 ns |
| Typed allow / deny, Access 1, long ID | 171 / 132 ns |

Target creation is 42–49 ns for numeric, short, UUID, 64-character, and
128-character inputs. Typed `action_id()` is 17–20 ns for sampled 8–100
character action IDs; `RoleAssignment::new` is 66–71 ns.

| One operation | Allocations | Bytes | Deallocations |
| --- | ---: | ---: | ---: |
| Typed `can`, allow, Access 1 or allow-last Access 100 | 6 | 45 | 6 |
| Typed `can`, deny, Access 1 or 100 | 3 | 29 | 3 |
| Runtime `can`, allow, Access 1 or 100 | 3 | 16 | 3 |
| Runtime `can`, deny, Access 1 or 100 | 0 | 0 | 0 |
| `Target::ident` numeric / short | 2 | 10 | 2 |
| `Target::ident` UUID / long | 2 | 43 / 145 | 2 |
| Normal `action_id()` | 1 | 19 | 1 |
| Build Access: 1 / 10 / 100 / 1,000 facts | 4 / 31 / 301 / 3,001 | 95 / 950 / 9,500 / 95,000 | same |

The allow path's extra three allocations are cloned `via_role` and target
metadata. Typed construction is a clear fixed cost, but not a microsecond cost.

## Access construction

| Operation | 1 | 10 | 100 | 1,000 | 10,000 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Collect cloned assignments | 84 ns | 670 ns | 6.37 µs | 66.30 µs | 652.63 µs |
| Construct assignments and Access | 89 ns | 685 ns | 6.56 µs | 67.49 µs | 684.20 µs |
| Incremental `empty` + `add_role` | 102 ns | 810 ns | 7.41 µs | 72.49 µs | — |

Cloning existing Access with 1,000 facts took 66.08 µs. An index therefore
would move non-trivial work into request-context construction.

## `can_any`

The 50-sample tiny-access reruns isolate candidate overhead.

| Access | Operation | Time |
| ---: | --- | ---: |
| 0 | Single runtime deny | 9.39 ns |
| 0 | `can_any` 2 / 3 / 4 all deny | 110.48 / 164.08 / 207.71 ns |
| 0 | Equivalent 2 / 3 / 4 prebuilt runtime checks | 12.16 / 22.34 / 29.89 ns |
| 1 | `can_any` 2 / 3 / 4, last allows | 203.79 / 250.99 / 297.12 ns |
| 1 | Equivalent prebuilt runtime sequence | 93.80 / 100.71 / 99.95 ns |

Most of the public `can_any` gap is typed action/target construction; its
prebuilt runtime sequence leaves only a small per-candidate residual, including
lock reacquisition. All-deny candidate matrices at Access 100 were 513 / 784 /
860 ns for 2 / 3 / 4 targets; four targets at 1,000 were 6.57 µs. These use
mixed target shapes and are not equivalent to four all-target-matching scans.

## Policy and request workloads

Grant lookup for one access fact was 104 ns (1×1 policy), 116 ns (100×5), 118
ns (1,000×20), and ~130 ns (10,000×20): policy size is minor versus Access.

| Operation | Time |
| --- | ---: |
| Build policy: 100×5 / 1,000×20 / 10,000×20 | 71.41 µs / 2.91 ms / 29.89 ms median |
| Replace small / 100×5 / 1,000×20 policy | 46 ns / 6.17 µs / 211.13 µs |
| Build 1,000×20 without / with known-action validation | 2.87 / 3.18 ms |
| Simple article owner check | 155 ns |
| Normal user allowed leaf / `can_any` Wiki allow | 156 / 236 ns |
| Access 25, 10 mixed checks | 2.06 µs (206 ns/check) |
| Access 100, 100 mixed checks | 42.71 µs (427 ns/check) |
| Access 5 / 25 / 100, 100 mixed checks | 15.46 / 21.52 / 42.38 µs |
| Access 100, 1,000 mixed checks | 424.63 µs (425 ns/check) |

The 10,000×20 policy build had one iteration/sample, so its 29.89 ms median
(29.79–30.04 ms) is reported rather than a slope estimate.

## Concurrency

Fixed-work throughput: 100,000 checks per reader. This is machine-specific,
not a latency/fairness claim.

| Readers | Access 1 immediate allow | Access 1 deny | Access 100 last allow |
| ---: | ---: | ---: | ---: |
| 1 | 6.98 M/s | 40.32 M/s | 0.67 M/s |
| 2 | 11.46 M/s | 28.91 M/s | 1.32 M/s |
| 4 | 5.27 M/s | 9.76 M/s | 2.41 M/s |
| 8 | 3.50 M/s | 5.31 M/s | 2.05 M/s |
| 16 | 2.83 M/s | 6.86 M/s | 2.45 M/s |

For small immediate allows, 4 readers plus replacements every 1,000 / 100
reader operations achieved 4.94 / 5.23 M/s; 8 readers achieved 3.42 / 3.59
M/s. No dramatic throughput collapse appeared at these rates, though this is
not a percentile-latency test.

## Conclusions (without changes)

- The current `Vec<RoleAssignment>` is effectively negligible through about 25
  facts and stays below 1 µs for full misses through 50. Indexing becomes
  evidence-backed only for sustained contexts around 500–1,000 facts, where
  worst cases exceed 5–10 µs. Construction/clone costs make it less free than
  lookup-only data suggests. There is no evidence to change it now.
- `can_any` adds typed-construction cost; repeated read-lock acquisition is
  measurable only as a tiny residual with prebuilt runtime values. It does not
  dominate the realistic workloads measured here.
- Future investigation, ranked by evidence: (1) indexing only for sustained
  large contexts, while preserving assignment metadata order; (2) typed owned
  string and allow-decision clone costs for tiny-check-heavy workloads; (3)
  `can_any` typed construction before any lock redesign. Each must be proven
  against this full suite and allocation harness.

The suite intentionally rejects timing logging, random-data generation,
fixture construction inside lookup loops, and treating typed `can_any` as
semantically equivalent to diagnostic prebuilt runtime sequences.
