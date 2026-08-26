# Error Handling and Dependencies

## Panic is not ordinary error handling

Do not casually use:

- `.unwrap()`;
- `.expect(...)`;
- `panic!()`;
- unchecked indexing;
- `todo!()` / `unimplemented!()` in a supposedly complete path.

Use `Result` or `Option` for expected or recoverable failure.

Panic paths can be appropriate for:

- tests/examples;
- programmer bugs;
- invariants already established by construction or validation;
- truly impossible internal states.

If using `expect`, explain the invariant rather than restating the operation.

Weak:

```rust
value.expect("value should exist")
```

Better:

```rust
value.expect("validated configuration always contains a primary endpoint")
```

A good message does not make an unjustified panic acceptable.

## Preserve useful error information

Be skeptical of:

```rust
Result<T, String>
Result<T, ()>
foo().map_err(|e| e.to_string())?
```

when callers may need structured information or a source chain.

Conversely, do not manufacture a huge custom error hierarchy where every caller handles every variant identically.

Design errors around meaningful failure categories and abstraction boundaries.

A library API may require structured errors so callers can react. An application boundary may appropriately aggregate/contextualize errors.

Do not expose lower-level implementation failures in public error types unless they are intentionally part of the contract.

## Do not ignore errors silently

Inspect uses like:

```rust
let _ = operation();
```

Only discard an error when failure is intentionally irrelevant and that intent is clear from context.

Do not make operations appear successful by swallowing failures.

## Dependency verification is mandatory

Never invent:

- crate names;
- versions;
- feature flags;
- modules;
- functions;
- methods;
- type names;
- Cargo features.

Before using a dependency:

1. inspect `Cargo.toml`;
2. prefer dependencies and APIs already used by the repository;
3. verify the version actually in use;
4. verify the API exists for that version;
5. verify required features;
6. compile against the repository configuration;
7. do not silently upgrade a dependency merely to obtain an API.

A plausible-looking API is not evidence that it exists.

## Adding dependencies

Before adding a crate ask:

- Does `std` already solve this adequately?
- Does an existing dependency solve it?
- Is a small local implementation clearer and safer?
- Does the dependency materially reduce complexity or correctness risk?
- Does it fit repository licensing/MSRV/platform requirements?

Do not optimize for either “fewest dependencies” or “maximum reuse.” Use justified dependencies.

Do not reimplement substantial established functionality simply to avoid a reasonable dependency.

## Serialization and external formats

When changing serialized/public data shapes:

- preserve compatibility requirements intentionally;
- inspect rename/default/deny-unknown-field behavior;
- test absent, malformed, unknown, and legacy values where relevant;
- do not infer wire formats from Rust type names alone.

External formats are API contracts even when the Rust compiler cannot enforce them.
