You are working in this repository:

`https://github.com/christopher-caldwell/rust_but_verify`

This project is still in an exploratory/draft stage. The public API is intentionally unstable and breaking changes are expected. Do not preserve existing API shapes merely for compatibility. The goal is to improve the model based on what we learned from actually using the examples.

Tests should cover the important semantics, but do not over-invest in exhaustive compatibility or edge-case testing yet. We are still learning what the API should be.

## Goal

Refine RustButVerify around a clearer distinction between:

1. long-lived authorization policy; and
2. role/access facts supplied for a particular actor or authorization context.

At the same time, keep the public API extremely natural to read.

Do **not** drift toward formal authorization terminology merely because it is technically precise. We previously used terms like `scope` and found them harder to reason about in normal application code.

A backend developer should be able to read the examples and understand the API without learning authorization-theory vocabulary first.

---

## Core Mental Model

The library should remain very small.

Conceptually:

```text
Policy:
    What actions may each role perform?

Access:
    What roles does this actor have on which targets?

Authorization:
    Can this access perform this action on this target?
```

Example:

```text
Policy:
    wiki_editor can article.edit
    article_owner can article.edit

Alice's access:
    wiki_editor on Wiki 42

Bob's access for this request:
    article_owner on Article 991
```

Then:

```text
Can Alice edit on Wiki 42?       YES
Can Bob edit on Article 991?     YES
Can Bob edit on Article 992?     NO
```

The evaluator should remain fundamentally based on exact target matching.

---

# Important Scaling Finding

We specifically examined this scenario:

* an application has 1,000 wikis;
* each wiki may have 10,000 articles;
* the application may therefore contain millions of articles.

RustButVerify must **not** require every article to be represented in memory just because a user has access to a wiki.

For example:

```text
Alice:
    wiki_editor on Wiki 42
```

is sufficient regardless of whether Wiki 42 contains:

```text
10 articles
10,000 articles
10,000,000 articles
```

When Alice edits Article 991, the application already knows that the article belongs to Wiki 42 and may authorize against:

```rust
Wiki::ident(42)
```

RustButVerify does not need to know that Article 991 belongs to Wiki 42.

Do not add relationship traversal or entity-graph storage to solve this.

---

# Leaf-Level Access Is Also Required

Do not assume every authorization relationship has a useful parent/group target.

Example:

Bob owns Article 991.

Bob is not part of the editorial staff of its wiki.

Bob should be allowed to edit his own article and nobody else's.

Conceptually:

```text
Policy:
    article_owner can article.edit

Bob's relevant access:
    article_owner on Article 991
```

Then:

```rust
authz.can(
    &access,
    EditArticle,
    Article::ident(991),
)
```

must allow.

Checking:

```rust
Article::ident(992)
```

must deny.

This means both of these are legitimate authorization paths:

```text
wiki_editor on Wiki 42
article_owner on Article 991
```

Neither one should be treated as more fundamental than the other.

Targets may represent broad authority or extremely narrow authority.

---

# Access Must Not Mean "The Complete ACL for This User"

This is a critical clarification.

Suppose Bob has authored 50,000 articles.

We should **not** expect the application to permanently materialize:

```text
article_owner on Article 1
article_owner on Article 17
article_owner on Article 38
...
50,000 assignments
```

into some giant in-memory object merely because Bob owns those articles.

The application may construct or augment `Access` with facts relevant to the current authorization operation.

For example:

```rust
let mut access = load_normal_user_access(user.id);

if article.owner_id == user.id {
    access.add_role(
        "article_owner",
        Article::ident(article.id),
    );
}
```

Then authorize.

The important distinction is:

```text
Policy
    long-lived

Access
    caller supplied
    may contain durable assignments
    may contain request-derived assignments
    does NOT imply completeness
```

Document this clearly.

The library must not require a model/policy rebuild merely because request-specific access facts change.

---

# Policy Lifetime

The current implementation separates these concerns fairly well internally.

The long-lived policy should contain relationships like:

```text
wiki_editor -> article.edit
article_owner -> article.edit
admin -> article.delete
```

That policy should generally be built:

* at application startup;
* after configuration changes;
* or whenever the consuming application intentionally refreshes authorization policy.

It should **not** be rebuilt per request.

Request-specific facts belong in `Access`, not in the policy.

Preserve atomic whole-policy replacement rather than adding incremental mutation merely to support request-specific authorization.

---

# Public Naming

Natural API design is a major requirement.

## Rename `Model` to `Policy`

`Model` is vague.

Prefer:

```rust
Policy
PolicyBuilder
PolicyError
```

The user-facing concept is:

> this is my authorization policy.

Do not introduce a public type named `RoleGrants` or similar.

Internally, grants are a fine implementation concept.

Publicly, `Policy` is clearer.

---

## Consider `grant(...)` -> `allow(...)`

Current conceptual usage:

```rust
policy.grant("editor", EditArticle);
```

Consider replacing it with:

```rust
policy.allow("editor", EditArticle);
```

This reads naturally:

> allow editor to EditArticle

Evaluate the resulting API across all examples.

Prefer natural reading at call sites over terminology borrowed from authorization theory.

If `allow` creates ambiguity or awkwardness elsewhere, make a reasoned decision, but bias toward it.

---

## Keep `Access`

Do **not** rename `Access` to something formal like:

```text
RoleAssignments
AuthorizationContext
PrincipalAssignments
```

unless there is an overwhelmingly strong usability reason discovered during implementation.

`Access` is deliberately simpler.

Instead, clarify its semantics in documentation:

> `Access` contains role assignments available to an authorization check. It does not need to represent every resource an actor could ever access.

---

## Keep `Target`

Do not revert to `Scope`.

We intentionally moved away from `scope`.

Even though "scope" may be technically precise, `Target` is easier to reason about in application code.

Keep the current general idea:

```rust
Wiki::ident(42)
Article::ident(991)
```

and typed target compatibility.

Internals may use whatever terminology is appropriate, but public application-facing code should remain natural.

---

# RoleAssignment Ergonomics

Review the current `RoleAssignment` construction API.

Currently the typed/common path is more verbose than necessary.

Aim for common code to look approximately like:

```rust
RoleAssignment::new(
    "wiki_editor",
    Wiki::ident(42),
)
```

rather than requiring users to think about erased/raw target keys.

If a low-level runtime constructor is still useful, give it an explicit escape-hatch name such as:

```rust
RoleAssignment::from_key(...)
```

or another natural equivalent.

The normal path should strongly favor typed targets.

Also preserve convenient APIs such as:

```rust
Access::with_role(...)
access.add_role(...)
```

Review whether `Access::new()` versus iterator construction can be made more ergonomic without overengineering.

Possible desirable style:

```rust
let mut access = Access::empty();

access
    .add_role("wiki_editor", Wiki::ident(42))
    .add_role("member", Organization::ident("acme"));
```

or an equivalent builder-like approach.

Use judgment.

---

# Multiple Valid Authorization Targets

The same application action may legitimately be authorized against multiple target types.

Example:

```rust
action EditArticle = "article.edit" for [Article, Wiki];
```

That declaration is meaningful.

It means:

```text
EditArticle may be authorized at an Article target
or at a Wiki target.
```

For an article edit endpoint, the application might have two legitimate authorization paths:

```text
Article 991
Wiki 42
```

because:

* the article owner may have direct Article access;
* an editor may have Wiki-wide access.

Do not infer the relationship between those targets.

The application explicitly supplies the legitimate targets.

---

# Add / Explore `can_any`

A first-class multi-target authorization helper now appears justified.

Conceptually:

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

should succeed if either supplied target independently authorizes the action.

Important semantics:

```text
for each supplied candidate target:
    check normal exact-match authorization

if any allows:
    ALLOW

otherwise:
    DENY
```

Do not perform:

* parent traversal;
* relationship inference;
* target expansion;
* wildcard matching;
* database lookup;
* ownership discovery.

The caller already established that these are valid authorization targets.

### Typed safety

Try to preserve the existing compile-time property that an action can only be checked against target types declared compatible with it.

The challenge is that `can_any` may need heterogeneous targets such as:

```text
Article
Wiki
Organization
```

Investigate a clean Rust API for this.

A tuple-based API may work well because tuple implementations can preserve heterogeneous compile-time checking.

For example:

```rust
(
    Article::ident(id),
    Wiki::ident(wiki_id),
)
```

is potentially preferable to forcing everything into a homogeneous erased `Vec`.

Do not build elaborate type machinery merely to make this work.

If a small, understandable typed implementation is possible, implement it.

If it becomes disproportionately complex, preserve the ordinary `can()` API and document/manual-test the equivalent:

```rust
authz.can(...Article...).is_allowed()
    || authz.can(...Wiki...).is_allowed()
```

The public API must remain understandable to ordinary Rust developers.

---

# Decision Metadata

The current allow result contains:

```rust
Decision::Allow {
    via_role
}
```

That becomes insufficient once an action may succeed through multiple candidate targets.

Prefer something approximately like:

```rust
Decision::Allow {
    via_role: RoleId,
    via_target: TargetKey,
}
```

or the best natural equivalent.

For example:

Bob:

```text
Allow
    via_role = article_owner
    via_target = article::991
```

Alice:

```text
Allow
    via_role = wiki_editor
    via_target = wiki::42
```

This is useful diagnostic information without introducing a full explanation/provenance engine.

Do not try to track:

```text
team membership source
database row source
ownership derivation
relationship path
```

RustButVerify does not know those things.

It only knows which supplied role/target pair actually matched.

---

# Global Role Definitions

Explicitly review and document this current semantic:

Role policy definitions are global.

Conceptually:

```text
wiki_editor -> article.read
wiki_editor -> article.edit
```

means that wherever someone has the `wiki_editor` role, that role has the same permissions.

Assignments are target-specific:

```text
Alice: wiki_editor on Wiki 42
Bob:   wiki_editor on Wiki 97
```

But the meaning of `wiki_editor` itself is not independently configured for each wiki.

Therefore this is naturally supported:

```text
Wiki 42:
    Alice has wiki_editor

Wiki 97:
    Bob has wiki_editor
```

But this is not directly modeled under the current design:

```text
Wiki 42's editor:
    read
    edit
    delete

Wiki 97's editor:
    read only
```

unless the application uses distinct runtime role identifiers.

Do **not** add per-target policy definitions in this iteration.

That would significantly change:

* policy cardinality;
* policy persistence expectations;
* policy loading;
* replacement semantics;
* the conceptual boundary of the library.

Instead, document the limitation clearly.

Consumers needing different role semantics may currently use role IDs such as:

```text
wiki_42_editor
wiki_97_editor
```

or otherwise normalize their policy before giving it to RustButVerify.

We can revisit domain/tenant-specific role definitions later if real usage proves it necessary.

---

# Preserve These Boundaries

RustButVerify should continue to do **none** of the following:

* persistence;
* SQL/database access;
* Redis access;
* relationship fetching;
* relationship traversal;
* ownership discovery;
* parent discovery;
* domain-object inspection;
* policy expressions;
* ABAC predicates;
* wildcard authorization;
* framework integration;
* HTTP status decisions;
* user identity management;
* group membership lookup;
* automatic resource loading.

The application supplies authorization facts.

RustButVerify evaluates them.

---

# Desired Core Shape

Aim for an API conceptually close to:

```rust
define_authorization! {
    target Article = "article";
    target Wiki = "wiki";

    action EditArticle = "article.edit" for [Article, Wiki];
    action DeleteArticle = "article.delete" for [Article, Wiki];
}

let mut policy = Policy::builder();

policy
    .allow("wiki_editor", EditArticle)
    .allow("article_owner", EditArticle)
    .allow("wiki_admin", DeleteArticle);

let authz = Authorizer::new(policy.build()?);
```

Alice:

```rust
let alice_access =
    Access::with_role("wiki_editor", Wiki::ident(42));

authz.can(
    &alice_access,
    EditArticle,
    Wiki::ident(42),
);
```

Bob:

```rust
let mut bob_access = Access::empty();

if article.owner_id == bob.id {
    bob_access.add_role(
        "article_owner",
        Article::ident(article.id),
    );
}

authz.can(
    &bob_access,
    EditArticle,
    Article::ident(article.id),
);
```

Combined endpoint:

```rust
authz.can_any(
    &access,
    EditArticle,
    (
        Article::ident(article.id),
        Wiki::ident(article.wiki_id),
    ),
);
```

Exact syntax is still subject to iteration.

Optimize for natural reading rather than mechanically implementing these examples.

---

# Examples

Update the examples so this lifecycle is obvious.

In particular, add or modify the knowledge-base/wiki example to demonstrate both authorization paths:

### Alice

```text
wiki_editor on Wiki 42
```

Alice can edit Article 991 because the application chooses Wiki 42 as a legitimate authorization target.

No Article-specific assignment is needed.

### Bob

```text
article_owner on Article 991
```

Bob can edit Article 991.

Bob cannot edit Article 992.

Bob does not need any Wiki-level role.

### Charlie

Has neither applicable access path.

Charlie cannot edit Article 991.

The example should make clear that the application's database/domain code determined:

```text
Article 991 belongs to Wiki 42
Bob owns Article 991
```

RustButVerify did not discover either relationship.

The application simply supplied the authorization facts/targets resulting from those facts.

---

# Documentation

Update the README and relevant docs to establish the lifecycle explicitly.

The important explanation should be roughly:

```text
Policy is long-lived authorization configuration.

Access contains role assignments supplied to an authorization check.

Access does not need to contain every resource an actor might ever access.

Applications may load durable assignments and derive additional
request-specific assignments from domain facts.

Targets are checked exactly.

RustButVerify does not infer relationships between targets.
```

Also update the existing example-friction documentation with the scaling finding.

Explain that:

```text
authority at Wiki 42
```

does not imply materializing an assignment for every Article within Wiki 42.

Likewise, ownership/direct-resource authorization can be represented by a request-specific assignment for the specific Article being operated on.

---

# Review Existing Code Before Editing

Inspect at least:

```text
src/model.rs
src/authorizer.rs
src/identifiers.rs
src/typed.rs
src/lib.rs

README.md
docs/example-api-friction.md
specs/init/spec.md
specs/vnext-ergonomics.md

examples/*
tests/*
```

Do not assume the descriptions in this prompt perfectly match the current implementation. Reconcile them with the actual repository state.

---

# Implementation Priorities

Prioritize, in order:

1. preserve the tiny exact-match evaluator;
2. make policy lifetime versus request/access lifetime obvious;
3. improve public naming and ergonomics;
4. ensure the design scales without materializing domain resources;
5. demonstrate both broad target access and direct resource access;
6. add `can_any` if it can remain simple and understandable;
7. improve decision metadata;
8. update docs/examples to teach the correct mental model.

Avoid adding abstractions merely because other authorization systems have them.

RustButVerify is intentionally narrower than Cedar, Casbin, or a ReBAC engine.

---

# Validation

At minimum, verify these cases:

```text
Policy:
    wiki_editor -> article.edit
    article_owner -> article.edit
```

### Alice

```text
wiki_editor on Wiki 42
```

Expected:

```text
EditArticle @ Wiki 42 -> ALLOW
EditArticle @ Wiki 43 -> DENY
EditArticle @ Article 991 -> DENY
```

unless an Article assignment was separately supplied.

### Bob

```text
article_owner on Article 991
```

Expected:

```text
EditArticle @ Article 991 -> ALLOW
EditArticle @ Article 992 -> DENY
EditArticle @ Wiki 42 -> DENY
```

### Multi-target

Given:

```text
candidate targets:
    Article 991
    Wiki 42
```

Alice should allow through Wiki 42.

Bob should allow through Article 991.

Charlie should deny through both.

If the decision reports metadata, it should identify the actual matching role and target.

### Policy refresh

Replacing the policy should remain atomic.

Changing request-specific `Access` must require no policy replacement.

---

# Do Not Overfit the Current API

This repository exists specifically so we can learn from implementation.

If a current type or method exists only because of an earlier design assumption, change it.

However, keep the library small.

The preferred outcome is **fewer concepts with clearer names**, not a larger authorization framework.

When finished:

1. run formatting;
2. run the test suite;
3. run/check the examples;
4. summarize the meaningful API changes;
5. call out any design decision where you intentionally deviated from the approximate API above and explain why.
