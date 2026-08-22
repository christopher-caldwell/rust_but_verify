use rust_but_verify::{
    Access, Action, ActionKey, Authorizer, Decision, Model, RoleAssignment, Scope, ScopeKey,
    ScopedBy,
};

#[derive(Clone, Copy)]
struct Edit;

impl Action for Edit {
    fn action_key(&self) -> ActionKey {
        ActionKey::new("scheduling", "edit")
    }
}

struct Child;

impl Scope for Child {
    const NAMESPACE: &'static str = "child";
}

impl ScopedBy<Child> for Edit {}

#[test]
fn consumer_can_authorize_with_typed_values() {
    let mut builder = Model::builder();
    builder.grant("parent", Edit);
    let model = builder.build().expect("model should be valid");
    let authorizer = Authorizer::new(model);
    let assignment = RoleAssignment::new("parent", ScopeKey::new("child", "123"));
    let access = Access::new([assignment]);

    assert_eq!(
        authorizer.can(&access, Edit, Child::ident("123")),
        Decision::Allow {
            via_role: "parent".into()
        }
    );
}
