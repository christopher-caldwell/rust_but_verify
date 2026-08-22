use rust_but_verify::{Action, ActionKey, Model, Scope, ScopeKey, ScopedBy};

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

struct Schedule;

impl Scope for Schedule {
    const NAMESPACE: &'static str = "schedule";
}

impl ScopedBy<Child> for Edit {}
impl ScopedBy<Schedule> for Edit {}

#[test]
fn consumer_declares_typed_actions_scopes_and_targets() {
    let child = Child::ident("123");
    let schedule = Schedule::ident(456_u64);

    assert_eq!(child.scope_key(), &ScopeKey::new("child", "123"));
    assert_eq!(schedule.scope_key(), &ScopeKey::new("schedule", "456"));

    let mut builder = Model::builder();
    builder.grant("parent", Edit);
    let model = builder.build().expect("typed action should build");

    assert!(model.role_grants(&"parent".into(), &ActionKey::new("scheduling", "edit")));
}
