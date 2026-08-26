# Concurrency, Unsafe, and Performance Audit

## Async audit

For newly async functions ask whether they perform meaningful asynchronous work or merely inherit async because an outer layer uses a runtime.

Pure computation and in-memory transformations usually do not need async.

Do not demand synchronous code when architecture intentionally requires an async trait/boundary.

## Spawn audit

For each `spawn`/`tokio::spawn` determine:

- owner/lifecycle;
- how failures are observed;
- whether completion is awaited;
- shutdown behavior;
- cancellation safety;
- whether the task may outlive its required state;
- whether task creation can grow without bound.

Task spawning used mainly to solve lifetime or ownership friction is a strong design-review signal.

## Lock audit

Inspect guards that may live across `.await`.

Determine whether data can be extracted and the guard released before suspension.

A synchronous mutex held across `.await` deserves especially strong scrutiny.

For `Arc<Mutex<T>>`, independently justify multiple ownership, thread sharing, mutation, and locking.

## Unbounded concurrency

Look for:

- spawning in arbitrary-size loops;
- `join_all` on unbounded input;
- unbounded channels/queues;
- unlimited stream concurrency.

Evaluate realistic workload rather than theoretical maximum, but report material resource-growth hazards.

## Unsafe audit

Search:

```text
unsafe {
unsafe fn
unsafe trait
unsafe impl
Send
Sync
from_raw
as_ptr
transmute
MaybeUninit
NonNull
extern "C"
```

For each unsafe operation, independently verify:

- pointer provenance/validity;
- alignment;
- initialization;
- aliasing rules;
- bounds;
- lifetime validity;
- ownership;
- thread safety;
- ABI assumptions;
- panic/unwind assumptions.

Do not accept a `// SAFETY` comment as proof. Confirm the surrounding safe API actually maintains the invariant.

`unsafe impl Send` and `unsafe impl Sync` require exceptional scrutiny because incorrect implementations can invalidate safety assumptions elsewhere.

## Collection audit

Evaluate data structures from actual operations:

- realistic size;
- lookup frequency;
- insertion/removal patterns;
- ordering requirements;
- memory overhead;
- iteration frequency.

Do not mechanically recommend `HashMap` over `Vec` based on big-O alone. Small linear collections can be simpler and faster. Conversely, repeated linear search at realistic large scales may be material.

If a data-structure decision is performance-driven and crossover behavior is uncertain, recommend benchmarks rather than intuition.

## Allocation audit

Search meaningful uses of:

```text
clone
to_string
to_owned
to_vec
collect
format!
Box
Arc
```

Pay extra attention in loops/hot paths.

Look for collect-then-iterate sequences whose intermediate collection provides no sorting, snapshot, repeated traversal, ownership, or borrow-management benefit.

Do not recommend convoluted zero-allocation code where the allocation is cheap and clarity is materially better.

## Algorithm/performance claims

Challenge claims such as:

- O(1) / constant-time;
- lock-free;
- zero-copy;
- allocation-free;
- scalable;
- optimized.

Require the implementation and, where appropriate, benchmark/profiling evidence to support them.

Flag speculative caches, specialized indexes, unsafe optimization, custom allocators, or complicated collection structures when realistic requirements do not warrant the complexity.
