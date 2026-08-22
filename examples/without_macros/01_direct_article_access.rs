//! The ordinary-Rust equivalent of Knowledge Base step 1.

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

impl ActionOn<Article> for EditArticle {}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();
    policy.allow("article_editor", EditArticle);
    let authz = Authorizer::new(policy.build()?);

    let bob = Access::with_role("article_editor", Article::ident(991));
    let edits_991 = authz
        .can(&bob, EditArticle, Article::ident(991))
        .is_allowed();
    let edits_992 = authz
        .can(&bob, EditArticle, Article::ident(992))
        .is_allowed();

    println!("Bob edits Article 991: {edits_991}");
    println!("Bob edits Article 992: {edits_992}");

    assert!(edits_991);
    assert!(!edits_992);
    Ok(())
}
