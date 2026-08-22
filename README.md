# RustButVerify

RustButVerify is an exploratory Rust authorization library for one question:

> Can any supplied role assignment perform this action within this exact scope?

Roles and grants are runtime data. Rust types make application call sites more
discoverable and prevent checking an action against a scope type that the
application has not declared compatible.

```rust
use rust_but_verify::{
    Access, Action, ActionId, Authorizer, Model, Scope, ScopedBy,
};

mod scheduling {
    use rust_but_verify::{Action, ActionId};

    #[derive(Clone, Copy)]
    pub struct Edit;

    pub const EDIT: Edit = Edit;

    impl Action for Edit {
        fn action_id(&self) -> ActionId {
            ActionId::new("scheduling.edit")
        }
    }
}

struct Show;

impl Scope for Show {
    const KIND: &'static str = "show";
}

impl ScopedBy<Show> for scheduling::Edit {}

let runtime_policy = [("show_producer", ActionId::new("scheduling.edit"))];
let mut builder = Model::builder();
for (role, action) in runtime_policy {
    builder.grant(role, action);
}

let authz = Authorizer::new(builder.build()?);
let access = Access::with_role("show_producer", Show::ident("midnight-signal"));

assert!(authz
    .can(&access, scheduling::EDIT, Show::ident("midnight-signal"))
    .is_allowed());
# Ok::<(), rust_but_verify::ModelError>(())
```

Read the examples in order. Each one adds one layer to the previous model:

1. `examples/01_internal_knowledge_base.rs` — **Role**: a user receives a role
   and the role grants article permissions. It uses macro declarations.
2. `examples/02_community_server.rs` — **Scope**: a user's role applies only in
   the named community.
3. `examples/03_deployment_platform.rs` — **Relationships**: users and service
   accounts act on projects and environments after the application establishes
   their relationship.
4. `examples/04_source_control_platform.rs` — **Composition**: team-derived
   access, direct grants, parent resources, authorship, and restricted
   sub-resources combine without granting more than intended.
5. `examples/05_internal_knowledge_base_without_macros.rs` — **Manual
   declarations**: functionally identical to Example 1, but intentionally
   written without `define_authorization!`.

See [`docs/example-api-friction.md`](docs/example-api-friction.md) for the API
friction uncovered while writing these examples.

## Boundary

The consuming application owns identity, role-assignment loading, persistence,
configuration loading, and domain relationships. For example, the application
must establish that a schedule belongs to a child before choosing a child scope
for an authorization check. RustButVerify does not inspect domain objects,
traverse relationships, infer ownership, or provide a policy language.

Runtime checks use exact `target_kind::ident` equality and default to deny.
An action ID is one opaque consumer-owned string, such as `"article.edit"`.
`Article::ident(1)` and `Wiki::ident(1)` remain distinct targets; idents may be
numeric, slugs, UUIDs, or composite values. The application establishes all
domain relationships before selecting a target to check.
Unknown roles and actions do not grant access. An optional known-action set can
reject misspelled runtime action keys while building a model.

## Model refresh

Build and validate a complete candidate model before installing it. Calling
`Authorizer::replace_model` swaps an already-built model atomically;
`Authorizer::try_replace` builds a candidate and leaves the active model intact
if validation fails.

## Declarations

Use `define_authorization!` for routine declarations, as Examples 1 through 4
do. For an equivalent non-macro declaration, see Example 5: it implements
`Scope`, `Action`, and `ScopedBy` directly while preserving the same typed
checks and behavior as Example 1.

The macro keeps routine action, target, and compatibility declarations concise.
The manual form remains ordinary Rust when a consumer prefers it. Runtime policy
loading also uses a small mutable builder loop so runtime permissions remain
visible and data-driven.

The public API is unstable and breaking changes are expected.
