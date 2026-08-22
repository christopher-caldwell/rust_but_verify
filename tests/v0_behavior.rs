use rust_but_verify::{
    Access, Action, ActionId, ActionOn, Authorizer, Decision, Policy, PolicyError, RoleAssignment,
    TargetKey, TargetType,
};

#[derive(Clone, Copy)]
struct Edit;

impl Action for Edit {
    fn action_id(&self) -> ActionId {
        ActionId::new("scheduling.edit")
    }
}

#[derive(Clone, Copy)]
struct Delete;

impl Action for Delete {
    fn action_id(&self) -> ActionId {
        ActionId::new("scheduling.delete")
    }
}

struct Child;

impl TargetType for Child {
    const KIND: &'static str = "child";
}

struct Schedule;

impl TargetType for Schedule {
    const KIND: &'static str = "schedule";
}

impl ActionOn<Child> for Edit {}
impl ActionOn<Schedule> for Edit {}
impl ActionOn<Child> for Delete {}

fn policy(permissions: &[(&str, ActionId)]) -> Result<Policy, PolicyError> {
    let mut builder = Policy::builder();
    for (role, action) in permissions {
        builder.allow(*role, action);
    }
    builder.build()
}

fn assignment(role: &str, namespace: &str, identifier: &str) -> RoleAssignment {
    RoleAssignment::from_key(role, TargetKey::new(namespace, identifier))
}

#[test]
fn minimum_v0_behavior_works_as_a_complete_consumer_flow() {
    let edit = ActionId::new("scheduling.edit");
    let authz = Authorizer::new(policy(&[("parent", edit.clone())]).unwrap());
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

    let mut invalid = Policy::builder();
    invalid
        .validate_actions([edit.clone()])
        .allow("parent", ActionId::new("scheduling.edti"));
    assert!(authz.try_replace(invalid).is_err());
    assert!(authz.can(&access, Edit, Child::ident("123")).is_allowed());

    authz.replace_policy(policy(&[("youth", edit)]).unwrap());
    assert_eq!(
        authz.can(&access, Edit, Child::ident("123")),
        Decision::Deny
    );
}
