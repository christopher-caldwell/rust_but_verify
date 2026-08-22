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
| Typed `can` | 88.03 ns | 75.42 ns |
| Typed `decide` | 152.18 ns | 75.61 ns |
| Runtime `can_runtime` | 29.31 ns | 15.97 ns |
| Runtime `decide_runtime` | 95.70 ns | 16.27 ns |

The allow-path delta is the expected provenance construction; denying does
not construct `Decision::Allow` provenance.

| Multi-target case | Mean |
| --- | ---: |
| Two-target tuple, all deny | 114.57 ns |
| Two-target tuple, second target allows | 134.43 ns |
| `Targets<EditArticle>` with four all-deny candidates | 240.67 ns |
| `Targets<EditArticle>` with ten all-deny candidates | 635.83 ns |

One Knowledge Base-style mixed workload, `normal_user_can_any_wiki_allow`,
measured 173.41 ns. The other selected realistic checks were 89.30 ns for a
simple article owner check, 91.53 ns for an allowed leaf, 1.73 µs for ten
mixed checks against Access 25, and 39.49 µs for 100 mixed checks against
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
| 1 + 1 | 13,594,021 | 13,594,021 | 0 | 3.003 s |
| 4 + 1 | 17,452,212 | 17,452,212 | 0 | 3.005 s |
| 8 + 1 | 9,802,519 | 9,802,519 | 0 | 3.005 s |

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
