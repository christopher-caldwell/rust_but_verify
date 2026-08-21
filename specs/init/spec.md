# RustButVerify — Functional PRD

**Status:** Draft / exploratory implementation
**Version:** v0
**Stability:** Unstable by design
**Primary goal:** Build the smallest functional version of the authorization model, use it ourselves, then iterate based on real DX and usage.

---

## 1. Purpose

RustButVerify is a small, Rust-native authorization library designed to answer one narrow question:

> **Can a role perform an action within a given scope?**

The project is intentionally narrower than general-purpose policy engines such as Cedar or Casbin.

The first implementation should prioritize:

* simple and predictable semantics;
* runtime-configurable roles and grants;
* application-controlled configuration and persistence;
* compile-time assistance where Rust can provide meaningful safety;
* a small runtime model;
* clear developer experience;
* easy iteration.

The goal of v0 is **not** to design the final API.

The goal is to implement the core behavior, exercise it in real Rust code, observe friction, and refine the API from evidence.

Breaking changes during this stage are expected.

---

# 2. Core Mental Model

RustButVerify models three primary relationships:

```text
Role → Action

Role Assignment → Scope

Authorization query:
Does any supplied role assignment permit this action
within this scope?
```

Example:

```text
Role:
parent

Action:
scheduling.edit

Scope:
child::123
```

The engine answers:

```text
Can `parent` perform `scheduling.edit`
within `child::123`?
```

The engine does not know what a parent, child, or schedule means.

---

# 3. Terminology

## Role

A runtime-defined identifier representing a grouping of allowed actions.

Examples:

```text
parent
youth
coach
administrator
```

Roles are policy data and must not require application recompilation.

---

## Action

An application capability that can be granted to roles.

Examples:

```text
scheduling.view
scheduling.edit
profile.view
billing.refund
```

Actions have stable runtime identities.

Application code should eventually be able to refer to actions through Rust values such as:

```rust
scheduling::EDIT
```

rather than repeatedly passing raw strings.

---

## Scope

A namespaced opaque identifier describing **where an authorization fact applies**.

Examples:

```text
schedule::456
child::123
organization::abc
```

A scope is not necessarily the object being acted upon.

For example, editing a schedule might legitimately be authorized by either:

```text
schedule::456
```

or:

```text
child::123
```

if the application has established that the child scope is a valid authorization path for that operation.

RustButVerify does not discover or validate that relationship.

---

## Role Assignment

A fact stating that a role applies within a particular scope.

Example:

```text
parent @ child::123
```

or:

```text
manager @ organization::abc
```

---

# 4. Functional Boundary

RustButVerify answers only:

> **Given supplied role assignments, does one of those roles grant this action within this exact scope?**

It does **not** answer:

```text
Does schedule 456 belong to child 123?

Is this user actually the parent of child 123?

Is this account suspended?

Is this user old enough?

Is the expense less than $5,000?

Does this document belong to the user?
```

Those are consuming-application concerns.

The caller is responsible for supplying truthful and relevant authorization facts.

---

# 5. Roles Are Runtime Data

Roles must be dynamically configurable.

Adding:

```text
parent
```

must not require recompiling or redeploying the application.

A policy model may contain:

```text
parent:
    scheduling.view
    scheduling.edit

youth:
    scheduling.view
```

The application may load this configuration from anywhere.

RustButVerify must not require any particular source such as:

* SQL;
* JSON;
* YAML;
* TOML;
* Redis;
* HTTP;
* an admin interface;
* hardcoded Rust configuration.

The library begins at the point where the application has normalized its data into values suitable for model construction.

---

# 6. Actions

Actions represent application-defined capabilities.

Conceptually, an action has:

```text
namespace + name
```

For example:

```text
namespace: scheduling
name: edit
```

which may be represented externally as:

```text
scheduling.edit
```

The library should expose a normalized runtime action identifier similar in concept to:

```rust
ActionKey::new("scheduling", "edit")
```

The exact type names and API are not fixed.

---

# 7. Typed Actions

Application code should be able to define typed action values.

Approximate desired usage:

```rust
authz.can(
    &access,
    scheduling::EDIT,
    Child::ident("123"),
)
```

The typed action should know its runtime `ActionKey`.

Its primary purpose is:

* discoverability;
* autocomplete;
* avoiding repeated raw strings;
* enabling compile-time action/scope compatibility checks.

The exact declaration syntax is still TBD.

---

# 8. Scopes

A scope consists of:

```text
namespace + opaque identifier
```

Examples:

```text
child::123
schedule::456
organization::abc
```

The identifier must remain opaque to RustButVerify.

The engine must never inspect or interpret it.

---

# 9. Application-Defined Scope Types

Applications should be able to define Rust types corresponding to scope namespaces.

Conceptually:

```rust
struct Child;
struct Schedule;
struct Organization;
```

with declarations similar to:

```rust
impl Scope for Child {
    const NAMESPACE: &'static str = "child";
}
```

The exact declaration API is still subject to iteration.

Application usage should be able to look approximately like:

```rust
Child::ident("123")
Schedule::ident("456")
```

which produce typed values that eventually erase to runtime values equivalent to:

```text
child::123
schedule::456
```

---

# 10. Identifier Types Must Remain Consumer-Controlled

RustButVerify must not force applications into typed IDs.

Applications should be able to use ordinary identifiers such as:

```rust
Child::ident("123")
Child::ident(uuid)
Child::ident(123_u64)
```

Applications that already define domain-specific IDs such as:

```rust
struct ChildId(Uuid);
```

should be able to integrate them naturally and optionally gain additional compile-time safety.

Typed IDs may be demonstrated in documentation as a useful pattern.

They must not be required by the library.

---

# 11. Actions May Support Multiple Scope Types

An action may be valid within more than one application-defined scope type.

Example:

```text
scheduling.edit
```

may be authorized within:

```text
Schedule
Child
Organization
```

Conceptually:

```rust
impl ScopedBy<Schedule> for Edit {}
impl ScopedBy<Child> for Edit {}
impl ScopedBy<Organization> for Edit {}
```

The exact API is TBD.

---

# 12. Compile-Time Scope Compatibility

Where practical, Rust's type system should prevent checking an action against a scope type the application has not declared as valid.

These should compile:

```rust
authz.can(
    &access,
    scheduling::EDIT,
    Schedule::ident("456"),
);
```

```rust
authz.can(
    &access,
    scheduling::EDIT,
    Child::ident("123"),
);
```

If `Document` has not been declared as a valid scope for `scheduling::EDIT`, this should ideally fail at compile time:

```rust
authz.can(
    &access,
    scheduling::EDIT,
    Document::ident("789"),
);
```

This is an application-facing safety mechanism only.

The runtime evaluator should not operate on these Rust types.

---

# 13. Multiple Paths to Authorization

The same action may succeed through different scopes.

Example access facts:

```text
parent @ schedule::456

parent @ child::123
```

Both may legitimately authorize:

```text
scheduling.edit
```

if the application has declared both `Schedule` and `Child` as valid scopes for that action.

The caller may therefore choose the scope representing the authorization path it has established.

For example:

```rust
authz.can(
    &access,
    scheduling::EDIT,
    Schedule::ident("456"),
);
```

or:

```rust
authz.can(
    &access,
    scheduling::EDIT,
    Child::ident("123"),
);
```

A future convenience API may support supplying several candidate scopes and succeeding if any authorize the action.

That convenience is **not required for v0**.

---

# 14. Relationship Resolution Is Outside the Library

Suppose:

```text
schedule::456 belongs to child::123
```

RustButVerify does not know that.

The host application may establish that relationship through its own domain logic and then authorize using:

```rust
Child::ident("123")
```

If the caller supplies the wrong child ID, that is an application bug.

RustButVerify must not attempt to verify entity relationships.

Doing so would move the library toward ReBAC/domain-policy evaluation, which is outside the intended boundary.

---

# 15. Role Assignments

The engine must be able to evaluate supplied role assignments such as:

```text
parent @ child::123
parent @ child::456
coach @ organization::abc
```

The consuming application owns:

* loading assignments;
* deriving assignments;
* storing assignments;
* deciding which assignments to provide for a request.

RustButVerify only evaluates the supplied facts.

The exact `Access` or role-assignment API is still TBD.

---

# 16. Core Authorization Rule

The initial authorization semantics are intentionally fixed.

Conceptually:

```text
for each supplied role assignment:
    if assignment.scope == requested_scope:
        if model says assignment.role grants requested_action:
            ALLOW

DENY
```

Important properties:

* exact scope matching;
* no predicate evaluation;
* no custom matcher expressions;
* no policy ordering;
* no resource inspection;
* no relationship traversal;
* default deny.

The fixed nature of this algorithm is intentional.

RustButVerify is not a configurable authorization-language engine.

---

# 17. Default Deny

If no supplied authorization path grants the requested action:

```text
DENY
```

Examples include:

* role does not exist;
* role exists but lacks action;
* scope does not match;
* no applicable role assignment exists;
* runtime action key typo causes no matching grant.

The library should fail closed.

---

# 18. Model Construction

RustButVerify must provide helpers for building its normalized policy model without owning the configuration source.

Desired flow:

```text
Consumer-controlled data
        ↓
Consumer-controlled loading/parsing
        ↓
RustButVerify model construction API
        ↓
Validation
        ↓
Immutable runtime model
```

The model builder should support both:

### Typed construction

Approximate example:

```rust
Model::builder()
    .role("parent")
        .grant(scheduling::VIEW)
        .grant(scheduling::EDIT)
    .build()
```

### Runtime-key construction

Approximate example:

```rust
builder.grant(
    RoleId::new("parent"),
    ActionKey::new("scheduling", "edit"),
);
```

This second path is important for applications loading arbitrary runtime configuration.

Exact builder syntax is TBD.

---

# 19. Core Model Should Not Require Duplicate Configuration

The default model-building path must not require applications to:

1. define an action in Rust;
2. register the same action in a catalog;
3. reference it again in runtime policy.

Runtime action keys must be usable directly when constructing the model.

The core engine can operate without knowing whether every action has a corresponding typed Rust declaration.

---

# 20. Optional Action Catalog

Consumers may optionally define a catalog or similar structure containing known actions.

Purpose:

> Validate that runtime action identifiers correspond to actions the consuming application considers valid.

Example mistake:

```text
scheduling.edti
```

Without catalog validation, this may be accepted as a runtime key but will simply fail to match the application's actual `scheduling.edit` action.

This naturally fails closed.

With optional validation enabled, model construction could reject the unknown action.

The existence of this optional capability is decided.

The following remain deliberately TBD:

* catalog representation;
* registration mechanism;
* whether it is called a catalog;
* validation API;
* whether action declarations can be discovered/composed automatically;
* whether macros eventually improve declaration ergonomics.

Do not prematurely hard-code these choices in v0 unless necessary.

---

# 21. Typed API vs Runtime Model

Compile-time types should exist primarily at the application boundary.

For example:

```rust
scheduling::EDIT
```

may statically express:

```text
valid with Child
valid with Schedule
valid with Organization
```

When passed into the engine it should erase to a simple runtime value such as:

```rust
ActionKey {
    namespace: "scheduling",
    action: "edit",
}
```

Similarly:

```rust
Child::ident("123")
```

should erase to something conceptually like:

```rust
ScopeKey {
    namespace: "child",
    id: "123",
}
```

The evaluator should operate on ordinary runtime data structures.

Avoid allowing public generics to infect every internal model type.

---

# 22. Model Refresh

Roles and action grants must be replaceable at runtime.

Conceptually:

```rust
let next = build_model_from_my_config()?;

authorizer.replace_model(next);
```

RustButVerify does not decide:

* when refresh occurs;
* how configuration changes are detected;
* whether the application polls;
* whether it uses pub/sub;
* where data comes from.

The library only provides the mechanism to replace the active model.

---

# 23. Replacement Semantics

Model replacement should be all-or-nothing.

Conceptually:

```text
current model
      ↓

build candidate
      ↓

validate candidate
      ↓

success:
    replace current model

failure:
    retain current model
```

Authorization must not observe a partially constructed policy model.

The exact synchronization primitive is an implementation detail.

Do not over-design distributed refresh or synchronization in v0.

---

# 24. Decision Results

The library must expose an allow/deny answer.

At minimum:

```text
Allow
Deny
```

It may also be useful to return lightweight explanation metadata such as:

```rust
Decision::Allow {
    via_role: RoleId,
}
```

The exact result API is not yet stable.

A convenience enforcement method such as:

```rust
authz.require(...)
```

may also exist.

It should return a library-specific authorization error rather than knowledge of HTTP status codes or frameworks.

---

# 25. Developer Experience

DX is a first-class requirement.

The project should actively prefer APIs that are:

* easy to read;
* easy to discover through autocomplete;
* hard to misuse accidentally;
* understandable without advanced type-level Rust knowledge;
* concise at common authorization call sites.

A target usage experience is:

```rust
authz.can(
    &access,
    scheduling::EDIT,
    Child::ident("123"),
)
```

A developer who understands normal Rust traits and generics should be able to understand the library from examples.

Advanced generic machinery may exist internally if useful.

It should not become the conceptual model users must understand.

---

# 26. Declaration Ergonomics

Initial declarations may contain some explicit boilerplate.

For example:

```rust
impl ScopedBy<Child> for Edit {}
impl ScopedBy<Schedule> for Edit {}
```

Do not add macros merely to reduce hypothetical boilerplate.

First implement the underlying API and use it.

If repeated declarations become meaningfully unpleasant, introduce convenience helpers based on real usage.

DX matters strongly, but premature abstraction should be avoided.

---

# 27. Explicit Non-Goals

The initial RustButVerify implementation will not provide:

* ABAC;
* ReBAC traversal;
* arbitrary conditions;
* resource attribute evaluation;
* subject attribute evaluation;
* expression languages;
* custom matcher languages;
* ownership inference;
* hierarchy traversal;
* relationship discovery;
* explicit domain object inspection;
* account-state rules;
* age rules;
* monetary threshold rules;
* wildcard resources;
* wildcard scopes;
* policy scripting;
* web-framework middleware;
* HTTP response handling;
* database adapters;
* persistence adapters;
* remote configuration clients;
* distributed synchronization;
* role-assignment loading;
* resource loading.

If a proposed feature requires RustButVerify to understand what a domain object **means**, it is probably outside the v0 boundary.

---

# 28. Intentional Limitations

The following are currently considered acceptable limitations rather than bugs:

### Exact scope matching

RustButVerify does not infer broader/narrower scopes.

---

### Caller-controlled relationship reasoning

The host application decides that:

```text
child::123
```

is a valid authorization scope for the operation it is performing.

---

### Unknown runtime actions without a catalog

If optional strict action validation is not enabled, unknown action identifiers may exist in the model and simply never match.

This fails closed.

---

### API instability

Names such as:

```text
Action
Scope
ScopedBy
Access
Model
ActionKey
ScopeKey
Authorizer
```

are working terminology, not stable API commitments.

---

# 29. Draft-Stage Implementation Guidance

This project is currently in an exploratory stage.

Implementation should favor:

> **functional and understandable over comprehensive and polished**

Do not attempt to finalize every abstraction before the first usable version exists.

The implementation model should expect:

* public APIs to change;
* type names to change;
* builder APIs to change;
* declaration syntax to change;
* internal structures to change;
* convenience APIs to be added or removed.

Avoid building compatibility layers for APIs that have not yet stabilized.

---

# 30. Testing Expectations for v0

Testing should be intentionally light but sufficient to protect the central semantics.

We are still iterating on the API, so avoid large test suites that tightly encode provisional syntax.

Prioritize small behavioral tests around the authorization engine.

At minimum, test:

### Grants succeed

```text
parent grants scheduling.edit
parent assigned to child::123
query scheduling.edit @ child::123
→ allow
```

### Missing action denies

```text
parent does not grant scheduling.delete
→ deny
```

### Wrong scope denies

```text
parent assigned to child::123
query child::456
→ deny
```

### Unknown role denies

```text
role does not exist in model
→ deny
```

### Multiple assignments

Authorization succeeds if any supplied matching role assignment grants the action.

### Model replacement

After installing a new valid model, subsequent authorization calls use the new grants.

### Failed candidate model

If model construction/validation fails, the previous active model remains usable.

### Compile-time scope compatibility

Add minimal compile-fail testing only if convenient.

Do not invest heavily in compile-fail infrastructure until the typed API shape is more stable.

---

# 31. Testing Philosophy

Tests at this stage should protect **behavior**, not provisional API aesthetics.

Prefer testing:

```text
role + action + matching scope = allow
```

over locking down the exact builder syntax.

Avoid exhaustive edge-case testing until the underlying public model has survived real usage.

The purpose of the initial tests is to make iteration safe, not to create an early maintenance burden.

---

# 32. Initial Success Criteria

The first implementation is successful if we can use RustButVerify in a real Rust application to:

1. define application actions;
2. define application scope namespaces;
3. constrain actions to valid scope types;
4. load roles and action grants from arbitrary application-controlled data;
5. construct a normalized model without adopting a RustButVerify-specific config file format;
6. supply scoped role assignments from application-controlled data;
7. ask whether an action is allowed within an exact namespaced scope;
8. receive deterministic allow/deny results;
9. support multiple valid scope types for the same action;
10. replace the policy model at runtime;
11. optionally validate runtime action names against a consumer-defined known-action set;
12. do all of this without introducing a policy language;
13. find the common API pleasant enough that we would choose to keep using it ourselves.

---

# 33. Suggested First Implementation Boundary

The initial implementation should be kept as small as reasonably possible.

A functional v0 likely needs concepts approximately equivalent to:

```text
RoleId

ActionKey
Action

ScopeKey
Scope
Target

RoleAssignment / Access

Model
ModelBuilder

Authorizer

Decision
Error
```

This list is illustrative, not an API mandate.

If a concept is not required to demonstrate the core authorization flow, defer it.

---

# 34. Questions Deliberately Left Open

The implementation should not treat these as settled unless required to make progress:

* exact action declaration syntax;
* exact scope declaration syntax;
* whether action constants or zero-sized types provide the best DX;
* exact `ScopedBy` mechanism;
* exact `Access` API;
* exact model builder chaining syntax;
* exact decision result shape;
* exact error types;
* catalog shape and registration API;
* whether multiple candidate scopes should have a convenience API;
* whether macros eventually improve declaration ergonomics;
* exact concurrency primitive used for model replacement.

When choosing a temporary answer, prefer the simplest implementation that preserves the functional boundary.

---

# 35. Iteration Strategy

Expected workflow:

```text
Build minimal core
      ↓
Exercise through examples
      ↓
Use in a real consuming application
      ↓
Identify awkward API surfaces
      ↓
Refine
      ↓
Only then consider stabilization
```

The project should resist solving hypothetical future requirements.

If real-world usage reveals a limitation, evaluate whether it fits the existing mental model before extending the engine.

Do not gradually evolve RustButVerify into a generic policy engine merely because broader authorization features are possible.

---

# 36. Core Product Statement

RustButVerify is:

> **Runtime-configurable RBAC with exact namespaced scopes and Rust-native compile-time assistance for valid action/scope combinations.**

Its value comes from keeping the authorization model deliberately fixed and understandable while using Rust's type system to improve the consuming application's developer experience.

The first version only needs to prove that this narrow model is both useful and pleasant to use.
