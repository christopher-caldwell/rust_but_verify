# Generated-Code Pattern Scan and Reporting

## Look for clusters, not isolated tells

No individual occurrence of a clone, trait, builder, `Arc`, `unwrap`, comment, iterator chain, or long README proves poor generated code.

Investigate combinations that suggest shallow local fixes.

### Compiler-appeasement cluster

```text
clone
+ Arc
+ Mutex/interior mutability
+ Box/dynamic dispatch
+ lifetime complexity
```

Question whether the ownership model was actually designed.

### Speculative architecture cluster

```text
trait
+ builder
+ factory
+ config object
+ one implementation
```

Question whether flexibility exists only hypothetically.

### Superficial-quality cluster

```text
large documentation
+ many comments
+ many tests
+ weak assertions
+ small/simple actual behavior
```

Presentation exceeding substance is worth investigating, but verbosity itself is not a violation.

### Error-erasure cluster

```text
unwrap/expect
+ map_err(to_string)
+ ignored Result
```

Question whether fallibility and abstraction boundaries were understood.

### Async escape-hatch cluster

```text
Arc<Mutex<_>>
+ cloned shared state
+ spawn
+ unobserved JoinHandle
```

Question lifecycle, ownership, cancellation, and failure handling.

Report the engineering consequence, never “this is AI slop” as the finding itself.

## Complexity audit

Count newly introduced concepts:

- types;
- traits;
- modules;
- configuration layers;
- wrappers;
- factories;
- builders;
- macros;
- adapters.

Ask which current requirements require each concept.

Look especially for one-implementation stacks with pass-through layers.

## Helper explosion

Flag many one-use helper functions when they obscure control flow and do not name meaningful concepts.

Conversely, flag enormous functions when meaningful domain operations are buried.

The goal is conceptual clarity, not arbitrary function size.

## Comment/documentation audit

Flag comments that narrate syntax rather than explain reasons/invariants.

Challenge unsupported statements such as:

- production-ready;
- thread-safe;
- scalable;
- lock-free;
- zero-copy;
- optimized.

A large README is not an AI tell. Investigate only when documentation claims exceed implemented behavior or compensate for unclear code.

## Final reasoning audit

Before concluding, answer:

### Ownership
- Is ownership of major values clear?
- Are clones intentional?
- Is shared ownership necessary?
- Are lifetime relationships natural rather than forced?

### API
- Is public surface intentional and limited?
- Do signatures communicate ownership?
- Are abstractions justified by current requirements?
- Does the API compose naturally with Rust conventions?

### Errors
- Are normal failures recoverable?
- Is useful information preserved at appropriate boundaries?
- Are panic paths actual invariants?

### Concurrency
- Is async necessary?
- Are spawned tasks owned/observable?
- Are locks scoped correctly?
- Is concurrency bounded where required?

### Safety
- Is unsafe necessary?
- Are invariants actually established?

### Performance
- Do structures/algorithms match realistic workloads?
- Are meaningful allocations justified?
- Are optimization claims evidenced?

### Tests
- Does each important test target a distinct risk?
- Are critical failure paths covered?
- Are bug fixes protected by regressions when practical?

### Architecture/scope
- Does the change fit the repository?
- Did it recreate existing concepts?
- Is the change larger than its requirement warrants?

## Weak explanations

Repeated reliance on these answers should trigger deeper investigation:

- “the compiler required it”;
- “it might be useful later”;
- “it is more flexible”;
- “it is cleaner”;
- “it is enterprise-ready”;
- “it is best practice”;
- “this pattern is common.”

Strong explanations tie decisions to domain behavior, Rust ownership semantics, repository architecture, API contracts, safety invariants, observed failure modes, measured workload, or actual extensibility requirements.

## Finding format

Use descending severity:

### `[Severity] Short issue title`

**Location:** `path`, symbol, and lines where practical.

**Problem:** explain the concrete issue.

**Why it matters:** explain the actual consequence.

**Recommended direction:** give the smallest responsible correction.

Do not rewrite large areas merely to demonstrate an alternative.

## Severity guidance

### Critical

Likely unsoundness, exploitable security issue, data corruption, or severe correctness/concurrency failure.

### High

Material correctness/design issue likely to cause bugs, operational failures, or significant maintenance cost.

### Medium

Meaningful ownership, API, complexity, edge-case, allocation, or maintainability problem.

### Low

Concrete but minor cleanup or polish.

### Observation

Not necessarily wrong, but potentially indicative of a larger issue.

## Final summary format

### Verdict

Choose exactly one:

- **Clean**
- **Clean with minor issues**
- **Needs revision**
- **Significant design concerns**
- **Unsafe / incorrect**

Do not base the verdict on presumed authorship.

### Highest-risk areas

Name only the most important few, not every minor finding.

### Verification performed

List checks actually executed and their outcomes.

### Scope confidence

State whether requested behavior appears satisfied and what could not be verified.

## What not to do

Do not:

- accuse the author of AI usage;
- treat style or verbosity as proof of AI;
- flag every clone or every unwrap automatically;
- demand either traits everywhere or concrete types everywhere;
- demand zero allocations;
- rewrite clear loops into iterators for style;
- rewrite clear iterator chains into loops for style;
- impose a preferred architecture over a coherent repository;
- recommend broad refactors for narrow issues;
- treat Clippy as infallible;
- treat tests as proof of correctness;
- reward complexity, test count, comment count, or documentation volume.

The audit succeeds when it identifies **unreasoned engineering decisions**, regardless of who or what wrote them.
