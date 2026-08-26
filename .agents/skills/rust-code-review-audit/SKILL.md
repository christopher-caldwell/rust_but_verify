---
name: rust-code-review-audit
description: Audit Rust code or diffs for low-quality generated implementation, compiler-appeasement, ownership mistakes, speculative abstractions, weak error handling, fabricated dependencies/APIs, concurrency or unsafe hazards, shallow tests, and unnecessary scope. Use for code review, PR review, generated-code auditing, or pre-merge quality review.
---

# Rust Code Review Audit

Use this skill to review Rust code for engineering quality, especially when code may have been generated or heavily assisted by an LLM.

Do **not** attempt to prove AI authorship from style.

The audit target is:

> **Unreasoned code: implementation choices that cannot be justified by the requirements, ownership semantics, repository architecture, safety invariants, or measured workload.**

The strongest recurring failure mode is code patched locally until the compiler/tests are green instead of code designed around the problem.

## Required audit workflow

### 1. Establish context before judging the diff

Before findings:

- inspect `Cargo.toml`, workspace structure, edition, MSRV, and features;
- inspect repository instructions, CI, `justfile`/`Makefile`, and contribution docs;
- inspect nearby modules and analogous implementations;
- identify the requested behavior and expected scope;
- inspect relevant existing tests;
- read the diff before evaluating new files in isolation.

Judge the code against a coherent existing repository rather than imposing a preferred architecture.

Load `references/01-repository-scope-and-ownership.md` for context, scope, and ownership audit rules.

### 2. Audit concrete design decisions

Review:

- ownership and borrowing;
- every meaningful clone/allocation;
- shared ownership/interior mutability;
- function signatures;
- traits, generics, builders, wrappers, and module boundaries;
- every new public item;
- error behavior;
- Cargo/dependency changes.

Load `references/02-api-errors-and-dependencies.md` for API, error, and dependency rules.

### 3. Apply heightened scrutiny to concurrency and unsafe code

For async/spawn/locks, verify lifecycle, boundedness, cancellation, and ownership. For unsafe code, independently verify invariants instead of trusting `// SAFETY` prose.

Also assess collections, algorithms, and allocations using realistic workload assumptions.

Load `references/03-concurrency-unsafe-and-performance.md`.

### 4. Evaluate tests by what they prove

Do not reward test count. Determine the distinct defect each test could catch, whether important failure/boundary cases are represented, and whether bug fixes have regression coverage.

Run repository-defined checks; otherwise run appropriate Cargo/Clippy verification.

Load `references/04-tests-and-verification.md`.

### 5. Perform a generated-code pattern scan

Look for **clusters**, not isolated syntax tells:

- compiler-appeasement clusters;
- speculative architecture stacks;
- superficial quality/presentation exceeding substance;
- error erasure;
- async escape hatches;
- duplicated repository concepts;
- unrelated diff churn.

Report the underlying engineering problem. Never report “this looks AI-generated” as a finding.

Load `references/05-patterns-and-reporting.md` for the pattern scan and final reporting format.

## Severity levels

Use these conservatively:

- **Critical** — likely unsoundness, security vulnerability, data corruption, severe concurrency/correctness failure.
- **High** — material correctness/design issue likely to cause bugs, operational problems, or major maintenance cost.
- **Medium** — meaningful maintainability, API, ownership, complexity, allocation, or edge-case problem.
- **Low** — minor cleanup or polish with concrete value.
- **Observation** — not necessarily wrong, but worth attention because it may indicate a larger problem.

Do not inflate severity to make the audit appear useful.

## Review principles

- Style is not evidence of AI authorship.
- No single `clone`, `unwrap`, trait, builder, `Arc`, comment, or long README is an AI tell or automatic quality problem.
- Clippy findings are signals to understand, not infallible verdicts.
- Passing tests do not prove design correctness.
- Prefer specific findings tied to consequences over generic “not idiomatic” criticism.
- Recommend the smallest responsible correction rather than broad rewrites.
- Do not impose personal architecture preferences on a coherent repository.
- Do not reward sophistication, line count, comment count, documentation volume, or number of tests.

## Finding format

For each material finding, report:

### `[Severity] Short issue title`

**Location:** file/function/lines where practical.

**Problem:** concrete implementation issue.

**Why it matters:** correctness, safety, maintainability, performance, API, or operational consequence.

**Recommended direction:** smallest appropriate correction; avoid rewriting the feature unless necessary.

## Final summary

End with:

### Verdict

Choose one:

- **Clean**
- **Clean with minor issues**
- **Needs revision**
- **Significant design concerns**
- **Unsafe / incorrect**

### Highest-risk areas

Name only the most important few.

### Verification performed

List commands actually run and results. Never imply checks were run when they were not.

### Scope confidence

State whether the change appears to satisfy the requested behavior and any important requirement that could not be verified.

## References

- `references/01-repository-scope-and-ownership.md` — repository context, scope, ownership, clones, borrowing, shared state.
- `references/02-api-errors-and-dependencies.md` — abstractions, public API, error handling, dependency/API verification.
- `references/03-concurrency-unsafe-and-performance.md` — async/spawn/locks, unsafe invariants, collections, allocations, performance.
- `references/04-tests-and-verification.md` — test-risk analysis, regression coverage, Cargo/Clippy checks.
- `references/05-patterns-and-reporting.md` — slop-pattern clusters, diff hygiene, final reasoning audit and reporting.
- `references/06-evidence-and-sources.md` — evidence and official guidance underlying the audit rules.
