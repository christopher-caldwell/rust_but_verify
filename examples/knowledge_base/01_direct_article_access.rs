//! Step 1: one role grants one action on one exact article.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

define_authorization! {
    target Article = "article" {
        action EditArticle = "article.edit";
    };
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    let mut policy = Policy::builder();
    policy.allow("article_editor", EditArticle);
    let authz = Authorizer::new(policy.build()?);

    // Bob has `article_editor` on Article 991, and nowhere else.
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
