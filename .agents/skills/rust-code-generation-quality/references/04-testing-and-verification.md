# Testing and Verification

## Tests should target risks

Do not generate tests simply to increase test count.

For each behavior consider the relevant subset of:

- normal behavior;
- empty/minimum input;
- meaningful maximum/boundary values;
- malformed/invalid input;
- duplicates;
- ordering assumptions;
- repeated operations/idempotency;
- state transitions;
- overflow/limit conditions;
- Unicode/encoding boundaries;
- concurrency/cancellation behavior;
- authorization boundaries;
- persistence/external failures;
- known regressions.

Only test cases relevant to the domain. Avoid checklist-driven test inflation.

## Every test should have a failure hypothesis

For each new test ask:

> What distinct defect could this catch?

These may not represent distinct coverage:

```rust
assert!(parse("alice").is_ok());
assert!(parse("bob").is_ok());
assert!(parse("charlie").is_ok());
```

unless those inputs activate different semantics.

Test observable behavior and invariants rather than reproducing the production algorithm inside the test.

## Assert meaningful output

Do not stop at:

```rust
assert!(result.is_ok());
```

when returned state/data matters.

Be cautious of:

- collection-length assertions with no content checks;
- giant snapshots with no targeted invariants;
- mock call assertions that ignore resulting behavior;
- tests tightly coupled to private implementation details.

White-box testing is valid when the internal behavior itself is part of what must be protected.

## Regression tests

For a reproducible bug fix, add a test that fails before the fix and passes afterward whenever practical.

Capture the broken behavior, not the implementation technique used to repair it.

## Use repository verification first

Before inventing commands, inspect:

- `justfile`;
- `Makefile`;
- CI workflows;
- repository documentation;
- workspace scripts.

Run the maintainer-defined workflow when available.

## Default Rust verification

When the repository defines no equivalent workflow, use as applicable:

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Where project feature design means all features are intended to compose, also consider:

```bash
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Do not blindly use `--all-features` when features are mutually exclusive or intentionally represent alternative platforms/backends.

For public library crates, also consider:

```bash
cargo test --doc
cargo doc --no-deps
```

Run project-specific MSRV, target, feature-matrix, Miri, sanitizer, benchmark, integration, or no-std checks when relevant to the changed area.

## Clippy discipline

Understand lint findings before fixing or suppressing them.

Do not automatically add `#[allow(...)]`.

When suppression is intentional, keep it narrow and explain non-obvious reasons when the repository's style warrants it.

Do not globally enable the entire Clippy `restriction` lint group as a universal quality policy; many restriction lints encode mutually exclusive preferences.

`pedantic`/`nursery` can be useful review signals but should be evaluated individually.

## Green tools are not semantic proof

Passing rustc, Clippy, and tests does not establish:

- correct architecture;
- correct domain behavior;
- sensible ownership;
- realistic scalability;
- sufficient boundary coverage.

Always perform a final manual diff review after tooling passes.
