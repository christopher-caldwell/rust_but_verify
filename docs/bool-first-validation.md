# Bool-first targeted validation: 2026-08-22

This report records the locked follow-up checks for the bool-first API. It
does not replace the previously established exhaustive evidence in
[`benchmark-results.md`](../benchmark-results.md); that historical baseline
remains intact and was not rerun.

The architecture remains unchanged: `Access` stores its assignments in a
`Vec`, identifiers remain owned, `Policy` uses `HashMap`/`HashSet`, and
`Authorizer` continues to protect policy replacement with `RwLock`.

## Focused timing

`cargo bench --bench authorization_baseline --
'decision_allow_vs_deny|locked_multi_target|realistic_workloads'` ran the
focused Criterion groups only. Values are mean estimates in nanoseconds on
this machine; typed calls construct their action and target inside the timed
closure, while runtime keys are prebuilt.

| Access 1 path | Allow | Deny |
| --- | ---: | ---: |
| Typed `can` | 87.18 ns | 75.43 ns |
| Typed `decide` | 152.98 ns | 76.12 ns |
| Runtime `can_runtime` | 27.39 ns | 16.15 ns |
| Runtime `decide_runtime` | 101.01 ns | 16.61 ns |

The allow-path delta is the expected provenance construction; denying does
not construct `Decision::Allow` provenance.

| Multi-target case | Mean |
| --- | ---: |
| Two-target tuple, all deny | 114.96 ns |
| Two-target tuple, second target allows | 145.35 ns |
| `Targets<EditArticle>` with four all-deny candidates | 238.56 ns |
| `Targets<EditArticle>` with ten all-deny candidates | 637.10 ns |

One Knowledge Base-style mixed workload, `normal_user_can_any_wiki_allow`,
measured 179.12 ns. The other selected realistic checks were 89.42 ns for a
simple article owner check, 95.25 ns for an allowed leaf, 1.77 µs for ten
mixed checks against Access 25, and 40.07 µs for 100 mixed checks against
Access 100.

## Allocation accounting

`cargo bench --bench allocation_counts` measured one release-mode operation
per `stats_alloc` region.

| Access 1 operation | Allow allocations / bytes | Deny allocations / bytes |
| --- | ---: | ---: |
| Typed `can` | 3 / 29 | 3 / 29 |
| Typed `decide` | 6 / 45 | 3 / 29 |
| Runtime `can_runtime` | 0 / 0 | 0 / 0 |
| Runtime `decide_runtime` | 3 / 16 | 0 / 0 |

Accordingly, successful boolean paths avoid the three provenance allocations
that `Decision::Allow` requires. Typed operations retain their independent
action/target construction allocations.

## Snapshot stress

`cargo bench --bench can_any_consistency` alternated policies with one writer
while each reader repeatedly checked a tuple whose candidates are jointly
allowable under either complete policy snapshot. A denial would therefore be
impossible if one read guard spans the complete operation.

| Readers + writer | Total calls | Allows | Impossible denies | Duration |
| --- | ---: | ---: | ---: | ---: |
| 1 + 1 | 10,639,840 | 10,639,840 | 0 | 3.006 s |
| 4 + 1 | 17,133,154 | 17,133,154 | 0 | 3.005 s |
| 8 + 1 | 10,037,499 | 10,037,499 | 0 | 3.005 s |

Every locked stress configuration reported zero impossible denials.

## Repository validation

The following commands passed without publishing:

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS=-D warnings cargo doc --no-deps
cargo check --manifest-path examples/Cargo.toml --bins
just -f examples/Justfile check
just package-check
```
