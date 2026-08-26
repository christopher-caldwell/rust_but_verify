# Repository, Scope, and Ownership Audit

## Establish repository context

Before judging implementation choices:

1. read relevant `Cargo.toml` files;
2. identify workspace members;
3. identify edition and `rust-version`/MSRV;
4. inspect enabled/default features;
5. inspect nearby modules and equivalent patterns;
6. read repository instructions (`AGENTS.md`, `CONTRIBUTING.md`, etc.);
7. inspect verification tooling and CI;
8. inspect existing tests for affected behavior;
9. identify the intended change scope.

Do not review new files as if they were standalone snippets.

A pattern that is questionable in isolation may be required by repository architecture. Conversely, generated code may duplicate functionality that was already present elsewhere.

## Scope audit

Determine what the change was actually supposed to accomplish.

Then inspect for unrelated expansion such as:

- dependency upgrades;
- generic infrastructure;
- broad renames;
- unrelated refactors;
- caching/telemetry;
- feature flags;
- new configuration systems;
- unrelated serialization changes;
- module reorganizations.

Ask:

> Did the problem require this additional concept/change, or did implementation generate adjacent work?

Flag unnecessary scope because it increases review surface and regression risk.

## Diff hygiene

Look for unrelated churn:

- formatting untouched files;
- arbitrary import reordering not caused by tooling;
- lockfile churn without relevant dependency changes;
- accidentally committed generated files;
- unrelated symbol renames.

Unnecessary churn is a reviewability problem even if behavior is correct.

## Ownership scan

Search changed code for meaningful uses of:

```text
.clone(
.to_owned(
.to_string(
.to_vec(
Arc<
Rc<
Mutex<
RwLock<
RefCell<
Box<
```

Do not flag mechanically.

For each meaningful occurrence ask:

- Why is ownership required?
- Why are two owners required?
- Could a borrow express the operation?
- Does the source become dead immediately afterward?
- Is the clone expensive or inside a loop/hot path?
- Did the ownership machinery appear mainly after compiler/lifetime friction?

A useful finding explains the specific ownership mismatch, not merely “avoid clone.”

Example:

> `request.user_id` is cloned before validation even though validation only reads it and the original is later moved into the domain value. Borrowing `&str` during validation avoids the allocation and better expresses ownership.

## Compiler-appeasement clusters

A single clone is weak evidence. A sequence of increasingly heavy mechanisms deserves investigation:

```text
clone -> extra clone -> Arc -> Mutex -> Box/lifetime complexity
```

Try to reconstruct the intended ownership model.

If the model is unclear, report that architectural issue rather than each individual syntax choice.

## Function signature audit

Be suspicious of:

```rust
fn foo(value: &String)
fn foo(values: &Vec<T>)
```

when `&str` or `&[T]` expresses the contract better.

Also inspect owned `String`, `Vec`, map, or domain parameters that are only observed and never retained.

Do not mechanically recommend borrowing. Ownership may be correct when the function stores, consumes, transfers, or transforms the value.

The question is whether the signature communicates lifecycle semantics accurately.

## Shared ownership audit

For `Arc`, `Rc`, locks, and interior mutability determine why each property is required:

1. multiple owners;
2. mutation;
3. cross-thread sharing where applicable;
4. locking/interior mutability as the right coordination model.

`Arc<Mutex<T>>` used primarily to make ownership convenient is a design smell.

Check whether ownership transfer, temporary borrowing, clearer state ownership, or message passing would more accurately represent the system.

## Duplicate concept search

Before accepting new infrastructure, search the repository for an existing:

- domain type;
- utility;
- parser;
- validation helper;
- error type;
- adapter;
- conversion;
- collection/index.

Generated code often independently recreates a concept already present in the repository.

Concept duplication is usually more important than superficial line duplication.
