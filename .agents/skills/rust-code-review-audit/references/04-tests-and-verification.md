# Test and Verification Audit

## Determine what every test proves

For each newly added/modified test ask:

> What distinct defect could this test catch?

Multiple arbitrary values that traverse identical behavior do not necessarily add coverage.

Do not reward test count.

## Boundary and failure audit

Depending on the domain, inspect the relevant subset of:

- empty/minimum inputs;
- boundary/maximum values;
- malformed/invalid values;
- duplicates;
- ordering;
- repeated calls/idempotency;
- state transitions;
- overflow/limits;
- Unicode/encoding;
- cancellation/concurrency;
- authorization boundaries;
- persistence/external failures;
- legacy/serialization compatibility.

Do not require irrelevant edge cases merely to fill a checklist.

## Error-path coverage

Generated suites often overrepresent happy paths.

Check important failures such as validation errors, unavailable dependencies, malformed responses, persistence failures, or state conflicts when applicable.

## Assertion quality

Be skeptical of tests that only:

```rust
assert!(result.is_ok());
```

when returned data/state matters.

Also inspect:

- collection length without content assertions;
- giant snapshots with no targeted invariant;
- mock-call assertions without behavior validation;
- duplicated implementation logic inside tests.

Prefer observable contracts and meaningful invariants.

## Regression tests

For bug fixes, identify the previous broken behavior and verify a regression test captures it whenever practical.

Ideally, the test would fail on the previous implementation and pass on the fix.

## Repository verification

Run maintainer-defined checks first, e.g. via:

- `justfile`;
- `Makefile`;
- CI workflow commands;
- project scripts/documentation.

Do not invent a parallel workflow when the repository already has one.

## Default verification when no repository workflow exists

As applicable:

```bash
cargo fmt --check
cargo check --workspace --all-targets
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Where feature composition is valid:

```bash
cargo check --workspace --all-targets --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Do not blindly use `--all-features` when features are mutually exclusive.

For public crates consider:

```bash
cargo test --doc
cargo doc --no-deps
```

Apply project-relevant MSRV, feature matrix, no-std, Miri, sanitizer, benchmark, integration, or cross-target checks when warranted.

## Clippy audit

Understand each material lint rather than mechanically applying or suppressing it.

Treat blanket `#[allow]` additions as suspicious unless repository policy explains them.

Do not globally demand the `restriction` group; it contains policy/style lints that can conflict.

## Report verification truthfully

Distinguish:

- checks actually run and passed;
- checks run and failed;
- checks not run;
- behavior that cannot be verified from available context.

Never imply a green validation step that was not executed.

Green tooling is necessary evidence but does not prove architecture, semantics, or sufficient edge-case coverage.
