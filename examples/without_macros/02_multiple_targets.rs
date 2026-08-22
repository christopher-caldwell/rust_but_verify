//! The ordinary-Rust equivalent of an action declared for Article and Wiki.

use rust_but_verify::{Access, Action, ActionId, ActionOn, Authorizer, Policy, TargetType};

#[derive(Clone, Copy)]
struct EditArticle;

impl Action for EditArticle {
    fn action_id(&self) -> ActionId {
        ActionId::new("article.edit")
    }
}

struct Article;
struct Wiki;

impl TargetType for Article {
    const KIND: &'static str = "article";
}

impl TargetType for Wiki {
    const KIND: &'static str = "wiki";
}

// This is the manual counterpart of:
// action EditArticle = "article.edit" for [Article, Wiki];
impl ActionOn<Article> for EditArticle {}
impl ActionOn<Wiki> for EditArticle {}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();
    policy.allow("wiki_editor", EditArticle);
    let authz = Authorizer::new(policy.build()?);

    let alice = Access::with_role("wiki_editor", Wiki::ident(42));
    let edits_991 = authz.can_any(
        &alice,
        EditArticle,
        (Article::ident(991), Wiki::ident(42)),
    );
    let edits_500 = authz.can_any(
        &alice,
        EditArticle,
        (Article::ident(500), Wiki::ident(7)),
    );

    println!("Alice edits Article 991 in Wiki 42: {edits_991}");
    println!("Alice edits Article 500 in Wiki 7: {edits_500}");

    assert!(edits_991);
    assert!(!edits_500);
    Ok(())
}
