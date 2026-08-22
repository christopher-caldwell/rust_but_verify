//! Example 1: a knowledge base with Wiki-wide and Article-specific access.
//!
//! The application supplies the relevant facts for this request. RustButVerify
//! does not load articles, verify ownership, or discover an article's Wiki.

use rust_but_verify::{Access, Authorizer, Policy, TargetType, define_authorization};

define_authorization! {
    target Article = "article";
    target Wiki = "wiki";
    action EditArticle = "article.edit" for [Article, Wiki];
}

struct ArticleRecord {
    id: u64,
    wiki_id: u64,
    owner_id: u64,
}

struct User {
    id: u64,
}

fn main() -> Result<(), rust_but_verify::PolicyError> {
    // Policy is long-lived, global role-to-action configuration. A role has
    // the same meaning wherever applications assign it.
    let mut policy = Policy::builder();
    policy
        .allow("wiki_editor", EditArticle)
        .allow("article_owner", EditArticle);
    let authz = Authorizer::new(policy.build()?);
    let article = ArticleRecord {
        id: 991,
        wiki_id: 42,
        owner_id: 7,
    };

    // A durable Wiki assignment covers every article in Wiki 42. No leaf
    // assignments are materialized.
    let alice = Access::with_role("wiki_editor", Wiki::ident(42));

    // The application loaded the request's article and verified Bob's owner
    // relation; it supplies the resulting leaf-level fact.
    let bob = User { id: 7 };
    let mut bob_access = Access::empty();
    if bob.id == article.owner_id {
        bob_access.add_role("article_owner", Article::ident(article.id));
    }
    let charlie = Access::empty();

    assert!(
        authz
            .can_any(
                &alice,
                EditArticle,
                (Article::ident(article.id), Wiki::ident(article.wiki_id))
            )
            .is_allowed()
    );
    assert!(
        authz
            .can_any(
                &bob_access,
                EditArticle,
                (Article::ident(article.id), Wiki::ident(article.wiki_id))
            )
            .is_allowed()
    );
    assert!(
        !authz
            .can_any(
                &charlie,
                EditArticle,
                (Article::ident(article.id), Wiki::ident(article.wiki_id))
            )
            .is_allowed()
    );
    assert!(
        !authz
            .can(&bob_access, EditArticle, Article::ident(992))
            .is_allowed()
    );
    Ok(())
}
