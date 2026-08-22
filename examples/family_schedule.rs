use rust_but_verify::{
    Access, Action, ActionKey, Authorizer, Decision, Model, ModelError, RoleAssignment, RoleId,
    Scope, ScopeKey, ScopedBy,
};

mod scheduling {
    use super::{Action, ActionKey};

    #[derive(Clone, Copy)]
    pub struct Edit;

    pub const EDIT: Edit = Edit;

    impl Action for Edit {
        fn action_key(&self) -> ActionKey {
            ActionKey::new("scheduling", "edit")
        }
    }
}

struct Child;

impl Scope for Child {
    const NAMESPACE: &'static str = "child";
}

struct Schedule;

impl Scope for Schedule {
    const NAMESPACE: &'static str = "schedule";
}

impl ScopedBy<Child> for scheduling::Edit {}
impl ScopedBy<Schedule> for scheduling::Edit {}

fn model_from_runtime_data(policy: &[(&str, ActionKey)]) -> Result<Model, ModelError> {
    let mut builder = Model::builder();
    for (role, action) in policy {
        builder.grant(*role, action);
    }
    builder.build()
}

fn main() -> Result<(), ModelError> {
    let edit_key = ActionKey::new("scheduling", "edit");
    let initial_policy = [("parent", edit_key.clone())];
    let authz = Authorizer::new(model_from_runtime_data(&initial_policy)?);
    let access = Access::new([
        RoleAssignment::new("parent", ScopeKey::new("child", "123")),
        RoleAssignment::new("parent", ScopeKey::new("schedule", "456")),
        RoleAssignment::new("youth", ScopeKey::new("child", "123")),
    ]);

    assert_eq!(
        authz.can(&access, scheduling::EDIT, Child::ident("123")),
        Decision::Allow {
            via_role: RoleId::new("parent")
        }
    );
    assert!(
        authz
            .can(&access, scheduling::EDIT, Schedule::ident("456"))
            .is_allowed()
    );
    assert_eq!(
        authz.can(&access, scheduling::EDIT, Child::ident("wrong-child")),
        Decision::Deny
    );

    let mut invalid = Model::builder();
    invalid
        .validate_actions([edit_key.clone()])
        .grant("parent", ActionKey::new("scheduling", "edti"));
    assert!(matches!(
        authz.try_replace(invalid),
        Err(ModelError::UnknownAction { .. })
    ));
    assert!(
        authz
            .can(&access, scheduling::EDIT, Child::ident("123"))
            .is_allowed()
    );

    let next_policy = [("youth", edit_key)];
    authz.replace_model(model_from_runtime_data(&next_policy)?);
    assert_eq!(
        authz.can(&access, scheduling::EDIT, Child::ident("123")),
        Decision::Allow {
            via_role: RoleId::new("youth")
        }
    );
    assert_eq!(
        authz.can(&access, scheduling::EDIT, Schedule::ident("456")),
        Decision::Deny
    );

    Ok(())
}
