use rust_but_verify::{
    Access, Action, ActionId, ActionOn, Authorizer, Decision, Policy, RoleAssignment, TargetKey,
    TargetType,
};

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

impl ActionOn<Child> for Edit {}

struct Article;

impl TargetType for Article {
    const KIND: &'static str = "article";
}

struct Wiki;

impl TargetType for Wiki {
    const KIND: &'static str = "wiki";
}

impl ActionOn<Article> for Edit {}
impl ActionOn<Wiki> for Edit {}

#[test]
fn consumer_can_authorize_with_typed_values() {
    let mut builder = Policy::builder();
    builder.allow("parent", Edit);
    let model = builder.build().expect("model should be valid");
    let authorizer = Authorizer::new(model);
    let assignment = RoleAssignment::from_key("parent", TargetKey::new("child", "123"));
    let access = Access::new([assignment]);

    assert_eq!(
        authorizer.can(&access, Edit, Child::ident("123")),
        Decision::Allow {
            via_role: "parent".into()
        }
    );
}

#[test]
fn typed_access_keeps_target_kinds_distinct_for_equal_idents() {
    let mut builder = Policy::builder();
    builder.allow("editor", Edit);
    let authorizer = Authorizer::new(builder.build().expect("model should be valid"));

    let article_access = Access::with_role("editor", Article::ident(1));

    assert!(
        authorizer
            .can(&article_access, Edit, Article::ident(1))
            .is_allowed()
    );
    assert_eq!(
        authorizer.can(&article_access, Edit, Wiki::ident(1)),
        Decision::Deny
    );
}

#[test]
fn target_idents_accept_slug_uuid_and_composite_values() {
    assert_eq!(
        Child::ident("engineering-handbook")
            .target_key()
            .identifier(),
        "engineering-handbook"
    );
    assert_eq!(
        Child::ident("550e8400-e29b-41d4-a716-446655440000")
            .target_key()
            .identifier(),
        "550e8400-e29b-41d4-a716-446655440000"
    );
    assert_eq!(
        Child::ident("acme:payments-api:production")
            .target_key()
            .identifier(),
        "acme:payments-api:production"
    );
}
