# RustButVerify

RustButVerify is an exploratory Rust authorization library for one question:

> Can any supplied role assignment perform this action within this exact scope?

Roles and grants are runtime data. Rust types make application call sites more
discoverable and prevent checking an action against a scope type that the
application has not declared compatible.

```rust
use rust_but_verify::{
    Access, Action, ActionKey, Authorizer, Model, RoleAssignment, Scope,
    ScopeKey, ScopedBy,
};

mod scheduling {
    use rust_but_verify::{Action, ActionKey};

    #[derive(Clone, Copy)]
    pub struct Edit;

    pub const EDIT: Edit = Edit;

    impl Action for Edit {
        fn action_key(&self) -> ActionKey {
            ActionKey::new("scheduling", "edit")
        }
    }
}

struct Show;

impl Scope for Show {
    const NAMESPACE: &'static str = "show";
}

impl ScopedBy<Show> for scheduling::Edit {}

let runtime_policy = [("show_producer", ActionKey::new("scheduling", "edit"))];
let mut builder = Model::builder();
for (role, action) in runtime_policy {
    builder.grant(role, action);
}

let authz = Authorizer::new(builder.build()?);
let access = Access::new([RoleAssignment::new(
    "show_producer",
    ScopeKey::new("show", "midnight-signal"),
)]);

assert!(authz
    .can(&access, scheduling::EDIT, Show::ident("midnight-signal"))
    .is_allowed());
# Ok::<(), rust_but_verify::ModelError>(())
```

Read the examples in order. Each one adds one layer to the previous model:

1. `examples/internal_knowledge_base.rs` — **Role**: a user receives a role
   and the role grants article permissions.
2. `examples/community_server.rs` — **Scope**: a user's role applies only in
   the named community.
3. `examples/deployment_platform.rs` — **Relationships**: users and service
   accounts act on projects and environments after the application establishes
   their relationship.
4. `examples/source_control_platform.rs` — **Composition**: team-derived
   access, direct grants, parent resources, authorship, and restricted
   sub-resources combine without granting more than intended.

See [`docs/example-api-friction.md`](docs/example-api-friction.md) for the API
friction uncovered while writing these examples.

## Boundary

The consuming application owns identity, role-assignment loading, persistence,
configuration loading, and domain relationships. For example, the application
must establish that a schedule belongs to a child before choosing a child scope
for an authorization check. RustButVerify does not inspect domain objects,
traverse relationships, infer ownership, or provide a policy language.

Runtime checks use exact `namespace::identifier` equality and default to deny.
Unknown roles and actions do not grant access. An optional known-action set can
reject misspelled runtime action keys while building a model.

## Model refresh

Build and validate a complete candidate model before installing it. Calling
`Authorizer::replace_model` swaps an already-built model atomically;
`Authorizer::try_replace` builds a candidate and leaves the active model intact
if validation fails.

## Current v0 friction

Action types, scope types, and `ScopedBy` implementations are deliberately
explicit. Runtime policy loading also uses a small mutable builder loop. This is
some boilerplate, but the concepts remain visible and ordinary Rust tooling can
navigate them. The v0 intentionally avoids macros or compatibility helpers
until repeated consumer use shows which declarations are genuinely awkward.

The public API is unstable and breaking changes are expected.
