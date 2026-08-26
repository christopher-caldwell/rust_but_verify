# Scope, Documentation, and Final Self-Review

## Keep scope proportional to the request

Do not spontaneously:

- redesign neighboring APIs;
- upgrade unrelated dependencies;
- rename unrelated types;
- add caching or telemetry;
- add feature flags;
- introduce generic infrastructure;
- reorganize unrelated modules;
- change serialization unrelated to the requested behavior.

Adjacent problems may be reported separately rather than folded into the implementation.

A narrow request that produces a broad pile of improvements is a common form of generated-code slop.

## Avoid conceptual fan-out

Count concepts, not just lines.

Review newly introduced:

- types;
- traits;
- modules;
- adapters;
- wrappers;
- builders;
- factories;
- macros;
- configuration layers.

Ask whether each concept is required to express the problem.

A 100-line direct solution can be simpler than a 50-line solution distributed across ten abstractions.

Watch especially for one-implementation stacks where each layer only forwards calls.

## Helpers should name concepts

Do not split a function into many tiny helpers solely to make the primary function look short.

Extract when doing so creates:

- a meaningful concept;
- reuse;
- independently useful testing;
- clearer control flow;
- a real abstraction boundary.

Likewise, do not preserve a giant function when meaningful concepts are hidden inside it.

## Comments explain why

Avoid narration:

```rust
// Increment the counter
counter += 1;
```

Comments should primarily explain:

- non-obvious reasons;
- invariants;
- external constraints;
- important tradeoffs;
- intentionally unusual behavior;
- safety reasoning.

Public documentation should help consumers understand behavior, ownership expectations, errors, panics where relevant, important side effects, and invariants.

Do not produce documentation filler for obvious private helpers.

## Prose must not outrun evidence

Do not claim code is:

- production ready;
- thread safe;
- lock free;
- zero copy;
- allocation free;
- constant time;
- scalable;
- optimized;

unless the implementation and evidence actually establish that claim.

Use precise technical descriptions instead of promotional language.

## No hidden placeholders

A completed implementation must not conceal:

- `todo!()`;
- `unimplemented!()`;
- fake success values;
- dummy branches;
- silently ignored errors;
- commented-out alternate implementations.

If something cannot be completed correctly, surface that explicitly rather than disguising incompleteness.

## Dedicated anti-slop pass

After all automated checks pass, inspect the diff again.

### Ownership scan

Search for:

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

Do not assume they are wrong. Verify each meaningful use has a semantic reason.

### Panic scan

Search for:

```text
unwrap(
expect(
panic!(
todo!(
unimplemented!(
```

Verify each production-path occurrence.

### Abstraction scan

Review every new:

- trait;
- generic parameter;
- builder;
- factory;
- wrapper;
- macro;
- configuration structure;
- module.

Ask what current requirement caused it to exist.

### API scan

Review every new `pub` and every newly exposed dependency/internal type.

### Dependency scan

Review every Cargo change and verify identity, version, features, API use, necessity, and relevant project constraints.

### Comment scan

Delete comments that restate syntax. Strengthen comments that claim to explain invariants but do not actually establish them.

### Test scan

For every new test ask what distinct defect it could catch.

### Scope scan

Compare the final diff against the original requirement. Remove unrelated “while I am here” changes unless necessary.

## Final questions

A reviewer should be able to answer clearly:

- Why does this type exist?
- Why does this function own or borrow its inputs?
- Why does this clone exist?
- Why does this trait/generic exist?
- Why is this item public?
- Why is this dependency required?
- What failures can occur and how are they represented?
- What invariant does each important test protect?
- Why is concurrency needed?
- Who owns spawned work?
- What justifies unsafe code?
- What happens at realistic scale?

Weak explanations include:

- “the compiler required it”;
- “it might be useful later”;
- “it is more flexible”;
- “it is cleaner”;
- “it is best practice.”

When those answers underpin important implementation choices, reconsider the design.
