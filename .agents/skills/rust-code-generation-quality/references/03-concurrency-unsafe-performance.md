# Concurrency, Unsafe, and Performance

## Async should represent asynchronous work

Do not make pure computation or in-memory transformations async merely because surrounding code uses Tokio or another runtime.

Async generally belongs where operations suspend while waiting for asynchronous work.

Avoid propagating `async` through layers that have no asynchronous behavior unless the architectural boundary requires it.

## Task spawning changes semantics

Do not use `spawn` to escape ownership or lifetime problems.

For every spawned task answer:

- Who owns its lifecycle?
- Who observes failure?
- Who waits for completion, if anyone?
- What happens during shutdown?
- Is cancellation safe?
- Can it outlive required resources/state?
- Can spawning become unbounded?

Detached fire-and-forget behavior must be intentional.

## Locks across await

Avoid holding inappropriate guards across `.await`.

Before awaiting with protected state:

1. determine whether required data can be extracted;
2. release the guard;
3. perform async work;
4. reacquire only if required.

A synchronous mutex across suspension deserves especially strong scrutiny.

## Unbounded concurrency

Inspect:

- `spawn` inside unbounded loops;
- `join_all` on arbitrary-size input;
- unbounded stream concurrency;
- unbounded queues/channels.

Use realistic workload assumptions rather than theoretical infinity, but bound work where resource growth can materially hurt the system.

## Unsafe Rust

Safe Rust is the default.

Do not introduce unsafe code merely for convenience or hypothetical performance.

Before every unsafe operation establish:

1. why safe Rust cannot reasonably express the operation;
2. which invariant the compiler cannot verify;
3. why the operation is valid;
4. how safe surrounding code preserves the invariant;
5. how to minimize and isolate the unsafe surface.

Safety reasoning may involve:

- pointer provenance;
- alignment;
- initialization;
- aliasing;
- bounds;
- lifetimes;
- ownership;
- thread safety;
- ABI assumptions;
- panic/unwind behavior.

Never write a vague comment such as:

```rust
// SAFETY: pointer is valid
```

unless the implementation establishes *why* it is valid for the exact operation.

Treat `unsafe impl Send`, `unsafe impl Sync`, raw pointers, FFI, `transmute`, `MaybeUninit`, and manual memory management as exceptional scrutiny areas.

## Data structures from access patterns

Do not choose a collection solely from asymptotic complexity.

Consider:

- realistic size;
- lookup frequency;
- insertion/removal behavior;
- ordering requirements;
- memory overhead;
- iteration frequency;
- key distribution.

For small collections, a `Vec` linear scan can be simpler and faster than hashing. For large repeatedly searched collections, an index may be warranted.

Use benchmarks/profiling when the decision materially affects performance and realistic crossover points are uncertain.

## Allocation discipline

Inspect meaningful uses of:

- `.clone()`;
- `.to_owned()`;
- `.to_string()`;
- `.to_vec()`;
- `.collect()`;
- `format!`;
- `Box`;
- temporary `Vec`/`String` creation.

Pay extra attention in loops and frequent paths.

Do not remove allocations at the cost of substantially worse clarity unless workload evidence justifies the complexity.

Watch for collect-then-iterate pipelines where the intermediate collection has no sorting, snapshot, repeated traversal, ownership, or borrow-management purpose.

## Optimization standard

Do not introduce custom caches, unsafe optimizations, specialized collections, manual allocation strategies, or complex indexing for hypothetical scale.

Likewise, do not ignore obvious quadratic behavior or repeated allocations when realistic requirements make them material.

The target is performance proportional to actual workload and measured evidence, not performative optimization.
