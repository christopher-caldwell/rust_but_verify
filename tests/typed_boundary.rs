use rust_but_verify::{Action, ActionId, ActionOn, Policy, TargetKey, TargetType};

#[derive(Clone, Copy)]
struct Edit;

impl Action for Edit {
    fn action_id(&self) -> ActionId {
        ActionId::new("scheduling.edit")
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

#[test]
fn consumer_declares_typed_actions_scopes_and_targets() {
    let child = Child::ident("123");
    let schedule = Schedule::ident(456_u64);

    assert_eq!(child.target_key(), &TargetKey::new("child", "123"));
    assert_eq!(schedule.target_key(), &TargetKey::new("schedule", "456"));

    let mut builder = Policy::builder();
    builder.allow("parent", Edit);
    let model = builder.build().expect("typed action should build");

    assert!(model.role_grants(&"parent".into(), &ActionId::new("scheduling.edit")));
}
