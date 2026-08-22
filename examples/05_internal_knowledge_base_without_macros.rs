//! Example 5: Example 1 with ordinary Rust declarations instead of the macro.

use rust_but_verify::{Access, Action, ActionId, ActionOn, Authorizer, Policy, TargetType};

#[derive(Clone, Copy)]
struct EditArticle;
impl Action for EditArticle {
    fn action_id(&self) -> ActionId {
        ActionId::new("article.edit")
    }
}
struct Article;
impl TargetType for Article {
    const KIND: &'static str = "article";
}
struct Wiki;
impl TargetType for Wiki {
    const KIND: &'static str = "wiki";
}
impl ActionOn<Article> for EditArticle {}
impl ActionOn<Wiki> for EditArticle {}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();
    policy
        .allow("wiki_editor", EditArticle)
        .allow("article_owner", EditArticle);
    let authz = Authorizer::new(policy.build()?);
    let alice = Access::with_role("wiki_editor", Wiki::ident(42));
    let bob_id = 7;
    let article_owner_id = 7;
    let mut bob = Access::empty();
    if bob_id == article_owner_id {
        bob.add_role("article_owner", Article::ident(991));
    }
    let charlie = Access::empty();
    assert!(
        authz
            .can_any(&alice, EditArticle, (Article::ident(991), Wiki::ident(42)))
            .is_allowed()
    );
    assert!(
        authz
            .can_any(&bob, EditArticle, (Article::ident(991), Wiki::ident(42)))
            .is_allowed()
    );
    assert!(
        !authz
            .can_any(
                &charlie,
                EditArticle,
                (Article::ident(991), Wiki::ident(42))
            )
            .is_allowed()
    );
    Ok(())
}
