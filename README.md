# RustButVerify

RustButVerify is an exploratory Rust authorization library for one question:

> Can any supplied role assignment perform this action on this exact target?

Roles and permissions are runtime data. Rust types make application call sites
more discoverable and prevent checking an action against a target type that the
application has not declared compatible.

```toml
[dependencies]
rust-but-verify = "0.1.0"
```

```rust
use rust_but_verify::{
    Access, Action, ActionId, ActionOn, Authorizer, Policy, TargetType,
};

#[derive(Clone, Copy)]
struct Edit;

impl Action for Edit {
    fn action_id(&self) -> ActionId {
        ActionId::new("scheduling.edit")
    }
}

struct Show;

impl TargetType for Show {
    const KIND: &'static str = "show";
}

impl ActionOn<Show> for Edit {}

let mut builder = Policy::builder();
builder.allow("show_producer", Edit);

let authz = Authorizer::new(builder.build()?);
let access = Access::with_role("show_producer", Show::ident("midnight-signal"));

if authz.can(&access, Edit, Show::ident("midnight-signal")) {
    // Allow the edit.
}
# Ok::<(), rust_but_verify::PolicyError>(())
```

## Authorization checks

Use `can` for the ordinary yes/no authorization question. It returns `bool`,
so it fits naturally in an `if` or assertion. When a caller also needs the
first successful authorization path, use `decide`; it returns
`Decision::Allow { via_role, via_target }` or `Decision::Deny`.

`Decision` is optional provenance, not an exhaustive explanation: if more than
one path could allow an action, it reports only the first successful path in
the API's evaluation order.

```rust
# use rust_but_verify::{Access, Action, ActionId, ActionOn, Authorizer, Decision, Policy, TargetType};
# #[derive(Clone, Copy)] struct Edit;
# impl Action for Edit { fn action_id(&self) -> ActionId { ActionId::new("show.edit") } }
# struct Show; impl TargetType for Show { const KIND: &'static str = "show"; }
# impl ActionOn<Show> for Edit {}
# let mut policy = Policy::builder(); policy.allow("show_producer", Edit);
# let authz = Authorizer::new(policy.build()?);
# let access = Access::with_role("show_producer", Show::ident("midnight-signal"));
match authz.decide(&access, Edit, Show::ident("midnight-signal")) {
    Decision::Allow { via_role, via_target } => println!("allowed by {via_role:?} at {via_target:?}"),
    Decision::Deny => println!("denied"),
}
# Ok::<(), rust_but_verify::PolicyError>(())
```

For a small “this OR that” check, `can_any` and `decide_any` accept an
exactly-two-target tuple. The tuple is evaluated left to right. They hold one
policy snapshot for the complete multi-target operation.

```rust
# use rust_but_verify::{Access, Action, ActionId, ActionOn, Authorizer, Policy, TargetType};
# #[derive(Clone, Copy)] struct Edit;
# impl Action for Edit { fn action_id(&self) -> ActionId { ActionId::new("show.edit") } }
# struct Show; impl TargetType for Show { const KIND: &'static str = "show"; }
# struct Collection; impl TargetType for Collection { const KIND: &'static str = "collection"; }
# impl ActionOn<Show> for Edit {}
# impl ActionOn<Collection> for Edit {}
# let authz = Authorizer::new(Policy::builder().build()?);
# let access = Access::empty();
if authz.can_any(&access, Edit, (Show::ident("midnight-signal"), Collection::ident("drama"))) {
    // Allow the edit.
}
# Ok::<(), rust_but_verify::PolicyError>(())
```

For zero, one, or more than two candidates, build `Targets<A>`. Each `add`
is checked at compile time for `ActionOn` compatibility; candidates retain
insertion order, are not deduplicated, and have no arbitrary maximum.
The application is responsible for supplying relevant targets (for example,
an article and its already-verified wiki); RustButVerify does not discover
domain relationships.

```rust
# use rust_but_verify::{Access, Action, ActionId, ActionOn, Authorizer, Policy, TargetType, Targets};
# #[derive(Clone, Copy)] struct Edit;
# impl Action for Edit { fn action_id(&self) -> ActionId { ActionId::new("show.edit") } }
# struct Show; impl TargetType for Show { const KIND: &'static str = "show"; }
# struct Collection; impl TargetType for Collection { const KIND: &'static str = "collection"; }
# impl ActionOn<Show> for Edit {}
# impl ActionOn<Collection> for Edit {}
# let mut policy = Policy::builder(); policy.allow("collection_editor", Edit);
# let authz = Authorizer::new(policy.build()?);
# let access = Access::with_role("collection_editor", Collection::ident("drama"));
let targets = Targets::<Edit>::new()
    .add(Show::ident("midnight-signal"))
    .add(Collection::ident("drama"));
assert!(authz.can_any(&access, Edit, targets));
# Ok::<(), rust_but_verify::PolicyError>(())
```

## Examples

The `examples/` directory is a separate, non-published Cargo package containing
standalone consumer binaries. Its domains are small tutorials: each successive
step keeps the application model familiar and introduces only the next
authorization concept.

```bash
cd examples
just example knowledge_base 01
```

The first domain is a Knowledge Base ramp:

1. direct Article access with one exact role assignment;
2. application-verified ownership supplied as an access fact;
3. Wiki-wide editing with `can_any`; and
4. ownership-derived and Wiki-wide paths together.

`define_authorization!` is optional convenience syntax. The
[`examples/without_macros/`](examples/without_macros/) reference binaries use
ordinary `TargetType`, `Action`, and `ActionOn` implementations while making
the same authorization calls. Run them with, for example,
`just example without_macros 01`.

## Policy and Access

`Policy` is long-lived global role-to-action configuration: `wiki_editor`
means the same thing wherever it is assigned. `Access` is a caller-supplied
collection of facts for one authorization context. Durable membership and
request-derived facts—such as a caller-verified article owner—can be combined
without rebuilding Policy.

## Boundary

The consuming application owns identity, role-assignment loading, persistence,
configuration loading, and domain relationships. It must establish that an
article belongs to a wiki or that a caller owns an article before choosing the
target or access fact to check. RustButVerify does not inspect domain objects,
traverse relationships, infer ownership, or provide a policy language.

It also provides no persistence adapters, wildcard matching, ABAC predicates,
framework integration, or identity management. Runtime checks use exact
`target_kind::ident` equality and default to deny. An action ID is one opaque
consumer-owned string, such as `"article.edit"`.

Unknown roles and actions do not allow access. An optional known-action set can
reject misspelled runtime action keys while building a policy.

## Policy refresh

Build and validate a complete candidate policy before installing it. Calling
`Authorizer::replace_policy` swaps an already-built policy atomically;
`Authorizer::try_replace` builds a candidate and leaves the active policy intact
if validation fails.

## Declarations

Use `define_authorization!` for routine declarations. It keeps action, target,
and compatibility declarations concise, while the manual form remains ordinary
Rust for consumers who prefer it. Runtime policy loading remains explicitly
data-driven through `Policy::builder`.

## Benchmarks

Run the baseline suite with `just bench`. The scenarios are documented in
[`benches/README.md`](benches/README.md); they measure the current implementation
without adding indexes or changing authorization behavior.

See [`docs/example-api-friction.md`](docs/example-api-friction.md) for the API
friction exposed while writing the tutorials. The public API is unstable and
breaking changes are expected.
