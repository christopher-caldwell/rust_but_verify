---
name: rust-code-generation-quality
description: Generate intentional, idiomatic Rust while avoiding compiler-appeasement, speculative abstractions, fabricated APIs, weak tests, excessive cloning, casual shared ownership, and other common forms of low-quality AI-generated code. Use when writing, modifying, refactoring, or completing Rust code.
---

# Rust Code Generation Quality

Use this skill whenever generating or materially modifying Rust code.

The objective is not to make generated code *look human*. The objective is to produce Rust whose important decisions can survive experienced human review.

The central failure mode to prevent is:

> **Resolving compiler errors locally instead of resolving design questions globally.**

A compiler-approved workaround is not automatically a good design.

## Required workflow

### 1. Understand the repository before coding

Before implementation:

- inspect the relevant `Cargo.toml` and workspace configuration;
- identify edition, MSRV/`rust-version`, enabled features, and existing dependencies;
- read nearby modules and analogous implementations;
- inspect tests that define current behavior;
- inspect repository instructions and verification tooling such as `AGENTS.md`, `CONTRIBUTING.md`, `justfile`, `Makefile`, and CI;
- determine the intended ownership, error, API, and architectural boundaries before adding code.

Prefer consistency with a coherent existing project over introducing a new local architecture.

### 2. Implement the smallest responsible design

Do not add flexibility, layers, traits, generics, builders, dependencies, shared state, async behavior, or public API surface without a present reason.

A useful question for every new concept is:

> **What current requirement causes this to exist?**

If the answer is only “future flexibility,” “best practice,” “cleaner,” or “the compiler required it,” reconsider it.

### 3. Treat ownership failures as design feedback

Never reflexively resolve borrow-checker friction with:

- `.clone()` / `.to_owned()` / `.to_string()` / `.to_vec()`;
- `Arc` / `Rc`;
- `Mutex` / `RwLock` / `RefCell`;
- `Box` or dynamic dispatch;
- extra lifetime parameters;
- spawning a task.

First determine who should own the value, whether the callee needs ownership, and whether the current abstraction is forcing the wrong relationship.

Load `references/01-ownership-and-api.md` for detailed ownership and API rules.

### 4. Make fallibility explicit

Do not use panic paths as ordinary error handling. Preserve useful error information and design error types around meaningful caller behavior rather than mirroring every lower-level failure.

Never invent crate names, APIs, feature flags, methods, versions, or modules. Verify dependencies against the actual repository and installed version.

Load `references/02-errors-and-dependencies.md` when touching fallible operations or external crates.

### 5. Be conservative with concurrency, unsafe, and optimization

`async`, task spawning, shared ownership, locks, unsafe code, caching, and specialized collections are architectural decisions—not convenient compiler escape hatches.

Use them only when the workload or semantics require them, and understand lifecycle and invariants before implementation.

Load `references/03-concurrency-unsafe-performance.md` when those concerns are present.

### 6. Test risks, not code volume

Tests must target behavior, boundaries, failure modes, state transitions, and regressions. Do not manufacture many equivalent happy-path tests merely to increase coverage or apparent completeness.

Run repository-defined verification first. When none exists, use the standard Rust checks described in `references/04-testing-and-verification.md`.

### 7. Perform a dedicated anti-slop review after tooling passes

Do not stop at green compilation or tests. Re-read the diff specifically for:

- unexplained clones and allocations;
- panic paths;
- speculative traits/generics/builders/wrappers;
- unnecessary `pub` items;
- new dependencies;
- excessive comments or documentation claims;
- unrelated scope expansion;
- placeholder behavior;
- tests that do not prove distinct risks.

Load `references/05-scope-comments-and-self-review.md` for the final pass.

## Non-negotiable rules

- Do not play compiler-error whack-a-mole.
- Do not invent dependencies or APIs.
- Do not clone merely to satisfy the borrow checker.
- Do not introduce `Arc<Mutex<_>>` merely because ownership is inconvenient.
- Do not introduce async or task spawning to escape lifetime problems.
- Do not introduce `unsafe` without explicit, verified invariants.
- Do not create abstractions for hypothetical future implementations.
- Do not expose implementation details publicly without a consumer need.
- Do not use `.unwrap()`, `.expect()`, or `panic!()` for normal recoverable runtime failure.
- Do not narrate obvious code with comments.
- Do not claim code is production-ready, lock-free, zero-copy, allocation-free, scalable, or optimized without establishing that claim.
- Do not broaden a narrow request into an unrelated refactor.
- Do not declare completion merely because the code compiles.

## Decision hierarchy

When several implementations are valid, prefer roughly in this order:

1. correctness;
2. safety;
3. consistency with the repository;
4. clear ownership and lifetime semantics;
5. simple public API;
6. low reasoning burden;
7. appropriate error behavior;
8. meaningful testability;
9. performance appropriate to realistic workloads;
10. extensibility required by known use cases;
11. brevity.

Do not sacrifice the top of this list to improve the bottom.

## Definition of done

Before declaring generated Rust complete, verify that:

- requested behavior is implemented without unrelated expansion;
- ownership is intentional and understandable;
- clones, allocations, smart pointers, and locks have semantic reasons;
- abstractions correspond to real concepts or architecture boundaries;
- public API surface is intentional;
- dependency names, versions, features, and APIs are verified;
- normal failure paths use appropriate error handling;
- concurrency has an understood lifecycle and boundedness where necessary;
- unsafe code, if any, has explicit and actually established invariants;
- tests cover meaningful risks and regressions;
- formatting, compilation, tests, and appropriate Clippy checks pass;
- the final diff has been manually reread after automated verification.

Finally ask:

> **If the compiler had accepted the first implementation immediately, would I still have designed it this way?**

If not, investigate why.

## References

Load only the references relevant to the current change:

- `references/01-ownership-and-api.md` — ownership, borrowing, API design, abstraction discipline, public surface.
- `references/02-errors-and-dependencies.md` — panic/error handling, dependency verification, API hallucination prevention.
- `references/03-concurrency-unsafe-performance.md` — async, spawning, locks, unsafe invariants, collections, allocations, optimization.
- `references/04-testing-and-verification.md` — test quality, edge cases, regression tests, Cargo/Clippy verification.
- `references/05-scope-comments-and-self-review.md` — implementation scope, comments/docs, final anti-slop diff scan.
- `references/06-evidence-and-sources.md` — research and official guidance underlying these rules.
