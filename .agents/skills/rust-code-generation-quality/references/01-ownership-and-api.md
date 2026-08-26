# Ownership and API Design

## Ownership is part of the design

Rust ownership signatures communicate semantics. Decide who owns each important value before treating borrow-checker diagnostics as local syntax problems.

When ownership fails, ask:

1. Who logically owns this value?
2. Does the callee merely observe it or retain it?
3. Should ownership move instead of being duplicated?
4. Is a temporary borrow sufficient?
5. Is the current module/API boundary forcing an unnatural relationship?

### Cloning

Every meaningful clone should answer:

> Why must two independently owned values exist?

Reasonable cases include:

- retaining a snapshot beyond the source lifetime;
- sending independent ownership to another thread/task;
- cloning a cheap shared handle such as an `Arc` when shared ownership is already intentional;
- satisfying an external API that genuinely requires owned data.

Bad reasons include:

- avoiding understanding the borrow checker;
- cloning immediately before the original becomes dead;
- cloning solely so a value can be borrowed from the copy;
- repeatedly cloning collections or strings inside loops with no semantic requirement.

Do not eliminate a legitimate clone merely to reduce allocations. The goal is intentional ownership, not clone prohibition.

## Borrowing at boundaries

Prefer the least restrictive parameter type that accurately expresses the function's semantics.

Be suspicious of:

```rust
fn process(value: &String)
fn process(values: &Vec<T>)
```

when the operation can naturally accept:

```rust
fn process(value: &str)
fn process(values: &[T])
```

Likewise, do not take `String`, `Vec<T>`, or another owned container when the function only reads it and does not retain ownership.

Take ownership when the function stores, consumes, transforms, transfers, or otherwise becomes responsible for the value.

Do not mechanically convert every owned parameter to a reference. Signatures should communicate the real lifecycle.

## Shared ownership

Treat these as architectural choices:

- `Arc`;
- `Rc`;
- `Mutex`;
- `RwLock`;
- `RefCell`.

Before introducing shared mutable state, establish why all relevant properties are required:

1. multiple owners;
2. mutation;
3. cross-thread access, if `Arc`/thread-safe locking is used;
4. locking/interior mutability as the correct coordination mechanism.

`Arc<Mutex<T>>` is not a generic solution to inconvenient ownership.

Consider instead whether ownership can move, data can be borrowed, state can live under a clearer owner, or message passing better represents the relationship.

## Rust API vocabulary

Do not translate object-oriented or TypeScript/Java/C# architecture into Rust mechanically.

Question unnecessary:

- getters and setters;
- `Manager`/`Service` wrappers that only forward calls;
- factories;
- traits that exist solely as one-to-one interfaces;
- dependency-injection scaffolding;
- duplicated DTO layers;
- builders for trivial construction;
- wrappers that enforce no invariant.

Use standard Rust mechanisms when they match the semantics:

- enums and pattern matching;
- `From` / `TryFrom`;
- `AsRef`;
- iterators;
- associated functions;
- newtypes;
- traits;
- `Option` / `Result`.

Do not use them ceremonially merely to appear idiomatic.

## Abstraction discipline

Do not create an abstraction because it *might* have multiple implementations later.

A trait is usually justified when at least one of these is true:

- multiple implementations exist;
- downstream consumers are expected to implement it;
- polymorphism is an actual domain requirement;
- the repository architecture intentionally uses a port/adaptor boundary;
- it isolates a meaningful external dependency;
- it creates a real testing boundary rather than merely making mocking possible.

Likewise, generics, associated types, builders, factories, macros, extension traits, and configuration layers should correspond to current requirements.

Prefer concrete code until an abstraction expresses a real concept or removes real duplication that changes for the same reason.

## Newtypes and semantic types

Use a newtype or enum when it materially:

- enforces an invariant;
- prevents confusion between same-shaped values;
- expresses important domain language;
- provides meaningful behavior;
- protects an abstraction boundary.

Do not wrap every primitive simply to demonstrate type safety.

Be especially cautious around APIs with many same-shaped primitives or booleans, e.g. multiple `String`s followed by `true, false`. When call-site meaning is ambiguous, semantic types may be warranted.

## Public API discipline

Default to private visibility.

For every newly public item ask:

> Does a downstream consumer actually need this?

Do not expose internal persistence representations, transport DTOs, dependency-specific types, helper traits, or internal identifiers without intentional contract reasons.

Public surface creates compatibility obligations.

Do not make something public merely because an arbitrary new module boundary requires access; consider whether the module split itself is useful.

## Conversions

Before inventing methods such as:

```rust
fn to_widget(...)
fn from_widget(...)
fn convert(...)
```

consider whether `From`, `TryFrom`, `AsRef`, `IntoIterator`, or another standard trait accurately describes the conversion.

Use a custom method when the conversion semantics do not fit a standard trait cleanly.

## Control-flow style

Use loops, iterators, `match`, `if let`, `let ... else`, and `?` according to clarity—not as markers of Rust sophistication.

Do not replace a clear loop with a dense iterator chain solely to look idiomatic.

Do not manually reproduce a standard Rust construct when the standard expression is materially clearer.

Avoid cleverness that forces the reviewer to decode generic bounds, lifetime tricks, macros, or nested combinators when the problem does not require that sophistication.
