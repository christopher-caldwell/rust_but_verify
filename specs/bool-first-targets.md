Implement the next RustButVerify iteration.

This iteration is now **locked in scope**. Do not broaden it into unrelated architecture work.

The goals are:

1. Make the common authorization API return `bool`.
2. Keep `Decision` as an opt-in provenance API.
3. Fix the confirmed `can_any` policy-snapshot correctness bug.
4. Replace the arbitrary 2–4 target tuple limit with:

   * tuple sugar for exactly 2 targets
   * `Targets<A>` for arbitrary numbers of typed targets
5. Tighten public/internal API boundaries.
6. Update examples/docs to teach the finalized shape.
7. Run only targeted performance validation after the change.

The comprehensive benchmark pass has already established that the current `Vec<RoleAssignment>`, policy representation, identifier representation, and `RwLock` are performant enough for the intended workload. Do not redesign them in this iteration.

---

# 1. Boolean authorization should be the default API

Change the public API so the natural authorization methods return `bool`.

The intended shape is:

```rust
authz.can(
    &access,
    EditArticle,
    Article::ident(991),
) -> bool
```

and:

```rust
authz.can_any(
    &access,
    EditArticle,
    (
        Article::ident(991),
        Wiki::ident(42),
    ),
) -> bool
```

The caller should no longer need:

```rust
authz.can(...).is_allowed()
```

The core mental model of this crate is a yes/no authorization question, so the default API should answer that directly.

---

# 2. Keep `Decision` as an opt-in provenance API

Do not remove `Decision`.

Instead, introduce:

```rust
authz.decide(
    &access,
    EditArticle,
    Article::ident(991),
) -> Decision
```

and:

```rust
authz.decide_any(
    &access,
    EditArticle,
    (
        Article::ident(991),
        Wiki::ident(42),
    ),
) -> Decision
```

Preserve the existing conceptual shape:

```rust
Decision::Allow {
    via_role,
    via_target,
}

Decision::Deny
```

The purpose of `Decision` is provenance/explanation:

> Which matching role and target caused this authorization check to succeed?

It is **not** an exhaustive explanation engine.

Document that clearly.

If multiple authorization paths could succeed, `Decision::Allow` represents the **first successful path according to evaluation order**.

For multi-target checks:

* tuple order is evaluation order
* `Targets<A>` insertion order is evaluation order

Do not attempt to return every possible authorization reason.

---

# 3. The bool path must not construct a `Decision`

Do not implement:

```rust
pub fn can(...) -> bool {
    self.decide(...).is_allowed()
}
```

The benchmark showed that successful `Decision::Allow` construction clones owned provenance and causes three additional allocations.

The boolean path should use the same matching logic but stop at:

```text
matching assignment found -> true
nothing matched -> false
```

Only `decide*` should clone the role/target data needed to produce an owned `Decision`.

The goal is semantic sharing without unnecessary result construction.

`can` and `decide` must not contain independent authorization algorithms that could drift apart.

Create an internal matching/evaluation primitive and have both public forms use it appropriately.

---

# 4. Make the runtime API symmetrical

The runtime API should follow the same convention.

Change:

```rust
can_runtime(...) -> Decision
```

to:

```rust
can_runtime(...) -> bool
```

and add:

```rust
decide_runtime(...) -> Decision
```

So the API families are symmetrical:

```text
Typed:
  can
  decide
  can_any
  decide_any

Runtime:
  can_runtime
  decide_runtime
```

The same underlying matching semantics must apply everywhere.

Again, `can_runtime` must not construct `Decision::Allow` internally.

---

# 5. Fix the confirmed `can_any` correctness bug

This is mandatory.

The benchmark/stress work reproduced a real correctness issue.

The current multi-target implementation independently acquires the policy read lock for each candidate target.

That means one `can_any` operation can observe:

```text
candidate 1 under Policy A
candidate 2 under Policy B
```

The alternating-policy stress test produced impossible `Deny` results even though **either complete policy snapshot would have allowed one candidate**.

This was reproduced at meaningful rates under concurrent policy replacement.

A single authorization operation must be evaluated against **one complete policy snapshot**.

The intended structure is approximately:

```text
can_any / decide_any
    acquire policy read guard ONCE
    evaluate candidate 1
    evaluate candidate 2
    ...
    release guard
```

Do not reacquire the policy lock per candidate.

The same rule applies whether the input is:

* the two-target tuple
* `Targets<A>`

This is primarily a correctness change. Any performance improvement is secondary.

---

# 6. Add a regression test for the policy-snapshot bug

Preserve the alternating-policy scenario used to reproduce the problem.

Conceptually:

```text
Access:
  role_b @ target_1
  role_a @ target_2

Policy A:
  role_a allows Edit
  role_b does not

Policy B:
  role_b allows Edit
  role_a does not
```

Therefore:

```text
Complete Policy A:
  target_1 denies
  target_2 allows

Complete Policy B:
  target_1 allows
  target_2 denies
```

There is **no valid complete policy snapshot where can_any(target_1, target_2) may deny**.

Add a concurrent regression test that repeatedly alternates the active policy while readers execute the multi-target check.

The assertion is:

```text
impossible Deny count == 0
```

Avoid making normal CI unnecessarily slow, but make the test substantial enough to exercise concurrent replacement.

More importantly, structure the implementation so it is obvious from the code that one read guard spans the complete evaluation. Do not rely only on probabilistic testing as proof of the design.

---

# 7. Remove the arbitrary 3- and 4-tuple API

The existing tuple support for 2, 3, and 4 candidate targets is an implementation artifact.

Do not merely raise the limit to 8, 12, or 16.

The public design should instead be:

## Exactly two targets: tuple convenience

Support:

```rust
authz.can_any(
    &access,
    EditArticle,
    (
        Article::ident(991),
        Wiki::ident(42),
    ),
)
```

This is intentionally the ergonomic:

> this OR that

form.

Tuple support should be **exactly 2 targets**.

Do not provide 3-, 4-, or arbitrary-N tuple implementations.

---

# 8. Add `Targets<A>` for arbitrary N

For more complex use cases, add a public:

```rust
Targets<A>
```

type.

The intended usage is:

```rust
let targets = Targets::<EditArticle>::new()
    .add(Article::ident(991))
    .add(Wiki::ident(42))
    .add(Organization::ident("acme"));

authz.can_any(
    &access,
    EditArticle,
    targets,
);
```

`Targets<A>` should carry the **action type**, not an action value.

Do not require this duplication:

```rust
Targets::for_action(EditArticle)
...
authz.can_any(..., EditArticle, targets)
```

Instead:

```rust
Targets::<EditArticle>::new()
```

uses the generic type purely to enforce valid target/action combinations while building the collection.

Internally it may erase validated typed targets into `TargetKey`s.

A reasonable conceptual representation is:

```rust
pub struct Targets<A> {
    targets: Vec<TargetKey>,
    _action: PhantomData<fn() -> A>,
}
```

The exact implementation may differ if there is a cleaner Rust shape.

---

# 9. `Targets<A>::add` must preserve compile-time compatibility

The key property is:

```rust
Targets::<EditArticle>::new()
    .add(target)
```

must only compile when:

```rust
EditArticle: ActionOn<TargetType>
```

Conceptually:

```rust
pub fn add<S>(mut self, target: Target<S>) -> Self
where
    A: ActionOn<S>,
    S: TargetType,
{
    ...
}
```

This preserves the same compile-time protection currently provided by typed `can`.

Invalid combinations should fail to compile.

For example, if `EditArticle` has not been declared valid on `Organization`, this must fail:

```rust
Targets::<EditArticle>::new()
    .add(Organization::ident("acme"));
```

Add compile-fail coverage for this.

---

# 10. `Targets<A>` has no arbitrary maximum

Do not impose a semantic target count limit.

A caller may add:

```text
1
3
10
100
1,000
```

targets if their domain genuinely requires it.

That performance cost belongs to the consuming application's authorization model.

RustButVerify should prevent invalid typed checks, but should not decide that an otherwise valid authorization model contains "too many" relevant targets.

Use a test with more than the previous four-target limit to prove this capability.

For example, 8–10 targets is enough to establish that this is not secretly capped at 4.

---

# 11. Empty and single-entry `Targets`

`Targets<A>` may naturally be assembled dynamically.

Allow:

```rust
Targets::<EditArticle>::new()
```

to contain zero targets.

Semantics:

```text
can_any(empty Targets) == false
decide_any(empty Targets) == Decision::Deny
```

Likewise, a `Targets<A>` containing one target is valid even though callers with a statically known single target would normally use `can`.

Do not introduce unnecessary minimum-size validation.

---

# 12. Preserve insertion order

`Targets<A>` should preserve insertion order.

Do not deduplicate or convert it to an unordered set.

Order matters because:

```rust
decide_any(...)
```

returns the first successful authorization path.

For:

```rust
Targets::<EditArticle>::new()
    .add(Article::ident(991))
    .add(Wiki::ident(42))
```

Article should be evaluated before Wiki.

Document this behavior.

---

# 13. Seal the multi-target input machinery

The current `CanAnyTargets`-style machinery should not be a downstream extension point.

Consumers should be able to pass only the supported public forms:

```text
(Target<A>, Target<B>)    // exactly two typed targets
Targets<Action>           // arbitrary typed collection
```

Use the normal Rust sealed-trait pattern.

If a public trait must appear in a public method bound, make it:

* sealed
* `#[doc(hidden)]`
* clearly implementation machinery

A downstream crate must not be able to implement the trait for arbitrary data and bypass the intended `ActionOn<T>` compile-time guarantees.

Do not make users learn this trait in normal documentation.

---

# 14. Keep `Targets` as the public terminology

Use:

```text
Target
Targets
```

Do not introduce:

```text
Scope
CandidateTargets
Resources
Objects
AnyOf
Alternatives
```

The library has already settled on **target** as the public concept.

`can_any` supplies the OR semantics.

`Targets` describes what the struct contains.

---

# 15. Make `Policy::role_grants` internal

`Policy::role_grants()` is authorization-engine implementation detail.

Consumers should not use `Policy` to manually reproduce the authorizer's decision process.

Make it crate-internal.

Also remove `grant` terminology from the internal method name while touching this code.

A name such as:

```rust
allows(...)
```

is preferable.

Do not expose a replacement public method unless there is a genuine consumer need.

---

# 16. Preserve everything benchmarks told us not to change

Do **not** change:

```text
Access = Vec<RoleAssignment>
```

The benchmark showed approximately:

```text
25 facts:    ~0.44 µs worst case
50 facts:    ~0.76 µs
100 facts:   ~1.4 µs
1,000 facts: ~13.7 µs
```

The intended 5–100 fact range is already healthy.

Do not add:

* HashMap-backed Access
* indexes
* cached lookup structures
* target indexes
* role indexes

Likewise, do not redesign:

* `RoleId`
* `ActionId`
* `TargetKey`
* owned String representation
* policy HashMap/HashSet representation
* `RwLock`
* policy replacement architecture

The exhaustive baseline did not justify those changes.

---

# 17. Do not optimize typed identifier allocations in this pass

The benchmark showed the typed API adds approximately 55–65 ns of fixed overhead in tiny cases because target/action IDs are created as owned values.

That is visible but not currently important enough to justify a representation redesign.

Do not introduce:

* `Cow`
* borrowed IDs
* interned strings
* static action IDs
* lifetime-heavy target APIs
* string caches

The bool-return change will naturally remove the `Decision::Allow` provenance allocations from the common authorization path. That is enough for this iteration.

---

# 18. Update unit tests around bool and Decision behavior

Add/adjust tests so the following invariants are explicit.

## Single target

```text
can == true
iff
decide == Decision::Allow
```

and:

```text
can == false
iff
decide == Decision::Deny
```

Test:

* matching target + permitted role
* wrong action
* wrong target
* unknown role
* empty Access

## Runtime

The same equivalence must hold for:

```text
can_runtime
decide_runtime
```

## Multi-target

The same equivalence must hold for:

```text
can_any
decide_any
```

using:

* first target allows
* second target allows
* neither allows
* empty `Targets`
* one-entry `Targets`
* many-entry `Targets`

---

# 19. Test deterministic provenance

For `decide` and `decide_any`, verify exactly which role/target is returned.

If two Access assignments can authorize the same target, preserve/document the current deterministic assignment iteration behavior.

For multi-target checks, target order must take precedence:

```text
evaluate target 1 against Access
if allowed, stop

otherwise evaluate target 2
...
```

Do not globally search all target/assignment combinations and then choose an arbitrary match.

---

# 20. Compile-time tests

Preserve and expand compile-time safety coverage.

At minimum verify:

## Valid two-target tuple

One action declared for both target types compiles.

## Invalid tuple target

An action not declared `ActionOn` one tuple target fails to compile.

## Valid `Targets<A>`

Different target types may be added when the action supports each one.

## Invalid `Targets<A>::add`

Adding an unsupported target type fails at compile time.

## Arbitrary count

A `Targets<A>` containing more than four entries compiles and works.

No need to add an elaborate compile-test framework if existing doctest/`compile_fail` coverage is sufficient.

---

# 21. Update the Knowledge Base examples

The examples package/domain work from the previous iteration should remain.

Update it to teach the new bool-first API.

The progression remains:

```text
knowledge_base/
  01_direct_article_access.rs
  02_article_ownership.rs
  03_wiki_access.rs
  04_combined_access.rs
```

## Step 01

Teach:

```rust
authz.can(...)
```

returning a boolean.

Do not introduce `Decision`.

## Step 02

Continue using the bool API while introducing application-derived ownership Access.

## Step 03

Introduce the exactly-two tuple form:

```rust
authz.can_any(
    &access,
    EditArticle,
    (
        Article::ident(article.id),
        Wiki::ident(article.wiki_id),
    ),
)
```

This should naturally read as:

> Article OR Wiki.

## Step 04

Combine ownership and Wiki-wide access.

Do not force `Targets<A>` into the beginner progression merely because it now exists.

The Knowledge Base domain only needs two relevant candidate target types, so the two-target tuple is the clearer teaching tool.

---

# 22. Document arbitrary-N `Targets` separately

`Targets<A>` still needs clear public documentation.

Add a concise README/rustdoc example showing something like:

```rust
let targets = Targets::<EditArticle>::new()
    .add(Article::ident(991))
    .add(Wiki::ident(42))
    .add(Organization::ident("acme"));

if authz.can_any(&access, EditArticle, targets) {
    // allowed through at least one relevant target
}
```

Use only target types on which the action is legitimately declared.

Explain:

* tuple syntax is convenience for exactly two targets
* `Targets<A>` is the arbitrary-N form
* `.add()` is compile-time checked
* order is preserved
* there is no arbitrary maximum
* the application is responsible for establishing why those targets are relevant
* RustButVerify does not discover resource relationships

Do not turn this into a relationship/policy language.

---

# 23. Update the non-macro examples

The `without_macros/` examples should continue demonstrating that:

```text
define_authorization!
```

is only convenience syntax over:

```text
TargetType
Action
ActionOn
```

Update them for the bool-return API where necessary.

If the multiple-target non-macro example is a natural place to demonstrate `Targets<A>`, it may do so, but keep the section small and focused on macro equivalence.

Do not create a second large tutorial suite.

---

# 24. Update README API examples

The root README should make the primary API immediately obvious.

Prefer examples like:

```rust
if authz.can(
    &access,
    EditArticle,
    Article::ident(991),
) {
    // ...
}
```

Then explain the richer optional form:

```rust
match authz.decide(...) {
    Decision::Allow { via_role, via_target } => { ... }
    Decision::Deny => { ... }
}
```

Do not present `.is_allowed()` as the normal workflow anymore.

Explicitly describe `Decision` as an optional provenance result.

---

# 25. Update rustdoc

Public method documentation should clearly distinguish:

```text
can
    yes/no authorization

decide
    authorization + first successful provenance

can_any
    yes/no across relevant targets

decide_any
    authorization + first successful target/role provenance
```

Document the one-policy-snapshot guarantee for multi-target evaluation.

That guarantee is now important enough to be part of the behavioral contract.

---

# 26. Preserve `Decision::is_allowed()` unless there is a compelling reason not to

Do not remove `Decision::is_allowed()` merely because `can` now returns bool.

It remains a reasonable convenience when callers explicitly requested a `Decision`.

This is not a necessary breaking cleanup.

---

# 27. Update benchmark harnesses for the API changes

The existing exhaustive benchmark suite/report is valuable baseline evidence.

Do not throw it away.

Adjust benchmark code as necessary because:

```text
can
can_any
can_runtime
```

now return bool.

Where a benchmark specifically measures provenance construction, use:

```text
decide
decide_any
decide_runtime
```

Preserve the distinction between:

```text
bool fast path
provenance path
```

---

# 28. Run targeted post-change benchmarks only

Do not rerun the entire exhaustive matrix.

We already answered the broad performance questions.

Run focused validation for the changes made here.

## Bool vs Decision

At minimum benchmark/allocation-count:

```text
typed can, Access 1 allow
typed decide, Access 1 allow

typed can, Access 1 deny
typed decide, Access 1 deny

can_runtime, Access 1 allow
decide_runtime, Access 1 allow

can_runtime, Access 1 deny
decide_runtime, Access 1 deny
```

Expected qualitative result:

* `can_runtime` should not allocate merely to return `true`
* typed `can` should retain typed target/action construction allocations but avoid provenance cloning
* `decide*` should retain provenance cost

Do not contort implementation merely to hit a benchmark number. Report what happens.

## `can_any`

Measure:

```text
2-target tuple, tiny Access, all deny
2-target tuple, tiny Access, second allows
Targets<A> with 4 targets, tiny Access, all deny
Targets<A> with 10 targets, tiny Access, all deny
```

This is sanity validation, not a new performance exploration.

## Realistic workload

Rerun one Knowledge Base mixed workload to make sure no accidental major regression occurred.

---

# 29. Re-run the concurrent snapshot stress test

This is the primary acceptance criterion for the correctness fix.

Use the same alternating-policy concept that previously reproduced impossible denials.

Report:

```text
total calls
allows
impossible denies
duration
```

The required result after the fix is:

```text
impossible denies = 0
```

Run with at least:

```text
1 reader + 1 writer
4 readers + 1 writer
8 readers + 1 writer
```

This test cannot mathematically prove all concurrency behavior, but combined with the code structure where one read guard spans the whole multi-target evaluation, it provides strong regression evidence.

---

# 30. Do not change release/publishing architecture

The publishing/release infrastructure added in the previous iteration is out of scope except for updates mechanically required by changed examples/docs/tests.

Do not redesign:

* Justfiles
* publish script
* release-plz
* CI architecture
* package metadata

Make only necessary compatibility adjustments.

---

# 31. Do not release or bump versions

This remains draft-stage API iteration.

Do not:

* publish to crates.io
* create tags
* prepare a release
* bump the package version solely because this API changed
* manufacture changelog history

The release tooling should continue to validate.

---

# 32. Validation

Before finishing, run the normal root validation, including whatever recipes already exist.

At minimum:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

Validate the standalone examples package as well:

```bash
cargo check --manifest-path examples/Cargo.toml --bins
```

Run its Justfile examples where useful to verify tutorial commands still work.

Run release/package dry validation if that is already part of the repository's normal validation flow.

Do not publish.

---

# 33. Final report

When finished, report:

1. final public API signatures
2. how `can` and `decide` share evaluation without forcing Decision construction
3. how the one-policy-snapshot guarantee is implemented
4. regression-test result for the impossible-deny race
5. final `Targets<A>` API
6. how exactly-two tuple support works
7. confirmation that 3/4 tuple implementations were removed
8. compile-time safety tests added
9. examples/docs updated
10. targeted benchmark results
11. allocation comparison between bool and Decision paths
12. any API friction discovered during implementation
13. all files materially changed
14. validation commands and results
15. confirmation that the broader performance architecture (`Vec`, IDs, Policy, `RwLock`) was intentionally left unchanged

Do not use the final report to propose another broad redesign unless implementation uncovered a concrete correctness or usability problem.

The purpose of this pass is to finish and stabilize this API iteration so we can move on.

