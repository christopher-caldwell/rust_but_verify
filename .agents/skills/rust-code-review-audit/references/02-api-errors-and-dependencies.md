# API, Error, and Dependency Audit

## Foreign-language architecture

Inspect unnecessary:

- getter/setter pairs;
- manager/service wrappers;
- factories;
- one-implementation interface-like traits;
- DTO mirrors;
- dependency-injection scaffolding;
- pass-through layers.

Do not flag because they resemble Java/TypeScript/C#. Flag when the abstraction adds concepts without expressing meaningful Rust/domain responsibilities.

## Trait audit

For every new trait ask:

- Are multiple implementations present?
- Must downstream users implement it?
- Does repository architecture intentionally define a port here?
- Is polymorphism a real requirement?
- Does it isolate a meaningful external dependency?
- Does it create a useful test boundary rather than merely enabling mocks?

“May have another implementation later” alone is weak justification.

## Generic audit

Inspect new generic structs/functions, associated types, complex trait bounds, and wrappers.

Ask what concrete requirement demands genericity.

Recommend a concrete design when it represents the actual domain, materially simplifies reasoning, and sacrifices no known use case.

Do not treat generic code as inherently superior or inferior.

## Builders and wrappers

Builders usually earn their cost through many optional settings, order-independent configuration, or staged invariants.

Flag ceremony such as a builder around a single required field when a constructor fully expresses the operation.

For newtypes/wrappers ask whether they:

- enforce an invariant;
- prevent meaningful confusion;
- express domain language;
- add behavior;
- protect an API boundary.

Do not remove useful semantic types merely to reduce line count.

## Boolean blindness

Inspect call sites such as:

```rust
create_user(name, true, false)
```

If booleans encode important domain choices and call-site meaning is unclear, an enum/config type may be appropriate.

Do not replace every boolean with a type automatically.

## Public API audit

Review every new `pub` item.

Ask:

> Does downstream code actually need this?

Watch for leaking:

- persistence representations;
- transport DTOs;
- dependency-specific types;
- internal numeric IDs;
- helpers;
- implementation-only traits.

Public surface is a compatibility commitment.

## Conversion API audit

Inspect custom methods like `to_*`, `from_*`, `convert`, `into_model` and consider whether a standard trait such as `From`, `TryFrom`, `AsRef`, or `IntoIterator` more accurately communicates the operation.

Do not force a standard trait when semantics do not fit.

## Panic-path audit

Search:

```text
.unwrap(
.expect(
panic!(
todo!(
unimplemented!(
```

and inspect uncertain indexing operations.

Allowed uses may include tests, examples, programmer bugs, and proven invariants.

Flag panic behavior for legitimate runtime failure.

Inspect `expect` messages. They should explain why the invariant guarantees success, not merely restate the action.

## Error flattening

Inspect:

```rust
Result<T, String>
Result<T, ()>
map_err(|e| e.to_string())
```

Determine whether structured information or source chains are lost at a boundary where callers need them.

Do not demand custom error types where no caller can make a meaningful distinction.

## Excessive error taxonomy

The opposite smell is an error enum that mirrors every dependency failure even though callers handle all variants identically.

Errors should expose meaningful semantic categories, not merely reconstruct the call graph.

## Ignored errors

Search for discarded `Result`s such as:

```rust
let _ = operation();
```

Determine whether failure is intentionally irrelevant. Flag silent operational failure.

## Cargo/dependency audit

Review every dependency change and verify:

- crate identity;
- version;
- feature names;
- API existence for that version;
- actual usage;
- compatibility with edition/MSRV/platform/licensing constraints where relevant;
- whether an existing dependency or `std` already provides the behavior;
- whether unrelated upgrades occurred.

Generated models can invent plausible crate APIs. Compile, but do not treat compilation as the only evidence: inspect the dependency contract and features being relied upon.

Flag unnecessary dependencies only when they add maintenance/supply-chain complexity without enough implementation value.
