use rust_but_verify::{Action, ActionId, Model, Scope, ScopedBy, TargetKey};

#[derive(Clone, Copy)]
struct Edit;

impl Action for Edit {
    fn action_id(&self) -> ActionId {
        ActionId::new("scheduling.edit")
    }
}

struct Child;

impl Scope for Child {
    const KIND: &'static str = "child";
}

struct Schedule;

impl Scope for Schedule {
    const KIND: &'static str = "schedule";
}

impl ScopedBy<Child> for Edit {}
impl ScopedBy<Schedule> for Edit {}

#[test]
fn consumer_declares_typed_actions_scopes_and_targets() {
    let child = Child::ident("123");
    let schedule = Schedule::ident(456_u64);

    assert_eq!(child.target_key(), &TargetKey::new("child", "123"));
    assert_eq!(schedule.target_key(), &TargetKey::new("schedule", "456"));

    let mut builder = Model::builder();
    builder.grant("parent", Edit);
    let model = builder.build().expect("typed action should build");

    assert!(model.role_grants(&"parent".into(), &ActionId::new("scheduling.edit")));
}
