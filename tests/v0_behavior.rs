use rust_but_verify::{
    Access, Action, ActionKey, Authorizer, Decision, Model, ModelError, RoleAssignment, Scope,
    ScopeKey, ScopedBy,
};

#[derive(Clone, Copy)]
struct Edit;

impl Action for Edit {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("scheduling", "edit")
    }
}

#[derive(Clone, Copy)]
struct Delete;

impl Action for Delete {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("scheduling", "delete")
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

impl ScopedBy<Child> for Edit {}
impl ScopedBy<Schedule> for Edit {}
impl ScopedBy<Child> for Delete {}

fn model(policy: &[(&str, ActionKey)]) -> Result<Model, ModelError> {
    let mut builder = Model::builder();
    for (role, action) in policy {
        builder.grant(*role, action);
    }
    builder.build()
}

fn assignment(role: &str, namespace: &str, identifier: &str) -> RoleAssignment {
    RoleAssignment::new(role, ScopeKey::new(namespace, identifier))
}

#[test]
fn minimum_v0_behavior_works_as_a_complete_consumer_flow() {
    let edit = ActionKey::new("scheduling", "edit");
    let authz = Authorizer::new(model(&[("parent", edit.clone())]).unwrap());
    let access = Access::new([
        assignment("unknown", "child", "123"),
        assignment("parent", "schedule", "456"),
        assignment("parent", "child", "123"),
    ]);

    assert!(authz.can(&access, Edit, Child::ident("123")).is_allowed());
    assert!(
        authz
            .can(&access, Edit, Schedule::ident("456"))
            .is_allowed()
    );
    assert_eq!(
        authz.can(&access, Delete, Child::ident("123")),
        Decision::Deny
    );
    assert_eq!(
        authz.can(&access, Edit, Child::ident("456")),
        Decision::Deny
    );
    assert_eq!(
        authz.can(
            &Access::new([assignment("unknown", "child", "123")]),
            Edit,
            Child::ident("123")
        ),
        Decision::Deny
    );

    let mut invalid = Model::builder();
    invalid
        .validate_actions([edit.clone()])
        .grant("parent", ActionKey::new("scheduling", "edti"));
    assert!(authz.try_replace(invalid).is_err());
    assert!(authz.can(&access, Edit, Child::ident("123")).is_allowed());

    authz.replace_model(model(&[("youth", edit)]).unwrap());
    assert_eq!(
        authz.can(&access, Edit, Child::ident("123")),
        Decision::Deny
    );
}
